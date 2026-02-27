use std::cell::{Cell, RefCell, UnsafeCell};
use std::ffi::{c_void, CStr};
use std::sync::Arc;

use super::component::MainThreadState;
use crate::params::{ParamId, ParamValue};
use crate::plugin::Plugin;
use crate::view::{ParentWindow, RawParent, View, ViewHost, ViewHostInner};
use vst3::Steinberg::Linux::{FileDescriptor, IEventHandlerTrait};
use vst3::Steinberg::Vst::{IComponentHandler, IComponentHandlerTrait};
use vst3::{Class, Interface, ComPtr, ComRef, ComWrapper, Steinberg::*};

pub struct Vst3ViewHost {
    pub handler: RefCell<Option<ComPtr<IComponentHandler>>>,
    // todo: not sure on best place, but this seems to match closest to the old code
    pub plug_frame: RefCell<Option<ComPtr<IPlugFrame>>>,
}

impl Vst3ViewHost {
    pub fn new() -> Vst3ViewHost {
        Vst3ViewHost {
            handler: RefCell::new(None),
            plug_frame: RefCell::new(None),
        }
    }
}

impl ViewHostInner for Vst3ViewHost {
    fn begin_gesture(&self, id: ParamId) {
        let handler = self.handler.borrow();
        if let Some(handler) = &*handler {
            unsafe {
                handler.beginEdit(id);
            }
        }
    }

    fn end_gesture(&self, id: ParamId) {
        let handler = self.handler.borrow();
        if let Some(handler) = &*handler {
            unsafe {
                handler.endEdit(id);
            }
        }
    }

    fn set_param(&self, id: ParamId, value: ParamValue) {
        let handler = self.handler.borrow();
        if let Some(handler) = &*handler {
            unsafe {
                handler.performEdit(id, value);
            }
        }
    }
}

pub struct PlugView<P: Plugin> {
    main_thread_state: Arc<UnsafeCell<MainThreadState<P>>>,
    /// Raw pointer allows accessing this PlugView as a COM object when needed.
    /// We use this rather than a ComPtr to avoid a circular reference / memory leak.
    /// It needs to be a cell so we can have interior mutability and actually populate this
    /// field after construction. This will allow us to later turn the IPlugView into
    /// IEventHandler / ITimerHandler.
    self_ptr: Cell<Option<*mut IPlugView>>,
}

// todo: not sure where to organize this
#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use vst3::Steinberg::Linux::*;

    impl<P: Plugin> IEventHandlerTrait for PlugView<P> {
        unsafe fn onFDIsSet(&self, _fd: FileDescriptor) {
            // todo: VERY NOT SURE if this is actually safe -
            // guard at least against re-entrant calls with Cell?
            let state = unsafe { &mut *self.main_thread_state.get() };
            state.view.as_mut().unwrap().poll();
        }
    }

    impl<P: Plugin> ITimerHandlerTrait for PlugView<P> {
        unsafe fn onTimer(&self) {
            let state = unsafe { &mut *self.main_thread_state.get() };
            state.view.as_mut().unwrap().poll();
        }
    }

    impl<P: Plugin> Class for PlugView<P> {
        type Interfaces = (IEventHandler, ITimerHandler, IPlugView);
    }

    pub(super) struct EventHandler<P: Plugin> {
        // TODO: This should be refactored to use a RefCell
        state: Arc<UnsafeCell<MainThreadState<P>>>,
    }

    impl<P: Plugin> EventHandler<P> {
        pub fn new(state: &Arc<UnsafeCell<MainThreadState<P>>>) -> EventHandler<P> {
            EventHandler {
                state: state.clone(),
            }
        }
    }

    impl<P: Plugin> Class for EventHandler<P> {
        type Interfaces = (IEventHandler, ITimerHandler);
    }

    impl<P: Plugin> IEventHandlerTrait for EventHandler<P> {
        unsafe fn onFDIsSet(&self, _fd: FileDescriptor) {
            // todo: VERY NOT SURE if this is actually safe
            let state = unsafe { &mut *self.state.get() };
            state.view.as_mut().unwrap().poll();
        }
    }

    impl<P: Plugin> ITimerHandlerTrait for EventHandler<P> {
        unsafe fn onTimer(&self) {
            let state = unsafe { &mut *self.state.get() };
            state.view.as_mut().unwrap().poll();
        }
    }
}

impl<P: Plugin> PlugView<P> {
    pub fn new(main_thread_state: &Arc<UnsafeCell<MainThreadState<P>>>) -> PlugView<P> {
        PlugView {
            main_thread_state: main_thread_state.clone(),
            self_ptr: Cell::new(None),
        }
    }

    // must be called after construction
    pub fn set_self_ptr(&self, ptr: *mut IPlugView) {
        self.self_ptr.set(Some(ptr));
    }
}

// todo: where would we like to put this?
/// Safely transitions from one COM interface type to another (like IPlugView to ITimerHandler)
/// using queryInterface.
macro_rules! query_interface {
    ($source_ptr:expr, $interface_type:ty) => {{
        let mut result_obj: *mut c_void = std::ptr::null_mut();
        let unknown_ptr = $source_ptr as *mut FUnknown;
        let iid = <$interface_type>::IID;

        let result = unsafe {
            ((*(*unknown_ptr).vtbl).queryInterface)(
                unknown_ptr,
                iid.as_ptr() as *const _,
                &mut result_obj,
            )
        };

        if result == kResultOk && !result_obj.is_null() {
            Some(result_obj as *mut $interface_type)
        } else {
            None
        }
    }};
}

impl<P: Plugin> IPlugViewTrait for PlugView<P> {
    unsafe fn isPlatformTypeSupported(&self, type_: FIDString) -> tresult {
        #[cfg(target_os = "windows")]
        if CStr::from_ptr(type_) == CStr::from_ptr(kPlatformTypeHWND) {
            return kResultTrue;
        }

        #[cfg(target_os = "macos")]
        if CStr::from_ptr(type_) == CStr::from_ptr(kPlatformTypeNSView) {
            return kResultTrue;
        }

        #[cfg(target_os = "linux")]
        if CStr::from_ptr(type_) == CStr::from_ptr(kPlatformTypeX11EmbedWindowID) {
            return kResultTrue;
        }

        kResultFalse
    }

    unsafe fn attached(&self, parent: *mut c_void, type_: FIDString) -> tresult {
        if self.isPlatformTypeSupported(type_) != kResultTrue {
            return kResultFalse;
        }

        #[cfg(target_os = "windows")]
        let raw_parent = RawParent::Win32(parent);

        #[cfg(target_os = "macos")]
        let raw_parent = RawParent::Cocoa(parent);

        #[cfg(target_os = "linux")]
        let raw_parent = RawParent::X11(parent as std::ffi::c_ulong);

        let main_thread_state = &mut *self.main_thread_state.get();

        let host = ViewHost::from_inner(main_thread_state.view_host.clone());
        let parent = ParentWindow::from_raw(raw_parent);
        let view = main_thread_state.plugin.view(host, &parent);
        main_thread_state.view = Some(view);

        #[cfg(target_os = "linux")]
        {
            use vst3::Steinberg::Linux::*;

            let Some(frame) = main_thread_state.view_host.plug_frame.borrow().clone() else {
                return kNotInitialized;
            };

            if let Some(run_loop) = frame.cast::<IRunLoop>() {
                if let Some(ptr) = self.self_ptr.get() {
                    if let Some(timer_handler_ptr) = query_interface!(ptr, ITimerHandler) {
                        run_loop.registerTimer(timer_handler_ptr, 16);

                        if let Some(fd) =
                            (*self.main_thread_state.get()).view.as_ref().unwrap().file_descriptor() {
                            if let Some(event_handler_ptr) = query_interface!(ptr, IEventHandler) {
                                run_loop.registerEventHandler(event_handler_ptr, 16);
                            }
                        }
                    }
                }
            }
        }

        kResultOk
    }

    unsafe fn removed(&self) -> tresult {
        let main_thread_state = &mut *self.main_thread_state.get();

        main_thread_state.view = None;

        kResultOk
    }

    unsafe fn onWheel(&self, _distance: f32) -> tresult {
        kResultFalse
    }

    unsafe fn onKeyDown(&self, _key: char16, _keyCode: int16, _modifiers: int16) -> tresult {
        kResultFalse
    }

    unsafe fn onKeyUp(&self, _key: char16, _keyCode: int16, _modifiers: int16) -> tresult {
        kResultFalse
    }

    unsafe fn getSize(&self, size: *mut ViewRect) -> tresult {
        if size.is_null() {
            return kResultFalse;
        }

        let main_thread_state = &*self.main_thread_state.get();

        if let Some(view) = &main_thread_state.view {
            let view_size = view.size();

            let rect = &mut *size;
            rect.left = 0;
            rect.top = 0;
            rect.right = view_size.width.round() as int32;
            rect.bottom = view_size.height.round() as int32;

            return kResultOk;
        }

        kResultFalse
    }

    unsafe fn onSize(&self, _newSize: *mut ViewRect) -> tresult {
        kNotImplemented
    }

    unsafe fn onFocus(&self, _state: TBool) -> tresult {
        kResultFalse
    }

    unsafe fn setFrame(&self, _frame: *mut IPlugFrame) -> tresult {
        let main_thread_state = &mut *self.main_thread_state.get();

        if let Some(frame) = ComRef::from_raw(_frame) {
            main_thread_state.view_host.plug_frame.replace(Some(frame.to_com_ptr()));
        }

        kResultOk
    }

    unsafe fn canResize(&self) -> tresult {
        kNotImplemented
    }

    unsafe fn checkSizeConstraint(&self, _rect: *mut ViewRect) -> tresult {
        kNotImplemented
    }
}

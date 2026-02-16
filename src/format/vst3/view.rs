use std::cell::{RefCell, UnsafeCell};
use std::ffi::{c_void, CStr};
use std::sync::Arc;

use vst3::Steinberg::Vst::{IComponentHandler, IComponentHandlerTrait};
use vst3::{Class, ComPtr, ComRef, ComWrapper, Steinberg::*};

use super::component::MainThreadState;
use crate::params::{ParamId, ParamValue};
use crate::plugin::Plugin;
use crate::view::{ParentWindow, RawParent, View, ViewHost, ViewHostInner};

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
    // TODO: not sure this should go here, or nested somewhere
    //  under plugin state.
    //  In old code, it lived right under view (essentially same as PlugView)
    //  so this is kinda the same.
    //  Also - do we really want to be bringing back EventHandler? I'm guessing
    //  it was removed for a reason.
    #[cfg(target_os = "linux")]
    handler: ComWrapper<linux::EventHandler<P>>,
}

// todo: not sure where to organize this
#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use vst3::Steinberg::Linux::*;

    pub(super) struct EventHandler<P: Plugin> {
        // TODO: old code had this as JUST an Arc,
        //  but in order to be able to call poll via this state, we
        //  need it to be an unsafecell. Is there a better way?
        //  I tend to think this is safe because, if it's all getting called
        //  from the "main thread", it shouldn't be getting concurrently accessed,
        //  but I don't trust that logic...
        state: Arc<UnsafeCell<MainThreadState<P>>>,
    }

    impl<P: Plugin> EventHandler<P> {
        pub fn new(state: &Arc<UnsafeCell<MainThreadState<P>>>,) -> EventHandler<P> {
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
            handler: ComWrapper::new(linux::EventHandler::new(main_thread_state)),
        }
    }
}

impl<P: Plugin> Class for PlugView<P> {
    type Interfaces = (IPlugView,);
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
                let timer_handler = self.handler.as_com_ref::<ITimerHandler>().unwrap();
                run_loop.registerTimer(timer_handler.as_ptr(), 16);

                if let Some(fd) = (*self.main_thread_state.get()).view.as_ref().unwrap().file_descriptor() {
                    let event_handler = self.handler.as_com_ref::<IEventHandler>().unwrap();
                    run_loop.registerEventHandler(event_handler.as_ptr(), fd);
                }
            }
        }

        // todo: old code had this - do we need it?
        // editor_state.editor.replace(Some(editor));

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
            main_thread_state
                .view_host
                .plug_frame
                .replace(Some(frame.to_com_ptr()));
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

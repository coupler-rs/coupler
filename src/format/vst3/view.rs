use std::cell::{RefCell, UnsafeCell};
use std::ffi::{CStr, c_void};
use std::sync::Arc;

use vst3::Steinberg::Vst::{IComponentHandler, IComponentHandlerTrait};
use vst3::{Class, ComPtr, ComRef, Steinberg::*};

use super::component::MainThreadState;
use crate::params::{ParamId, ParamValue};
use crate::plugin::Plugin;
use crate::view::{ParentWindow, RawParent, View, ViewHost, ViewHostInner};

pub struct Vst3ViewHost {
    pub handler: RefCell<Option<ComPtr<IComponentHandler>>>,
}

impl Vst3ViewHost {
    pub fn new() -> Vst3ViewHost {
        Vst3ViewHost {
            handler: RefCell::new(None),
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
}

impl<P: Plugin> PlugView<P> {
    pub fn new(main_thread_state: &Arc<UnsafeCell<MainThreadState<P>>>) -> PlugView<P> {
        PlugView {
            main_thread_state: main_thread_state.clone(),
        }
    }
}

impl<P: Plugin> Class for PlugView<P> {
    type Interfaces = (IPlugView,);
}

impl<P: Plugin> IPlugViewTrait for PlugView<P> {
    unsafe fn isPlatformTypeSupported(&self, type_: FIDString) -> tresult {
        #[cfg(target_os = "windows")]
        if unsafe { CStr::from_ptr(type_) } == unsafe { CStr::from_ptr(kPlatformTypeHWND) } {
            return kResultTrue;
        }

        #[cfg(target_os = "macos")]
        if unsafe { CStr::from_ptr(type_) } == unsafe { CStr::from_ptr(kPlatformTypeNSView) } {
            return kResultTrue;
        }

        #[cfg(target_os = "linux")]
        if unsafe { CStr::from_ptr(type_) }
            == unsafe { CStr::from_ptr(kPlatformTypeX11EmbedWindowID) }
        {
            return kResultTrue;
        }

        kResultFalse
    }

    unsafe fn attached(&self, parent: *mut c_void, type_: FIDString) -> tresult {
        if unsafe { self.isPlatformTypeSupported(type_) } != kResultTrue {
            return kResultFalse;
        }

        #[cfg(target_os = "windows")]
        let raw_parent = RawParent::Win32(parent);

        #[cfg(target_os = "macos")]
        let raw_parent = RawParent::Cocoa(parent);

        #[cfg(target_os = "linux")]
        let raw_parent = RawParent::X11(parent as std::ffi::c_ulong);

        let main_thread_state = unsafe { &mut *self.main_thread_state.get() };

        let host = ViewHost::from_inner(main_thread_state.view_host.clone());
        let parent = unsafe { ParentWindow::from_raw(raw_parent) };
        let view = main_thread_state.plugin.view(host, &parent);
        main_thread_state.view = Some(view);

        kResultOk
    }

    unsafe fn removed(&self) -> tresult {
        let main_thread_state = unsafe { &mut *self.main_thread_state.get() };

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

        let main_thread_state = unsafe { &*self.main_thread_state.get() };

        if let Some(view) = &main_thread_state.view {
            let view_size = view.size();

            let rect = unsafe { &mut *size };
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

    unsafe fn setFrame(&self, frame: *mut IPlugFrame) -> tresult {
        let main_thread_state = unsafe { &mut *self.main_thread_state.get() };
        main_thread_state.frame =
            unsafe { ComRef::from_raw(frame) }.map(|frame| frame.to_com_ptr());

        kResultOk
    }

    unsafe fn canResize(&self) -> tresult {
        kNotImplemented
    }

    unsafe fn checkSizeConstraint(&self, _rect: *mut ViewRect) -> tresult {
        kNotImplemented
    }
}

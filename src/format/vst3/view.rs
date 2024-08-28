use std::cell::UnsafeCell;
use std::ffi::{c_void, CStr};
use std::sync::Arc;

use vst3::{Class, Steinberg::*};

use super::component::MainThreadState;
use crate::editor::{Editor, ParentWindow, RawParent};
use crate::plugin::Plugin;

pub struct View<P: Plugin> {
    main_thread_state: Arc<UnsafeCell<MainThreadState<P>>>,
}

impl<P: Plugin> View<P> {
    pub fn new(main_thread_state: &Arc<UnsafeCell<MainThreadState<P>>>) -> View<P> {
        View {
            main_thread_state: main_thread_state.clone(),
        }
    }
}

impl<P: Plugin> Class for View<P> {
    type Interfaces = (IPlugView,);
}

impl<P: Plugin> IPlugViewTrait for View<P> {
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

        let editor = main_thread_state.plugin.editor(&ParentWindow::from_raw(raw_parent));
        main_thread_state.editor = Some(editor);

        kResultOk
    }

    unsafe fn removed(&self) -> tresult {
        let main_thread_state = &mut *self.main_thread_state.get();

        main_thread_state.editor = None;

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

        if let Some(editor) = &main_thread_state.editor {
            let editor_size = editor.size();

            let rect = &mut *size;
            rect.left = 0;
            rect.top = 0;
            rect.right = editor_size.width.round() as int32;
            rect.bottom = editor_size.height.round() as int32;

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
        kNotImplemented
    }

    unsafe fn canResize(&self) -> tresult {
        kNotImplemented
    }

    unsafe fn checkSizeConstraint(&self, _rect: *mut ViewRect) -> tresult {
        kNotImplemented
    }
}

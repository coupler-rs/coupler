use std::ffi::{CStr, c_void};
use std::rc::Rc;
use std::sync::Arc;

use vst3::Steinberg::Vst::{IComponentHandler, IComponentHandlerTrait};
use vst3::{Class, ComPtr, ComRef, Steinberg::*};

use super::component::MainThreadState;
use crate::editor::{Editor, EditorHost, EditorHostInner, ParentWindow, RawParent};
use crate::params::{ParamId, ParamValue};
use crate::plugin::Plugin;
use crate::sync::{sync_cell::SyncCell, thread_cell::ThreadCell};
use crate::util::RequireSendSync;

pub struct Vst3EditorHost {
    pub handler: Option<ComPtr<IComponentHandler>>,
}

impl EditorHostInner for Vst3EditorHost {
    fn begin_gesture(&self, id: ParamId) {
        if let Some(handler) = &self.handler {
            unsafe {
                handler.beginEdit(id);
            }
        }
    }

    fn end_gesture(&self, id: ParamId) {
        if let Some(handler) = &self.handler {
            unsafe {
                handler.endEdit(id);
            }
        }
    }

    fn set_param(&self, id: ParamId, value: ParamValue) {
        if let Some(handler) = &self.handler {
            unsafe {
                handler.performEdit(id, value);
            }
        }
    }
}

pub struct PlugView<P: Plugin> {
    main_thread_state: Arc<SyncCell<MainThreadState<P>>>,
}

impl<P: Plugin> RequireSendSync for PlugView<P> {}

impl<P: Plugin> PlugView<P> {
    pub fn new(main_thread_state: &Arc<SyncCell<MainThreadState<P>>>) -> PlugView<P> {
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

        let mut main_thread_state = self.main_thread_state.borrow();

        let host = EditorHost::from_inner(Rc::new(Vst3EditorHost {
            handler: main_thread_state.handler.clone(),
        }));
        let parent = unsafe { ParentWindow::from_raw(raw_parent) };
        let editor = main_thread_state.plugin.editor(host, &parent);
        main_thread_state.editor = Some(ThreadCell::new(editor));

        kResultOk
    }

    unsafe fn removed(&self) -> tresult {
        let mut main_thread_state = self.main_thread_state.borrow();

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

        let main_thread_state = self.main_thread_state.borrow();

        let editor_size = if let Some(editor) = &main_thread_state.editor {
            editor.size()
        } else {
            main_thread_state.plugin.editor_size()
        };

        let rect = unsafe { &mut *size };
        rect.left = 0;
        rect.top = 0;
        rect.right = editor_size.width.round() as int32;
        rect.bottom = editor_size.height.round() as int32;

        kResultOk
    }

    unsafe fn onSize(&self, _newSize: *mut ViewRect) -> tresult {
        kNotImplemented
    }

    unsafe fn onFocus(&self, _state: TBool) -> tresult {
        kResultFalse
    }

    unsafe fn setFrame(&self, frame: *mut IPlugFrame) -> tresult {
        let mut main_thread_state = self.main_thread_state.borrow();
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

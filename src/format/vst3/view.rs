use std::ffi::c_void;
use std::sync::{Arc, Mutex};

use vst3::{Class, Steinberg::*};

use super::component::MainThreadState;
use crate::Plugin;

pub struct View<P> {
    main_thread_state: Arc<Mutex<MainThreadState<P>>>,
}

impl<P: Plugin> View<P> {
    pub fn new(main_thread_state: &Arc<Mutex<MainThreadState<P>>>) -> View<P> {
        View {
            main_thread_state: main_thread_state.clone(),
        }
    }
}

impl<P: Plugin> Class for View<P> {
    type Interfaces = (IPlugView,);
}

impl<P: Plugin> IPlugViewTrait for View<P> {
    unsafe fn isPlatformTypeSupported(&self, _type_: FIDString) -> tresult {
        kNotImplemented
    }

    unsafe fn attached(&self, _parent: *mut c_void, _type_: FIDString) -> tresult {
        kNotImplemented
    }

    unsafe fn removed(&self) -> tresult {
        kNotImplemented
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

    unsafe fn getSize(&self, _size: *mut ViewRect) -> tresult {
        kNotImplemented
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

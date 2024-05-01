use std::sync::RwLock;

use vst3::ComPtr;
use vst3::Steinberg::Vst::{IComponentHandler, IComponentHandlerTrait};

use crate::host::HostInner;
use crate::params::{ParamId, ParamValue};

pub struct Vst3Host {
    pub handler: RwLock<Option<ComPtr<IComponentHandler>>>,
}

impl Vst3Host {
    pub fn new() -> Vst3Host {
        Vst3Host {
            handler: RwLock::new(None),
        }
    }
}

impl HostInner for Vst3Host {
    fn begin_gesture(&self, id: ParamId) {
        let handler = self.handler.read().unwrap();
        if let Some(handler) = &*handler {
            // TODO: only call this on main thread
            unsafe {
                handler.beginEdit(id);
            }
        }
    }

    fn end_gesture(&self, id: ParamId) {
        let handler = self.handler.read().unwrap();
        if let Some(handler) = &*handler {
            // TODO: only call this on main thread
            unsafe {
                handler.endEdit(id);
            }
        }
    }

    fn set_param(&self, id: ParamId, value: ParamValue) {
        let handler = self.handler.read().unwrap();
        if let Some(handler) = &*handler {
            // TODO: only call this on main thread
            unsafe {
                handler.performEdit(id, value);
            }
        }
    }
}

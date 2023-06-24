use std::ffi::c_void;
use std::marker::PhantomData;
use std::sync::Arc;

use vst3_bindgen::{Class, ComWrapper, Steinberg::Vst::*, Steinberg::*};

use super::{Vst3Info, Vst3Plugin};
use crate::{Plugin, PluginInfo};

pub struct Factory<P> {
    info: Arc<PluginInfo>,
    vst3_info: Vst3Info,
    _marker: PhantomData<P>,
}

impl<P: Plugin + Vst3Plugin> Factory<P> {
    pub fn new() -> Factory<P> {
        Factory {
            info: Arc::new(P::info()),
            vst3_info: P::vst3_info(),
            _marker: PhantomData,
        }
    }
}

impl<P: Plugin + Vst3Plugin> Class for Factory<P> {
    type Interfaces = (IPluginFactory3,);
}

impl<P: Plugin + Vst3Plugin> IPluginFactoryTrait for Factory<P> {
    unsafe fn getFactoryInfo(&self, info: *mut PFactoryInfo) -> tresult {
        kNotImplemented
    }

    unsafe fn countClasses(&self) -> int32 {
        1
    }

    unsafe fn getClassInfo(&self, index: int32, info: *mut PClassInfo) -> tresult {
        kNotImplemented
    }

    unsafe fn createInstance(
        &self,
        cid: FIDString,
        _iid: FIDString,
        obj: *mut *mut c_void,
    ) -> tresult {
        kNotImplemented
    }
}

impl<P: Plugin + Vst3Plugin> IPluginFactory2Trait for Factory<P> {
    unsafe fn getClassInfo2(&self, index: int32, info: *mut PClassInfo2) -> tresult {
        kNotImplemented
    }
}

impl<P: Plugin + Vst3Plugin> IPluginFactory3Trait for Factory<P> {
    unsafe fn getClassInfoUnicode(&self, index: int32, info: *mut PClassInfoW) -> tresult {
        kNotImplemented
    }

    unsafe fn setHostContext(&self, context: *mut FUnknown) -> tresult {
        kNotImplemented
    }
}

use std::ffi::{c_void, CStr};
use std::marker::PhantomData;
use std::sync::Arc;

use vst3_bindgen::{uid, Class, Steinberg::Vst::*, Steinberg::*};

use super::util::{copy_cstring, copy_wstring};
use super::{Uuid, Vst3Info, Vst3Plugin};
use crate::{Plugin, PluginInfo};

fn uuid_to_tuid(uuid: &Uuid) -> TUID {
    uid(uuid.0, uuid.1, uuid.2, uuid.3)
}

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
        let info = &mut *info;

        copy_cstring(&self.info.vendor, &mut info.vendor);
        copy_cstring(&self.info.url, &mut info.url);
        copy_cstring(&self.info.email, &mut info.email);
        info.flags = PFactoryInfo_::FactoryFlags_::kUnicode as int32;

        kResultOk
    }

    unsafe fn countClasses(&self) -> int32 {
        1
    }

    unsafe fn getClassInfo(&self, index: int32, info: *mut PClassInfo) -> tresult {
        if index == 0 {
            let info = &mut *info;

            info.cid = uuid_to_tuid(&self.vst3_info.class_id);
            info.cardinality = PClassInfo_::ClassCardinality_::kManyInstances as int32;
            copy_cstring("Audio Module Class", &mut info.category);
            copy_cstring(&self.info.name, &mut info.name);

            return kResultOk;
        }

        kInvalidArgument
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
        if index == 0 {
            let info = &mut *info;

            info.cid = uuid_to_tuid(&self.vst3_info.class_id);
            info.cardinality = PClassInfo_::ClassCardinality_::kManyInstances as int32;
            copy_cstring("Audio Module Class", &mut info.category);
            copy_cstring(&self.info.name, &mut info.name);
            info.classFlags = 0;
            copy_cstring("Fx", &mut info.subCategories);
            copy_cstring(&self.info.vendor, &mut info.vendor);
            copy_cstring("", &mut info.version);
            let version_str = CStr::from_ptr(SDKVersionString).to_str().unwrap();
            copy_cstring(version_str, &mut info.sdkVersion);

            return kResultOk;
        }

        kInvalidArgument
    }
}

impl<P: Plugin + Vst3Plugin> IPluginFactory3Trait for Factory<P> {
    unsafe fn getClassInfoUnicode(&self, index: int32, info: *mut PClassInfoW) -> tresult {
        if index == 0 {
            let info = &mut *info;

            info.cid = uuid_to_tuid(&self.vst3_info.class_id);
            info.cardinality = PClassInfo_::ClassCardinality_::kManyInstances as int32;
            copy_cstring("Audio Module Class", &mut info.category);
            copy_wstring(&self.info.name, &mut info.name);
            info.classFlags = 0;
            copy_cstring("Fx", &mut info.subCategories);
            copy_wstring(&self.info.vendor, &mut info.vendor);
            copy_wstring("", &mut info.version);
            let version_str = CStr::from_ptr(SDKVersionString).to_str().unwrap();
            copy_wstring(version_str, &mut info.sdkVersion);

            return kResultOk;
        }

        kInvalidArgument
    }

    unsafe fn setHostContext(&self, _context: *mut FUnknown) -> tresult {
        kResultOk
    }
}

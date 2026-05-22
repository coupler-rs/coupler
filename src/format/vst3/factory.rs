use std::ffi::{CStr, c_void};
use std::marker::PhantomData;

use vst3::{Class, ComWrapper, Steinberg::Vst::*, Steinberg::*, uid};

use super::component::Component;
use super::util::{copy_wstring, with_vst3_info};
use super::{Uuid, Vst3Plugin};
use crate::plugin::Plugin;
use crate::util::{RequireSendSync, copy_cstring, with_info};

fn uuid_to_tuid(uuid: &Uuid) -> TUID {
    uid(uuid.0, uuid.1, uuid.2, uuid.3)
}

pub struct Factory<P> {
    class_id: Uuid,
    _marker: PhantomData<fn() -> P>,
}

impl<P: Plugin> RequireSendSync for Factory<P> {}

impl<P: Plugin + Vst3Plugin> Factory<P> {
    pub fn new() -> Factory<P> {
        let mut class_id = None;
        with_vst3_info::<P, _>(|info| {
            class_id = Some(info.class_id);
        });

        Factory {
            class_id: class_id.unwrap(),
            _marker: PhantomData,
        }
    }
}

impl<P: Plugin> Class for Factory<P> {
    type Interfaces = (IPluginFactory3,);
}

impl<P: Plugin> IPluginFactoryTrait for Factory<P> {
    unsafe fn getFactoryInfo(&self, info: *mut PFactoryInfo) -> tresult {
        let info = unsafe { &mut *info };

        info.flags = PFactoryInfo_::FactoryFlags_::kUnicode as int32;

        with_info::<P, _>(|plugin_info| {
            copy_cstring(plugin_info.vendor, &mut info.vendor);
            copy_cstring(plugin_info.url, &mut info.url);
            copy_cstring(plugin_info.email, &mut info.email);
        });

        kResultOk
    }

    unsafe fn countClasses(&self) -> int32 {
        1
    }

    unsafe fn getClassInfo(&self, index: int32, info: *mut PClassInfo) -> tresult {
        if index == 0 {
            let info = unsafe { &mut *info };

            info.cid = uuid_to_tuid(&self.class_id);
            info.cardinality = PClassInfo_::ClassCardinality_::kManyInstances as int32;
            copy_cstring("Audio Module Class", &mut info.category);

            with_info::<P, _>(|plugin_info| {
                copy_cstring(plugin_info.name, &mut info.name);
            });

            return kResultOk;
        }

        kInvalidArgument
    }

    unsafe fn createInstance(
        &self,
        cid: FIDString,
        iid: FIDString,
        obj: *mut *mut c_void,
    ) -> tresult {
        let cid = unsafe { &*(cid as *const TUID) };
        let class_id = uuid_to_tuid(&self.class_id);
        if cid == &class_id {
            let component = ComWrapper::new(Component::<P>::new());
            let unknown = component.as_com_ref::<FUnknown>().unwrap();
            let ptr = unknown.as_ptr();
            return unsafe { ((*(*ptr).vtbl).queryInterface)(ptr, iid as *const TUID, obj) };
        }

        kInvalidArgument
    }
}

impl<P: Plugin> IPluginFactory2Trait for Factory<P> {
    unsafe fn getClassInfo2(&self, index: int32, info: *mut PClassInfo2) -> tresult {
        if index == 0 {
            let info = unsafe { &mut *info };

            info.cid = uuid_to_tuid(&self.class_id);
            info.cardinality = PClassInfo_::ClassCardinality_::kManyInstances as int32;
            copy_cstring("Audio Module Class", &mut info.category);
            info.classFlags = 0;
            copy_cstring("Fx", &mut info.subCategories);
            let version_str = unsafe { CStr::from_ptr(SDKVersionString) }.to_str().unwrap();
            copy_cstring(version_str, &mut info.sdkVersion);

            with_info::<P, _>(|plugin_info| {
                copy_cstring(plugin_info.name, &mut info.name);
                copy_cstring(plugin_info.vendor, &mut info.vendor);
                copy_cstring(plugin_info.version, &mut info.version);
            });

            return kResultOk;
        }

        kInvalidArgument
    }
}

impl<P: Plugin> IPluginFactory3Trait for Factory<P> {
    unsafe fn getClassInfoUnicode(&self, index: int32, info: *mut PClassInfoW) -> tresult {
        if index == 0 {
            let info = unsafe { &mut *info };

            info.cid = uuid_to_tuid(&self.class_id);
            info.cardinality = PClassInfo_::ClassCardinality_::kManyInstances as int32;
            copy_cstring("Audio Module Class", &mut info.category);
            info.classFlags = 0;
            copy_cstring("Fx", &mut info.subCategories);
            let version_str = unsafe { CStr::from_ptr(SDKVersionString) }.to_str().unwrap();
            copy_wstring(version_str, &mut info.sdkVersion);

            with_info::<P, _>(|plugin_info| {
                copy_wstring(plugin_info.name, &mut info.name);
                copy_wstring(plugin_info.vendor, &mut info.vendor);
                copy_wstring(plugin_info.version, &mut info.version);
            });

            return kResultOk;
        }

        kInvalidArgument
    }

    unsafe fn setHostContext(&self, _context: *mut FUnknown) -> tresult {
        kResultOk
    }
}

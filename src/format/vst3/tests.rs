use std::error::Error;
use std::ffi::CStr;
use std::fmt::{self, Formatter};
use std::io::{self, Read, Write};
use std::{ptr, slice};

use crate::buffers::Buffers;
use crate::engine::{Config, Engine};
use crate::events::Events;
use crate::host::Host;
use crate::params::{ParamId, ParamValue};
use crate::plugin::{Plugin, PluginInfo};
use crate::view::{ParentWindow, Size, View, ViewHost};

use vst3::Steinberg::Vst::{IComponent, SDKVersionString};
use vst3::Steinberg::{char16, char8, int32};
use vst3::Steinberg::{
    kInvalidArgument, kResultOk, FIDString, IPluginFactory, IPluginFactory2, IPluginFactory2Trait,
    IPluginFactory3, IPluginFactory3Trait, IPluginFactoryTrait, PClassInfo, PClassInfo2,
    PClassInfoW, PClassInfo_, PFactoryInfo, PFactoryInfo_,
};
use vst3::{uid, ComPtr, Interface};

use super::{get_plugin_factory, Uuid, Vst3Info, Vst3Plugin};

const NAME: &str = "test plugin";
const VERSION: &str = "1.2.3";
const VENDOR: &str = "test vendor";
const URL: &str = "https://example.com/";
const EMAIL: &str = "example@example.com";
const CLASS_ID: [u32; 4] = [0x11111111, 0x22222222, 0x33333333, 0x44444444];

struct TestPlugin;

impl Plugin for TestPlugin {
    type Engine = TestEngine;
    type View = TestView;

    fn info() -> PluginInfo {
        PluginInfo {
            name: NAME.to_string(),
            version: VERSION.to_string(),
            vendor: VENDOR.to_string(),
            url: URL.to_string(),
            email: EMAIL.to_string(),
            buses: Vec::new(),
            layouts: vec![],
            params: Vec::new(),
            has_view: false,
        }
    }
    fn new(_host: Host) -> Self {
        TestPlugin
    }
    fn set_param(&mut self, _id: ParamId, _value: ParamValue) {}
    fn get_param(&self, _id: ParamId) -> ParamValue {
        0.0
    }
    fn parse_param(&self, _id: ParamId, _text: &str) -> Option<ParamValue> {
        None
    }
    fn display_param(
        &self,
        _id: ParamId,
        _value: ParamValue,
        _fmt: &mut Formatter,
    ) -> Result<(), fmt::Error> {
        Ok(())
    }
    fn save(&self, _output: &mut impl Write) -> io::Result<()> {
        Ok(())
    }
    fn load(&mut self, _input: &mut impl Read) -> io::Result<()> {
        Ok(())
    }
    fn engine(&mut self, _config: Config) -> Self::Engine {
        TestEngine
    }
    fn view(&mut self, _host: ViewHost, _parent: &ParentWindow) -> Self::View {
        TestView
    }

    #[allow(unused_variables)]
    fn latency(&self, _config: &Config) -> u64 {
        0
    }
}

impl Vst3Plugin for TestPlugin {
    fn vst3_info() -> Vst3Info {
        Vst3Info {
            class_id: Uuid(CLASS_ID[0], CLASS_ID[1], CLASS_ID[2], CLASS_ID[3]),
        }
    }
}

struct TestEngine;

impl Engine for TestEngine {
    fn reset(&mut self) {}
    fn flush(&mut self, _events: Events) {}
    fn process(&mut self, _buffers: Buffers, _events: Events) {}
}

struct TestView;

impl View for TestView {
    fn size(&self) -> Size {
        Size {
            width: 0.0,
            height: 0.0,
        }
    }
    fn param_changed(&mut self, _id: ParamId, _value: ParamValue) {}
}

fn str_from_chars(chars: &[char8]) -> Result<&str, Box<dyn Error>> {
    let bytes = unsafe { slice::from_raw_parts(chars.as_ptr() as *const u8, chars.len()) };
    Ok(CStr::from_bytes_until_nul(bytes)?.to_str()?)
}

fn string_from_wchars(wchars: &[char16]) -> Result<String, std::char::DecodeUtf16Error> {
    let utf16 = unsafe { slice::from_raw_parts(wchars.as_ptr() as *const u16, wchars.len()) };
    char::decode_utf16(utf16.iter().copied().take_while(|c| *c != 0)).collect()
}

#[test]
fn factory() {
    let ptr = get_plugin_factory::<TestPlugin>() as *mut IPluginFactory;
    let factory = unsafe { ComPtr::from_raw(ptr) }.unwrap();

    let mut factory_info = PFactoryInfo {
        vendor: [0; 64],
        url: [0; 256],
        email: [0; 128],
        flags: 0,
    };
    let result = unsafe { factory.getFactoryInfo(&mut factory_info) };
    assert_eq!(result, kResultOk);

    assert_eq!(str_from_chars(&factory_info.vendor).unwrap(), VENDOR);
    assert_eq!(str_from_chars(&factory_info.url).unwrap(), URL);
    assert_eq!(str_from_chars(&factory_info.email).unwrap(), EMAIL);
    assert_eq!(
        factory_info.flags,
        PFactoryInfo_::FactoryFlags_::kUnicode as int32
    );

    let class_count = unsafe { factory.countClasses() };
    assert_eq!(class_count, 1);

    let mut class_info = PClassInfo {
        cid: [0; 16],
        cardinality: 0,
        category: [0; 32],
        name: [0; 64],
    };
    unsafe { factory.getClassInfo(0, &mut class_info) };

    assert_eq!(
        class_info.cid,
        uid(CLASS_ID[0], CLASS_ID[1], CLASS_ID[2], CLASS_ID[3])
    );
    assert_eq!(
        class_info.cardinality,
        PClassInfo_::ClassCardinality_::kManyInstances as int32
    );

    assert_eq!(
        str_from_chars(&class_info.category).unwrap(),
        "Audio Module Class"
    );
    assert_eq!(str_from_chars(&class_info.name).unwrap(), NAME);

    let result = unsafe { factory.getClassInfo(1, &mut class_info) };
    assert_eq!(result, kInvalidArgument);

    let factory_2 = factory.cast::<IPluginFactory2>().unwrap();

    let mut class_info_2 = PClassInfo2 {
        cid: [0; 16],
        cardinality: 0,
        category: [0; 32],
        name: [0; 64],
        classFlags: 0,
        subCategories: [0; 128],
        vendor: [0; 64],
        version: [0; 64],
        sdkVersion: [0; 64],
    };
    let result = unsafe { factory_2.getClassInfo2(0, &mut class_info_2) };
    assert_eq!(result, kResultOk);

    let sdk_version = unsafe { CStr::from_ptr(SDKVersionString) }.to_str().unwrap();

    assert_eq!(
        class_info_2.cid,
        uid(CLASS_ID[0], CLASS_ID[1], CLASS_ID[2], CLASS_ID[3])
    );
    assert_eq!(
        class_info_2.cardinality,
        PClassInfo_::ClassCardinality_::kManyInstances as int32
    );
    assert_eq!(
        str_from_chars(&class_info_2.category).unwrap(),
        "Audio Module Class"
    );
    assert_eq!(str_from_chars(&class_info_2.name).unwrap(), NAME);
    assert_eq!(class_info_2.classFlags, 0);
    assert_eq!(str_from_chars(&class_info_2.subCategories).unwrap(), "Fx");
    assert_eq!(str_from_chars(&class_info_2.vendor).unwrap(), VENDOR);
    assert_eq!(str_from_chars(&class_info_2.version).unwrap(), VERSION);
    assert_eq!(
        str_from_chars(&class_info_2.sdkVersion).unwrap(),
        sdk_version
    );

    let result = unsafe { factory_2.getClassInfo(1, &mut class_info) };
    assert_eq!(result, kInvalidArgument);

    let factory_3 = factory.cast::<IPluginFactory3>().unwrap();

    let mut class_info_w = PClassInfoW {
        cid: [0; 16],
        cardinality: 0,
        category: [0; 32],
        name: [0; 64],
        classFlags: 0,
        subCategories: [0; 128],
        vendor: [0; 64],
        version: [0; 64],
        sdkVersion: [0; 64],
    };
    let result = unsafe { factory_3.getClassInfoUnicode(0, &mut class_info_w) };
    assert_eq!(result, kResultOk);

    assert_eq!(
        class_info_w.cid,
        uid(CLASS_ID[0], CLASS_ID[1], CLASS_ID[2], CLASS_ID[3])
    );
    assert_eq!(
        class_info_w.cardinality,
        PClassInfo_::ClassCardinality_::kManyInstances as int32
    );
    assert_eq!(
        str_from_chars(&class_info_w.category).unwrap(),
        "Audio Module Class"
    );
    assert_eq!(string_from_wchars(&class_info_w.name).unwrap(), NAME);
    assert_eq!(class_info_w.classFlags, 0);
    assert_eq!(str_from_chars(&class_info_w.subCategories).unwrap(), "Fx");
    assert_eq!(string_from_wchars(&class_info_w.vendor).unwrap(), VENDOR);
    assert_eq!(string_from_wchars(&class_info_w.version).unwrap(), VERSION);
    assert_eq!(
        string_from_wchars(&class_info_w.sdkVersion).unwrap(),
        sdk_version
    );

    let result = unsafe { factory_3.getClassInfoUnicode(1, &mut class_info_w) };
    assert_eq!(result, kInvalidArgument);

    let mut obj = ptr::null_mut();
    let result = unsafe {
        factory.createInstance(
            uid(CLASS_ID[0], CLASS_ID[1], CLASS_ID[2], CLASS_ID[3]).as_ptr(),
            IComponent::IID.as_ptr() as FIDString,
            &mut obj,
        )
    };
    assert_eq!(result, kResultOk);

    unsafe { ComPtr::from_raw(obj as *mut IComponent) }.unwrap();
}

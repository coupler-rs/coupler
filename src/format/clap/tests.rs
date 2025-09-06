use std::ffi::{c_char, CStr};
use std::fmt::{self, Formatter};
use std::io::{self, Read, Write};

use crate::buffers::Buffers;
use crate::events::Events;
use crate::view::{ParentWindow, Size, View, ViewHost};

use clap_sys::plugin_factory::{clap_plugin_factory, CLAP_PLUGIN_FACTORY_ID};
use clap_sys::version::CLAP_VERSION;

use crate::engine::{Config, Engine};
use crate::host::Host;
use crate::params::{ParamId, ParamValue};
use crate::plugin::{Plugin, PluginInfo};

use super::{ClapInfo, ClapPlugin, Factory};

const NAME: &str = "test plugin";
const VERSION: &str = "1.2.3";
const VENDOR: &str = "test vendor";
const URL: &str = "https://example.com/";
const EMAIL: &str = "example@example.com";
const ID: &str = "com.example.plugin";

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

impl ClapPlugin for TestPlugin {
    fn clap_info() -> ClapInfo {
        ClapInfo { id: ID.to_string() }
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

unsafe fn str_from_ptr<'a>(ptr: *const c_char) -> Result<&'a str, std::str::Utf8Error> {
    CStr::from_ptr(ptr).to_str()
}

#[test]
fn factory() {
    let factory = Factory::<TestPlugin>::new();

    let result = unsafe { factory.init() };
    assert!(result);

    let plugin_factory =
        unsafe { factory.get(CLAP_PLUGIN_FACTORY_ID.as_ptr()) as *const clap_plugin_factory };

    let plugin_count = unsafe { ((*plugin_factory).get_plugin_count).unwrap()(plugin_factory) };
    assert_eq!(plugin_count, 1);

    let desc_ptr = unsafe { ((*plugin_factory).get_plugin_descriptor).unwrap()(plugin_factory, 1) };
    assert!(desc_ptr.is_null());

    let desc_ptr = unsafe { ((*plugin_factory).get_plugin_descriptor).unwrap()(plugin_factory, 0) };
    assert!(!desc_ptr.is_null());

    let desc = unsafe { &*desc_ptr };
    assert_eq!(desc.clap_version.major, CLAP_VERSION.major);
    assert_eq!(desc.clap_version.minor, CLAP_VERSION.minor);
    assert_eq!(desc.clap_version.revision, CLAP_VERSION.revision);
    assert_eq!(unsafe { str_from_ptr(desc.id).unwrap() }, ID);
    assert_eq!(unsafe { str_from_ptr(desc.name).unwrap() }, NAME);
    assert_eq!(unsafe { str_from_ptr(desc.vendor).unwrap() }, VENDOR);
    assert_eq!(unsafe { str_from_ptr(desc.url).unwrap() }, URL);
    assert_eq!(unsafe { str_from_ptr(desc.manual_url).unwrap() }, "");
    assert_eq!(unsafe { str_from_ptr(desc.support_url).unwrap() }, "");
    assert_eq!(unsafe { str_from_ptr(desc.version).unwrap() }, VERSION);
    assert_eq!(unsafe { str_from_ptr(desc.description).unwrap() }, "");
    assert!(unsafe { *desc.features }.is_null());

    unsafe { factory.deinit() };
}

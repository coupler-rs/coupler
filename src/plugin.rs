use std::io::{self, Read, Write};
use std::sync::Arc;

use crate::bus::{BusInfo, Layout};
use crate::editor::{Editor, Parent};
use crate::params::{ParamId, ParamInfo, ParamValue};
use crate::process::{Config, Processor};

pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub vendor: String,
    pub url: String,
    pub email: String,
    pub buses: Vec<BusInfo>,
    pub layouts: Vec<Layout>,
    pub params: Vec<ParamInfo>,
    pub has_editor: bool,
}

impl Default for PluginInfo {
    fn default() -> PluginInfo {
        PluginInfo {
            name: String::new(),
            version: String::new(),
            vendor: String::new(),
            url: String::new(),
            email: String::new(),
            buses: Vec::new(),
            layouts: Vec::new(),
            params: Vec::new(),
            has_editor: false,
        }
    }
}

pub trait HostInner {}

#[derive(Clone)]
pub struct Host {
    _inner: Arc<dyn HostInner>,
}

impl Host {
    pub fn from_inner(inner: Arc<dyn HostInner>) -> Host {
        Host { _inner: inner }
    }
}

pub trait Plugin: Send + Sized + 'static {
    type Processor: Processor;
    type Editor: Editor;

    fn info() -> PluginInfo;
    fn new(host: Host) -> Self;
    fn set_param(&mut self, id: ParamId, value: ParamValue);
    fn get_param(&self, id: ParamId) -> ParamValue;
    fn save(&self, output: &mut impl Write) -> io::Result<()>;
    fn load(&mut self, input: &mut impl Read) -> io::Result<()>;
    fn processor(&self, config: Config) -> Self::Processor;
    fn editor(&self, parent: Parent) -> Self::Editor;

    #[allow(unused_variables)]
    fn latency(&self, config: &Config) -> u64 {
        0
    }
}

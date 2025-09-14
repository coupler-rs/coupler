use std::fmt::{self, Formatter};
use std::io::{self, Read, Write};

use crate::bus::{BusInfo, Layout};
use crate::engine::{Config, Engine};
use crate::host::Host;
use crate::params::{ParamId, ParamInfo, ParamValue};
use crate::view::{ParentWindow, View, ViewHost};

pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub vendor: String,
    pub url: String,
    pub email: String,
    pub buses: Vec<BusInfo>,
    pub layouts: Vec<Layout>,
    pub params: Vec<ParamInfo>,
    pub has_view: bool,
}

#[allow(clippy::derivable_impls)]
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
            has_view: false,
        }
    }
}

pub trait Plugin: Send + Sized + 'static {
    type Engine: Engine;
    type View: View;

    fn info() -> PluginInfo;
    fn new(host: Host) -> Self;
    fn set_param(&mut self, id: ParamId, value: ParamValue);
    fn get_param(&self, id: ParamId) -> ParamValue;
    fn parse_param(&self, id: ParamId, text: &str) -> Option<ParamValue>;
    fn display_param(
        &self,
        id: ParamId,
        value: ParamValue,
        fmt: &mut Formatter,
    ) -> Result<(), fmt::Error>;
    fn save(&self, output: &mut impl Write) -> io::Result<()>;
    fn load(&mut self, input: &mut impl Read) -> io::Result<()>;
    fn engine(&mut self, config: Config) -> Self::Engine;
    fn view(&mut self, host: ViewHost, parent: &ParentWindow) -> Self::View;

    #[allow(unused_variables)]
    fn latency(&self, config: &Config) -> u64 {
        0
    }
}

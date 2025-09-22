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
        }
    }
}

pub trait Plugin: Send + Sized + 'static {
    type Engine: Engine;
    type View: View;

    fn info() -> PluginInfo;
    fn new(host: Host) -> Self;

    fn buses(&self) -> Vec<BusInfo>;
    fn layouts(&self) -> Vec<Layout>;

    fn params(&self) -> Vec<ParamInfo>;
    fn set_param(&mut self, id: ParamId, value: ParamValue);
    fn get_param(&self, id: ParamId) -> ParamValue;
    fn parse_param(&self, id: ParamId, text: &str) -> Option<ParamValue>;
    fn display_param(
        &self,
        id: ParamId,
        value: ParamValue,
        fmt: &mut Formatter,
    ) -> Result<(), fmt::Error>;

    fn save(&self, output: impl Write) -> io::Result<()>;
    fn load(&mut self, input: impl Read) -> io::Result<()>;

    fn engine(&mut self, config: &Config) -> Self::Engine;

    fn has_view(&self) -> bool;
    fn view(&mut self, host: ViewHost, parent: &ParentWindow) -> Self::View;

    #[allow(unused_variables)]
    fn latency(&self, config: &Config) -> u64 {
        0
    }
}

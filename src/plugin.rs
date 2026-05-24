use std::{fmt, io};

use crate::bus::{BusConfig, BusInfo};
use crate::editor::{Editor, EditorHost, ParentWindow, Size};
use crate::host::Host;
use crate::params::{ParamId, ParamInfo, ParamValue};
use crate::process::{Config, Processor};

#[derive(Default)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub vendor: String,
    pub url: String,
    pub email: String,
}

pub trait Plugin: Send + Sized + 'static {
    type Processor: Processor;
    type Editor: Editor;

    fn info() -> PluginInfo;
    fn new(host: Host) -> Self;

    fn buses(&self) -> Vec<BusInfo>;
    fn bus_configs(&self) -> Vec<BusConfig>;

    fn params(&self) -> Vec<ParamInfo>;
    fn set_param(&mut self, id: ParamId, value: ParamValue);
    fn get_param(&self, id: ParamId) -> ParamValue;
    fn parse_param(&self, id: ParamId, text: &str) -> Option<ParamValue>;
    fn display_param(
        &self,
        id: ParamId,
        value: ParamValue,
        write: impl fmt::Write,
    ) -> Result<(), fmt::Error>;

    fn save(&self, output: impl io::Write) -> io::Result<()>;
    fn load(&mut self, input: impl io::Read) -> io::Result<()>;

    fn processor(&mut self, config: Config) -> Self::Processor;

    fn has_editor(&self) -> bool;
    fn editor_size(&self) -> Size;
    fn editor(&mut self, host: EditorHost, parent: &ParentWindow) -> Self::Editor;

    #[allow(unused_variables)]
    fn latency(&self, config: Config) -> u64 {
        0
    }
}

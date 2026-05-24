use std::{fmt, io};

use crate::bus::{BuildBusConfigs, BuildBuses};
use crate::editor::{Editor, EditorHost, ParentWindow, Size};
use crate::host::Host;
use crate::params::{ParamId, ParamInfo, ParamValue};
use crate::process::{Config, Processor};

#[derive(Default)]
pub struct PluginInfo<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub vendor: &'a str,
    pub url: &'a str,
    pub email: &'a str,
}

pub trait BuildInfo {
    fn info(self, info: PluginInfo);
}

pub trait Plugin: Send + Sized + 'static {
    type Processor: Processor;
    type Editor: Editor;

    fn info(build: impl BuildInfo);
    fn new(host: Host) -> Self;

    fn buses(&self, build: impl BuildBuses);
    fn bus_configs(&self, build: impl BuildBusConfigs);

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

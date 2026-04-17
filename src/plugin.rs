use std::fmt::{self, Formatter};
use std::io::{self, Read, Write};

use crate::bus::{BusInfo, Layout};
use crate::editor::{Editor, EditorHost, ParentWindow, Size};
use crate::host::Host;
use crate::params::{ParamId, ParamInfo, ParamValue};
use crate::process::{Config, Processor};

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
    type Processor: Processor;
    type Editor: Editor;

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

    fn processor(&mut self, config: &Config) -> Self::Processor;

    fn has_editor(&self) -> bool;
    fn editor_size(&self) -> Size;
    fn editor(&mut self, host: EditorHost, parent: &ParentWindow) -> Self::Editor;

    #[allow(unused_variables)]
    fn latency(&self, config: &Config) -> u64 {
        0
    }
}

use std::io::{self, Read, Write};

use crate::bus::{BusInfo, Layout};
use crate::editor::{Editor, EditorHost, ParentWindow};
use crate::host::Host;
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
            has_editor: false,
        }
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
    fn processor(&mut self, config: Config) -> Self::Processor;
    fn editor(&mut self, host: EditorHost, parent: &ParentWindow) -> Self::Editor;

    #[allow(unused_variables)]
    fn latency(&self, config: &Config) -> u64 {
        0
    }
}

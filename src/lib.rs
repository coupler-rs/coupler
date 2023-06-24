use std::io::{self, Read, Write};

pub mod bus;
pub mod format;
pub mod param;
pub mod process;

use bus::{BusInfo, Layout};
use param::ParamInfo;
use process::{Config, ProcessInfo};

pub type ParamId = u32;

pub type ParamValue = f64;

pub struct PluginInfo {
    pub name: String,
    pub vendor: String,
    pub url: String,
    pub email: String,
    pub inputs: Vec<BusInfo>,
    pub outputs: Vec<BusInfo>,
    pub layouts: Vec<Layout>,
    pub params: Vec<ParamInfo>,
}

pub trait Plugin: Send + Sync + Sized + 'static {
    type Processor: Processor<Self>;
    type Editor: Editor<Self>;

    fn info() -> PluginInfo;
    fn create() -> Self;
    fn set_param(&self, id: ParamId, value: ParamValue);
    fn get_param(&self, id: ParamId) -> ParamValue;
    fn save(&self, output: &mut impl Write) -> io::Result<()>;
    fn load(&self, input: &mut impl Read) -> io::Result<()>;
}

pub struct Buffers {}

pub struct Events {}

pub trait Processor<P>: Send + Sized + 'static {
    fn create(plugin: &P, config: Config) -> Self;
    fn info(&self) -> ProcessInfo;
    fn set_param(&mut self, id: ParamId, value: ParamValue);
    fn reset(&mut self);
    fn process(&mut self, buffers: Buffers, events: Events);
}

pub struct EditorContext {}

pub struct ParentWindow {}

pub struct Size {}

pub trait Editor<P>: Sized + 'static {
    fn create(plugin: &P, context: EditorContext, parent: &ParentWindow) -> Self;
    fn size(&self) -> Size;
    fn set_param(&mut self, id: ParamId, value: ParamValue);
}

use std::io::{self, Read, Write};

pub mod buffers;
pub mod bus;
pub mod events;
pub mod format;
pub mod param;

mod sync;

use buffers::Buffers;
use bus::{BusInfo, Layout};
use events::Events;
use param::ParamInfo;

pub type ParamId = u32;

pub type ParamValue = f64;

pub struct PluginInfo {
    pub name: String,
    pub vendor: String,
    pub url: String,
    pub email: String,
    pub buses: Vec<BusInfo>,
    pub layouts: Vec<Layout>,
    pub params: Vec<ParamInfo>,
}

pub struct Host {}

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
    fn editor(&self, container: Container) -> Self::Editor;

    #[allow(unused_variables)]
    fn latency(&self, config: &Config) -> u64 {
        0
    }
}

#[derive(Clone)]
pub struct Config {
    pub layout: Layout,
    pub sample_rate: f64,
    pub max_buffer_size: usize,
}

pub trait Processor: Send + Sized + 'static {
    fn set_param(&mut self, id: ParamId, value: ParamValue);
    fn reset(&mut self);
    fn process(&mut self, buffers: Buffers, events: Events);
}

pub struct Container {}

pub struct Size {}

pub trait Editor: Sized + 'static {
    fn exists() -> bool {
        true
    }

    fn size(&self) -> Size;
    fn set_param(&mut self, id: ParamId, value: ParamValue);
}

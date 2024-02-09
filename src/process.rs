use crate::buffers::Buffers;
use crate::bus::Layout;
use crate::events::Events;
use crate::params::{ParamId, ParamValue};

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

use crate::buffers::Buffers;
use crate::bus::Layout;
use crate::events::Events;

#[derive(Clone)]
pub struct Config {
    pub layout: Layout,
    pub sample_rate: f64,
    pub max_buffer_size: usize,
}

pub trait Engine: Send + Sized + 'static {
    fn reset(&mut self);
    fn flush(&mut self, events: Events);
    fn process(&mut self, buffers: Buffers, events: Events);
}

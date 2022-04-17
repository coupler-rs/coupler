use crate::{buffer::*, bus::*, param::*, plugin::*};

pub struct ProcessContext<'a> {
    sample_rate: f64,
    inputs: &'a [BusState],
    outputs: &'a [BusState],
}

impl<'a> ProcessContext<'a> {
    pub fn new(
        sample_rate: f64,
        inputs: &'a [BusState],
        outputs: &'a [BusState],
    ) -> ProcessContext<'a> {
        ProcessContext { sample_rate, inputs, outputs }
    }

    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }

    pub fn inputs(&self) -> &'a [BusState] {
        self.inputs
    }

    pub fn outputs(&self) -> &'a [BusState] {
        self.outputs
    }
}

pub struct Event {
    pub offset: usize,
    pub event: EventType,
}

pub enum EventType {
    ParamChange(ParamChange),
}

pub struct ParamChange {
    pub id: ParamId,
    pub value_normalized: f64,
}

pub trait Processor: Send + Sized {
    type Plugin: Plugin;

    fn create(plugin: PluginHandle<Self::Plugin>, context: &ProcessContext) -> Self;
    fn reset(&mut self, context: &ProcessContext);
    fn process(&mut self, context: &ProcessContext, buffers: Buffers, events: &[Event]);
}

use crate::{buffer::*, bus::*, param::*, plugin::*};

pub struct ProcessContext<'a> {
    sample_rate: f64,
    input_layouts: &'a [BusLayout],
    output_layouts: &'a [BusLayout],
}

impl<'a> ProcessContext<'a> {
    pub fn new(
        sample_rate: f64,
        input_layouts: &'a [BusLayout],
        output_layouts: &'a [BusLayout],
    ) -> ProcessContext<'a> {
        ProcessContext { sample_rate, input_layouts, output_layouts }
    }

    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }

    pub fn input_layouts(&self) -> &'a [BusLayout] {
        self.input_layouts
    }

    pub fn output_layouts(&self) -> &'a [BusLayout] {
        self.output_layouts
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
    fn process(&mut self, context: &ProcessContext, buffers: &mut Buffers, events: &[Event]);
}

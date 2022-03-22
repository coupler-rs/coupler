use crate::{buffer::*, bus::*, param::*, plugin::*};

pub struct ProcessContext<'a> {
    pub sample_rate: f64,
    pub input_layouts: &'a [BusLayout],
    pub output_layouts: &'a [BusLayout],
}

pub struct ParamChange {
    pub offset: usize,
    pub id: ParamId,
    pub value: f64,
}

pub trait Processor: Send + Sized {
    type Plugin: Plugin;

    fn create(plugin: &Self::Plugin, context: &ProcessContext) -> Self;
    fn reset(&mut self, context: &ProcessContext);
    fn process(
        &mut self,
        context: &ProcessContext,
        buffers: &mut Buffers,
        param_changes: &[ParamChange],
    );
}

use crate::{buffer::*, bus::*, params::*};

pub struct ProcessContext<'a> {
    pub(crate) sample_rate: f64,
    pub(crate) input_layouts: &'a [BusLayout],
    pub(crate) output_layouts: &'a [BusLayout],
    pub(crate) param_list: &'a ParamList,
    pub(crate) param_values: &'a [f64],
}

impl<'a> ProcessContext<'a> {
    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }

    pub fn input_layouts(&self) -> &[BusLayout] {
        self.input_layouts
    }

    pub fn output_layouts(&self) -> &[BusLayout] {
        self.output_layouts
    }

    pub fn get_param(&self, id: ParamId) -> f64 {
        self.param_values[self.param_list.indices[&id]]
    }
}

pub struct ParamChange {
    pub id: ParamId,
    pub offset: usize,
    pub value: f64,
}

pub trait Processor: Send + Sized {
    fn process(
        &mut self,
        context: &ProcessContext,
        buffers: &mut AudioBuffers,
        param_changes: &[ParamChange],
    );
    fn reset(&mut self, context: &ProcessContext);
}

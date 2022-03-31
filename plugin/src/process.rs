use crate::{atomic::AtomicF64, buffer::*, bus::*, param::*, plugin::*};

pub struct ProcessContext<'a> {
    sample_rate: f64,
    input_layouts: &'a [BusLayout],
    output_layouts: &'a [BusLayout],
    param_states: &'a ParamStates,
}

impl<'a> ProcessContext<'a> {
    pub fn new(
        sample_rate: f64,
        input_layouts: &'a [BusLayout],
        output_layouts: &'a [BusLayout],
        param_states: &'a ParamStates,
    ) -> ProcessContext<'a> {
        ProcessContext { sample_rate, input_layouts, output_layouts, param_states }
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

    #[inline]
    pub fn get_param<P: Param + 'static>(&self, key: ParamKey<P>) -> P::Value {
        self.param_states.get_param(key)
    }
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

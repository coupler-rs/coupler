use crate::{atomic::AtomicF64, buffer::*, bus::*, param::*, plugin::*};

pub struct ProcessContext<'a> {
    sample_rate: f64,
    input_layouts: &'a [BusLayout],
    output_layouts: &'a [BusLayout],
    param_list: &'a ParamList,
    param_values: &'a [AtomicF64],
}

impl<'a> ProcessContext<'a> {
    pub fn new(
        sample_rate: f64,
        input_layouts: &'a [BusLayout],
        output_layouts: &'a [BusLayout],
        param_list: &'a ParamList,
        param_values: &'a [AtomicF64],
    ) -> ProcessContext<'a> {
        ProcessContext { sample_rate, input_layouts, output_layouts, param_list, param_values }
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
        let index = self.param_list.get_param_index(key.id).expect("Invalid parameter key");
        let param_info = self.param_list.get_param(key.id).unwrap();
        let param = param_info.param.downcast_ref::<P>().expect("Incorrect parameter type");
        param.from_normalized(self.param_values[index].load())
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

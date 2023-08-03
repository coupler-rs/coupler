use std::sync::atomic::Ordering;

use crate::sync::{bitset::AtomicBitset, float::AtomicF64};
use crate::{ParamInfo, ParamValue};
use crate::{Plugin, Processor};

pub struct ParamValues {
    values: Vec<AtomicF64>,
    plugin_dirty: AtomicBitset,
    processor_dirty: AtomicBitset,
}

impl ParamValues {
    pub fn new(params: &[ParamInfo]) -> ParamValues {
        ParamValues {
            values: params.iter().map(|p| AtomicF64::new(p.default)).collect(),
            plugin_dirty: AtomicBitset::with_len(params.len()),
            processor_dirty: AtomicBitset::with_len(params.len()),
        }
    }

    pub fn set_from_processor(&self, index: usize, value: ParamValue) {
        self.values[index].store(value, Ordering::Relaxed);
        self.plugin_dirty.set(index, Ordering::Release);
    }

    pub fn sync_processor<P: Processor>(&self, params: &[ParamInfo], processor: &mut P) {
        for index in self.processor_dirty.drain(Ordering::Acquire) {
            let id = params[index].id;
            let value = self.values[index].load(Ordering::Relaxed);
            processor.set_param(id, value);
        }
    }

    pub fn set_from_plugin(&self, index: usize, value: ParamValue) {
        self.values[index].store(value, Ordering::Relaxed);
        self.processor_dirty.set(index, Ordering::Release);
    }

    pub fn sync_plugin<P: Plugin>(&self, params: &[ParamInfo], plugin: &mut P) {
        for index in self.plugin_dirty.drain(Ordering::Acquire) {
            let id = params[index].id;
            let value = self.values[index].load(Ordering::Relaxed);
            plugin.set_param(id, value);
        }
    }
}

use std::sync::atomic::Ordering;

use super::bitset::{self, AtomicBitset};
use super::float::AtomicF64;
use crate::params::{ParamInfo, ParamValue};

pub struct ParamValues {
    values: Vec<AtomicF64>,
    dirty: AtomicBitset,
}

impl ParamValues {
    pub fn new(params: &[ParamInfo]) -> ParamValues {
        ParamValues {
            values: params.iter().map(|p| AtomicF64::new(p.default)).collect(),
            dirty: AtomicBitset::with_len(params.len()),
        }
    }

    pub fn set(&self, index: usize, value: ParamValue) {
        self.values[index].store(value, Ordering::Relaxed);
        self.dirty.set(index, Ordering::Release);
    }

    pub fn poll(&self) -> ParamChanges {
        ParamChanges {
            values: &self.values,
            iter: self.dirty.drain(Ordering::Acquire),
        }
    }
}

pub struct ParamChanges<'a> {
    values: &'a [AtomicF64],
    iter: bitset::Drain<'a>,
}

impl<'a> Iterator for ParamChanges<'a> {
    type Item = (usize, ParamValue);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.iter.next() {
            Some((index, self.values[index].load(Ordering::Relaxed)))
        } else {
            None
        }
    }
}

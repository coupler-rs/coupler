use std::sync::atomic::Ordering;

use super::bitset::{self, AtomicBitset};
use super::float::AtomicF64;
use crate::params::ParamValue;

pub struct ParamValues {
    values: Vec<AtomicF64>,
    dirty: AtomicBitset,
}

impl ParamValues {
    pub fn with_count(count: usize) -> ParamValues {
        ParamValues {
            values: (0..count).map(|_| AtomicF64::new(0.0)).collect(),
            dirty: AtomicBitset::with_len(count),
        }
    }

    pub fn set(&self, index: usize, value: ParamValue) {
        self.values[index].store(value, Ordering::Relaxed);
        self.dirty.set(index, true, Ordering::Release);
    }

    pub fn poll(&self) -> Poll {
        Poll {
            values: &self.values,
            iter: self.dirty.drain(Ordering::Acquire),
        }
    }
}

pub struct Poll<'a> {
    values: &'a [AtomicF64],
    iter: bitset::Drain<'a>,
}

impl<'a> Iterator for Poll<'a> {
    type Item = (usize, ParamValue);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.iter.next() {
            Some((index, self.values[index].load(Ordering::Relaxed)))
        } else {
            None
        }
    }
}

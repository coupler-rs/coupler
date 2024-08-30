use std::sync::atomic::Ordering;

use super::bitset::{self, AtomicBitset};
use super::float::AtomicF64;
use crate::params::{ParamInfo, ParamValue};

pub struct ParamGestures {
    values: Vec<AtomicF64>,
    gestures: AtomicBitset,
    dirty: AtomicBitset,
}

impl ParamGestures {
    pub fn new(params: &[ParamInfo]) -> ParamGestures {
        ParamGestures {
            values: params.iter().map(|p| AtomicF64::new(p.default)).collect(),
            gestures: AtomicBitset::with_len(params.len()),
            dirty: AtomicBitset::with_len(params.len()),
        }
    }

    pub fn begin_gesture(&self, index: usize) {
        self.gestures.set(index, Ordering::Relaxed);
        self.dirty.set(index, Ordering::Release);
    }

    pub fn end_gesture(&self, index: usize) {
        self.gestures.reset(index, Ordering::Relaxed);
        self.dirty.set(index, Ordering::Release);
    }

    pub fn set_value(&self, index: usize, value: ParamValue) {
        self.values[index].store(value, Ordering::Relaxed);
        self.dirty.set(index, Ordering::Release);
    }

    pub fn poll(&self) -> Poll {
        Poll {
            values: &self.values,
            gestures: &self.gestures,
            iter: self.dirty.drain(Ordering::Acquire),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum GestureState {
    Off,
    On,
}

#[derive(Copy, Clone)]
pub struct GestureUpdate {
    value: ParamValue,
    state: GestureState,
}

pub struct Poll<'a> {
    values: &'a [AtomicF64],
    gestures: &'a AtomicBitset,
    iter: bitset::Drain<'a>,
}

impl<'a> Iterator for Poll<'a> {
    type Item = (usize, GestureUpdate);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.iter.next() {
            let value = self.values[index].load(Ordering::Relaxed);
            let state = if self.gestures.get(index, Ordering::Relaxed) {
                GestureState::On
            } else {
                GestureState::Off
            };

            Some((index, GestureUpdate { value, state }))
        } else {
            None
        }
    }
}

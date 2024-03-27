use std::sync::atomic::{AtomicU64, Ordering};

use super::bitset::{self, AtomicBitset};
use super::float::AtomicF64;
use crate::params::{ParamInfo, ParamValue};

pub struct ParamGestures {
    values: Vec<AtomicF64>,
    gestures: Vec<AtomicU64>,
    dirty: AtomicBitset,
}

impl ParamGestures {
    pub fn new(params: &[ParamInfo]) -> ParamGestures {
        ParamGestures {
            values: params.iter().map(|p| AtomicF64::new(p.default)).collect(),
            gestures: params.iter().map(|_| AtomicU64::new(0)).collect(),
            dirty: AtomicBitset::with_len(params.len()),
        }
    }

    pub fn begin_gesture(&self, index: usize, value: ParamValue) {
        self.gestures[index].fetch_add(1, Ordering::Relaxed);
        self.dirty.set(index, Ordering::Release);
    }

    pub fn end_gesture(&self, index: usize, value: ParamValue) {
        self.gestures[index].fetch_sub(1, Ordering::Relaxed);
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
    gestures: &'a [AtomicU64],
    iter: bitset::Drain<'a>,
}

impl<'a> Iterator for Poll<'a> {
    type Item = (usize, GestureUpdate);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.iter.next() {
            let value = self.values[index].load(Ordering::Relaxed);
            let state = if self.gestures[index].load(Ordering::Relaxed) == 0 {
                GestureState::Off
            } else {
                GestureState::On
            };

            Some((index, GestureUpdate { value, state }))
        } else {
            None
        }
    }
}

use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::slice;

#[derive(Eq, PartialEq, Clone)]
pub enum BusLayout {
    Stereo,
}

impl BusLayout {
    pub fn channel_count(&self) -> usize {
        match self {
            BusLayout::Stereo => 2,
        }
    }
}

pub struct AudioBuses<'a> {
    pub(crate) samples: usize,
    pub(crate) inputs: &'a [AudioBus<'a>],
    pub(crate) outputs: &'a mut [AudioBus<'a>],
}

impl<'a> AudioBuses<'a> {
    pub fn samples(&self) -> usize {
        self.samples
    }

    pub fn inputs(&self) -> &[AudioBus<'a>] {
        self.inputs
    }

    pub fn outputs(&mut self) -> &mut [AudioBus<'a>] {
        self.outputs
    }
}

pub struct AudioBus<'a> {
    pub(crate) enabled: bool,
    pub(crate) layout: &'a BusLayout,
    pub(crate) samples: usize,
    pub(crate) channels: &'a mut [AudioBuffer<'a>],
}

impl<'a> AudioBus<'a> {
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn layout(&self) -> &BusLayout {
        self.layout
    }

    pub fn samples(&self) -> usize {
        self.samples
    }

    pub fn channels(&self) -> &[AudioBuffer<'a>] {
        self.channels
    }

    pub fn channels_mut(&mut self) -> &mut [AudioBuffer<'a>] {
        self.channels
    }
}

pub struct AudioBuffer<'a> {
    pub(crate) ptr: *mut f32,
    pub(crate) len: usize,
    pub(crate) phantom: PhantomData<&'a ()>,
}

impl<'a> Deref for AudioBuffer<'a> {
    type Target = [f32];

    fn deref(&self) -> &'a [f32] {
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl<'a> DerefMut for AudioBuffer<'a> {
    fn deref_mut(&mut self) -> &'a mut [f32] {
        unsafe { slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

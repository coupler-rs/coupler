use crate::bus::BusState;

use std::marker::PhantomData;
use std::slice;

pub struct Buffers<'a> {
    offset: usize,
    len: usize,
    input_states: &'a [BusState],
    input_indices: &'a [(usize, usize)],
    input_ptrs: &'a [*const f32],
    output_states: &'a [BusState],
    output_indices: &'a [(usize, usize)],
    output_ptrs: &'a [*mut f32],
    phantom: PhantomData<(&'a f32, &'a mut f32)>,
}

impl<'a> Buffers<'a> {
    pub(crate) unsafe fn new(
        len: usize,
        input_states: &'a [BusState],
        input_indices: &'a [(usize, usize)],
        input_ptrs: &'a [*const f32],
        output_states: &'a [BusState],
        output_indices: &'a [(usize, usize)],
        output_ptrs: &'a [*mut f32],
    ) -> Buffers<'a> {
        Buffers {
            offset: 0,
            len,
            input_states,
            input_indices,
            input_ptrs,
            output_states,
            output_indices,
            output_ptrs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn samples(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn inputs(&self) -> Buses<'a> {
        Buses {
            offset: self.offset,
            len: self.len,
            states: self.input_states,
            indices: self.input_indices,
            ptrs: self.input_ptrs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn outputs(&mut self) -> BusesMut<'a> {
        BusesMut {
            offset: self.offset,
            len: self.len,
            states: self.output_states,
            indices: self.output_indices,
            ptrs: self.output_ptrs,
            phantom: PhantomData,
        }
    }
}

pub struct Buses<'a> {
    offset: usize,
    len: usize,
    states: &'a [BusState],
    indices: &'a [(usize, usize)],
    ptrs: &'a [*const f32],
    phantom: PhantomData<&'a f32>,
}

impl<'a> Buses<'a> {
    #[inline]
    pub fn samples(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn buses(&self) -> usize {
        self.indices.len()
    }

    #[inline]
    pub fn bus(&self, index: usize) -> Option<Bus<'a>> {
        if let Some((start, end)) = self.indices.get(index) {
            Some(Bus {
                offset: self.offset,
                len: self.len,
                state: &self.states[index],
                ptrs: &self.ptrs[*start..*end],
                phantom: PhantomData,
            })
        } else {
            None
        }
    }
}

pub struct Bus<'a> {
    offset: usize,
    len: usize,
    state: &'a BusState,
    ptrs: &'a [*const f32],
    phantom: PhantomData<&'a f32>,
}

impl<'a> Bus<'a> {
    #[inline]
    pub fn samples(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn channels(&self) -> usize {
        self.ptrs.len()
    }

    #[inline]
    pub fn enabled(&self) -> bool {
        self.state.enabled
    }

    #[inline]
    pub fn channel(&self, index: usize) -> Option<&[f32]> {
        if let Some(ptr) = self.ptrs.get(index) {
            if self.len != 0 {
                unsafe { Some(slice::from_raw_parts(ptr.add(self.offset), self.len)) }
            } else {
                Some(&[])
            }
        } else {
            None
        }
    }
}

pub struct BusesMut<'a> {
    offset: usize,
    len: usize,
    states: &'a [BusState],
    indices: &'a [(usize, usize)],
    ptrs: &'a [*mut f32],
    phantom: PhantomData<&'a mut f32>,
}

impl<'a> BusesMut<'a> {
    #[inline]
    pub fn samples(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn buses(&self) -> usize {
        self.indices.len()
    }

    #[inline]
    pub fn bus(&mut self, index: usize) -> Option<BusMut<'a>> {
        if let Some((start, end)) = self.indices.get(index) {
            Some(BusMut {
                offset: self.offset,
                len: self.len,
                state: &self.states[index],
                ptrs: &self.ptrs[*start..*end],
                phantom: PhantomData,
            })
        } else {
            None
        }
    }
}

pub struct BusMut<'a> {
    offset: usize,
    len: usize,
    state: &'a BusState,
    ptrs: &'a [*mut f32],
    phantom: PhantomData<&'a mut f32>,
}

impl<'a> BusMut<'a> {
    #[inline]
    pub fn samples(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn channels(&self) -> usize {
        self.ptrs.len()
    }

    #[inline]
    pub fn enabled(&self) -> bool {
        self.state.enabled
    }

    #[inline]
    pub fn channel(&mut self, index: usize) -> Option<&mut [f32]> {
        if let Some(ptr) = self.ptrs.get(index) {
            if self.len != 0 {
                unsafe { Some(slice::from_raw_parts_mut(ptr.add(self.offset), self.len)) }
            } else {
                Some(&mut [])
            }
        } else {
            None
        }
    }
}

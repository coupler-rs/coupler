use crate::bus::BusState;

use std::marker::PhantomData;
use std::slice;

pub struct Buffers<'a, 'b, 'c> {
    offset: usize,
    len: usize,
    input_states: &'a [BusState],
    input_indices: &'a [(usize, usize)],
    input_ptrs: &'a [*const f32],
    output_states: &'a [BusState],
    output_indices: &'a [(usize, usize)],
    output_ptrs: &'a [*mut f32],
    phantom: PhantomData<(&'b f32, &'c mut f32)>,
}

impl<'a, 'b, 'c> Buffers<'a, 'b, 'c> {
    pub(crate) unsafe fn new(
        len: usize,
        input_states: &'a [BusState],
        input_indices: &'a [(usize, usize)],
        input_ptrs: &'a [*const f32],
        output_states: &'a [BusState],
        output_indices: &'a [(usize, usize)],
        output_ptrs: &'a [*mut f32],
    ) -> Buffers<'a, 'b, 'c> {
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
    pub fn inputs(&self) -> Buses {
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
    pub fn outputs(&mut self) -> BusesMut {
        BusesMut {
            offset: self.offset,
            len: self.len,
            states: self.output_states,
            indices: self.output_indices,
            ptrs: self.output_ptrs,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn split(&mut self) -> (Buses, BusesMut) {
        let inputs = Buses {
            offset: self.offset,
            len: self.len,
            states: self.input_states,
            indices: self.input_indices,
            ptrs: self.input_ptrs,
            phantom: PhantomData,
        };

        let outputs = BusesMut {
            offset: self.offset,
            len: self.len,
            states: self.output_states,
            indices: self.output_indices,
            ptrs: self.output_ptrs,
            phantom: PhantomData,
        };

        (inputs, outputs)
    }
}

pub struct Buses<'a, 'b> {
    offset: usize,
    len: usize,
    states: &'a [BusState],
    indices: &'a [(usize, usize)],
    ptrs: &'a [*const f32],
    phantom: PhantomData<&'b f32>,
}

impl<'a, 'b> Buses<'a, 'b> {
    #[inline]
    pub fn samples(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn buses(&self) -> usize {
        self.indices.len()
    }

    #[inline]
    pub fn bus(&self, index: usize) -> Option<Bus> {
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

pub struct Bus<'a, 'b> {
    offset: usize,
    len: usize,
    state: &'a BusState,
    ptrs: &'a [*const f32],
    phantom: PhantomData<&'b f32>,
}

impl<'a, 'b> Bus<'a, 'b> {
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

pub struct BusesMut<'a, 'b> {
    offset: usize,
    len: usize,
    states: &'a [BusState],
    indices: &'a [(usize, usize)],
    ptrs: &'a [*mut f32],
    phantom: PhantomData<&'b mut f32>,
}

impl<'a, 'b> BusesMut<'a, 'b> {
    #[inline]
    pub fn samples(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn buses(&self) -> usize {
        self.indices.len()
    }

    #[inline]
    pub fn bus(&mut self, index: usize) -> Option<BusMut> {
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

pub struct BusMut<'a, 'b> {
    offset: usize,
    len: usize,
    state: &'a BusState,
    ptrs: &'a [*mut f32],
    phantom: PhantomData<&'b mut f32>,
}

impl<'a, 'b> BusMut<'a, 'b> {
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

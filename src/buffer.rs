use crate::bus::BusState;
use crate::process::Event;

use std::marker::PhantomData;
use std::mem::MaybeUninit;
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
    pub fn borrow(&self) -> Buffers {
        Buffers {
            offset: self.offset,
            len: self.len,
            input_states: self.input_states,
            input_indices: self.input_indices,
            input_ptrs: self.input_ptrs,
            output_states: self.output_states,
            output_indices: self.output_indices,
            output_ptrs: self.output_ptrs,
            phantom: PhantomData,
        }
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
    pub fn split(self) -> (Buses<'a, 'b>, BusesMut<'a, 'c>) {
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

    #[inline]
    pub fn split_at(self, sample: usize) -> (Buffers<'a, 'b, 'c>, Buffers<'a, 'b, 'c>) {
        assert!(sample <= self.len);

        let first = Buffers {
            offset: self.offset,
            len: sample,
            input_states: self.input_states,
            input_indices: self.input_indices,
            input_ptrs: self.input_ptrs,
            output_states: self.output_states,
            output_indices: self.output_indices,
            output_ptrs: self.output_ptrs,
            phantom: PhantomData,
        };

        let second = Buffers {
            offset: self.offset + sample,
            len: self.len - sample,
            input_states: self.input_states,
            input_indices: self.input_indices,
            input_ptrs: self.input_ptrs,
            output_states: self.output_states,
            output_indices: self.output_indices,
            output_ptrs: self.output_ptrs,
            phantom: PhantomData,
        };

        (first, second)
    }

    #[inline]
    pub fn chunks(self, chunk_size: usize) -> Chunks<'a, 'b, 'c> {
        Chunks { buffers: self, chunk_size }
    }

    #[inline]
    pub fn split_at_events<'e>(self, events: &'e [Event]) -> SplitAtEvents<'a, 'b, 'c, 'e> {
        SplitAtEvents { buffers: self, events, offset: 0 }
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

    #[inline]
    pub fn all_buses<const N: usize>(&self) -> Option<[Bus; N]> {
        if N != self.indices.len() {
            return None;
        }

        let buses: MaybeUninit<[Bus; N]> = MaybeUninit::uninit();
        for (index, (start, end)) in self.indices.iter().enumerate() {
            unsafe {
                (buses.as_ptr() as *mut Bus).add(index).write(Bus {
                    offset: self.offset,
                    len: self.len,
                    state: &self.states[index],
                    ptrs: &self.ptrs[*start..*end],
                    phantom: PhantomData,
                });
            }
        }

        Some(unsafe { buses.assume_init() })
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

    #[inline]
    pub fn all_buses<const N: usize>(&mut self) -> Option<[BusMut; N]> {
        if N != self.indices.len() {
            return None;
        }

        let buses: MaybeUninit<[BusMut; N]> = MaybeUninit::uninit();
        for (index, (start, end)) in self.indices.iter().enumerate() {
            unsafe {
                (buses.as_ptr() as *mut BusMut).add(index).write(BusMut {
                    offset: self.offset,
                    len: self.len,
                    state: &self.states[index],
                    ptrs: &self.ptrs[*start..*end],
                    phantom: PhantomData,
                });
            }
        }

        Some(unsafe { buses.assume_init() })
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

pub struct Chunks<'a, 'b, 'c> {
    buffers: Buffers<'a, 'b, 'c>,
    chunk_size: usize,
}

impl<'a, 'b, 'c> Iterator for Chunks<'a, 'b, 'c> {
    type Item = Buffers<'a, 'b, 'c>;

    #[inline]
    fn next(&mut self) -> Option<Buffers<'a, 'b, 'c>> {
        if self.buffers.len == 0 {
            return None;
        }

        let chunk_size = self.chunk_size.min(self.buffers.len);

        let chunk = Buffers {
            offset: self.buffers.offset,
            len: chunk_size,
            input_states: self.buffers.input_states,
            input_indices: self.buffers.input_indices,
            input_ptrs: self.buffers.input_ptrs,
            output_states: self.buffers.output_states,
            output_indices: self.buffers.output_indices,
            output_ptrs: self.buffers.output_ptrs,
            phantom: PhantomData,
        };

        self.buffers.offset += chunk_size;
        self.buffers.len -= chunk_size;

        Some(chunk)
    }
}

pub struct SplitAtEvents<'a, 'b, 'c, 'e> {
    buffers: Buffers<'a, 'b, 'c>,
    events: &'e [Event],
    offset: usize,
}

impl<'a, 'b, 'c, 'e> Iterator for SplitAtEvents<'a, 'b, 'c, 'e> {
    type Item = (Buffers<'a, 'b, 'c>, &'e [Event]);

    #[inline]
    fn next(&mut self) -> Option<(Buffers<'a, 'b, 'c>, &'e [Event])> {
        if self.buffers.len == 0 && self.events.is_empty() {
            return None;
        }

        let mut event_count = 0;
        let mut chunk_size = self.buffers.len;
        for event in self.events {
            if event.offset > self.offset {
                chunk_size = (event.offset - self.offset).min(self.buffers.len);
                break;
            }

            event_count += 1;
        }

        let (events, rest) = self.events.split_at(event_count);
        self.events = rest;

        let chunk = Buffers {
            offset: self.buffers.offset,
            len: chunk_size,
            input_states: self.buffers.input_states,
            input_indices: self.buffers.input_indices,
            input_ptrs: self.buffers.input_ptrs,
            output_states: self.buffers.output_states,
            output_indices: self.buffers.output_indices,
            output_ptrs: self.buffers.output_ptrs,
            phantom: PhantomData,
        };

        self.buffers.offset += chunk_size;
        self.buffers.len -= chunk_size;

        self.offset += chunk_size;

        Some((chunk, events))
    }
}

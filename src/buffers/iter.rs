use std::marker::PhantomData;

use super::{
    Buffer, BufferData, BufferMut, BufferView, Buffers, Offset, Sample, SampleMut, Samples,
};
use crate::events::Events;

pub struct SamplesIter<'a, 'b> {
    buffers: &'a [BufferData],
    ptrs: &'a [*mut f32],
    offset: isize,
    end: isize,
    _marker: PhantomData<&'b mut f32>,
}

impl<'a, 'b> SamplesIter<'a, 'b> {
    pub(crate) fn new(buffers: Buffers<'a, 'b>) -> SamplesIter<'a, 'b> {
        SamplesIter {
            buffers: buffers.buffers,
            ptrs: buffers.ptrs,
            offset: buffers.offset,
            end: buffers.offset + buffers.len as isize,
            _marker: buffers._marker,
        }
    }
}

impl<'a, 'b> Iterator for SamplesIter<'a, 'b> {
    type Item = Samples<'a, 'b>;

    #[inline]
    fn next(&mut self) -> Option<Samples<'a, 'b>> {
        if self.offset < self.end {
            let offset = self.offset;
            self.offset += 1;

            unsafe { Some(Samples::from_raw_parts(self.buffers, self.ptrs, offset)) }
        } else {
            None
        }
    }
}

pub struct SampleIter<'a, 'b> {
    ptrs: &'a [*mut f32],
    offset: isize,
    end: isize,
    _marker: PhantomData<&'b f32>,
}

impl<'a, 'b> SampleIter<'a, 'b> {
    pub(crate) fn new(buffer: Buffer<'a, 'b>) -> SampleIter<'a, 'b> {
        SampleIter {
            ptrs: buffer.ptrs,
            offset: buffer.offset,
            end: buffer.offset + buffer.len as isize,
            _marker: buffer._marker,
        }
    }
}

impl<'a, 'b> Iterator for SampleIter<'a, 'b> {
    type Item = Sample<'a, 'b>;

    #[inline]
    fn next(&mut self) -> Option<Sample<'a, 'b>> {
        if self.offset < self.end {
            let offset = self.offset;
            self.offset += 1;

            unsafe { Some(Sample::from_raw_parts(self.ptrs, offset)) }
        } else {
            None
        }
    }
}

pub struct SampleIterMut<'a, 'b> {
    ptrs: &'a [*mut f32],
    offset: isize,
    end: isize,
    _marker: PhantomData<&'b mut f32>,
}

impl<'a, 'b> SampleIterMut<'a, 'b> {
    pub(crate) fn new(buffer: BufferMut<'a, 'b>) -> SampleIterMut<'a, 'b> {
        SampleIterMut {
            ptrs: buffer.ptrs,
            offset: buffer.offset,
            end: buffer.offset + buffer.len as isize,
            _marker: buffer._marker,
        }
    }
}

impl<'a, 'b> Iterator for SampleIterMut<'a, 'b> {
    type Item = SampleMut<'a, 'b>;

    #[inline]
    fn next(&mut self) -> Option<SampleMut<'a, 'b>> {
        if self.offset < self.end {
            let offset = self.offset;
            self.offset += 1;

            unsafe { Some(SampleMut::from_raw_parts(self.ptrs, offset)) }
        } else {
            None
        }
    }
}

pub struct SplitAtEvents<'e, B: BufferView> {
    raw: B::Raw,
    len: usize,
    events: Events<'e>,
    time: i64,
    _marker: PhantomData<B>,
}

impl<'e, B: BufferView> SplitAtEvents<'e, B> {
    #[inline]
    pub(crate) fn new(buffer: B, events: Events<'e>) -> SplitAtEvents<'e, B> {
        let (raw, len) = buffer.into_raw_parts();

        SplitAtEvents {
            raw,
            len,
            events,
            time: 0,
            _marker: PhantomData,
        }
    }
}

impl<'e, B: BufferView> Iterator for SplitAtEvents<'e, B> {
    type Item = (B, Events<'e>);

    #[inline]
    fn next(&mut self) -> Option<(B, Events<'e>)> {
        if self.len == 0 {
            if self.events.len() == 0 {
                return None;
            }

            // If we've reached the end of the buffer, yield all remaining events in one go:
            let buffers = unsafe { B::from_raw_parts(self.raw, 0) };

            let events = self.events;
            self.events = Events::new(&[]);

            return Some((buffers, events));
        }

        // Find the first event with a timestamp greater than the current one:
        let mut event_count = 0;
        let mut split = self.len;
        for event in self.events {
            if event.time > self.time {
                let offset = (event.time - self.time) as usize;
                if offset < self.len {
                    split = offset;
                }

                self.time = event.time;

                break;
            }

            event_count += 1;
        }

        let buffer = unsafe { B::from_raw_parts(self.raw, split) };
        self.raw = unsafe { self.raw.offset(split as isize) };
        self.len -= split;

        let events = self.events.slice(..event_count).unwrap();
        self.events = self.events.slice(event_count..).unwrap();

        Some((buffer, events))
    }
}

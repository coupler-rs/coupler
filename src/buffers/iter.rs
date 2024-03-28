use std::marker::PhantomData;

use super::{Buffer, BufferData, BufferMut, Buffers, Sample, SampleMut, Samples};
use crate::events::Events;

pub trait IntoSamples {
    type Sample;
    type SampleIter: Iterator<Item = Self::Sample>;

    fn into_samples(self) -> Self::SampleIter;
}

impl<'a, 'b> IntoSamples for Buffers<'a, 'b> {
    type Sample = Samples<'a, 'b>;
    type SampleIter = SamplesIter<'a, 'b>;

    #[inline]
    fn into_samples(self) -> Self::SampleIter {
        SamplesIter::new(self)
    }
}

pub struct SamplesIter<'a, 'b> {
    buffers: &'a [BufferData],
    ptrs: &'a [*mut f32],
    offset: isize,
    end: isize,
    _marker: PhantomData<&'b mut f32>,
}

impl<'a, 'b> SamplesIter<'a, 'b> {
    fn new(buffers: Buffers<'a, 'b>) -> SamplesIter<'a, 'b> {
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

impl<'a, 'b> IntoSamples for Buffer<'a, 'b> {
    type Sample = Sample<'a, 'b>;
    type SampleIter = SampleIter<'a, 'b>;

    #[inline]
    fn into_samples(self) -> Self::SampleIter {
        SampleIter::new(self)
    }
}

pub struct SampleIter<'a, 'b> {
    ptrs: &'a [*mut f32],
    offset: isize,
    end: isize,
    _marker: PhantomData<&'b f32>,
}

impl<'a, 'b> SampleIter<'a, 'b> {
    fn new(buffer: Buffer<'a, 'b>) -> SampleIter<'a, 'b> {
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

impl<'a, 'b> IntoSamples for BufferMut<'a, 'b> {
    type Sample = SampleMut<'a, 'b>;
    type SampleIter = SampleIterMut<'a, 'b>;

    #[inline]
    fn into_samples(self) -> Self::SampleIter {
        SampleIterMut::new(self)
    }
}

pub struct SampleIterMut<'a, 'b> {
    ptrs: &'a [*mut f32],
    offset: isize,
    end: isize,
    _marker: PhantomData<&'b mut f32>,
}

impl<'a, 'b> SampleIterMut<'a, 'b> {
    fn new(buffer: BufferMut<'a, 'b>) -> SampleIterMut<'a, 'b> {
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

pub trait IntoBlocks {
    type Block;
    type BlockIter: BlockIterator<Block = Self::Block>;

    fn into_blocks(self) -> Self::BlockIter;
}

pub trait BlockIterator {
    type Block;

    fn len(&self) -> usize;
    fn next_block(&mut self, len: usize) -> Self::Block;

    #[inline]
    fn split_at_events(self, events: Events) -> SplitAtEvents<Self>
    where
        Self: Sized,
    {
        SplitAtEvents::new(self, events)
    }
}

impl<'a, 'b> IntoBlocks for Buffers<'a, 'b> {
    type Block = Buffers<'a, 'b>;
    type BlockIter = BlocksIter<'a, 'b>;

    #[inline]
    fn into_blocks(self) -> Self::BlockIter {
        BlocksIter::new(self)
    }
}

pub struct BlocksIter<'a, 'b> {
    buffers: &'a [BufferData],
    ptrs: &'a [*mut f32],
    offset: isize,
    end: isize,
    _marker: PhantomData<&'b mut f32>,
}

impl<'a, 'b> BlocksIter<'a, 'b> {
    fn new(buffers: Buffers<'a, 'b>) -> BlocksIter<'a, 'b> {
        BlocksIter {
            buffers: buffers.buffers,
            ptrs: buffers.ptrs,
            offset: buffers.offset,
            end: buffers.offset + buffers.len as isize,
            _marker: buffers._marker,
        }
    }
}

impl<'a, 'b> BlockIterator for BlocksIter<'a, 'b> {
    type Block = Buffers<'a, 'b>;

    #[inline]
    fn len(&self) -> usize {
        (self.end - self.offset) as usize
    }

    #[inline]
    fn next_block(&mut self, len: usize) -> Self::Block {
        let remainder = self.len();
        let len = if len > remainder { remainder } else { len };

        let offset = self.offset;
        self.offset += len as isize;

        unsafe { Buffers::from_raw_parts(self.buffers, self.ptrs, offset, len) }
    }
}

impl<'a, 'b> IntoBlocks for Buffer<'a, 'b> {
    type Block = Buffer<'a, 'b>;
    type BlockIter = BlockIter<'a, 'b>;

    #[inline]
    fn into_blocks(self) -> Self::BlockIter {
        BlockIter::new(self)
    }
}

pub struct BlockIter<'a, 'b> {
    ptrs: &'a [*mut f32],
    offset: isize,
    end: isize,
    _marker: PhantomData<&'b f32>,
}

impl<'a, 'b> BlockIter<'a, 'b> {
    fn new(buffer: Buffer<'a, 'b>) -> BlockIter<'a, 'b> {
        BlockIter {
            ptrs: buffer.ptrs,
            offset: buffer.offset,
            end: buffer.offset + buffer.len as isize,
            _marker: buffer._marker,
        }
    }
}

impl<'a, 'b> BlockIterator for BlockIter<'a, 'b> {
    type Block = Buffer<'a, 'b>;

    #[inline]
    fn len(&self) -> usize {
        (self.end - self.offset) as usize
    }

    #[inline]
    fn next_block(&mut self, len: usize) -> Self::Block {
        let remainder = self.len();
        let len = if len > remainder { remainder } else { len };

        let offset = self.offset;
        self.offset += len as isize;

        unsafe { Buffer::from_raw_parts(self.ptrs, offset, len) }
    }
}

impl<'a, 'b> IntoBlocks for BufferMut<'a, 'b> {
    type Block = BufferMut<'a, 'b>;
    type BlockIter = BlockIterMut<'a, 'b>;

    #[inline]
    fn into_blocks(self) -> Self::BlockIter {
        BlockIterMut::new(self)
    }
}

pub struct BlockIterMut<'a, 'b> {
    ptrs: &'a [*mut f32],
    offset: isize,
    end: isize,
    _marker: PhantomData<&'b mut f32>,
}

impl<'a, 'b> BlockIterMut<'a, 'b> {
    fn new(buffer: BufferMut<'a, 'b>) -> BlockIterMut<'a, 'b> {
        BlockIterMut {
            ptrs: buffer.ptrs,
            offset: buffer.offset,
            end: buffer.offset + buffer.len as isize,
            _marker: buffer._marker,
        }
    }
}

impl<'a, 'b> BlockIterator for BlockIterMut<'a, 'b> {
    type Block = BufferMut<'a, 'b>;

    #[inline]
    fn len(&self) -> usize {
        (self.end - self.offset) as usize
    }

    #[inline]
    fn next_block(&mut self, len: usize) -> Self::Block {
        let remainder = self.len();
        let len = if len > remainder { remainder } else { len };

        let offset = self.offset;
        self.offset += len as isize;

        unsafe { BufferMut::from_raw_parts(self.ptrs, offset, len) }
    }
}

pub struct SplitAtEvents<'e, B> {
    blocks: B,
    events: Events<'e>,
    time: i64,
}

impl<'e, B> SplitAtEvents<'e, B> {
    fn new(blocks: B, events: Events<'e>) -> SplitAtEvents<'e, B> {
        SplitAtEvents {
            blocks,
            events,
            time: 0,
        }
    }
}

impl<'e, B: BlockIterator> Iterator for SplitAtEvents<'e, B> {
    type Item = (B::Block, Events<'e>);

    #[inline]
    fn next(&mut self) -> Option<(B::Block, Events<'e>)> {
        let len = self.blocks.len();

        if len == 0 {
            if self.events.len() == 0 {
                return None;
            }

            // If we've reached the end of the buffer, yield all remaining events in one go:
            let buffers = self.blocks.next_block(0);

            let events = self.events;
            self.events = Events::new(&[]);

            return Some((buffers, events));
        }

        // Find the first event with a timestamp greater than the current one:
        let mut event_count = 0;
        let mut split = len;
        for event in self.events {
            if event.time > self.time {
                let offset = (event.time - self.time) as usize;
                if offset < len {
                    split = offset;
                }

                self.time = event.time;

                break;
            }

            event_count += 1;
        }

        let buffer = self.blocks.next_block(split);

        let events = self.events.slice(..event_count).unwrap();
        self.events = self.events.slice(event_count..).unwrap();

        Some((buffer, events))
    }
}

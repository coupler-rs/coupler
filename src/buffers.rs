use std::marker::PhantomData;
use std::ops::{Index, IndexMut, Range};
use std::{array, slice};

pub mod collect;
pub mod iter;

use crate::events::Events;
use collect::FromBuffers;
use iter::{BlockIterator, IntoBlocks, IntoSamples};

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum BufferType {
    Const,
    Mut,
}

pub enum AnyBuffer<'a, 'b> {
    Const(Buffer<'a, 'b>),
    Mut(BufferMut<'a, 'b>),
}

impl<'a, 'b> AnyBuffer<'a, 'b> {
    #[inline]
    pub unsafe fn from_raw_parts(
        buffer_type: BufferType,
        ptrs: &'a [*mut f32],
        offset: isize,
        len: usize,
    ) -> AnyBuffer<'a, 'b> {
        match buffer_type {
            BufferType::Const => AnyBuffer::Const(Buffer::from_raw_parts(ptrs, offset, len)),
            BufferType::Mut => AnyBuffer::Mut(BufferMut::from_raw_parts(ptrs, offset, len)),
        }
    }
}

pub struct BufferData {
    pub buffer_type: BufferType,
    pub start: usize,
    pub end: usize,
}

pub struct Buffers<'a, 'b> {
    buffers: &'a [BufferData],
    ptrs: &'a [*mut f32],
    offset: isize,
    len: usize,
    _marker: PhantomData<&'b mut f32>,
}

impl<'a, 'b> Buffers<'a, 'b> {
    #[inline]
    pub unsafe fn from_raw_parts(
        buffers: &'a [BufferData],
        ptrs: &'a [*mut f32],
        offset: isize,
        len: usize,
    ) -> Buffers<'a, 'b> {
        Buffers {
            buffers,
            ptrs,
            offset,
            len,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn buffer_count(&self) -> usize {
        self.buffers.len()
    }

    #[inline]
    pub fn reborrow<'c>(&'c mut self) -> Buffers<'a, 'c> {
        Buffers {
            buffers: self.buffers,
            ptrs: self.ptrs,
            offset: self.offset,
            len: self.len,
            _marker: self._marker,
        }
    }

    #[inline]
    pub fn get(&mut self, index: usize) -> Option<AnyBuffer> {
        if let Some(buffer) = self.buffers.get(index) {
            unsafe {
                Some(AnyBuffer::from_raw_parts(
                    buffer.buffer_type,
                    &self.ptrs[buffer.start..buffer.end],
                    self.offset,
                    self.len,
                ))
            }
        } else {
            None
        }
    }

    #[inline]
    pub fn collect<B: FromBuffers<'a, 'b>>(self) -> Option<B> {
        let mut iter = self.into_iter();

        let result = B::bind(&mut iter)?;

        if iter.next().is_none() {
            Some(result)
        } else {
            None
        }
    }

    #[inline]
    pub fn slice(&mut self, range: Range<usize>) -> Option<Buffers> {
        if range.start > range.end || range.end > self.len {
            None
        } else {
            Some(Buffers {
                buffers: self.buffers,
                ptrs: self.ptrs,
                offset: self.offset + range.start as isize,
                len: range.end - range.start,
                _marker: self._marker,
            })
        }
    }

    #[inline]
    pub fn samples<'c>(&'c mut self) -> iter::SamplesIter<'a, 'c> {
        self.reborrow().into_samples()
    }

    #[inline]
    pub fn split_at_events<'c, 'e>(
        &'c mut self,
        events: Events<'e>,
    ) -> iter::SplitAtEvents<'e, iter::BlocksIter<'a, 'c>> {
        self.reborrow().into_blocks().split_at_events(events)
    }
}

impl<'a, 'b> IntoIterator for Buffers<'a, 'b> {
    type Item = AnyBuffer<'a, 'b>;
    type IntoIter = BufferIter<'a, 'b>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        BufferIter {
            iter: self.buffers.iter(),
            ptrs: self.ptrs,
            offset: self.offset,
            len: self.len,
            _marker: PhantomData,
        }
    }
}

pub struct BufferIter<'a, 'b> {
    iter: slice::Iter<'a, BufferData>,
    ptrs: &'a [*mut f32],
    offset: isize,
    len: usize,
    _marker: PhantomData<&'b mut f32>,
}

impl<'a, 'b> Iterator for BufferIter<'a, 'b> {
    type Item = AnyBuffer<'a, 'b>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(buffer) = self.iter.next() {
            unsafe {
                Some(AnyBuffer::from_raw_parts(
                    buffer.buffer_type,
                    &self.ptrs[buffer.start..buffer.end],
                    self.offset,
                    self.len,
                ))
            }
        } else {
            None
        }
    }
}

pub enum AnySample<'a, 'b> {
    Const(Sample<'a, 'b>),
    Mut(SampleMut<'a, 'b>),
}

impl<'a, 'b> AnySample<'a, 'b> {
    #[inline]
    pub unsafe fn from_raw_parts(
        buffer_type: BufferType,
        ptrs: &'a [*mut f32],
        offset: isize,
    ) -> AnySample<'a, 'b> {
        match buffer_type {
            BufferType::Const => AnySample::Const(Sample::from_raw_parts(ptrs, offset)),
            BufferType::Mut => AnySample::Mut(SampleMut::from_raw_parts(ptrs, offset)),
        }
    }
}

pub struct Samples<'a, 'b> {
    buffers: &'a [BufferData],
    ptrs: &'a [*mut f32],
    offset: isize,
    _marker: PhantomData<&'b mut f32>,
}

impl<'a, 'b> Samples<'a, 'b> {
    #[inline]
    pub unsafe fn from_raw_parts(
        buffers: &'a [BufferData],
        ptrs: &'a [*mut f32],
        offset: isize,
    ) -> Samples<'a, 'b> {
        Samples {
            buffers,
            ptrs,
            offset,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn buffer_count(&self) -> usize {
        self.buffers.len()
    }

    #[inline]
    pub fn get(&mut self, index: usize) -> Option<AnySample> {
        if let Some(buffer) = self.buffers.get(index) {
            unsafe {
                Some(AnySample::from_raw_parts(
                    buffer.buffer_type,
                    &self.ptrs[buffer.start..buffer.end],
                    self.offset,
                ))
            }
        } else {
            None
        }
    }
}

impl<'a, 'b> IntoIterator for Samples<'a, 'b> {
    type Item = AnySample<'a, 'b>;
    type IntoIter = BufferSampleIter<'a, 'b>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        BufferSampleIter {
            iter: self.buffers.iter(),
            ptrs: self.ptrs,
            offset: self.offset,
            _marker: PhantomData,
        }
    }
}

pub struct BufferSampleIter<'a, 'b> {
    iter: slice::Iter<'a, BufferData>,
    ptrs: &'a [*mut f32],
    offset: isize,
    _marker: PhantomData<&'b mut f32>,
}

impl<'a, 'b> Iterator for BufferSampleIter<'a, 'b> {
    type Item = AnySample<'a, 'b>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(buffer) = self.iter.next() {
            unsafe {
                Some(AnySample::from_raw_parts(
                    buffer.buffer_type,
                    &self.ptrs[buffer.start..buffer.end],
                    self.offset,
                ))
            }
        } else {
            None
        }
    }
}

#[derive(Copy, Clone)]
pub struct Buffer<'a, 'b> {
    ptrs: &'a [*mut f32],
    offset: isize,
    len: usize,
    _marker: PhantomData<&'b f32>,
}

impl<'a, 'b> Buffer<'a, 'b> {
    #[inline]
    pub unsafe fn from_raw_parts(
        ptrs: &'a [*mut f32],
        offset: isize,
        len: usize,
    ) -> Buffer<'a, 'b> {
        Buffer {
            ptrs,
            offset,
            len,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn channel_count(&self) -> usize {
        self.ptrs.len()
    }

    #[inline]
    pub fn collect<const N: usize>(self) -> Option<[&'b [f32]; N]> {
        if self.channel_count() != N {
            return None;
        }

        Some(array::from_fn(|i| unsafe {
            slice::from_raw_parts(self.ptrs[i].offset(self.offset), self.len)
        }))
    }

    #[inline]
    pub fn samples(&self) -> iter::SampleIter<'a, 'b> {
        self.into_samples()
    }

    #[inline]
    pub fn split_at_events<'e>(
        &self,
        events: Events<'e>,
    ) -> iter::SplitAtEvents<'e, iter::BlockIter<'a, 'b>> {
        self.into_blocks().split_at_events(events)
    }
}

impl<'a, 'b> Index<usize> for Buffer<'a, 'b> {
    type Output = [f32];

    #[inline]
    fn index(&self, index: usize) -> &[f32] {
        unsafe { slice::from_raw_parts(self.ptrs[index].offset(self.offset), self.len) }
    }
}

impl<'a, 'b> IntoIterator for Buffer<'a, 'b> {
    type Item = &'b [f32];
    type IntoIter = Channels<'a, 'b>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Channels {
            iter: self.ptrs.iter(),
            offset: self.offset,
            len: self.len,
            _marker: PhantomData,
        }
    }
}

pub struct Channels<'a, 'b> {
    iter: slice::Iter<'a, *mut f32>,
    offset: isize,
    len: usize,
    _marker: PhantomData<&'b f32>,
}

impl<'a, 'b> Iterator for Channels<'a, 'b> {
    type Item = &'b [f32];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ptr) = self.iter.next() {
            unsafe { Some(slice::from_raw_parts(ptr.offset(self.offset), self.len)) }
        } else {
            None
        }
    }
}

pub struct Sample<'a, 'b> {
    ptrs: &'a [*mut f32],
    offset: isize,
    _marker: PhantomData<&'b f32>,
}

impl<'a, 'b> Sample<'a, 'b> {
    #[inline]
    pub unsafe fn from_raw_parts(ptrs: &'a [*mut f32], offset: isize) -> Sample<'a, 'b> {
        Sample {
            ptrs,
            offset,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn channel_count(&self) -> usize {
        self.ptrs.len()
    }
}

impl<'a, 'b> Index<usize> for Sample<'a, 'b> {
    type Output = f32;

    #[inline]
    fn index(&self, index: usize) -> &f32 {
        unsafe { &*self.ptrs[index].offset(self.offset) }
    }
}

impl<'a, 'b> IntoIterator for Sample<'a, 'b> {
    type Item = &'b f32;
    type IntoIter = SampleChannels<'a, 'b>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        SampleChannels {
            iter: self.ptrs.iter(),
            offset: self.offset,
            _marker: PhantomData,
        }
    }
}

pub struct SampleChannels<'a, 'b> {
    iter: slice::Iter<'a, *mut f32>,
    offset: isize,
    _marker: PhantomData<&'b f32>,
}

impl<'a, 'b> Iterator for SampleChannels<'a, 'b> {
    type Item = &'b f32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ptr) = self.iter.next() {
            unsafe { Some(&*ptr.offset(self.offset)) }
        } else {
            None
        }
    }
}

pub struct BufferMut<'a, 'b> {
    ptrs: &'a [*mut f32],
    offset: isize,
    len: usize,
    _marker: PhantomData<&'b mut f32>,
}

impl<'a, 'b> BufferMut<'a, 'b> {
    #[inline]
    pub unsafe fn from_raw_parts(
        ptrs: &'a [*mut f32],
        offset: isize,
        len: usize,
    ) -> BufferMut<'a, 'b> {
        BufferMut {
            ptrs,
            offset,
            len,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn channel_count(&self) -> usize {
        self.ptrs.len()
    }

    #[inline]
    pub fn reborrow<'c>(&'c mut self) -> BufferMut<'a, 'c> {
        BufferMut {
            ptrs: self.ptrs,
            offset: self.offset,
            len: self.len,
            _marker: self._marker,
        }
    }

    #[inline]
    pub fn collect<const N: usize>(self) -> Option<[&'b mut [f32]; N]> {
        if self.channel_count() != N {
            return None;
        }

        Some(array::from_fn(|i| unsafe {
            slice::from_raw_parts_mut(self.ptrs[i].offset(self.offset), self.len)
        }))
    }

    #[inline]
    pub fn samples<'c>(&'c mut self) -> iter::SampleIterMut<'a, 'c> {
        self.reborrow().into_samples()
    }

    #[inline]
    pub fn split_at_events<'c, 'e>(
        &'c mut self,
        events: Events<'e>,
    ) -> iter::SplitAtEvents<'e, iter::BlockIterMut<'a, 'c>> {
        self.reborrow().into_blocks().split_at_events(events)
    }
}

impl<'a, 'b> Index<usize> for BufferMut<'a, 'b> {
    type Output = [f32];

    #[inline]
    fn index(&self, index: usize) -> &[f32] {
        unsafe { slice::from_raw_parts(self.ptrs[index].offset(self.offset), self.len) }
    }
}

impl<'a, 'b> IndexMut<usize> for BufferMut<'a, 'b> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut [f32] {
        unsafe { slice::from_raw_parts_mut(self.ptrs[index].offset(self.offset), self.len) }
    }
}

impl<'a, 'b> IntoIterator for BufferMut<'a, 'b> {
    type Item = &'b mut [f32];
    type IntoIter = ChannelsMut<'a, 'b>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ChannelsMut {
            iter: self.ptrs.iter(),
            offset: self.offset,
            len: self.len,
            _marker: PhantomData,
        }
    }
}

pub struct ChannelsMut<'a, 'b> {
    iter: slice::Iter<'a, *mut f32>,
    offset: isize,
    len: usize,
    _marker: PhantomData<&'b mut f32>,
}

impl<'a, 'b> Iterator for ChannelsMut<'a, 'b> {
    type Item = &'b mut [f32];

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ptr) = self.iter.next() {
            unsafe { Some(slice::from_raw_parts_mut(ptr.offset(self.offset), self.len)) }
        } else {
            None
        }
    }
}

pub struct SampleMut<'a, 'b> {
    ptrs: &'a [*mut f32],
    offset: isize,
    _marker: PhantomData<&'b mut f32>,
}

impl<'a, 'b> SampleMut<'a, 'b> {
    #[inline]
    pub unsafe fn from_raw_parts(ptrs: &'a [*mut f32], offset: isize) -> SampleMut<'a, 'b> {
        SampleMut {
            ptrs,
            offset,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn channel_count(&self) -> usize {
        self.ptrs.len()
    }
}

impl<'a, 'b> Index<usize> for SampleMut<'a, 'b> {
    type Output = f32;

    #[inline]
    fn index(&self, index: usize) -> &f32 {
        unsafe { &*self.ptrs[index].offset(self.offset) }
    }
}

impl<'a, 'b> IndexMut<usize> for SampleMut<'a, 'b> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut f32 {
        unsafe { &mut *self.ptrs[index].offset(self.offset) }
    }
}

impl<'a, 'b> IntoIterator for SampleMut<'a, 'b> {
    type Item = &'b mut f32;
    type IntoIter = SampleChannelsMut<'a, 'b>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        SampleChannelsMut {
            iter: self.ptrs.iter(),
            offset: self.offset,
            _marker: PhantomData,
        }
    }
}

pub struct SampleChannelsMut<'a, 'b> {
    iter: slice::Iter<'a, *mut f32>,
    offset: isize,
    _marker: PhantomData<&'b mut f32>,
}

impl<'a, 'b> Iterator for SampleChannelsMut<'a, 'b> {
    type Item = &'b mut f32;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ptr) = self.iter.next() {
            unsafe { Some(&mut *ptr.offset(self.offset)) }
        } else {
            None
        }
    }
}

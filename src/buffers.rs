use std::marker::PhantomData;
use std::ops::{Index, IndexMut, Range};
use std::{array, slice};

pub mod collect;
mod buffer_view;
pub mod iter;

pub use buffer_view::{BufferView, Offset, SampleView};

use collect::FromBuffers;

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
        raw: RawBuffer<'a>,
        len: usize,
    ) -> AnyBuffer<'a, 'b> {
        match buffer_type {
            BufferType::Const => AnyBuffer::Const(Buffer::from_raw_parts(raw, len)),
            BufferType::Mut => AnyBuffer::Mut(BufferMut::from_raw_parts(raw, len)),
        }
    }
}

pub struct BufferData {
    pub buffer_type: BufferType,
    pub start: usize,
    pub end: usize,
}

#[derive(Copy, Clone)]
pub struct RawBuffers<'a> {
    pub buffers: &'a [BufferData],
    pub ptrs: &'a [*mut f32],
    pub offset: isize,
}

pub struct Buffers<'a, 'b> {
    raw: RawBuffers<'a>,
    len: usize,
    _marker: PhantomData<&'b mut f32>,
}

impl<'a, 'b> Buffers<'a, 'b> {
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn buffer_count(&self) -> usize {
        self.raw.buffers.len()
    }

    #[inline]
    pub fn reborrow<'c>(&'c mut self) -> Buffers<'a, 'c> {
        Buffers {
            raw: self.raw,
            len: self.len,
            _marker: self._marker,
        }
    }

    #[inline]
    pub fn get(&mut self, index: usize) -> Option<AnyBuffer> {
        if let Some(buffer) = self.raw.buffers.get(index) {
            unsafe {
                Some(AnyBuffer::from_raw_parts(
                    buffer.buffer_type,
                    RawBuffer {
                        ptrs: &self.raw.ptrs[buffer.start..buffer.end],
                        offset: self.raw.offset,
                    },
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
                raw: RawBuffers {
                    buffers: self.raw.buffers,
                    ptrs: self.raw.ptrs,
                    offset: self.raw.offset + range.start as isize,
                },
                len: range.end - range.start,
                _marker: self._marker,
            })
        }
    }
}

impl<'a, 'b> IntoIterator for Buffers<'a, 'b> {
    type Item = AnyBuffer<'a, 'b>;
    type IntoIter = BufferIter<'a, 'b>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        BufferIter {
            iter: self.raw.buffers.into_iter(),
            ptrs: self.raw.ptrs,
            offset: self.raw.offset,
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
                    RawBuffer {
                        ptrs: &self.ptrs[buffer.start..buffer.end],
                        offset: self.offset,
                    },
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
    pub unsafe fn from_raw(buffer_type: BufferType, raw: RawBuffer<'a>) -> AnySample<'a, 'b> {
        match buffer_type {
            BufferType::Const => AnySample::Const(Sample::from_raw(raw)),
            BufferType::Mut => AnySample::Mut(SampleMut::from_raw(raw)),
        }
    }
}

pub struct BufferSamples<'a, 'b> {
    raw: RawBuffers<'a>,
    _marker: PhantomData<&'b mut f32>,
}

impl<'a, 'b> BufferSamples<'a, 'b> {
    #[inline]
    pub fn buffer_count(&self) -> usize {
        self.raw.buffers.len()
    }

    #[inline]
    pub fn get(&mut self, index: usize) -> Option<AnySample> {
        if let Some(buffer) = self.raw.buffers.get(index) {
            unsafe {
                Some(AnySample::from_raw(
                    buffer.buffer_type,
                    RawBuffer {
                        ptrs: &self.raw.ptrs[buffer.start..buffer.end],
                        offset: self.raw.offset,
                    },
                ))
            }
        } else {
            None
        }
    }
}

impl<'a, 'b> IntoIterator for BufferSamples<'a, 'b> {
    type Item = AnySample<'a, 'b>;
    type IntoIter = BufferSampleIter<'a, 'b>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        BufferSampleIter {
            iter: self.raw.buffers.into_iter(),
            ptrs: self.raw.ptrs,
            offset: self.raw.offset,
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
                Some(AnySample::from_raw(
                    buffer.buffer_type,
                    RawBuffer {
                        ptrs: &self.ptrs[buffer.start..buffer.end],
                        offset: self.offset,
                    },
                ))
            }
        } else {
            None
        }
    }
}

#[derive(Copy, Clone)]
pub struct RawBuffer<'a> {
    pub ptrs: &'a [*mut f32],
    pub offset: isize,
}

#[derive(Copy, Clone)]
pub struct Buffer<'a, 'b> {
    raw: RawBuffer<'a>,
    len: usize,
    _marker: PhantomData<&'b f32>,
}

impl<'a, 'b> Buffer<'a, 'b> {
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn channel_count(&self) -> usize {
        self.raw.ptrs.len()
    }

    #[inline]
    pub fn collect<const N: usize>(self) -> Option<[&'b [f32]; N]> {
        if self.channel_count() != N {
            return None;
        }

        Some(array::from_fn(|i| unsafe {
            slice::from_raw_parts(self.raw.ptrs[i].offset(self.raw.offset), self.len)
        }))
    }
}

impl<'a, 'b> Index<usize> for Buffer<'a, 'b> {
    type Output = [f32];

    #[inline]
    fn index(&self, index: usize) -> &[f32] {
        unsafe { slice::from_raw_parts(self.raw.ptrs[index].offset(self.raw.offset), self.len) }
    }
}

impl<'a, 'b> IntoIterator for Buffer<'a, 'b> {
    type Item = &'b [f32];
    type IntoIter = Channels<'a, 'b>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Channels {
            iter: self.raw.ptrs.into_iter(),
            offset: self.raw.offset,
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
    raw: RawBuffer<'a>,
    _marker: PhantomData<&'b f32>,
}

impl<'a, 'b> Sample<'a, 'b> {
    #[inline]
    pub fn channel_count(&self) -> usize {
        self.raw.ptrs.len()
    }
}

impl<'a, 'b> Index<usize> for Sample<'a, 'b> {
    type Output = f32;

    #[inline]
    fn index(&self, index: usize) -> &f32 {
        unsafe { &*self.raw.ptrs[index].offset(self.raw.offset) }
    }
}

impl<'a, 'b> IntoIterator for Sample<'a, 'b> {
    type Item = &'b f32;
    type IntoIter = SampleChannels<'a, 'b>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        SampleChannels {
            iter: self.raw.ptrs.into_iter(),
            offset: self.raw.offset,
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
    raw: RawBuffer<'a>,
    len: usize,
    _marker: PhantomData<&'b mut f32>,
}

impl<'a, 'b> BufferMut<'a, 'b> {
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn channel_count(&self) -> usize {
        self.raw.ptrs.len()
    }

    #[inline]
    pub fn reborrow<'c>(&'c mut self) -> BufferMut<'a, 'c> {
        BufferMut {
            raw: self.raw,
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
            slice::from_raw_parts_mut(self.raw.ptrs[i].offset(self.raw.offset), self.len)
        }))
    }
}

impl<'a, 'b> Index<usize> for BufferMut<'a, 'b> {
    type Output = [f32];

    #[inline]
    fn index(&self, index: usize) -> &[f32] {
        unsafe { slice::from_raw_parts(self.raw.ptrs[index].offset(self.raw.offset), self.len) }
    }
}

impl<'a, 'b> IndexMut<usize> for BufferMut<'a, 'b> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut [f32] {
        unsafe { slice::from_raw_parts_mut(self.raw.ptrs[index].offset(self.raw.offset), self.len) }
    }
}

impl<'a, 'b> IntoIterator for BufferMut<'a, 'b> {
    type Item = &'b mut [f32];
    type IntoIter = ChannelsMut<'a, 'b>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        ChannelsMut {
            iter: self.raw.ptrs.into_iter(),
            offset: self.raw.offset,
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
    raw: RawBuffer<'a>,
    _marker: PhantomData<&'b mut f32>,
}

impl<'a, 'b> SampleMut<'a, 'b> {
    #[inline]
    pub fn channel_count(&self) -> usize {
        self.raw.ptrs.len()
    }
}

impl<'a, 'b> Index<usize> for SampleMut<'a, 'b> {
    type Output = f32;

    #[inline]
    fn index(&self, index: usize) -> &f32 {
        unsafe { &*self.raw.ptrs[index].offset(self.raw.offset) }
    }
}

impl<'a, 'b> IndexMut<usize> for SampleMut<'a, 'b> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut f32 {
        unsafe { &mut *self.raw.ptrs[index].offset(self.raw.offset) }
    }
}

impl<'a, 'b> IntoIterator for SampleMut<'a, 'b> {
    type Item = &'b mut f32;
    type IntoIter = SampleChannelsMut<'a, 'b>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        SampleChannelsMut {
            iter: self.raw.ptrs.into_iter(),
            offset: self.raw.offset,
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

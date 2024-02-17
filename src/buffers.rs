use std::marker::PhantomData;
use std::ops::{Index, IndexMut, Range};
use std::slice;

pub mod bind;
pub mod iter;
mod sample_buffer;

pub use sample_buffer::{Offset, SampleBuffer};

use bind::{BindBuffers, BindBuffersError};

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
    pub fn bind<B: BindBuffers<'a, 'b>>(self) -> Result<B, BindBuffersError> {
        let mut iter = self.into_iter();

        let result = B::bind(&mut iter)?;

        if iter.next().is_none() {
            Ok(result)
        } else {
            Err(BindBuffersError(()))
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
    pub fn channel_count(&self) -> usize {
        self.raw.ptrs.len()
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

pub struct BufferMut<'a, 'b> {
    raw: RawBuffer<'a>,
    len: usize,
    _marker: PhantomData<&'b mut f32>,
}

impl<'a, 'b> BufferMut<'a, 'b> {
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

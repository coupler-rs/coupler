use std::marker::PhantomData;
use std::ops::{Index, IndexMut, Range};
use std::slice;

use crate::events::Events;

pub mod bind;
pub mod iter;

use bind::BindBuffers;
use iter::SplitAtEvents;

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
        offset: usize,
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
    offset: usize,
    len: usize,
    _marker: PhantomData<&'b mut f32>,
}

impl<'a, 'b> Buffers<'a, 'b> {
    #[inline]
    pub unsafe fn from_raw_parts(
        buffers: &'a [BufferData],
        ptrs: &'a [*mut f32],
        offset: usize,
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
    pub fn bind<B: BindBuffers<'a, 'b>>(self) -> Option<B> {
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
                offset: self.offset + range.start,
                len: range.end - range.start,
                _marker: self._marker,
            })
        }
    }

    #[inline]
    pub fn split_at_events<'e>(self, events: Events<'e>) -> SplitAtEvents<'a, 'b, 'e> {
        SplitAtEvents::new(self, events)
    }
}

impl<'a, 'b> IntoIterator for Buffers<'a, 'b> {
    type Item = AnyBuffer<'a, 'b>;
    type IntoIter = BufferIter<'a, 'b>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        BufferIter {
            iter: self.buffers.into_iter(),
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
    offset: usize,
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

pub struct Buffer<'a, 'b> {
    ptrs: &'a [*mut f32],
    offset: usize,
    len: usize,
    _marker: PhantomData<&'b f32>,
}

impl<'a, 'b> Buffer<'a, 'b> {
    #[inline]
    pub unsafe fn from_raw_parts(
        ptrs: &'a [*mut f32],
        offset: usize,
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
    pub fn channel_count(&self) -> usize {
        self.ptrs.len()
    }
}

impl<'a, 'b> Index<usize> for Buffer<'a, 'b> {
    type Output = [f32];

    #[inline]
    fn index(&self, index: usize) -> &[f32] {
        unsafe { slice::from_raw_parts(self.ptrs[index].add(self.offset), self.len) }
    }
}

pub struct BufferMut<'a, 'b> {
    ptrs: &'a [*mut f32],
    offset: usize,
    len: usize,
    _marker: PhantomData<&'b mut f32>,
}

impl<'a, 'b> BufferMut<'a, 'b> {
    #[inline]
    pub unsafe fn from_raw_parts(
        ptrs: &'a [*mut f32],
        offset: usize,
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
    pub fn channel_count(&self) -> usize {
        self.ptrs.len()
    }
}

impl<'a, 'b> Index<usize> for BufferMut<'a, 'b> {
    type Output = [f32];

    #[inline]
    fn index(&self, index: usize) -> &[f32] {
        unsafe { slice::from_raw_parts(self.ptrs[index].add(self.offset), self.len) }
    }
}

impl<'a, 'b> IndexMut<usize> for BufferMut<'a, 'b> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut [f32] {
        unsafe { slice::from_raw_parts_mut(self.ptrs[index].add(self.offset), self.len) }
    }
}

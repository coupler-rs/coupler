use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::slice;

pub struct Buffers {}

pub struct Inputs {}

pub struct Outputs {}

pub struct Buffer<'a, 'b> {
    ptrs: &'a [*const f32],
    offset: usize,
    len: usize,
    _marker: PhantomData<&'b f32>,
}

impl<'a, 'b> Buffer<'a, 'b> {
    #[inline]
    pub unsafe fn from_raw_parts(
        ptrs: &'a [*const f32],
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

    fn index(&self, index: usize) -> &[f32] {
        unsafe { slice::from_raw_parts(self.ptrs[index].add(self.offset), self.len) }
    }
}

pub struct BufferMut<'a, 'b> {
    ptrs: &'a [*mut f32],
    offset: usize,
    len: usize,
    _marker: PhantomData<&'b f32>,
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

    fn index(&self, index: usize) -> &[f32] {
        unsafe { slice::from_raw_parts(self.ptrs[index].add(self.offset), self.len) }
    }
}

impl<'a, 'b> IndexMut<usize> for BufferMut<'a, 'b> {
    fn index_mut(&mut self, index: usize) -> &mut [f32] {
        unsafe { slice::from_raw_parts_mut(self.ptrs[index].add(self.offset), self.len) }
    }
}

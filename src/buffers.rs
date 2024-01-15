use std::marker::PhantomData;
use std::ops::{Index, IndexMut, Range};
use std::slice;

use crate::bus::BusDir;

pub enum BufferDir<'a, 'b> {
    In(Buffer<'a, 'b>),
    Out(BufferMut<'a, 'b>),
    InOut(BufferMut<'a, 'b>),
}

impl<'a, 'b> BufferDir<'a, 'b> {
    #[inline]
    pub unsafe fn from_raw_parts(
        dir: BusDir,
        ptrs: &'a [*mut f32],
        offset: isize,
        len: usize,
    ) -> BufferDir<'a, 'b> {
        match dir {
            BusDir::In => BufferDir::In(Buffer::from_raw_parts(ptrs, offset, len)),
            BusDir::Out => BufferDir::Out(BufferMut::from_raw_parts(ptrs, offset, len)),
            BusDir::InOut => BufferDir::InOut(BufferMut::from_raw_parts(ptrs, offset, len)),
        }
    }
}

pub struct BusData {
    pub dir: BusDir,
    pub start: usize,
    pub end: usize,
}

pub struct Buffers<'a, 'b> {
    buses: &'a [BusData],
    ptrs: &'a [*mut f32],
    offset: isize,
    len: usize,
    _marker: PhantomData<&'b mut f32>,
}

impl<'a, 'b> Buffers<'a, 'b> {
    #[inline]
    pub unsafe fn from_raw_parts(
        buses: &'a [BusData],
        ptrs: &'a [*mut f32],
        offset: isize,
        len: usize,
    ) -> Buffers<'a, 'b> {
        Buffers {
            buses,
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
    pub fn bus_count(&self) -> usize {
        self.buses.len()
    }

    #[inline]
    pub fn get(&mut self, index: usize) -> Option<BufferDir> {
        if let Some(bus) = self.buses.get(index) {
            unsafe {
                Some(BufferDir::from_raw_parts(
                    bus.dir,
                    &self.ptrs[bus.start..bus.end],
                    self.offset,
                    self.len,
                ))
            }
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
                buses: self.buses,
                ptrs: self.ptrs,
                offset: self.offset.checked_add_unsigned(range.start).unwrap(),
                len: range.end - range.start,
                _marker: self._marker,
            })
        }
    }
}

impl<'a, 'b> IntoIterator for Buffers<'a, 'b> {
    type Item = BufferDir<'a, 'b>;
    type IntoIter = Buses<'a, 'b>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Buses {
            iter: self.buses.into_iter(),
            ptrs: self.ptrs,
            offset: self.offset,
            len: self.len,
            _marker: PhantomData,
        }
    }
}

pub struct Buses<'a, 'b> {
    iter: slice::Iter<'a, BusData>,
    ptrs: &'a [*mut f32],
    offset: isize,
    len: usize,
    _marker: PhantomData<&'b mut f32>,
}

impl<'a, 'b> Iterator for Buses<'a, 'b> {
    type Item = BufferDir<'a, 'b>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(bus) = self.iter.next() {
            unsafe {
                Some(BufferDir::from_raw_parts(
                    bus.dir,
                    &self.ptrs[bus.start..bus.end],
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
    pub fn channel_count(&self) -> usize {
        self.ptrs.len()
    }
}

impl<'a, 'b> Index<usize> for Buffer<'a, 'b> {
    type Output = [f32];

    #[inline]
    fn index(&self, index: usize) -> &[f32] {
        unsafe { slice::from_raw_parts(self.ptrs[index].offset(self.offset), self.len) }
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
    pub fn channel_count(&self) -> usize {
        self.ptrs.len()
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

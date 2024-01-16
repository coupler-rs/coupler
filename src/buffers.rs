use std::marker::PhantomData;
use std::ops::{Index, IndexMut, Range};
use std::slice;

use crate::bus::BusDir;

pub mod bind;

use bind::BindBuffers;

pub enum BufferDir<'a> {
    In(Buffer<'a>),
    Out(BufferMut<'a>),
    InOut(BufferMut<'a>),
}

impl<'a> BufferDir<'a> {
    #[inline]
    pub unsafe fn from_raw_parts(
        dir: BusDir,
        ptrs: &'a [*mut f32],
        offset: isize,
        len: usize,
    ) -> BufferDir<'a> {
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

pub struct Buffers<'a> {
    buses: &'a [BusData],
    ptrs: &'a [*mut f32],
    offset: isize,
    len: usize,
    _marker: PhantomData<&'a mut f32>,
}

impl<'a> Buffers<'a> {
    #[inline]
    pub unsafe fn from_raw_parts(
        buses: &'a [BusData],
        ptrs: &'a [*mut f32],
        offset: isize,
        len: usize,
    ) -> Buffers<'a> {
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
    pub fn reborrow(&mut self) -> Buffers {
        Buffers {
            buses: self.buses,
            ptrs: self.ptrs,
            offset: self.offset,
            len: self.len,
            _marker: self._marker,
        }
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
    pub fn bind<'b, B: BindBuffers<'b>>(&'b mut self) -> Option<B> {
        let mut iter = self.reborrow().into_iter();

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
                buses: self.buses,
                ptrs: self.ptrs,
                offset: self.offset.checked_add_unsigned(range.start).unwrap(),
                len: range.end - range.start,
                _marker: self._marker,
            })
        }
    }
}

impl<'a> IntoIterator for Buffers<'a> {
    type Item = BufferDir<'a>;
    type IntoIter = Buses<'a>;

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

pub struct Buses<'a> {
    iter: slice::Iter<'a, BusData>,
    ptrs: &'a [*mut f32],
    offset: isize,
    len: usize,
    _marker: PhantomData<&'a mut f32>,
}

impl<'a> Iterator for Buses<'a> {
    type Item = BufferDir<'a>;

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

pub struct Buffer<'a> {
    ptrs: &'a [*mut f32],
    offset: isize,
    len: usize,
    _marker: PhantomData<&'a f32>,
}

impl<'a> Buffer<'a> {
    #[inline]
    pub unsafe fn from_raw_parts(ptrs: &'a [*mut f32], offset: isize, len: usize) -> Buffer<'a> {
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

impl<'a> Index<usize> for Buffer<'a> {
    type Output = [f32];

    #[inline]
    fn index(&self, index: usize) -> &[f32] {
        unsafe { slice::from_raw_parts(self.ptrs[index].offset(self.offset), self.len) }
    }
}

pub struct BufferMut<'a> {
    ptrs: &'a [*mut f32],
    offset: isize,
    len: usize,
    _marker: PhantomData<&'a mut f32>,
}

impl<'a> BufferMut<'a> {
    #[inline]
    pub unsafe fn from_raw_parts(ptrs: &'a [*mut f32], offset: isize, len: usize) -> BufferMut<'a> {
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

impl<'a> Index<usize> for BufferMut<'a> {
    type Output = [f32];

    #[inline]
    fn index(&self, index: usize) -> &[f32] {
        unsafe { slice::from_raw_parts(self.ptrs[index].offset(self.offset), self.len) }
    }
}

impl<'a> IndexMut<usize> for BufferMut<'a> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut [f32] {
        unsafe { slice::from_raw_parts_mut(self.ptrs[index].offset(self.offset), self.len) }
    }
}

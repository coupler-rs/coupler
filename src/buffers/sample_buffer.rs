use std::marker::PhantomData;

use super::{Buffer, BufferMut, Buffers, RawBuffer, RawBuffers};

pub trait Offset {
    unsafe fn offset(self, count: isize) -> Self;
}

pub trait SampleBuffer: Sized {
    type Raw: Copy + Clone + Offset;

    fn sample_count(&self) -> usize;
    fn into_raw_parts(self) -> (Self::Raw, usize);
    unsafe fn from_raw_parts(raw: Self::Raw, sample_count: usize) -> Self;
}

impl<'a> Offset for RawBuffers<'a> {
    #[inline]
    unsafe fn offset(self, count: isize) -> Self {
        RawBuffers {
            offset: self.offset + count,
            ..self
        }
    }
}

impl<'a, 'b> SampleBuffer for Buffers<'a, 'b> {
    type Raw = RawBuffers<'a>;

    #[inline]
    fn sample_count(&self) -> usize {
        self.len
    }

    #[inline]
    fn into_raw_parts(self) -> (Self::Raw, usize) {
        (self.raw, self.len)
    }

    #[inline]
    unsafe fn from_raw_parts(raw: Self::Raw, sample_count: usize) -> Self {
        Buffers {
            raw,
            len: sample_count,
            _marker: PhantomData,
        }
    }
}

impl<'a> Offset for RawBuffer<'a> {
    #[inline]
    unsafe fn offset(self, count: isize) -> Self {
        RawBuffer {
            offset: self.offset + count,
            ..self
        }
    }
}

impl<'a, 'b> SampleBuffer for Buffer<'a, 'b> {
    type Raw = RawBuffer<'a>;

    #[inline]
    fn sample_count(&self) -> usize {
        self.len
    }

    #[inline]
    fn into_raw_parts(self) -> (Self::Raw, usize) {
        (self.raw, self.len)
    }

    #[inline]
    unsafe fn from_raw_parts(raw: Self::Raw, sample_count: usize) -> Self {
        Buffer {
            raw,
            len: sample_count,
            _marker: PhantomData,
        }
    }
}

impl<'a, 'b> SampleBuffer for BufferMut<'a, 'b> {
    type Raw = RawBuffer<'a>;

    #[inline]
    fn sample_count(&self) -> usize {
        self.len
    }

    #[inline]
    fn into_raw_parts(self) -> (Self::Raw, usize) {
        (self.raw, self.len)
    }

    #[inline]
    unsafe fn from_raw_parts(raw: Self::Raw, sample_count: usize) -> Self {
        BufferMut {
            raw,
            len: sample_count,
            _marker: PhantomData,
        }
    }
}

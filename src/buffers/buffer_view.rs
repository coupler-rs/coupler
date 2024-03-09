use super::iter::{Samples, SplitAtEvents};
use super::{Buffer, BufferData, BufferMut, BufferSamples, Buffers, Sample, SampleMut};
use crate::events::Events;

pub trait Offset {
    unsafe fn offset(self, count: isize) -> Self;
}

pub trait SampleView {
    type Raw: Copy + Clone;

    unsafe fn from_raw(raw: Self::Raw) -> Self;
}

pub trait BufferView: Sized {
    type Raw: Copy + Clone + Offset;
    type Sample: SampleView<Raw = Self::Raw>;

    fn into_raw_parts(self) -> (Self::Raw, usize);
    unsafe fn from_raw_parts(raw: Self::Raw, len: usize) -> Self;

    #[inline]
    fn samples(self) -> Samples<Self> {
        Samples::new(self)
    }

    #[inline]
    fn split_at_events<'e>(self, events: Events<'e>) -> SplitAtEvents<'e, Self> {
        SplitAtEvents::new(self, events)
    }
}

#[derive(Copy, Clone)]
pub struct RawBuffers<'a> {
    pub buffers: &'a [BufferData],
    pub ptrs: &'a [*mut f32],
    pub offset: isize,
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

impl<'a, 'b> SampleView for BufferSamples<'a, 'b> {
    type Raw = RawBuffers<'a>;

    #[inline]
    unsafe fn from_raw(raw: Self::Raw) -> Self {
        BufferSamples::from_raw_parts(raw.buffers, raw.ptrs, raw.offset)
    }
}

impl<'a, 'b> BufferView for Buffers<'a, 'b> {
    type Raw = RawBuffers<'a>;
    type Sample = BufferSamples<'a, 'b>;

    #[inline]
    fn into_raw_parts(self) -> (Self::Raw, usize) {
        (
            RawBuffers {
                buffers: self.buffers,
                ptrs: self.ptrs,
                offset: self.offset,
            },
            self.len,
        )
    }

    #[inline]
    unsafe fn from_raw_parts(raw: Self::Raw, len: usize) -> Self {
        Buffers::from_raw_parts(raw.buffers, raw.ptrs, raw.offset, len)
    }
}

#[derive(Copy, Clone)]
pub struct RawBuffer<'a> {
    pub ptrs: &'a [*mut f32],
    pub offset: isize,
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

impl<'a, 'b> SampleView for Sample<'a, 'b> {
    type Raw = RawBuffer<'a>;

    #[inline]
    unsafe fn from_raw(raw: Self::Raw) -> Self {
        Sample::from_raw_parts(raw.ptrs, raw.offset)
    }
}

impl<'a, 'b> BufferView for Buffer<'a, 'b> {
    type Raw = RawBuffer<'a>;
    type Sample = Sample<'a, 'b>;

    #[inline]
    fn into_raw_parts(self) -> (Self::Raw, usize) {
        (
            RawBuffer {
                ptrs: self.ptrs,
                offset: self.offset,
            },
            self.len,
        )
    }

    #[inline]
    unsafe fn from_raw_parts(raw: Self::Raw, len: usize) -> Self {
        Buffer::from_raw_parts(raw.ptrs, raw.offset, len)
    }
}

impl<'a, 'b> SampleView for SampleMut<'a, 'b> {
    type Raw = RawBuffer<'a>;

    #[inline]
    unsafe fn from_raw(raw: Self::Raw) -> Self {
        SampleMut::from_raw_parts(raw.ptrs, raw.offset)
    }
}

impl<'a, 'b> BufferView for BufferMut<'a, 'b> {
    type Raw = RawBuffer<'a>;
    type Sample = SampleMut<'a, 'b>;

    #[inline]
    fn into_raw_parts(self) -> (Self::Raw, usize) {
        (
            RawBuffer {
                ptrs: self.ptrs,
                offset: self.offset,
            },
            self.len,
        )
    }

    #[inline]
    unsafe fn from_raw_parts(raw: Self::Raw, len: usize) -> Self {
        BufferMut::from_raw_parts(raw.ptrs, raw.offset, len)
    }
}

use std::marker::PhantomData;

use super::iter::{Samples, SplitAtEvents};
use super::{Buffer, BufferMut, BufferSamples, Buffers, RawBuffer, RawBuffers, Sample, SampleMut};
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
    fn sample(self, index: usize) -> Option<Self::Sample> {
        let (raw, len) = self.into_raw_parts();

        if index < len {
            unsafe { Some(Self::Sample::from_raw(raw.offset(index as isize))) }
        } else {
            None
        }
    }

    #[inline]
    fn samples(self) -> Samples<Self> {
        Samples::new(self)
    }

    #[inline]
    fn split_at_events<'e>(self, events: Events<'e>) -> SplitAtEvents<'e, Self> {
        SplitAtEvents::new(self, events)
    }
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
        BufferSamples {
            raw,
            _marker: PhantomData,
        }
    }
}

impl<'a, 'b> BufferView for Buffers<'a, 'b> {
    type Raw = RawBuffers<'a>;
    type Sample = BufferSamples<'a, 'b>;

    #[inline]
    fn into_raw_parts(self) -> (Self::Raw, usize) {
        (self.raw, self.len)
    }

    #[inline]
    unsafe fn from_raw_parts(raw: Self::Raw, len: usize) -> Self {
        Buffers {
            raw,
            len,
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

impl<'a, 'b> SampleView for Sample<'a, 'b> {
    type Raw = RawBuffer<'a>;

    #[inline]
    unsafe fn from_raw(raw: Self::Raw) -> Self {
        Sample {
            raw,
            _marker: PhantomData,
        }
    }
}

impl<'a, 'b> BufferView for Buffer<'a, 'b> {
    type Raw = RawBuffer<'a>;
    type Sample = Sample<'a, 'b>;

    #[inline]
    fn into_raw_parts(self) -> (Self::Raw, usize) {
        (self.raw, self.len)
    }

    #[inline]
    unsafe fn from_raw_parts(raw: Self::Raw, len: usize) -> Self {
        Buffer {
            raw,
            len,
            _marker: PhantomData,
        }
    }
}

impl<'a, 'b> SampleView for SampleMut<'a, 'b> {
    type Raw = RawBuffer<'a>;

    #[inline]
    unsafe fn from_raw(raw: Self::Raw) -> Self {
        SampleMut {
            raw,
            _marker: PhantomData,
        }
    }
}

impl<'a, 'b> BufferView for BufferMut<'a, 'b> {
    type Raw = RawBuffer<'a>;
    type Sample = SampleMut<'a, 'b>;

    #[inline]
    fn into_raw_parts(self) -> (Self::Raw, usize) {
        (self.raw, self.len)
    }

    #[inline]
    unsafe fn from_raw_parts(raw: Self::Raw, len: usize) -> Self {
        BufferMut {
            raw,
            len,
            _marker: PhantomData,
        }
    }
}

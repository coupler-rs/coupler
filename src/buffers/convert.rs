use std::error::Error;
use std::{array, fmt, slice};

use super::{AnyBuffer, Buffer, BufferMut, Buffers};

impl<'a, 'b> TryFrom<AnyBuffer<'a, 'b>> for Buffer<'a, 'b> {
    type Error = AnyBuffer<'a, 'b>;

    #[inline]
    fn try_from(value: AnyBuffer<'a, 'b>) -> Result<Buffer<'a, 'b>, Self::Error> {
        match value {
            AnyBuffer::Const(buffer) => Ok(buffer),
            _ => Err(value),
        }
    }
}

impl<'a, 'b> TryFrom<AnyBuffer<'a, 'b>> for BufferMut<'a, 'b> {
    type Error = AnyBuffer<'a, 'b>;

    #[inline]
    fn try_from(value: AnyBuffer<'a, 'b>) -> Result<BufferMut<'a, 'b>, Self::Error> {
        match value {
            AnyBuffer::Mut(buffer) => Ok(buffer),
            _ => Err(value),
        }
    }
}

impl<'a, 'b, const N: usize> TryFrom<Buffer<'a, 'b>> for [&'b [f32]; N] {
    type Error = Buffer<'a, 'b>;

    #[inline]
    fn try_from(value: Buffer<'a, 'b>) -> Result<[&'b [f32]; N], Self::Error> {
        if value.channel_count() == N {
            Ok(array::from_fn(|i| unsafe {
                slice::from_raw_parts(value.ptrs[i].offset(value.offset), value.len)
            }))
        } else {
            Err(value)
        }
    }
}

impl<'a, 'b, const N: usize> TryFrom<BufferMut<'a, 'b>> for [&'b mut [f32]; N] {
    type Error = BufferMut<'a, 'b>;

    #[inline]
    fn try_from(value: BufferMut<'a, 'b>) -> Result<[&'b mut [f32]; N], Self::Error> {
        if value.channel_count() == N {
            Ok(array::from_fn(|i| unsafe {
                slice::from_raw_parts_mut(value.ptrs[i].offset(value.offset), value.len)
            }))
        } else {
            Err(value)
        }
    }
}

#[derive(Debug)]
pub struct TryFromBuffersError;

impl fmt::Display for TryFromBuffersError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        "buffer layout does not match".fmt(fmt)
    }
}

impl Error for TryFromBuffersError {}

macro_rules! try_from_buffers {
    ($($buffer:ident),*) => {
        impl<'a, 'b, $($buffer),*> TryFrom<Buffers<'a, 'b>> for ($($buffer,)*)
        where
            $($buffer: TryFrom<AnyBuffer<'a, 'b>>),*
        {
            type Error = TryFromBuffersError;

            #[inline]
            fn try_from(value: Buffers<'a, 'b>) -> Result<Self, Self::Error> {
                let mut iter = value.into_iter();

                let result = (
                    $({
                        let next = iter.next().ok_or(TryFromBuffersError)?;
                        $buffer::try_from(next).map_err(|_| TryFromBuffersError)?
                    },)*
                );

                if iter.next().is_none() {
                    Ok(result)
                } else {
                    Err(TryFromBuffersError)
                }
            }
        }
    }
}

try_from_buffers!();
try_from_buffers!(B0);
try_from_buffers!(B0, B1);
try_from_buffers!(B0, B1, B2);
try_from_buffers!(B0, B1, B2, B3);
try_from_buffers!(B0, B1, B2, B3, B4);
try_from_buffers!(B0, B1, B2, B3, B4, B5);
try_from_buffers!(B0, B1, B2, B3, B4, B5, B6);
try_from_buffers!(B0, B1, B2, B3, B4, B5, B6, B7);
try_from_buffers!(B0, B1, B2, B3, B4, B5, B6, B7, B8);
try_from_buffers!(B0, B1, B2, B3, B4, B5, B6, B7, B8, B9);

use std::array;

use super::{AnyBuffer, Buffer, BufferMut};

pub trait FromBuffers<'a, 'b>: Sized {
    fn from_buffers<I>(buffers: &mut I) -> Option<Self>
    where
        I: Iterator<Item = AnyBuffer<'a, 'b>>;
}

impl<'a, 'b> FromBuffers<'a, 'b> for Buffer<'a, 'b> {
    #[inline]
    fn from_buffers<I>(buffers: &mut I) -> Option<Self>
    where
        I: Iterator<Item = AnyBuffer<'a, 'b>>,
    {
        match buffers.next()? {
            AnyBuffer::Const(buffer) => Some(buffer),
            _ => None,
        }
    }
}

impl<'a, 'b, const N: usize> FromBuffers<'a, 'b> for [&'b [f32]; N] {
    #[inline]
    fn from_buffers<I>(buffers: &mut I) -> Option<Self>
    where
        I: Iterator<Item = AnyBuffer<'a, 'b>>,
    {
        Buffer::from_buffers(buffers)?.collect()
    }
}

impl<'a, 'b> FromBuffers<'a, 'b> for BufferMut<'a, 'b> {
    #[inline]
    fn from_buffers<I>(buffers: &mut I) -> Option<Self>
    where
        I: Iterator<Item = AnyBuffer<'a, 'b>>,
    {
        match buffers.next()? {
            AnyBuffer::Mut(buffer) => Some(buffer),
            _ => None,
        }
    }
}

impl<'a, 'b, const N: usize> FromBuffers<'a, 'b> for [&'b mut [f32]; N] {
    #[inline]
    fn from_buffers<I>(buffers: &mut I) -> Option<Self>
    where
        I: Iterator<Item = AnyBuffer<'a, 'b>>,
    {
        BufferMut::from_buffers(buffers)?.collect()
    }
}

macro_rules! from_buffers {
    ($($buffer:ident),*) => {
        impl<'a, 'b, $($buffer),*> FromBuffers<'a, 'b> for ($($buffer,)*)
        where
            $($buffer: FromBuffers<'a, 'b>),*
        {
            #[inline]
            fn from_buffers<I>(buffers: &mut I) -> Option<Self>
            where
                I: Iterator<Item = AnyBuffer<'a, 'b>>,
            {
                Some((
                    $(
                        $buffer::from_buffers(buffers)?,
                    )*
                ))
            }
        }
    }
}

from_buffers!(B0);
from_buffers!(B0, B1);
from_buffers!(B0, B1, B2);
from_buffers!(B0, B1, B2, B3);
from_buffers!(B0, B1, B2, B3, B4);
from_buffers!(B0, B1, B2, B3, B4, B5);
from_buffers!(B0, B1, B2, B3, B4, B5, B6);
from_buffers!(B0, B1, B2, B3, B4, B5, B6, B7);
from_buffers!(B0, B1, B2, B3, B4, B5, B6, B7, B8);
from_buffers!(B0, B1, B2, B3, B4, B5, B6, B7, B8, B9);

impl<'a, 'b, const N: usize, B> FromBuffers<'a, 'b> for [B; N]
where
    B: FromBuffers<'a, 'b>,
{
    #[inline]
    fn from_buffers<I>(buffers: &mut I) -> Option<Self>
    where
        I: Iterator<Item = AnyBuffer<'a, 'b>>,
    {
        let mut results = array::from_fn(|_| None);

        for result in results.iter_mut() {
            *result = Some(B::from_buffers(buffers)?);
        }

        Some(results.map(|result| result.unwrap()))
    }
}

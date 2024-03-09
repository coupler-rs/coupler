use std::array;

use super::{AnyBuffer, Buffer, BufferMut};

pub trait FromBuffers<'a, 'b>: Sized {
    fn bind<I>(buffers: &mut I) -> Option<Self>
    where
        I: Iterator<Item = AnyBuffer<'a, 'b>>;
}

impl<'a, 'b> FromBuffers<'a, 'b> for Buffer<'a, 'b> {
    #[inline]
    fn bind<I>(buffers: &mut I) -> Option<Self>
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
    fn bind<I>(buffers: &mut I) -> Option<Self>
    where
        I: Iterator<Item = AnyBuffer<'a, 'b>>,
    {
        Buffer::bind(buffers)?.collect()
    }
}

impl<'a, 'b> FromBuffers<'a, 'b> for BufferMut<'a, 'b> {
    #[inline]
    fn bind<I>(buffers: &mut I) -> Option<Self>
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
    fn bind<I>(buffers: &mut I) -> Option<Self>
    where
        I: Iterator<Item = AnyBuffer<'a, 'b>>,
    {
        BufferMut::bind(buffers)?.collect()
    }
}

macro_rules! bind_buffers {
    ($($binding:ident),*) => {
        impl<'a, 'b, $($binding),*> FromBuffers<'a, 'b> for ($($binding,)*)
        where
            $($binding: FromBuffers<'a, 'b>),*
        {
            #[inline]
            fn bind<I>(buffers: &mut I) -> Option<Self>
            where
                I: Iterator<Item = AnyBuffer<'a, 'b>>,
            {
                Some((
                    $(
                        $binding::bind(buffers)?,
                    )*
                ))
            }
        }
    }
}

bind_buffers!(B0);
bind_buffers!(B0, B1);
bind_buffers!(B0, B1, B2);
bind_buffers!(B0, B1, B2, B3);
bind_buffers!(B0, B1, B2, B3, B4);
bind_buffers!(B0, B1, B2, B3, B4, B5);
bind_buffers!(B0, B1, B2, B3, B4, B5, B6);
bind_buffers!(B0, B1, B2, B3, B4, B5, B6, B7);
bind_buffers!(B0, B1, B2, B3, B4, B5, B6, B7, B8);
bind_buffers!(B0, B1, B2, B3, B4, B5, B6, B7, B8, B9);

impl<'a, 'b, const N: usize, B> FromBuffers<'a, 'b> for [B; N]
where
    B: FromBuffers<'a, 'b>,
{
    #[inline]
    fn bind<I>(buffers: &mut I) -> Option<Self>
    where
        I: Iterator<Item = AnyBuffer<'a, 'b>>,
    {
        let mut results = array::from_fn(|_| None);

        for result in results.iter_mut() {
            *result = Some(B::bind(buffers)?);
        }

        Some(results.map(|result| result.unwrap()))
    }
}

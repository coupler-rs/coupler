use super::{AnyBuffer, Buffer, BufferMut};

pub trait BindBuffers<'a, 'b>: Sized {
    fn bind<I>(buffers: &mut I) -> Option<Self>
    where
        I: Iterator<Item = AnyBuffer<'a, 'b>>;
}

impl<'a, 'b> BindBuffers<'a, 'b> for Buffer<'a, 'b> {
    #[inline]
    fn bind<I>(buffers: &mut I) -> Option<Self>
    where
        I: Iterator<Item = AnyBuffer<'a, 'b>>,
    {
        match buffers.next() {
            Some(AnyBuffer::Const(buffer)) => Some(buffer),
            _ => None,
        }
    }
}

impl<'a, 'b> BindBuffers<'a, 'b> for BufferMut<'a, 'b> {
    #[inline]
    fn bind<I>(buffers: &mut I) -> Option<Self>
    where
        I: Iterator<Item = AnyBuffer<'a, 'b>>,
    {
        match buffers.next() {
            Some(AnyBuffer::Mut(buffer)) => Some(buffer),
            _ => None,
        }
    }
}

macro_rules! bind_buffers {
    ($($binding:ident),*) => {
        impl<'a, 'b, $($binding),*> BindBuffers<'a, 'b> for ($($binding,)*)
        where
            $($binding: BindBuffers<'a, 'b>),*
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

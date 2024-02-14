use super::{AnyBuffer, Buffer, BufferMut, Buffers};

#[derive(Copy, Clone, Debug)]
pub struct BindBuffersError(pub(crate) ());

pub trait BindBuffers<'a, 'b>: Sized {
    fn bind<I>(buffers: &mut I) -> Result<Self, BindBuffersError>
    where
        I: Iterator<Item = AnyBuffer<'a, 'b>>;
}

impl<'a, 'b> TryFrom<AnyBuffer<'a, 'b>> for Buffer<'a, 'b> {
    type Error = BindBuffersError;

    #[inline]
    fn try_from(value: AnyBuffer<'a, 'b>) -> Result<Self, Self::Error> {
        match value {
            AnyBuffer::Const(buffer) => Ok(buffer),
            _ => Err(BindBuffersError(())),
        }
    }
}

impl<'a, 'b> BindBuffers<'a, 'b> for Buffer<'a, 'b> {
    #[inline]
    fn bind<I>(buffers: &mut I) -> Result<Self, BindBuffersError>
    where
        I: Iterator<Item = AnyBuffer<'a, 'b>>,
    {
        match buffers.next() {
            Some(buffer) => buffer.try_into(),
            _ => Err(BindBuffersError(())),
        }
    }
}

impl<'a, 'b> TryFrom<Buffers<'a, 'b>> for Buffer<'a, 'b> {
    type Error = BindBuffersError;

    #[inline]
    fn try_from(value: Buffers<'a, 'b>) -> Result<Self, Self::Error> {
        value.bind()
    }
}

impl<'a, 'b> TryFrom<AnyBuffer<'a, 'b>> for BufferMut<'a, 'b> {
    type Error = BindBuffersError;

    #[inline]
    fn try_from(value: AnyBuffer<'a, 'b>) -> Result<Self, Self::Error> {
        match value {
            AnyBuffer::Mut(buffer) => Ok(buffer),
            _ => Err(BindBuffersError(())),
        }
    }
}

impl<'a, 'b> BindBuffers<'a, 'b> for BufferMut<'a, 'b> {
    #[inline]
    fn bind<I>(buffers: &mut I) -> Result<Self, BindBuffersError>
    where
        I: Iterator<Item = AnyBuffer<'a, 'b>>,
    {
        match buffers.next() {
            Some(buffer) => buffer.try_into(),
            _ => Err(BindBuffersError(())),
        }
    }
}

impl<'a, 'b> TryFrom<Buffers<'a, 'b>> for BufferMut<'a, 'b> {
    type Error = BindBuffersError;

    #[inline]
    fn try_from(value: Buffers<'a, 'b>) -> Result<Self, Self::Error> {
        value.bind()
    }
}

macro_rules! bind_buffers {
    ($($binding:ident),*) => {
        impl<'a, 'b, $($binding),*> BindBuffers<'a, 'b> for ($($binding,)*)
        where
            $($binding: BindBuffers<'a, 'b>),*
        {
            #[inline]
            fn bind<I>(buffers: &mut I) -> Result<Self, BindBuffersError>
            where
                I: Iterator<Item = AnyBuffer<'a, 'b>>,
            {
                Ok((
                    $(
                        $binding::bind(buffers)?,
                    )*
                ))
            }
        }

        impl<'a, 'b, $($binding),*> TryFrom<Buffers<'a, 'b>> for ($($binding,)*)
        where
            $($binding: BindBuffers<'a, 'b>),*
        {
            type Error = BindBuffersError;

            #[inline]
            fn try_from(value: Buffers<'a, 'b>) -> Result<Self, Self::Error> {
                value.bind()
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

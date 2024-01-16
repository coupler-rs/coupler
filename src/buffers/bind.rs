use super::{Buffer, BufferDir, BufferMut};

pub trait BindBuffers<'a>: Sized {
    fn bind<I>(buffers: &mut I) -> Option<Self>
    where
        I: Iterator<Item = BufferDir<'a>>;
}

pub struct In<'a>(pub Buffer<'a>);

impl<'a> BindBuffers<'a> for In<'a> {
    #[inline]
    fn bind<I>(buffers: &mut I) -> Option<Self>
    where
        I: Iterator<Item = BufferDir<'a>>,
    {
        match buffers.next() {
            Some(BufferDir::In(buffer)) => Some(In(buffer)),
            _ => None,
        }
    }
}

pub struct Out<'a>(pub BufferMut<'a>);

impl<'a> BindBuffers<'a> for Out<'a> {
    #[inline]
    fn bind<I>(buffers: &mut I) -> Option<Self>
    where
        I: Iterator<Item = BufferDir<'a>>,
    {
        match buffers.next() {
            Some(BufferDir::Out(buffer)) => Some(Out(buffer)),
            _ => None,
        }
    }
}

pub struct InOut<'a>(pub BufferMut<'a>);

impl<'a> BindBuffers<'a> for InOut<'a> {
    #[inline]
    fn bind<I>(buffers: &mut I) -> Option<Self>
    where
        I: Iterator<Item = BufferDir<'a>>,
    {
        match buffers.next() {
            Some(BufferDir::InOut(buffer)) => Some(InOut(buffer)),
            _ => None,
        }
    }
}

macro_rules! bind_buffers {
    ($($binding:ident),*) => {
        impl<'a, $($binding),*> BindBuffers<'a> for ($($binding,)*)
        where
            $($binding: BindBuffers<'a>),*
        {
            #[inline]
            fn bind<I>(buffers: &mut I) -> Option<Self>
            where
                I: Iterator<Item = BufferDir<'a>>,
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

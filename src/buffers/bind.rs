use super::{Buffer, BufferDir, BufferMut};

pub trait BindBuffers<'a, 'b>: Sized {
    fn bind<I>(buffers: &mut I) -> Option<Self>
    where
        I: Iterator<Item = BufferDir<'a, 'b>>;
}

pub struct In<'a, 'b>(pub Buffer<'a, 'b>);

impl<'a, 'b> BindBuffers<'a, 'b> for In<'a, 'b> {
    #[inline]
    fn bind<I>(buffers: &mut I) -> Option<Self>
    where
        I: Iterator<Item = BufferDir<'a, 'b>>,
    {
        match buffers.next() {
            Some(BufferDir::In(buffer)) => Some(In(buffer)),
            _ => None,
        }
    }
}

pub struct Out<'a, 'b>(pub BufferMut<'a, 'b>);

impl<'a, 'b> BindBuffers<'a, 'b> for Out<'a, 'b> {
    #[inline]
    fn bind<I>(buffers: &mut I) -> Option<Self>
    where
        I: Iterator<Item = BufferDir<'a, 'b>>,
    {
        match buffers.next() {
            Some(BufferDir::Out(buffer)) => Some(Out(buffer)),
            _ => None,
        }
    }
}

pub struct InOut<'a, 'b>(pub BufferMut<'a, 'b>);

impl<'a, 'b> BindBuffers<'a, 'b> for InOut<'a, 'b> {
    #[inline]
    fn bind<I>(buffers: &mut I) -> Option<Self>
    where
        I: Iterator<Item = BufferDir<'a, 'b>>,
    {
        match buffers.next() {
            Some(BufferDir::InOut(buffer)) => Some(InOut(buffer)),
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
                I: Iterator<Item = BufferDir<'a, 'b>>,
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

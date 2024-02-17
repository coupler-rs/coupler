use super::{Buffer, BufferMut, Buffers};

pub trait SampleBuffer {
    fn sample_count(&self) -> usize;
}

impl<'a, 'b> SampleBuffer for Buffers<'a, 'b> {
    #[inline]
    fn sample_count(&self) -> usize {
        self.len
    }
}

impl<'a, 'b> SampleBuffer for Buffer<'a, 'b> {
    #[inline]
    fn sample_count(&self) -> usize {
        self.len
    }
}

impl<'a, 'b> SampleBuffer for BufferMut<'a, 'b> {
    #[inline]
    fn sample_count(&self) -> usize {
        self.len
    }
}

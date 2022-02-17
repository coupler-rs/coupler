use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::slice;

pub struct Buffers<'a, 'b> {
    pub(crate) inputs: &'a [Bus<'a>],
    pub(crate) outputs: &'a mut [BusMut<'b>],
    pub(crate) samples: usize,
}

impl<'a, 'b> Buffers<'a, 'b> {
    pub fn samples(&self) -> usize {
        self.samples
    }

    pub fn inputs(&self) -> &[Bus] {
        self.inputs
    }

    pub fn outputs(&mut self) -> &mut [BusMut<'b>] {
        self.outputs
    }
}

pub struct Bus<'a> {
    pub(crate) channels: Vec<Buffer<'a>>,
}

impl<'a> Bus<'a> {
    pub fn enabled(&self) -> bool {
        !self.channels.is_empty()
    }
}

impl<'a> Deref for Bus<'a> {
    type Target = [Buffer<'a>];

    fn deref(&self) -> &[Buffer<'a>] {
        &self.channels
    }
}

pub struct BusMut<'a> {
    pub(crate) channels: Vec<BufferMut<'a>>,
}

impl<'a> BusMut<'a> {
    pub fn enabled(&self) -> bool {
        !self.channels.is_empty()
    }
}

impl<'a> Deref for BusMut<'a> {
    type Target = [BufferMut<'a>];

    fn deref(&self) -> &[BufferMut<'a>] {
        &self.channels
    }
}

impl<'a> DerefMut for BusMut<'a> {
    fn deref_mut(&mut self) -> &mut [BufferMut<'a>] {
        &mut self.channels
    }
}

pub struct Buffer<'a> {
    pub(crate) ptr: *const f32,
    pub(crate) samples: usize,
    pub(crate) phantom: PhantomData<&'a [f32]>,
}

impl<'a> Deref for Buffer<'a> {
    type Target = [f32];

    fn deref(&self) -> &[f32] {
        unsafe { slice::from_raw_parts(self.ptr, self.samples) }
    }
}

pub struct BufferMut<'a> {
    pub(crate) ptr: *mut f32,
    pub(crate) samples: usize,
    pub(crate) phantom: PhantomData<&'a mut [f32]>,
}

impl<'a> Deref for BufferMut<'a> {
    type Target = [f32];

    fn deref(&self) -> &[f32] {
        unsafe { slice::from_raw_parts(self.ptr, self.samples) }
    }
}

impl<'a> DerefMut for BufferMut<'a> {
    fn deref_mut(&mut self) -> &mut [f32] {
        unsafe { slice::from_raw_parts_mut(self.ptr, self.samples) }
    }
}

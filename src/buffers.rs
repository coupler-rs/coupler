use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::slice;

pub struct Buffers<'a, 'b, 'c> {
    input_ptrs: &'a [*const f32],
    input_data: &'a [InputData],
    output_ptrs: &'a [*mut f32],
    output_data: &'a [OutputData],
    offset: usize,
    len: usize,
    _marker: PhantomData<(&'b f32, &'c mut f32)>,
}

impl<'a, 'b, 'c> Buffers<'a, 'b, 'c> {
    #[inline]
    pub unsafe fn from_raw_parts(
        input_ptrs: &'a [*const f32],
        input_data: &'a [InputData],
        output_ptrs: &'a [*mut f32],
        output_data: &'a [OutputData],
        offset: usize,
        len: usize,
    ) -> Buffers<'a, 'b, 'c> {
        Buffers {
            input_ptrs,
            input_data,
            output_ptrs,
            output_data,
            offset,
            len,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn inputs(&mut self) -> Inputs {
        unsafe { Inputs::from_raw_parts(self.input_ptrs, self.input_data, self.offset, self.len) }
    }

    #[inline]
    pub fn outputs(&mut self) -> Outputs {
        unsafe {
            Outputs::from_raw_parts(self.output_ptrs, self.output_data, self.offset, self.len)
        }
    }

    #[inline]
    pub fn split(&mut self) -> (Inputs, Outputs) {
        unsafe {
            (
                Inputs::from_raw_parts(self.input_ptrs, self.input_data, self.offset, self.len),
                Outputs::from_raw_parts(self.output_ptrs, self.output_data, self.offset, self.len),
            )
        }
    }
}

pub struct InputData {
    pub start: usize,
    pub end: usize,
}

pub struct Inputs<'a, 'b> {
    ptrs: &'a [*const f32],
    data: &'a [InputData],
    offset: usize,
    len: usize,
    _marker: PhantomData<&'b f32>,
}

impl<'a, 'b> Inputs<'a, 'b> {
    #[inline]
    pub unsafe fn from_raw_parts(
        ptrs: &'a [*const f32],
        data: &'a [InputData],
        offset: usize,
        len: usize,
    ) -> Inputs<'a, 'b> {
        Inputs {
            ptrs,
            data,
            offset,
            len,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn bus_count(&self) -> usize {
        self.ptrs.len()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<Buffer> {
        if let Some(data) = self.data.get(index) {
            unsafe {
                Some(Buffer::from_raw_parts(
                    &self.ptrs[data.start..data.end],
                    self.offset,
                    self.len,
                ))
            }
        } else {
            None
        }
    }
}

pub struct OutputData {
    pub start: usize,
    pub end: usize,
}

pub struct Outputs<'a, 'b> {
    ptrs: &'a [*mut f32],
    data: &'a [OutputData],
    offset: usize,
    len: usize,
    _marker: PhantomData<&'b f32>,
}

impl<'a, 'b> Outputs<'a, 'b> {
    #[inline]
    pub unsafe fn from_raw_parts(
        ptrs: &'a [*mut f32],
        data: &'a [OutputData],
        offset: usize,
        len: usize,
    ) -> Outputs<'a, 'b> {
        Outputs {
            ptrs,
            data,
            offset,
            len,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn bus_count(&self) -> usize {
        self.ptrs.len()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<BufferMut> {
        if let Some(data) = self.data.get(index) {
            unsafe {
                Some(BufferMut::from_raw_parts(
                    &self.ptrs[data.start..data.end],
                    self.offset,
                    self.len,
                ))
            }
        } else {
            None
        }
    }
}

pub struct Buffer<'a, 'b> {
    ptrs: &'a [*const f32],
    offset: usize,
    len: usize,
    _marker: PhantomData<&'b f32>,
}

impl<'a, 'b> Buffer<'a, 'b> {
    #[inline]
    pub unsafe fn from_raw_parts(
        ptrs: &'a [*const f32],
        offset: usize,
        len: usize,
    ) -> Buffer<'a, 'b> {
        Buffer {
            ptrs,
            offset,
            len,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn channel_count(&self) -> usize {
        self.ptrs.len()
    }
}

impl<'a, 'b> Index<usize> for Buffer<'a, 'b> {
    type Output = [f32];

    fn index(&self, index: usize) -> &[f32] {
        unsafe { slice::from_raw_parts(self.ptrs[index].add(self.offset), self.len) }
    }
}

pub struct BufferMut<'a, 'b> {
    ptrs: &'a [*mut f32],
    offset: usize,
    len: usize,
    _marker: PhantomData<&'b f32>,
}

impl<'a, 'b> BufferMut<'a, 'b> {
    #[inline]
    pub unsafe fn from_raw_parts(
        ptrs: &'a [*mut f32],
        offset: usize,
        len: usize,
    ) -> BufferMut<'a, 'b> {
        BufferMut {
            ptrs,
            offset,
            len,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn channel_count(&self) -> usize {
        self.ptrs.len()
    }
}

impl<'a, 'b> Index<usize> for BufferMut<'a, 'b> {
    type Output = [f32];

    fn index(&self, index: usize) -> &[f32] {
        unsafe { slice::from_raw_parts(self.ptrs[index].add(self.offset), self.len) }
    }
}

impl<'a, 'b> IndexMut<usize> for BufferMut<'a, 'b> {
    fn index_mut(&mut self, index: usize) -> &mut [f32] {
        unsafe { slice::from_raw_parts_mut(self.ptrs[index].add(self.offset), self.len) }
    }
}

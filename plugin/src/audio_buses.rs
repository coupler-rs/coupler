use std::slice;

pub struct AudioBuses<'a> {
    pub(crate) frames: usize,
    pub(crate) input_buses: &'a [(usize, usize)],
    pub(crate) input_channels: &'a [*const f32],
    pub(crate) output_buses: &'a [(usize, usize)],
    pub(crate) output_channels: &'a [*mut f32],
}

impl<'a> AudioBuses<'a> {
    pub fn frames(&self) -> usize {
        self.frames
    }

    pub fn input_count(&self) -> usize {
        self.input_buses.len()
    }

    pub fn output_count(&self) -> usize {
        self.output_buses.len()
    }

    pub fn input(&self, index: usize) -> Option<InputBus<'a>> {
        if let Some(&(start, end)) = self.input_buses.get(index) {
            Some(InputBus { frames: self.frames, channels: &self.input_channels[start..end] })
        } else {
            None
        }
    }

    pub fn output(&mut self, index: usize) -> Option<OutputBus<'a>> {
        if let Some(&(start, end)) = self.input_buses.get(index) {
            Some(OutputBus { frames: self.frames, channels: &self.output_channels[start..end] })
        } else {
            None
        }
    }
}

pub struct InputBus<'a> {
    frames: usize,
    channels: &'a [*const f32],
}

impl<'a> InputBus<'a> {
    pub fn frames(&self) -> usize {
        self.frames
    }

    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    pub fn channel(&self, index: usize) -> Option<&[f32]> {
        if let Some(&ptr) = self.channels.get(index) {
            Some(unsafe { slice::from_raw_parts(ptr, self.frames) })
        } else {
            None
        }
    }
}

pub struct OutputBus<'a> {
    frames: usize,
    channels: &'a [*mut f32],
}

impl<'a> OutputBus<'a> {
    pub fn frames(&self) -> usize {
        self.frames
    }

    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    pub fn channel(&self, index: usize) -> Option<&'a [f32]> {
        if let Some(&ptr) = self.channels.get(index) {
            Some(unsafe { slice::from_raw_parts(ptr, self.frames) })
        } else {
            None
        }
    }

    pub fn channel_mut(&mut self, index: usize) -> Option<&'a mut [f32]> {
        if let Some(&ptr) = self.channels.get(index) {
            Some(unsafe { slice::from_raw_parts_mut(ptr, self.frames) })
        } else {
            None
        }
    }
}

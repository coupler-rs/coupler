use std::ptr::NonNull;

use vst3_bindgen::Steinberg::Vst::ProcessData;

use super::util::slice_from_raw_parts_checked;
use crate::buffers::{Buffers, InputData, OutputData};
use crate::Config;

pub struct ScratchBuffers {
    buffers: Vec<f32>,
    silence: Vec<f32>,
    input_ptrs: Vec<*const f32>,
    input_data: Vec<InputData>,
    output_ptrs: Vec<*mut f32>,
    output_data: Vec<OutputData>,
}

impl ScratchBuffers {
    pub fn new() -> ScratchBuffers {
        ScratchBuffers {
            buffers: Vec::new(),
            silence: Vec::new(),
            input_ptrs: Vec::new(),
            input_data: Vec::new(),
            output_ptrs: Vec::new(),
            output_data: Vec::new(),
        }
    }

    pub fn resize(&mut self, config: &Config) {
        self.input_data.clear();
        let mut input_channels = 0;
        for input in &config.layout.inputs {
            let channel_count = input.channel_count();
            self.input_data.push(InputData {
                start: input_channels,
                end: input_channels + channel_count,
            });
            input_channels += channel_count;
        }

        self.output_data.clear();
        let mut output_channels = 0;
        for output in &config.layout.outputs {
            let channel_count = output.channel_count();
            self.output_data.push(OutputData {
                start: output_channels,
                end: output_channels + channel_count,
            });
            output_channels += channel_count;
        }

        // Each output buffer can either alias an input buffer or belong to an inactive output bus,
        // so the worst-case number of scratch buffers is given by the number of output channels.
        let scratch_space = config.max_buffer_size * output_channels;
        self.buffers.resize(scratch_space, 0.0);

        // Silence buffer, to be used for inactive input buses
        self.silence.resize(config.max_buffer_size, 0.0);

        let dangling = NonNull::dangling().as_ptr();

        self.input_ptrs.clear();
        self.input_ptrs.resize(input_channels, dangling);

        self.output_ptrs.clear();
        self.output_ptrs.resize(output_channels, dangling);
    }

    pub unsafe fn get_buffers(
        &mut self,
        config: &Config,
        inputs_active: &[bool],
        outputs_active: &[bool],
        data: &ProcessData,
    ) -> Result<Buffers, ()> {
        let len = data.numSamples as usize;
        if len > config.max_buffer_size {
            return Err(());
        }

        let mut scratch = &mut self.buffers[..];

        if len == 0 {
            let dangling = NonNull::dangling().as_ptr();

            self.input_ptrs.fill(dangling);
            self.output_ptrs.fill(dangling);
        } else {
            let input_count = data.numInputs as usize;
            if input_count != config.layout.inputs.len() {
                return Err(());
            }

            let inputs = slice_from_raw_parts_checked(data.inputs, input_count);
            for i in 0..inputs.len() {
                if inputs_active[i] {
                    let channel_count = inputs[i].numChannels as usize;
                    if channel_count != config.layout.inputs[i].channel_count() {
                        return Err(());
                    }

                    let channels_ptr = inputs[i].__field0.channelBuffers32 as *const *const f32;
                    let channels = slice_from_raw_parts_checked(channels_ptr, channel_count);

                    let input_data = &self.input_data[i];
                    self.input_ptrs[input_data.start..input_data.end].copy_from_slice(channels);
                } else {
                    let silence = self.silence.as_ptr();

                    let input_data = &self.input_data[i];
                    self.input_ptrs[input_data.start..input_data.end].fill(silence);
                }
            }

            let output_count = data.numOutputs as usize;
            if output_count != config.layout.outputs.len() {
                return Err(());
            }

            let outputs = slice_from_raw_parts_checked(data.outputs, output_count);
            for i in 0..outputs.len() {
                if outputs_active[i] {
                    let channel_count = outputs[i].numChannels as usize;
                    if channel_count != config.layout.outputs[i].channel_count() {
                        return Err(());
                    }

                    let channels_ptr = outputs[i].__field0.channelBuffers32;
                    let channels = slice_from_raw_parts_checked(channels_ptr, channel_count);

                    let output_data = &self.output_data[i];
                    self.output_ptrs[output_data.start..output_data.end].copy_from_slice(channels);
                } else {
                    let output_data = &self.output_data[i];
                    for ptr in &mut self.output_ptrs[output_data.start..output_data.end] {
                        let (first, rest) = scratch.split_at_mut(config.max_buffer_size);
                        scratch = rest;

                        *ptr = first.as_mut_ptr();
                    }
                }
            }
        }

        Ok(Buffers::from_raw_parts(
            &self.input_ptrs,
            &self.input_data,
            &self.output_ptrs,
            &self.output_data,
            0,
            len,
        ))
    }
}

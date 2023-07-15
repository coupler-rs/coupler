use std::cmp::Ordering;
use std::ptr::NonNull;
use std::slice;

use vst3_bindgen::Steinberg::Vst::ProcessData;

use super::util::slice_from_raw_parts_checked;
use crate::buffers::{Buffers, BusData};
use crate::bus::{BusDir, BusInfo};
use crate::Config;

pub struct ScratchBuffers {
    buffers: Vec<f32>,
    silence: Vec<f32>,
    inputs_sorted: Vec<usize>,
    outputs_sorted: Vec<usize>,
    ptrs: Vec<*mut f32>,
    buses: Vec<BusData>,
}

impl ScratchBuffers {
    pub fn new() -> ScratchBuffers {
        ScratchBuffers {
            buffers: Vec::new(),
            silence: Vec::new(),
            inputs_sorted: Vec::new(),
            outputs_sorted: Vec::new(),
            ptrs: Vec::new(),
            buses: Vec::new(),
        }
    }

    pub fn resize(&mut self, buses: &[BusInfo], config: &Config) {
        self.buses.clear();
        let mut total_channels = 0;
        let mut input_channels = 0;
        let mut output_channels = 0;
        for (info, format) in buses.iter().zip(config.layout.formats.iter()) {
            let channel_count = format.channel_count();

            self.buses.push(BusData {
                start: total_channels,
                end: total_channels + channel_count,
                dir: info.dir,
            });

            total_channels += channel_count;
            match info.dir {
                BusDir::In => input_channels += channel_count,
                BusDir::Out => output_channels += channel_count,
            }
        }

        // Each output buffer can either alias an input buffer or belong to an inactive output bus,
        // so the worst-case number of scratch buffers is given by the number of output channels.
        let scratch_space = config.max_buffer_size * output_channels;
        self.buffers.resize(scratch_space, 0.0);

        // Silence buffer, to be used for inactive input buses
        self.silence.resize(config.max_buffer_size, 0.0);

        self.inputs_sorted.clear();
        self.inputs_sorted.reserve(input_channels);
        self.inputs_sorted.shrink_to(input_channels);

        self.outputs_sorted.clear();
        self.outputs_sorted.reserve(input_channels);
        self.outputs_sorted.shrink_to(input_channels);

        self.ptrs.resize(total_channels, NonNull::dangling().as_ptr());
    }

    pub unsafe fn get_buffers(
        &mut self,
        buses: &[BusInfo],
        input_bus_map: &[usize],
        output_bus_map: &[usize],
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
            self.ptrs.fill(NonNull::dangling().as_ptr());
        } else {
            let input_count = data.numInputs as usize;
            let output_count = data.numOutputs as usize;
            if input_count != input_bus_map.len() || output_count != output_bus_map.len() {
                return Err(());
            }

            let inputs = slice_from_raw_parts_checked(data.inputs, input_count);
            let outputs = slice_from_raw_parts_checked(data.outputs, output_count);

            let mut input_index = 0;
            let mut output_index = 0;

            for (bus_index, bus_info) in buses.iter().enumerate() {
                match bus_info.dir {
                    BusDir::In => {
                        if inputs_active[input_index] {
                            let channel_count = inputs[input_index].numChannels as usize;
                            if channel_count != config.layout.formats[bus_index].channel_count() {
                                return Err(());
                            }

                            let channels = slice_from_raw_parts_checked(
                                inputs[input_index].__field0.channelBuffers32,
                                channel_count,
                            );

                            let bus_data = &self.buses[bus_index];
                            self.ptrs[bus_data.start..bus_data.end].copy_from_slice(channels);
                        } else {
                            // For inactive input buses, provide pointers to the silence buffer.

                            let silence = self.silence.as_ptr() as *mut f32;

                            let bus_data = &self.buses[bus_index];
                            self.ptrs[bus_data.start..bus_data.end].fill(silence);
                        }

                        input_index += 1;
                    }
                    BusDir::Out => {
                        if outputs_active[output_index] {
                            let channel_count = outputs[output_index].numChannels as usize;
                            if channel_count != config.layout.formats[bus_index].channel_count() {
                                return Err(());
                            }

                            let channels = slice_from_raw_parts_checked(
                                outputs[output_index].__field0.channelBuffers32,
                                channel_count,
                            );

                            let bus_data = &self.buses[bus_index];
                            self.ptrs[bus_data.start..bus_data.end].copy_from_slice(channels);
                        } else {
                            // For inactive output buses, allocate a scratch buffer for each
                            // channel.

                            let bus_data = &self.buses[bus_index];
                            for ptr in &mut self.ptrs[bus_data.start..bus_data.end] {
                                let (first, rest) = scratch.split_at_mut(config.max_buffer_size);
                                scratch = rest;

                                *ptr = first.as_mut_ptr();
                            }
                        }

                        output_index += 1;
                    }
                }
            }

            // Detect input buffers which are aliased by an output buffer, and copy each aliased
            // input to a scratch buffer.
            //
            // We do this by sorting the host-provided input and output pointers as integers, then
            // iterating through both sorted arrays in tandem and checking if any input pointer is
            // equal to any output pointer.

            self.inputs_sorted.clear();
            for (bus_index, active) in input_bus_map.iter().zip(inputs_active.iter()) {
                if *active {
                    let bus_data = &self.buses[*bus_index];
                    self.inputs_sorted.extend(bus_data.start..bus_data.end);
                }
            }
            self.inputs_sorted.sort_unstable_by_key(|i| self.ptrs[*i]);

            self.outputs_sorted.clear();
            for (bus_index, active) in output_bus_map.iter().zip(outputs_active.iter()) {
                if *active {
                    let bus_data = &self.buses[*bus_index];
                    self.outputs_sorted.extend(bus_data.start..bus_data.end);
                }
            }
            self.outputs_sorted.sort_unstable_by_key(|i| self.ptrs[*i]);

            let mut inputs_sorted = self.inputs_sorted.iter().copied().peekable();
            let mut outputs_sorted = self.outputs_sorted.iter().copied().peekable();
            while let (Some(input), Some(output)) = (inputs_sorted.peek(), outputs_sorted.peek()) {
                match self.ptrs[*input].cmp(&(self.ptrs[*output])) {
                    Ordering::Less => {
                        inputs_sorted.next();
                    }
                    Ordering::Greater => {
                        outputs_sorted.next();
                    }
                    Ordering::Equal => {
                        let (first, rest) = scratch.split_at_mut(config.max_buffer_size);
                        scratch = rest;

                        let input_slice = slice::from_raw_parts(self.ptrs[*input], len);
                        first[0..len].copy_from_slice(input_slice);
                        self.ptrs[*input] = first.as_mut_ptr();
                    }
                }
            }

            self.inputs_sorted.clear();
            self.outputs_sorted.clear();
        }

        Ok(Buffers::from_raw_parts(&self.buses, &self.ptrs, len))
    }
}

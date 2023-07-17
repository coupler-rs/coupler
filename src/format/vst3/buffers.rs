use std::iter::zip;
use std::ptr::NonNull;
use std::slice;

use vst3_bindgen::Steinberg::Vst::ProcessData;

use super::util::slice_from_raw_parts_checked;
use crate::buffers::{Buffers, BusData};
use crate::bus::{BusDir, BusInfo};
use crate::Config;

pub struct ScratchBuffers {
    buses: Vec<BusData>,
    ptrs: Vec<*mut f32>,
    buffers: Vec<f32>,
    silence: Vec<f32>,
    output_ptrs: Vec<*mut f32>,
}

impl ScratchBuffers {
    pub fn new() -> ScratchBuffers {
        ScratchBuffers {
            buses: Vec::new(),
            ptrs: Vec::new(),
            buffers: Vec::new(),
            silence: Vec::new(),
            output_ptrs: Vec::new(),
        }
    }

    pub fn resize(&mut self, buses: &[BusInfo], config: &Config) {
        self.buses.clear();
        let mut total_channels = 0;
        let mut output_channels = 0;
        for (info, format) in zip(buses, &config.layout.formats) {
            let channel_count = format.channel_count();

            self.buses.push(BusData {
                start: total_channels,
                end: total_channels + channel_count,
                dir: info.dir,
            });

            total_channels += channel_count;
            if info.dir == BusDir::Out || info.dir == BusDir::InOut {
                output_channels += channel_count;
            }
        }

        // Each input buffer can be aliased by an output buffer, and each output buffer can belong
        // to an inactive output bus.
        let scratch_space = config.max_buffer_size * total_channels;
        self.buffers.resize(scratch_space, 0.0);

        // Silence buffer, to be used for inactive input buses
        self.silence.resize(config.max_buffer_size, 0.0);

        self.output_ptrs.clear();
        self.output_ptrs.reserve(output_channels);

        self.ptrs.resize(total_channels, NonNull::dangling().as_ptr());
    }

    pub unsafe fn get_buffers(
        &mut self,
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
            return Ok(Buffers::from_raw_parts(&self.buses, &self.ptrs, len));
        }

        let input_count = data.numInputs as usize;
        let output_count = data.numOutputs as usize;
        if input_count != input_bus_map.len() || output_count != output_bus_map.len() {
            return Err(());
        }

        let inputs = slice_from_raw_parts_checked(data.inputs, input_count);
        let outputs = slice_from_raw_parts_checked(data.outputs, output_count);

        // Validate that the host has provided us with the correct number of channels for each bus.
        for (&bus_index, input) in zip(input_bus_map, inputs) {
            if input.numChannels as usize != config.layout.formats[bus_index].channel_count() {
                return Err(());
            }
        }
        for (&bus_index, output) in zip(output_bus_map, outputs) {
            if output.numChannels as usize != config.layout.formats[bus_index].channel_count() {
                return Err(());
            }
        }

        // Set up output pointers.
        for (output_index, &bus_index) in output_bus_map.iter().enumerate() {
            let bus_data = &self.buses[bus_index];
            if outputs_active[output_index] {
                let output = &outputs[output_index];
                let channels = slice_from_raw_parts_checked(
                    output.__field0.channelBuffers32,
                    output.numChannels as usize,
                );

                self.ptrs[bus_data.start..bus_data.end].copy_from_slice(channels);
            } else {
                // For inactive output buses, allocate a scratch buffer for each channel.
                for ptr in &mut self.ptrs[bus_data.start..bus_data.end] {
                    let (first, rest) = scratch.split_at_mut(len);
                    scratch = rest;

                    *ptr = first.as_mut_ptr();
                }
            }
        }

        // Set up input pointers.
        let has_inputs = self.buses.iter().any(|bus| bus.dir == BusDir::In);
        if has_inputs {
            // Build a sorted list of output pointers, so that we can check for each input pointer
            // whether it is aliased by an output pointer.
            self.output_ptrs.clear();
            for (output_index, output) in outputs.iter().enumerate() {
                if outputs_active[output_index] {
                    let channels = slice_from_raw_parts_checked(
                        output.__field0.channelBuffers32,
                        output.numChannels as usize,
                    );
                    self.output_ptrs.extend_from_slice(&channels);
                }
            }
            self.output_ptrs.sort_unstable();

            for (input_index, &bus_index) in input_bus_map.iter().enumerate() {
                let bus_data = &self.buses[bus_index];
                if bus_data.dir == BusDir::In {
                    if inputs_active[input_index] {
                        let input = &inputs[input_index];
                        let channels = slice_from_raw_parts_checked(
                            input.__field0.channelBuffers32,
                            input.numChannels as usize,
                        );

                        let ptrs = &mut self.ptrs[bus_data.start..bus_data.end];
                        for (channel, ptr) in zip(channels, ptrs) {
                            // If an input buffer is aliased by some output buffer, copy its
                            // contents to a scratch buffer.
                            if self.output_ptrs.binary_search(channel).is_err() {
                                *ptr = *channel;
                            } else {
                                let (first, rest) = scratch.split_at_mut(len);
                                scratch = rest;

                                let input_slice = slice::from_raw_parts(*channel, len);
                                first.copy_from_slice(input_slice);
                                *ptr = first.as_mut_ptr();
                            }
                        }
                    } else {
                        // For inactive input buses, provide pointers to the silence buffer.
                        let silence = self.silence.as_ptr() as *mut f32;
                        self.ptrs[bus_data.start..bus_data.end].fill(silence);
                    }
                }
            }

            self.output_ptrs.clear();
        }

        // For in-out buses, copy input buffers to corresponding output buffers where necessary.
        //
        // TODO: Detect the case where input buffers are aliased by non-corresponding output
        // buffers. This can happen when the host is attempting to do in-place processing but the
        // host and the plugin disagree on which input channels map to which output channels.
        //
        // When this is the case, we have to be careful to perform copies in the correct order such
        // that inputs don't get overwritten. In the general case, this requires scratch space.
        for (input_index, &bus_index) in input_bus_map.iter().enumerate() {
            let bus_data = &self.buses[bus_index];
            if bus_data.dir == BusDir::InOut {
                if inputs_active[input_index] {
                    let input = &inputs[input_index];
                    let channels = slice_from_raw_parts_checked(
                        input.__field0.channelBuffers32,
                        input.numChannels as usize,
                    );

                    let ptrs = &self.ptrs[bus_data.start..bus_data.end];
                    for (src, dst) in zip(channels, ptrs) {
                        if src != dst {
                            let src = slice::from_raw_parts(*src, len);
                            let dst = slice::from_raw_parts_mut(*dst, len);
                            dst.copy_from_slice(src);
                        }
                    }
                } else {
                    for dst in &self.ptrs[bus_data.start..bus_data.end] {
                        let dst = slice::from_raw_parts_mut(*dst, len);
                        dst.fill(0.0);
                    }
                }
            }
        }

        Ok(Buffers::from_raw_parts(&self.buses, &self.ptrs, len))
    }
}

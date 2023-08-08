use std::iter::zip;
use std::ptr::NonNull;
use std::slice;

use vst3::Steinberg::Vst::ProcessData;

use super::util::slice_from_raw_parts_checked;
use crate::buffers::{Buffers, BusData};
use crate::bus::{BusDir, BusInfo};
use crate::Config;

pub struct ScratchBuffers {
    inputs_active: Vec<bool>,
    outputs_active: Vec<bool>,
    buses: Vec<BusData>,
    ptrs: Vec<*mut f32>,
    buffers: Vec<f32>,
    silence: Vec<f32>,
    output_ptrs: Vec<*mut f32>,
    moves: Vec<(*const f32, *mut f32)>,
}

impl ScratchBuffers {
    pub fn new(input_count: usize, output_count: usize) -> ScratchBuffers {
        ScratchBuffers {
            inputs_active: vec![true; input_count],
            outputs_active: vec![true; output_count],
            buses: Vec::new(),
            ptrs: Vec::new(),
            buffers: Vec::new(),
            silence: Vec::new(),
            output_ptrs: Vec::new(),
            moves: Vec::new(),
        }
    }

    pub fn set_input_active(&mut self, index: usize, active: bool) {
        self.inputs_active[index] = active;
    }

    pub fn set_output_active(&mut self, index: usize, active: bool) {
        self.outputs_active[index] = active;
    }

    pub fn resize(&mut self, buses: &[BusInfo], config: &Config) {
        self.buses.clear();
        let mut total_channels = 0;
        let mut output_channels = 0;
        let mut in_out_channels = 0;
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
            if info.dir == BusDir::InOut {
                in_out_channels += channel_count;
            }
        }

        self.ptrs.resize(total_channels, NonNull::dangling().as_ptr());

        // Each input buffer can be aliased by an output buffer, each output buffer can belong to an
        // inactive bus, and each input provided to an in-out bus might need to be copied to
        // scratch space temporarily while copying inputs to outputs.
        let scratch_space = config.max_buffer_size * (total_channels + in_out_channels);
        self.buffers.resize(scratch_space, 0.0);

        // Silence buffer, to be used for inactive input buses
        self.silence.resize(config.max_buffer_size, 0.0);

        self.output_ptrs.clear();
        self.output_ptrs.reserve(output_channels);

        self.moves.clear();
        self.moves.reserve(in_out_channels);
    }

    pub unsafe fn get_buffers(
        &mut self,
        input_bus_map: &[usize],
        output_bus_map: &[usize],
        config: &Config,
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
        self.output_ptrs.clear();
        for (output_index, &bus_index) in output_bus_map.iter().enumerate() {
            let bus_data = &self.buses[bus_index];
            if self.outputs_active[output_index] {
                let output = &outputs[output_index];
                let channels = slice_from_raw_parts_checked(
                    output.__field0.channelBuffers32,
                    output.numChannels as usize,
                );

                self.ptrs[bus_data.start..bus_data.end].copy_from_slice(channels);
                self.output_ptrs.extend_from_slice(&channels);
            } else {
                // For inactive output buses, allocate a scratch buffer for each channel.
                for ptr in &mut self.ptrs[bus_data.start..bus_data.end] {
                    let (first, rest) = scratch.split_at_mut(len);
                    scratch = rest;

                    *ptr = first.as_mut_ptr();
                }
            }
        }

        // Sort the list of output pointers so that we can use binary search to check if input
        // pointers are aliased by output pointers.
        self.output_ptrs.sort_unstable();

        // Set up input pointers.
        for (input_index, &bus_index) in input_bus_map.iter().enumerate() {
            let bus_data = &self.buses[bus_index];
            if bus_data.dir == BusDir::In {
                if self.inputs_active[input_index] {
                    let input = &inputs[input_index];
                    let channels = slice_from_raw_parts_checked(
                        input.__field0.channelBuffers32,
                        input.numChannels as usize,
                    );

                    let ptrs = &mut self.ptrs[bus_data.start..bus_data.end];
                    for (&channel, ptr) in zip(channels, ptrs) {
                        // If an input buffer is aliased by some output buffer, copy its contents to
                        // a scratch buffer.
                        if self.output_ptrs.binary_search(&channel).is_ok() {
                            let (first, rest) = scratch.split_at_mut(len);
                            scratch = rest;

                            let input_slice = slice::from_raw_parts(channel, len);
                            first.copy_from_slice(input_slice);
                            *ptr = first.as_mut_ptr();
                        } else {
                            *ptr = channel;
                        }
                    }
                } else {
                    // For inactive input buses, provide pointers to the silence buffer.
                    let silence = self.silence.as_ptr() as *mut f32;
                    self.ptrs[bus_data.start..bus_data.end].fill(silence);
                }
            }
        }

        // If the host has passed us separate input and output buffers for an in-out bus, copy
        // inputs to outputs.
        self.moves.clear();
        for (input_index, &bus_index) in input_bus_map.iter().enumerate() {
            let bus_data = &self.buses[bus_index];
            if bus_data.dir == BusDir::InOut {
                if self.inputs_active[input_index] {
                    let input = &inputs[input_index];
                    let channels = slice_from_raw_parts_checked(
                        input.__field0.channelBuffers32,
                        input.numChannels as usize,
                    );

                    let ptrs = &self.ptrs[bus_data.start..bus_data.end];
                    for (&src, &dst) in zip(channels, ptrs) {
                        // Only perform a copy if input and output pointers are not equal.
                        if src != dst {
                            // If an input buffer is aliased by an output buffer, we might overwrite
                            // it when performing copies, so save its contents in a scratch
                            // buffer.
                            if self.output_ptrs.binary_search(&src).is_ok() {
                                let (first, rest) = scratch.split_at_mut(len);
                                scratch = rest;

                                let input_slice = slice::from_raw_parts(src, len);
                                first.copy_from_slice(input_slice);
                                self.moves.push((first.as_ptr(), dst));
                            } else {
                                self.moves.push((src, dst));
                            }
                        }
                    }
                } else {
                    // For inactive input buses, copy from the silence buffer.
                    for &dst in &self.ptrs[bus_data.start..bus_data.end] {
                        self.moves.push((self.silence.as_ptr(), dst));
                    }
                }
            }
        }

        // Now that any aliased input buffers have been copied to scratch space, actually perform
        // the copies.
        for (src, dst) in self.moves.drain(..) {
            let src = slice::from_raw_parts(src, len);
            let dst = slice::from_raw_parts_mut(dst, len);
            dst.copy_from_slice(src);
        }

        self.output_ptrs.clear();

        Ok(Buffers::from_raw_parts(&self.buses, &self.ptrs, len))
    }
}

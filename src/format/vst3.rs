#![allow(non_snake_case)]

// use crate::process::{Event, ProcessContext, *};
// use crate::{buffer::*, bus::*, editor::*, param::*, plugin::*};

// use std::cell::{Cell, UnsafeCell};
use std::collections::HashSet;
// use std::ffi::{c_void, CStr};
// use std::marker::PhantomData;
// use std::os::raw::{c_char, c_int};
// use std::rc::Rc;
use std::sync::atomic::Ordering;
use std::sync::Arc;
// use std::{io, ptr, slice};
use std::cell::UnsafeCell;
use std::ffi::{c_void, CStr};
use std::marker::PhantomData;
use std::{ptr, slice};

// use raw_window_handle::RawWindowHandle;

// use vst3_sys::{BusInfo, *};
use vst3_bindgen::{uid, Class, ComWrapper, Steinberg::Vst::*, Steinberg::*};

use super::util::{self, copy_cstring};
use crate::atomic::AtomicBitset;
use crate::bus::{BusConfig, BusConfigList, BusFormat, BusList, BusState};
use crate::plugin::{Plugin, PluginHandle, PluginInfo};

// macro_rules! offset_of {
//     ($struct:ty, $field:ident) => {{
//         let dummy = std::mem::MaybeUninit::<$struct>::uninit();
//         let base = dummy.as_ptr();
//         let field = std::ptr::addr_of!((*base).$field);

//         (field as *const c_void).offset_from(base as *const c_void)
//     }};
// }

fn copy_wstring(src: &str, dst: &mut [TChar]) {
    let mut len = 0;
    for (src, dst) in src.encode_utf16().zip(dst.iter_mut()) {
        *dst = src as TChar;
        len += 1;
    }

    if len < dst.len() {
        dst[len] = 0;
    } else if let Some(last) = dst.last_mut() {
        *last = 0;
    }
}

unsafe fn len_wstring(string: *const i16) -> usize {
    let mut len = 0;

    while *string.offset(len) != 0 {
        len += 1;
    }

    len as usize
}

fn bus_format_to_speaker_arrangement(bus_format: &BusFormat) -> SpeakerArrangement {
    match bus_format {
        BusFormat::Stereo => SpeakerArr::kStereo,
    }
}

fn speaker_arrangement_to_bus_format(speaker_arrangement: SpeakerArrangement) -> Option<BusFormat> {
    match speaker_arrangement {
        SpeakerArr::kStereo => Some(BusFormat::Stereo),
        _ => None,
    }
}

// struct Vst3EditorContext<P> {
//     component_handler: Cell<Option<*mut *const IComponentHandler>>,
//     plug_frame: Cell<Option<*mut *const IPlugFrame>>,
//     plugin: PluginHandle<P>,
//     param_states: Arc<ParamStates>,
// }

// impl<P> Drop for Vst3EditorContext<P> {
//     fn drop(&mut self) {
//         if let Some(handler) = self.component_handler.take() {
//             unsafe {
//                 ((*(*handler)).unknown.release)(handler as *mut c_void);
//             }
//         }

//         if let Some(frame) = self.plug_frame.take() {
//             unsafe {
//                 ((*(*frame)).unknown.release)(frame as *mut c_void);
//             }
//         }
//     }
// }

// impl<P> EditorContextHandler<P> for Vst3EditorContext<P> {
//     fn begin_edit(&self, id: ParamId) {
//         let _ = PluginHandle::params(&self.plugin)
//             .index_of(id)
//             .expect("Invalid parameter id");

//         if let Some(component_handler) = self.component_handler.get() {
//             unsafe {
//                 ((*(*component_handler)).begin_edit)(component_handler as *mut c_void, id);
//             }
//         }
//     }

//     fn perform_edit(&self, id: ParamId, value: f64) {
//         let param_index = PluginHandle::params(&self.plugin)
//             .index_of(id)
//             .expect("Invalid parameter id");
//         let param_info = &PluginHandle::params(&self.plugin).params()[param_index];

//         param_info.get_accessor().set(&self.plugin, value);

//         let value_normalized = param_info.get_mapping().unmap(value);

//         if let Some(component_handler) = self.component_handler.get() {
//             unsafe {
//                 ((*(*component_handler)).perform_edit)(
//                     component_handler as *mut c_void,
//                     id,
//                     value_normalized,
//                 );
//             }
//         }

//         self.param_states
//             .dirty_processor
//             .set(param_index, Ordering::Release);
//     }

//     fn end_edit(&self, id: ParamId) {
//         let _ = PluginHandle::params(&self.plugin)
//             .index_of(id)
//             .expect("Invalid parameter id");

//         if let Some(component_handler) = self.component_handler.get() {
//             unsafe {
//                 ((*(*component_handler)).end_edit)(component_handler as *mut c_void, id);
//             }
//         }
//     }

//     fn poll_params(&self) -> PollParams<P> {
//         PollParams {
//             iter: self
//                 .param_states
//                 .dirty_editor
//                 .drain_indices(Ordering::Acquire),
//             param_list: PluginHandle::params(&self.plugin),
//         }
//     }
// }

struct BusStates {
    inputs: Vec<BusState>,
    outputs: Vec<BusState>,
}

struct ParamStates {
    dirty_processor: AtomicBitset,
    dirty_editor: AtomicBitset,
}

// struct ProcessorState<P: Plugin> {
//     sample_rate: f64,
//     max_buffer_size: usize,
//     needs_reset: bool,
//     input_channels: usize,
//     input_indices: Vec<(usize, usize)>,
//     input_ptrs: Vec<*const f32>,
//     output_channels: usize,
//     output_indices: Vec<(usize, usize)>,
//     output_ptrs: Vec<*mut f32>,
//     // Scratch buffers for copying inputs to when the host uses the same
//     // buffers for inputs and outputs
//     scratch_buffers: Vec<f32>,
//     output_ptr_set: Vec<*mut f32>,
//     aliased_inputs: Vec<usize>,
//     events: Vec<Event>,
//     processor: Option<P::Processor>,
// }

// struct EditorState<P: Plugin> {
//     context: Rc<Vst3EditorContext<P>>,
//     editor: Option<P::Editor>,
// }

struct Wrapper<P: Plugin> {
    // has_editor: bool,
    bus_list: BusList,
    bus_config_list: BusConfigList,
    bus_config_set: HashSet<BusConfig>,
    // We only form an &mut to bus_states in set_bus_arrangements and
    // activate_bus, which aren't called concurrently with any other methods on
    // IComponent or IAudioProcessor per the spec.
    bus_states: UnsafeCell<BusStates>,
    param_states: Arc<ParamStates>,
    plugin: PluginHandle<P>,
    // processor_state: UnsafeCell<ProcessorState<P>>,
    // editor_state: UnsafeCell<EditorState<P>>,
}

impl<P: Plugin> Wrapper<P> {
    pub fn new(info: &PluginInfo) -> Wrapper<P> {
        let bus_list = P::buses();
        let bus_config_list = P::bus_configs();

        util::validate_bus_configs(&bus_list, &bus_config_list);

        let bus_config_set = bus_config_list
            .get_configs()
            .iter()
            .cloned()
            .collect::<HashSet<BusConfig>>();

        let default_config = bus_config_list.get_default().unwrap();

        let mut inputs = Vec::with_capacity(bus_list.get_inputs().len());
        for format in default_config.get_inputs() {
            inputs.push(BusState::new(format.clone(), true));
        }

        let mut outputs = Vec::with_capacity(bus_list.get_outputs().len());
        for format in default_config.get_outputs() {
            outputs.push(BusState::new(format.clone(), true));
        }

        let bus_states = UnsafeCell::new(BusStates { inputs, outputs });

        let plugin = PluginHandle::<P>::new();

        let param_count = PluginHandle::params(&plugin).params().len();

        let dirty_processor = AtomicBitset::with_len(param_count);
        let dirty_editor = AtomicBitset::with_len(param_count);
        let param_states = Arc::new(ParamStates {
            dirty_processor,
            dirty_editor,
        });

        //         let input_indices = Vec::with_capacity(bus_list.get_inputs().len());
        //         let input_ptrs = Vec::new();

        //         let output_indices = Vec::with_capacity(bus_list.get_outputs().len());
        //         let output_ptrs = Vec::new();

        //         let processor_state = UnsafeCell::new(ProcessorState {
        //             sample_rate: 0.0,
        //             max_buffer_size: 0,
        //             needs_reset: false,
        //             input_channels: 0,
        //             input_indices,
        //             input_ptrs,
        //             output_channels: 0,
        //             output_indices,
        //             output_ptrs,
        //             scratch_buffers: Vec::new(),
        //             output_ptr_set: Vec::new(),
        //             aliased_inputs: Vec::new(),
        //             // We can't know the maximum number of param changes in a
        //             // block, so make a reasonable guess and hope we don't have to
        //             // allocate more
        //             events: Vec::with_capacity(1024 + 4 * param_count),
        //             processor: None,
        //         });

        //         let editor_context = Rc::new(Vst3EditorContext {
        //             component_handler: Cell::new(None),
        //             plug_frame: Cell::new(None),
        //             plugin: plugin.clone(),
        //             param_states: param_states.clone(),
        //         });

        //         let editor_state = UnsafeCell::new(EditorState {
        //             context: editor_context,
        //             editor: None,
        //         });

        Wrapper {
            // has_editor: info.get_has_editor(),
            bus_list,
            bus_config_list,
            bus_config_set,
            bus_states,
            param_states,
            plugin,
            // processor_state,
            // editor_state,
        }
    }
}

impl<P: Plugin> Class for Wrapper<P> {
    type Interfaces = (
        IPluginBase,
        IComponent,
        IAudioProcessor,
        IProcessContextRequirements,
        IEditController,
    );
}

impl<P: Plugin> IPluginBaseTrait for Wrapper<P> {
    unsafe fn initialize(&self, _context: *mut FUnknown) -> tresult {
        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        kResultOk
    }
}

impl<P: Plugin> IComponentTrait for Wrapper<P> {
    unsafe fn getControllerClassId(&self, classId: *mut TUID) -> tresult {
        kNotImplemented
    }

    unsafe fn setIoMode(&self, mode: IoMode) -> tresult {
        kResultOk
    }

    unsafe fn getBusCount(&self, type_: MediaType, dir: BusDirection) -> int32 {
        match type_ as MediaTypes {
            MediaTypes_::kAudio => match dir as BusDirections {
                BusDirections_::kInput => self.bus_list.get_inputs().len() as int32,
                BusDirections_::kOutput => self.bus_list.get_outputs().len() as int32,
                _ => 0,
            },
            MediaTypes_::kEvent => 0,
            _ => 0,
        }
    }

    unsafe fn getBusInfo(
        &self,
        type_: MediaType,
        dir: BusDirection,
        index: int32,
        bus: *mut BusInfo,
    ) -> tresult {
        let bus_states = &*self.bus_states.get();

        match type_ as MediaTypes {
            MediaTypes_::kAudio => {
                let bus_info = match dir as BusDirections {
                    BusDirections_::kInput => self.bus_list.get_inputs().get(index as usize),
                    BusDirections_::kOutput => self.bus_list.get_outputs().get(index as usize),
                    _ => None,
                };

                let bus_state = match dir {
                    BusDirections_::kInput => bus_states.inputs.get(index as usize),
                    BusDirections_::kOutput => bus_states.outputs.get(index as usize),
                    _ => None,
                };

                if let (Some(bus_info), Some(bus_state)) = (bus_info, bus_state) {
                    let bus = &mut *bus;

                    bus.mediaType = MediaTypes_::kAudio as MediaType;
                    bus.direction = dir;
                    bus.channelCount = bus_state.format().channels() as int32;
                    copy_wstring(bus_info.get_name(), &mut bus.name);
                    bus.busType = if index == 0 {
                        BusTypes_::kMain as BusType
                    } else {
                        BusTypes_::kAux as BusType
                    };
                    bus.flags = BusInfo_::BusFlags_::kDefaultActive as uint32;

                    return kResultOk;
                }
            }
            MediaTypes_::kEvent => {}
            _ => {}
        }

        kInvalidArgument
    }

    unsafe fn getRoutingInfo(
        &self,
        inInfo: *mut RoutingInfo,
        outInfo: *mut RoutingInfo,
    ) -> tresult {
        kNotImplemented
    }

    unsafe fn activateBus(
        &self,
        type_: MediaType,
        dir: BusDirection,
        index: int32,
        state: TBool,
    ) -> tresult {
        let bus_states = &mut *self.bus_states.get();

        match type_ as MediaTypes {
            MediaTypes_::kAudio => {
                let bus_state = match dir as BusDirections {
                    BusDirections_::kInput => bus_states.inputs.get_mut(index as usize),
                    BusDirections_::kOutput => bus_states.outputs.get_mut(index as usize),
                    _ => None,
                };

                if let Some(bus_state) = bus_state {
                    bus_state.set_enabled(if state == 0 { false } else { true });
                    return kResultOk;
                }
            }
            MediaTypes_::kEvent => {}
            _ => {}
        }

        kInvalidArgument
    }

    unsafe fn setActive(&self, state: TBool) -> tresult {
        kResultOk
    }

    unsafe fn setState(&self, state: *mut IBStream) -> tresult {
        kResultOk
    }

    unsafe fn getState(&self, state: *mut IBStream) -> tresult {
        kResultOk
    }
}

impl<P: Plugin> IAudioProcessorTrait for Wrapper<P> {
    unsafe fn setBusArrangements(
        &self,
        inputs: *mut SpeakerArrangement,
        numIns: int32,
        outputs: *mut SpeakerArrangement,
        numOuts: int32,
    ) -> tresult {
        let bus_states = &mut *self.bus_states.get();

        if numIns as usize != self.bus_list.get_inputs().len()
            || numOuts as usize != self.bus_list.get_outputs().len()
        {
            return kResultFalse;
        }

        let mut candidate = BusConfig::new();

        // Don't use from_raw_parts for zero-length inputs, since the pointer
        // may be null or unaligned
        let inputs = if numIns > 0 {
            slice::from_raw_parts(inputs, numIns as usize)
        } else {
            &[]
        };
        for input in inputs {
            if let Some(bus_format) = speaker_arrangement_to_bus_format(*input) {
                candidate = candidate.input(bus_format);
            } else {
                return kResultFalse;
            }
        }

        // Don't use from_raw_parts for zero-length inputs, since the pointer
        // may be null or unaligned
        let outputs = if numOuts > 0 {
            slice::from_raw_parts(outputs, numOuts as usize)
        } else {
            &[]
        };
        for output in outputs {
            if let Some(bus_format) = speaker_arrangement_to_bus_format(*output) {
                candidate = candidate.output(bus_format);
            } else {
                return kResultFalse;
            }
        }

        if self.bus_config_set.contains(&candidate) {
            for (input, bus_state) in candidate
                .get_inputs()
                .iter()
                .zip(bus_states.inputs.iter_mut())
            {
                bus_state.set_format(input.clone());
            }

            for (output, bus_state) in candidate
                .get_outputs()
                .iter()
                .zip(bus_states.outputs.iter_mut())
            {
                bus_state.set_format(output.clone());
            }

            return kResultTrue;
        }

        kResultFalse
    }

    unsafe fn getBusArrangement(
        &self,
        dir: BusDirection,
        index: int32,
        arr: *mut SpeakerArrangement,
    ) -> tresult {
        let bus_states = &*self.bus_states.get();

        let bus_state = match dir as BusDirections {
            BusDirections_::kInput => bus_states.inputs.get(index as usize),
            BusDirections_::kOutput => bus_states.outputs.get(index as usize),
            _ => None,
        };

        if let Some(bus_state) = bus_state {
            *arr = bus_format_to_speaker_arrangement(bus_state.format());
            return kResultOk;
        }

        kInvalidArgument
    }

    unsafe fn canProcessSampleSize(&self, symbolicSampleSize: int32) -> tresult {
        match symbolicSampleSize as SymbolicSampleSizes {
            SymbolicSampleSizes_::kSample32 => kResultTrue,
            SymbolicSampleSizes_::kSample64 => kResultFalse,
            _ => kInvalidArgument,
        }
    }

    unsafe fn getLatencySamples(&self) -> uint32 {
        0
    }

    unsafe fn setupProcessing(&self, setup: *mut ProcessSetup) -> tresult {
        kResultOk
    }

    unsafe fn setProcessing(&self, state: TBool) -> tresult {
        kResultOk
    }

    unsafe fn process(&self, data: *mut ProcessData) -> tresult {
        kResultOk
    }

    unsafe fn getTailSamples(&self) -> uint32 {
        kInfiniteTail
    }
}

impl<P: Plugin> IProcessContextRequirementsTrait for Wrapper<P> {
    unsafe fn getProcessContextRequirements(&self) -> uint32 {
        0
    }
}

impl<P: Plugin> IEditControllerTrait for Wrapper<P> {
    unsafe fn setComponentState(&self, state: *mut IBStream) -> tresult {
        kResultOk
    }

    unsafe fn setState(&self, state: *mut IBStream) -> tresult {
        kResultOk
    }

    unsafe fn getState(&self, state: *mut IBStream) -> tresult {
        kResultOk
    }

    unsafe fn getParameterCount(&self) -> int32 {
        PluginHandle::params(&self.plugin).params().len() as int32
    }

    unsafe fn getParameterInfo(&self, paramIndex: int32, info: *mut ParameterInfo) -> tresult {
        let params = PluginHandle::params(&self.plugin);
        if let Some(param_info) = params.params().get(paramIndex as usize) {
            let info = &mut *info;

            info.id = param_info.get_id();
            copy_wstring(&param_info.get_name(), &mut info.title);
            copy_wstring(&param_info.get_name(), &mut info.shortTitle);
            copy_wstring(&param_info.get_label(), &mut info.units);
            info.stepCount = if let Some(steps) = param_info.get_steps() {
                (steps.max(2) - 1) as i32
            } else {
                0
            };
            info.defaultNormalizedValue = param_info.get_mapping().unmap(param_info.get_default());
            info.unitId = 0;
            info.flags = ParameterInfo_::ParameterFlags_::kCanAutomate as int32;

            kResultOk
        } else {
            kInvalidArgument
        }
    }

    unsafe fn getParamStringByValue(
        &self,
        id: ParamID,
        valueNormalized: ParamValue,
        string: *mut String128,
    ) -> tresult {
        if let Some(param_info) = PluginHandle::params(&self.plugin).get(id) {
            let mut display = String::new();
            let value = param_info.get_mapping().map(valueNormalized);
            param_info.get_format().display(value, &mut display);
            copy_wstring(&display, &mut *string);

            return kResultOk;
        }

        kInvalidArgument
    }

    unsafe fn getParamValueByString(
        &self,
        id: ParamID,
        string: *mut TChar,
        valueNormalized: *mut ParamValue,
    ) -> tresult {
        if let Some(param_info) = PluginHandle::params(&self.plugin).get(id) {
            let len = len_wstring(string);
            if let Ok(string) = String::from_utf16(slice::from_raw_parts(string as *const u16, len))
            {
                if let Ok(value) = param_info.get_format().parse(&string) {
                    *valueNormalized = param_info.get_mapping().unmap(value);
                    return kResultOk;
                }
            }
        }

        kInvalidArgument
    }

    unsafe fn normalizedParamToPlain(
        &self,
        id: ParamID,
        valueNormalized: ParamValue,
    ) -> ParamValue {
        if let Some(param_info) = PluginHandle::params(&self.plugin).get(id) {
            return param_info.get_mapping().map(valueNormalized);
        }

        0.0
    }

    unsafe fn plainParamToNormalized(&self, id: ParamID, plainValue: ParamValue) -> ParamValue {
        if let Some(param_info) = PluginHandle::params(&self.plugin).get(id) {
            return param_info.get_mapping().unmap(plainValue);
        }

        0.0
    }

    unsafe fn getParamNormalized(&self, id: ParamID) -> ParamValue {
        if let Some(param_info) = PluginHandle::params(&self.plugin).get(id) {
            let value = param_info.get_accessor().get(&self.plugin);
            return param_info.get_mapping().unmap(value);
        }

        0.0
    }

    unsafe fn setParamNormalized(&self, id: ParamID, value: ParamValue) -> tresult {
        if let Some(param_info) = PluginHandle::params(&self.plugin).get(id) {
            let param_index = PluginHandle::params(&self.plugin).index_of(id).unwrap();

            let value = param_info.get_mapping().map(value);
            param_info.get_accessor().set(&self.plugin, value);

            self.param_states
                .dirty_processor
                .set(param_index, Ordering::Release);
            self.param_states
                .dirty_editor
                .set(param_index, Ordering::Release);

            return kResultOk;
        }

        kInvalidArgument
    }

    unsafe fn setComponentHandler(&self, handler: *mut IComponentHandler) -> tresult {
        kResultOk
    }

    unsafe fn createView(&self, name: FIDString) -> *mut IPlugView {
        ptr::null_mut()
    }
}

//     unsafe extern "system" fn set_active(this: *mut c_void, state: TBool) -> TResult {
//         let wrapper = &*(this.offset(-offset_of!(Self, component)) as *const Wrapper<P>);
//         let bus_states = &mut *wrapper.bus_states.get();
//         let processor_state = &mut *wrapper.processor_state.get();

//         match state {
//             0 => {
//                 processor_state.processor = None;
//             }
//             _ => {
//                 let context = ProcessContext::new(
//                     processor_state.sample_rate,
//                     processor_state.max_buffer_size,
//                     &bus_states.inputs[..],
//                     &bus_states.outputs[..],
//                 );
//                 processor_state.processor =
//                     Some(P::Processor::create(wrapper.plugin.clone(), &context));

//                 // Prepare buffer indices and ensure that buffer pointer Vecs are the correct size:

//                 processor_state.input_indices.clear();
//                 let mut total_channels = 0;
//                 for bus_state in bus_states.inputs.iter() {
//                     let channels = if bus_state.enabled() {
//                         bus_state.format().channels()
//                     } else {
//                         0
//                     };
//                     processor_state
//                         .input_indices
//                         .push((total_channels, total_channels + channels));
//                     total_channels += channels;
//                 }
//                 processor_state.input_channels = total_channels;

//                 processor_state
//                     .input_ptrs
//                     .reserve(processor_state.input_channels);
//                 processor_state
//                     .input_ptrs
//                     .shrink_to(processor_state.input_channels);

//                 processor_state.output_indices.clear();
//                 let mut total_channels = 0;
//                 for bus_state in bus_states.outputs.iter() {
//                     let channels = if bus_state.enabled() {
//                         bus_state.format().channels()
//                     } else {
//                         0
//                     };
//                     processor_state
//                         .output_indices
//                         .push((total_channels, total_channels + channels));
//                     total_channels += channels;
//                 }
//                 processor_state.output_channels = total_channels;

//                 processor_state
//                     .output_ptrs
//                     .reserve(processor_state.output_channels);
//                 processor_state
//                     .output_ptrs
//                     .shrink_to(processor_state.output_channels);

//                 // Ensure enough scratch buffer space for any number of aliased input buffers:

//                 let scratch_buffer_size = processor_state.max_buffer_size
//                     * processor_state
//                         .input_channels
//                         .min(processor_state.output_channels);
//                 processor_state.scratch_buffers.reserve(scratch_buffer_size);
//                 processor_state
//                     .scratch_buffers
//                     .shrink_to(scratch_buffer_size);

//                 processor_state
//                     .output_ptr_set
//                     .reserve(processor_state.output_channels);
//                 processor_state
//                     .output_ptr_set
//                     .shrink_to(processor_state.output_channels);

//                 processor_state
//                     .aliased_inputs
//                     .reserve(processor_state.input_channels);
//                 processor_state
//                     .aliased_inputs
//                     .shrink_to(processor_state.input_channels);
//             }
//         }

//         result::OK
//     }

//     unsafe extern "system" fn component_set_state(
//         this: *mut c_void,
//         state: *mut *const IBStream,
//     ) -> TResult {
//         struct StreamReader(*mut *const IBStream);

//         impl io::Read for StreamReader {
//             fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
//                 let mut bytes: i32 = 0;
//                 let result = unsafe {
//                     ((*(*self.0)).read)(
//                         self.0 as *mut c_void,
//                         buf.as_mut_ptr() as *mut c_void,
//                         buf.len() as i32,
//                         &mut bytes,
//                     )
//                 };

//                 if result == result::OK {
//                     Ok(bytes as usize)
//                 } else {
//                     Err(io::Error::new(
//                         io::ErrorKind::Other,
//                         "Failed to read from stream",
//                     ))
//                 }
//             }
//         }

//         let wrapper = &*(this.offset(-offset_of!(Self, component)) as *const Wrapper<P>);

//         match wrapper.plugin.deserialize(&mut StreamReader(state)) {
//             Ok(_) => {
//                 wrapper
//                     .param_states
//                     .dirty_processor
//                     .set_all(Ordering::Release);
//                 wrapper.param_states.dirty_editor.set_all(Ordering::Release);

//                 result::OK
//             }
//             Err(_) => result::FALSE,
//         }
//     }

//     unsafe extern "system" fn component_get_state(
//         this: *mut c_void,
//         state: *mut *const IBStream,
//     ) -> TResult {
//         struct StreamWriter(*mut *const IBStream);

//         impl io::Write for StreamWriter {
//             fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
//                 let mut bytes: i32 = 0;
//                 let result = unsafe {
//                     ((*(*self.0)).write)(
//                         self.0 as *mut c_void,
//                         buf.as_ptr() as *mut c_void,
//                         buf.len() as i32,
//                         &mut bytes,
//                     )
//                 };

//                 if result == result::OK {
//                     Ok(bytes as usize)
//                 } else {
//                     Err(io::Error::new(
//                         io::ErrorKind::Other,
//                         "Failed to write to stream",
//                     ))
//                 }
//             }

//             fn flush(&mut self) -> io::Result<()> {
//                 Ok(())
//             }
//         }

//         let wrapper = &*(this.offset(-offset_of!(Self, component)) as *const Wrapper<P>);

//         match wrapper.plugin.serialize(&mut StreamWriter(state)) {
//             Ok(_) => result::OK,
//             Err(_) => result::FALSE,
//         }
//     }

//     unsafe extern "system" fn setup_processing(
//         this: *mut c_void,
//         setup: *mut ProcessSetup,
//     ) -> TResult {
//         let wrapper = &*(this.offset(-offset_of!(Self, audio_processor)) as *const Wrapper<P>);
//         let processor_state = &mut *wrapper.processor_state.get();

//         let setup = &*setup;

//         processor_state.sample_rate = setup.sample_rate;
//         processor_state.max_buffer_size = setup.max_samples_per_block as usize;

//         result::OK
//     }

//     unsafe extern "system" fn set_processing(this: *mut c_void, state: TBool) -> TResult {
//         let wrapper = &*(this.offset(-offset_of!(Self, audio_processor)) as *const Wrapper<P>);
//         let bus_states = &*wrapper.bus_states.get();
//         let processor_state = &mut *wrapper.processor_state.get();

//         if processor_state.processor.is_none() {
//             return result::NOT_INITIALIZED;
//         }

//         if state != 0 {
//             // Don't need to call reset() the first time set_processing() is
//             // called with true.
//             if !processor_state.needs_reset {
//                 processor_state.needs_reset = true;
//                 return result::OK;
//             }

//             let context = ProcessContext::new(
//                 processor_state.sample_rate,
//                 processor_state.max_buffer_size,
//                 &bus_states.inputs[..],
//                 &bus_states.outputs[..],
//             );
//             processor_state.processor.as_mut().unwrap().reset(&context);
//         }

//         result::OK
//     }

//     unsafe extern "system" fn process(this: *mut c_void, data: *mut ProcessData) -> TResult {
//         let wrapper = &*(this.offset(-offset_of!(Self, audio_processor)) as *const Wrapper<P>);
//         let bus_states = &*wrapper.bus_states.get();
//         let processor_state = &mut *wrapper.processor_state.get();

//         if processor_state.processor.is_none() {
//             return result::NOT_INITIALIZED;
//         }

//         processor_state.events.clear();

//         for index in wrapper
//             .param_states
//             .dirty_processor
//             .drain_indices(Ordering::Acquire)
//         {
//             let param_info = &PluginHandle::params(&wrapper.plugin).params()[index];
//             let value = param_info.get_accessor().get(&wrapper.plugin);

//             processor_state.events.push(Event {
//                 offset: 0,
//                 event: EventType::ParamChange(ParamChange {
//                     id: param_info.get_id(),
//                     value,
//                 }),
//             });
//         }

//         let process_data = &*data;

//         let param_changes = process_data.input_parameter_changes;
//         if !param_changes.is_null() {
//             let param_count =
//                 ((*(*param_changes)).get_parameter_count)(param_changes as *mut c_void);
//             for index in 0..param_count {
//                 let param_data =
//                     ((*(*param_changes)).get_parameter_data)(param_changes as *mut c_void, index);

//                 if param_data.is_null() {
//                     continue;
//                 }

//                 let id = ((*(*param_data)).get_parameter_id)(param_data as *mut c_void);
//                 let point_count = ((*(*param_data)).get_point_count)(param_data as *mut c_void);

//                 if let Some(param_index) = PluginHandle::params(&wrapper.plugin).index_of(id) {
//                     for index in 0..point_count {
//                         let mut offset = 0;
//                         let mut value_normalized = 0.0;
//                         let result = ((*(*param_data)).get_point)(
//                             param_data as *mut c_void,
//                             index,
//                             &mut offset,
//                             &mut value_normalized,
//                         );

//                         if result != result::OK {
//                             continue;
//                         }

//                         let param_info =
//                             &PluginHandle::params(&wrapper.plugin).params()[param_index];
//                         let value = param_info.get_mapping().map(value_normalized);
//                         param_info.get_accessor().set(&wrapper.plugin, value);
//                         wrapper
//                             .param_states
//                             .dirty_editor
//                             .set(param_index, Ordering::Release);

//                         processor_state.events.push(Event {
//                             offset: offset as usize,
//                             event: EventType::ParamChange(ParamChange { id, value }),
//                         });
//                     }
//                 }
//             }
//         }

//         processor_state
//             .events
//             .sort_by_key(|param_change| param_change.offset);

//         processor_state.input_ptrs.clear();
//         processor_state.output_ptrs.clear();

//         let samples = process_data.num_samples as usize;

//         if samples > 0 {
//             if wrapper.bus_list.get_inputs().len() > 0 {
//                 if process_data.num_inputs as usize != wrapper.bus_list.get_inputs().len() {
//                     return result::INVALID_ARGUMENT;
//                 }

//                 let inputs =
//                     slice::from_raw_parts(process_data.inputs, process_data.num_inputs as usize);

//                 for (input, bus_state) in inputs.iter().zip(bus_states.inputs.iter()) {
//                     if !bus_state.enabled() || bus_state.format().channels() == 0 {
//                         continue;
//                     }

//                     if input.num_channels as usize != bus_state.format().channels() {
//                         return result::INVALID_ARGUMENT;
//                     }

//                     let channels = slice::from_raw_parts(
//                         input.channel_buffers as *const *const f32,
//                         input.num_channels as usize,
//                     );
//                     processor_state.input_ptrs.extend_from_slice(channels);
//                 }
//             }

//             if wrapper.bus_list.get_outputs().len() > 0 {
//                 if process_data.num_outputs as usize != wrapper.bus_list.get_outputs().len() {
//                     return result::INVALID_ARGUMENT;
//                 }

//                 let outputs =
//                     slice::from_raw_parts(process_data.outputs, process_data.num_outputs as usize);

//                 for (output, bus_state) in outputs.iter().zip(bus_states.outputs.iter()) {
//                     if !bus_state.enabled() || bus_state.format().channels() == 0 {
//                         continue;
//                     }

//                     if output.num_channels as usize != bus_state.format().channels() {
//                         return result::INVALID_ARGUMENT;
//                     }

//                     let channels = slice::from_raw_parts(
//                         output.channel_buffers as *const *mut f32,
//                         output.num_channels as usize,
//                     );
//                     processor_state.output_ptrs.extend_from_slice(channels);
//                 }
//             }

//             // Copy aliased input buffers into scratch buffers

//             processor_state
//                 .output_ptr_set
//                 .extend_from_slice(&processor_state.output_ptrs);
//             processor_state.output_ptr_set.sort();
//             processor_state.output_ptr_set.dedup();

//             for (channel, input_ptr) in processor_state.input_ptrs.iter().enumerate() {
//                 if processor_state
//                     .output_ptr_set
//                     .binary_search(&(*input_ptr as *mut f32))
//                     .is_ok()
//                 {
//                     processor_state.aliased_inputs.push(channel);

//                     let input_buffer = slice::from_raw_parts(*input_ptr, samples);
//                     processor_state
//                         .scratch_buffers
//                         .extend_from_slice(input_buffer);
//                 }
//             }

//             for (index, channel) in processor_state.aliased_inputs.iter().enumerate() {
//                 let offset = index * processor_state.max_buffer_size;
//                 let ptr = processor_state.scratch_buffers.as_ptr().add(offset) as *mut f32;
//                 processor_state.input_ptrs[*channel] = ptr;
//             }

//             processor_state.output_ptr_set.clear();
//             processor_state.aliased_inputs.clear();
//         } else {
//             processor_state
//                 .input_ptrs
//                 .resize(processor_state.input_channels, ptr::null());
//             processor_state
//                 .output_ptrs
//                 .resize(processor_state.output_channels, ptr::null_mut());
//         }

//         let buffers = Buffers::new(
//             samples,
//             &bus_states.inputs,
//             &processor_state.input_indices,
//             &processor_state.input_ptrs,
//             &bus_states.outputs,
//             &processor_state.output_indices,
//             &processor_state.output_ptrs,
//         );

//         let context = ProcessContext::new(
//             processor_state.sample_rate,
//             processor_state.max_buffer_size,
//             &bus_states.inputs[..],
//             &bus_states.outputs[..],
//         );

//         if let Some(processor) = &mut processor_state.processor {
//             processor.process(&context, buffers, &processor_state.events[..]);
//         }

//         processor_state.scratch_buffers.clear();

//         processor_state.input_ptrs.clear();
//         processor_state.output_ptrs.clear();

//         processor_state.events.clear();

//         result::OK
//     }

//     unsafe extern "system" fn set_component_state(
//         _this: *mut c_void,
//         _state: *mut *const IBStream,
//     ) -> TResult {
//         result::OK
//     }

//     unsafe extern "system" fn edit_controller_set_state(
//         _this: *mut c_void,
//         _state: *mut *const IBStream,
//     ) -> TResult {
//         result::OK
//     }

//     unsafe extern "system" fn edit_controller_get_state(
//         _this: *mut c_void,
//         _state: *mut *const IBStream,
//     ) -> TResult {
//         result::OK
//     }

//     unsafe extern "system" fn set_component_handler(
//         this: *mut c_void,
//         handler: *mut *const IComponentHandler,
//     ) -> TResult {
//         let wrapper = &*(this.offset(-offset_of!(Self, edit_controller)) as *const Wrapper<P>);
//         let editor_state = &*wrapper.editor_state.get();

//         if let Some(prev_handler) = editor_state.context.component_handler.take() {
//             ((*(*prev_handler)).unknown.release)(prev_handler as *mut c_void);
//         }

//         if !handler.is_null() {
//             ((*(*handler)).unknown.add_ref)(handler as *mut c_void);
//             editor_state.context.component_handler.set(Some(handler));
//         }

//         result::OK
//     }

//     unsafe extern "system" fn create_view(
//         this: *mut c_void,
//         name: *const c_char,
//     ) -> *mut *const IPlugView {
//         let wrapper = &*(this.offset(-offset_of!(Self, edit_controller)) as *const Wrapper<P>);

//         if !wrapper.has_editor {
//             return ptr::null_mut();
//         }

//         if CStr::from_ptr(name) == CStr::from_ptr(view_types::EDITOR) {
//             Self::add_ref(this.offset(-offset_of!(Self, edit_controller)));
//             return this.offset(-offset_of!(Self, edit_controller) + offset_of!(Self, plug_view))
//                 as *mut *const IPlugView;
//         }

//         ptr::null_mut()
//     }

//     unsafe extern "system" fn is_platform_type_supported(
//         _this: *mut c_void,
//         platform_type: *const c_char,
//     ) -> TResult {
//         #[cfg(target_os = "windows")]
//         if CStr::from_ptr(platform_type) == CStr::from_ptr(platform_types::HWND) {
//             return result::TRUE;
//         }

//         #[cfg(target_os = "macos")]
//         if CStr::from_ptr(platform_type) == CStr::from_ptr(platform_types::NS_VIEW) {
//             return result::TRUE;
//         }

//         #[cfg(target_os = "linux")]
//         if CStr::from_ptr(platform_type) == CStr::from_ptr(platform_types::X11_EMBED_WINDOW_ID) {
//             return result::TRUE;
//         }

//         result::FALSE
//     }

//     unsafe extern "system" fn attached(
//         this: *mut c_void,
//         parent: *mut c_void,
//         platform_type: *const c_char,
//     ) -> TResult {
//         if Self::is_platform_type_supported(this, platform_type) != result::TRUE {
//             return result::NOT_IMPLEMENTED;
//         }

//         let wrapper = &*(this.offset(-offset_of!(Self, plug_view)) as *const Wrapper<P>);
//         let editor_state = &mut *wrapper.editor_state.get();

//         #[cfg(target_os = "macos")]
//         let parent = {
//             use raw_window_handle::macos::MacOSHandle;
//             RawWindowHandle::MacOS(MacOSHandle {
//                 ns_view: parent,
//                 ..MacOSHandle::empty()
//             })
//         };

//         #[cfg(target_os = "windows")]
//         let parent = {
//             use raw_window_handle::windows::WindowsHandle;
//             RawWindowHandle::Windows(WindowsHandle {
//                 hwnd: parent,
//                 ..WindowsHandle::empty()
//             })
//         };

//         #[cfg(target_os = "linux")]
//         let parent = {
//             use raw_window_handle::unix::XcbHandle;
//             RawWindowHandle::Xcb(XcbHandle {
//                 window: parent as u32,
//                 ..XcbHandle::empty()
//             })
//         };

//         let context = EditorContext::new(editor_state.context.clone());

//         let editor = P::Editor::open(wrapper.plugin.clone(), context, Some(&ParentWindow(parent)));

//         #[cfg(target_os = "linux")]
//         {
//             let frame = editor_state.context.plug_frame.get();
//             if frame.is_none() {
//                 return result::NOT_INITIALIZED;
//             }
//             let frame = frame.unwrap();

//             let mut obj = ptr::null_mut();
//             let result = ((*(*frame)).unknown.query_interface)(
//                 frame as *mut c_void,
//                 &IRunLoop::IID,
//                 &mut obj,
//             );

//             if result == result::OK {
//                 let run_loop = obj as *mut *const IRunLoop;

//                 let timer_handler = this
//                     .offset(-offset_of!(Self, plug_view) + offset_of!(Self, timer_handler))
//                     as *mut *const ITimerHandler;
//                 ((*(*run_loop)).register_timer)(run_loop as *mut c_void, timer_handler, 16);

//                 if let Some(file_descriptor) = editor.file_descriptor() {
//                     let event_handler = this
//                         .offset(-offset_of!(Self, plug_view) + offset_of!(Self, event_handler))
//                         as *mut *const IEventHandler;
//                     ((*(*run_loop)).register_event_handler)(
//                         run_loop as *mut c_void,
//                         event_handler,
//                         file_descriptor,
//                     );
//                 }

//                 ((*(*run_loop)).unknown.release)(run_loop as *mut c_void);
//             }
//         }

//         editor_state.editor = Some(editor);

//         result::OK
//     }

//     unsafe extern "system" fn removed(this: *mut c_void) -> TResult {
//         let wrapper = &*(this.offset(-offset_of!(Self, plug_view)) as *const Wrapper<P>);
//         let editor_state = &mut *wrapper.editor_state.get();

//         if let Some(mut editor) = editor_state.editor.take() {
//             editor.close();
//         }

//         #[cfg(target_os = "linux")]
//         {
//             if let Some(frame) = editor_state.context.plug_frame.get() {
//                 let mut obj = ptr::null_mut();
//                 let result = ((*(*frame)).unknown.query_interface)(
//                     frame as *mut c_void,
//                     &IRunLoop::IID,
//                     &mut obj,
//                 );

//                 if result == result::OK {
//                     let run_loop = obj as *mut *const IRunLoop;

//                     let event_handler = this
//                         .offset(-offset_of!(Self, plug_view) + offset_of!(Self, event_handler))
//                         as *mut *const IEventHandler;
//                     ((*(*run_loop)).unregister_event_handler)(
//                         run_loop as *mut c_void,
//                         event_handler,
//                     );

//                     let timer_handler = this
//                         .offset(-offset_of!(Self, plug_view) + offset_of!(Self, timer_handler))
//                         as *mut *const ITimerHandler;
//                     ((*(*run_loop)).unregister_timer)(run_loop as *mut c_void, timer_handler);

//                     ((*(*run_loop)).unknown.release)(run_loop as *mut c_void);
//                 }
//             }
//         }

//         result::OK
//     }

//     unsafe extern "system" fn on_wheel(_this: *mut c_void, _distance: f32) -> TResult {
//         result::NOT_IMPLEMENTED
//     }

//     unsafe extern "system" fn on_key_down(
//         _this: *mut c_void,
//         _key: i16,
//         _key_code: i16,
//         _modifiers: i16,
//     ) -> TResult {
//         result::NOT_IMPLEMENTED
//     }

//     unsafe extern "system" fn on_key_up(
//         _this: *mut c_void,
//         _key: i16,
//         _key_code: i16,
//         _modifiers: i16,
//     ) -> TResult {
//         result::NOT_IMPLEMENTED
//     }

//     unsafe extern "system" fn get_size(_this: *mut c_void, size: *mut ViewRect) -> TResult {
//         let (width, height) = P::Editor::size();

//         let size = &mut *size;
//         size.top = 0;
//         size.left = 0;
//         size.right = width.round() as i32;
//         size.bottom = height.round() as i32;

//         result::OK
//     }

//     unsafe extern "system" fn on_size(_this: *mut c_void, _new_size: *const ViewRect) -> TResult {
//         result::NOT_IMPLEMENTED
//     }

//     unsafe extern "system" fn on_focus(_this: *mut c_void, _state: TBool) -> TResult {
//         result::NOT_IMPLEMENTED
//     }

//     unsafe extern "system" fn set_frame(
//         this: *mut c_void,
//         frame: *mut *const IPlugFrame,
//     ) -> TResult {
//         let wrapper = &*(this.offset(-offset_of!(Self, plug_view)) as *const Wrapper<P>);
//         let editor_state = &*wrapper.editor_state.get();

//         if let Some(prev_frame) = editor_state.context.plug_frame.take() {
//             ((*(*prev_frame)).unknown.release)(prev_frame as *mut c_void);
//         }

//         if !frame.is_null() {
//             ((*(*frame)).unknown.add_ref)(frame as *mut c_void);
//             editor_state.context.plug_frame.set(Some(frame));
//         }

//         result::OK
//     }

//     unsafe extern "system" fn can_resize(_this: *mut c_void) -> TResult {
//         result::FALSE
//     }

//     unsafe extern "system" fn check_size_constraint(
//         _this: *mut c_void,
//         _rect: *mut ViewRect,
//     ) -> TResult {
//         result::NOT_IMPLEMENTED
//     }

//     #[cfg(target_os = "linux")]
//     unsafe extern "system" fn on_fd_is_set(this: *mut c_void, _fd: c_int) {
//         let wrapper = &*(this.offset(-offset_of!(Self, event_handler)) as *const Wrapper<P>);
//         let editor_state = &mut *wrapper.editor_state.get();

//         if let Some(editor) = &mut editor_state.editor {
//             editor.poll();
//         }
//     }

//     #[cfg(not(target_os = "linux"))]
//     unsafe extern "system" fn on_fd_is_set(_this: *mut c_void, _fd: c_int) {}

//     #[cfg(target_os = "linux")]
//     unsafe extern "system" fn on_timer(this: *mut c_void) {
//         let wrapper = &*(this.offset(-offset_of!(Self, timer_handler)) as *const Wrapper<P>);
//         let editor_state = &mut *wrapper.editor_state.get();

//         if let Some(editor) = &mut editor_state.editor {
//             editor.poll();
//         }
//     }

//     #[cfg(not(target_os = "linux"))]
//     unsafe extern "system" fn on_timer(_this: *mut c_void) {}
// }

struct Factory<P> {
    vst3_info: Vst3Info,
    info: PluginInfo,
    _marker: PhantomData<P>,
}

impl<P: Plugin + Vst3Plugin> Factory<P> {
    pub fn new() -> Factory<P> {
        Factory {
            vst3_info: P::vst3_info(),
            info: P::info(),
            _marker: PhantomData,
        }
    }
}

impl<P: Plugin + Vst3Plugin> Class for Factory<P> {
    type Interfaces = (IPluginFactory3,);
}

impl<P: Plugin + Vst3Plugin> IPluginFactoryTrait for Factory<P> {
    unsafe fn getFactoryInfo(&self, info: *mut PFactoryInfo) -> tresult {
        let info = &mut *info;

        copy_cstring(&self.info.get_vendor(), &mut info.vendor);
        copy_cstring(&self.info.get_url(), &mut info.url);
        copy_cstring(&self.info.get_email(), &mut info.email);
        info.flags = PFactoryInfo_::FactoryFlags_::kUnicode as int32;

        kResultOk
    }

    unsafe fn countClasses(&self) -> int32 {
        1
    }

    unsafe fn getClassInfo(&self, index: int32, info: *mut PClassInfo) -> tresult {
        if index != 0 {
            return kInvalidArgument;
        }

        let info = &mut *info;

        info.cid = self.vst3_info.get_class_id().0;
        info.cardinality = PClassInfo_::ClassCardinality_::kManyInstances as int32;
        copy_cstring("Audio Module Class", &mut info.category);
        copy_cstring(&self.info.get_name(), &mut info.name);

        kResultOk
    }

    unsafe fn createInstance(
        &self,
        cid: FIDString,
        iid: FIDString,
        obj: *mut *mut c_void,
    ) -> tresult {
        let cid = &*(cid as *const TUID);
        if cid != &self.vst3_info.get_class_id().0 {
            return kInvalidArgument;
        }

        let wrapper = ComWrapper::new(Wrapper::<P>::new(&self.info));
        let unknown = wrapper.to_com_ptr::<FUnknown>().unwrap();
        let ptr = unknown.as_ptr();
        ((*(*ptr).vtbl).queryInterface)(ptr, iid as *const TUID, obj)
    }
}

impl<P: Plugin + Vst3Plugin> IPluginFactory2Trait for Factory<P> {
    unsafe fn getClassInfo2(&self, index: int32, info: *mut PClassInfo2) -> tresult {
        if index != 0 {
            return kInvalidArgument;
        }

        let info = &mut *info;

        info.cid = self.vst3_info.get_class_id().0;
        info.cardinality = PClassInfo_::ClassCardinality_::kManyInstances as int32;
        copy_cstring("Audio Module Class", &mut info.category);
        copy_cstring(&self.info.get_name(), &mut info.name);
        info.classFlags = 0;
        copy_cstring("Fx", &mut info.subCategories);
        copy_cstring(&self.info.get_vendor(), &mut info.vendor);
        copy_cstring("", &mut info.version);
        let version_str = CStr::from_ptr(SDKVersionString).to_str().unwrap();
        copy_cstring(version_str, &mut info.sdkVersion);

        kResultOk
    }
}

impl<P: Plugin + Vst3Plugin> IPluginFactory3Trait for Factory<P> {
    unsafe fn getClassInfoUnicode(&self, index: int32, info: *mut PClassInfoW) -> tresult {
        if index != 0 {
            return kInvalidArgument;
        }

        let info = &mut *info;

        info.cid = self.vst3_info.get_class_id().0;
        info.cardinality = PClassInfo_::ClassCardinality_::kManyInstances as int32;
        copy_cstring("Audio Module Class", &mut info.category);
        copy_wstring(&self.info.get_name(), &mut info.name);
        info.classFlags = 0;
        copy_cstring("Fx", &mut info.subCategories);
        copy_wstring(&self.info.get_vendor(), &mut info.vendor);
        copy_wstring("", &mut info.version);
        let version_str = CStr::from_ptr(SDKVersionString).to_str().unwrap();
        copy_wstring(version_str, &mut info.sdkVersion);

        kResultOk
    }

    unsafe fn setHostContext(&self, _context: *mut FUnknown) -> tresult {
        kNotImplemented
    }
}

#[derive(Copy, Clone)]
pub struct Uid(TUID);

impl Uid {
    pub const fn new(a: u32, b: u32, c: u32, d: u32) -> Uid {
        Uid(uid(a, b, c, d))
    }
}

pub struct Vst3Info {
    class_id: Uid,
}

impl Vst3Info {
    #[inline]
    pub fn with_class_id(class_id: Uid) -> Vst3Info {
        Vst3Info { class_id }
    }

    #[inline]
    pub fn class_id(mut self, class_id: Uid) -> Self {
        self.class_id = class_id;
        self
    }

    #[inline]
    pub fn get_class_id(&self) -> Uid {
        self.class_id
    }
}

pub trait Vst3Plugin {
    fn vst3_info() -> Vst3Info;
}

#[doc(hidden)]
pub fn get_plugin_factory<P: Plugin + Vst3Plugin>() -> *mut c_void {
    ComWrapper::new(Factory::<P>::new())
        .to_com_ptr::<IPluginFactory>()
        .unwrap()
        .into_raw() as *mut c_void
}

#[macro_export]
macro_rules! vst3 {
    ($plugin:ty) => {
        #[cfg(target_os = "windows")]
        #[no_mangle]
        extern "system" fn InitDll() -> bool {
            true
        }

        #[cfg(target_os = "windows")]
        #[no_mangle]
        extern "system" fn ExitDll() -> bool {
            true
        }

        #[cfg(target_os = "macos")]
        #[no_mangle]
        extern "system" fn BundleEntry(_bundle_ref: *mut ::std::ffi::c_void) -> bool {
            true
        }

        #[cfg(target_os = "macos")]
        #[no_mangle]
        extern "system" fn BundleExit() -> bool {
            true
        }

        #[cfg(target_os = "linux")]
        #[no_mangle]
        extern "system" fn ModuleEntry(_library_handle: *mut ::std::ffi::c_void) -> bool {
            true
        }

        #[cfg(target_os = "linux")]
        #[no_mangle]
        extern "system" fn ModuleExit() -> bool {
            true
        }

        #[no_mangle]
        extern "system" fn GetPluginFactory() -> *mut ::std::ffi::c_void {
            ::coupler::format::vst3::get_plugin_factory::<$plugin>()
        }
    };
}

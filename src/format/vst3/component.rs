use std::cell::UnsafeCell;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::{ptr, slice};

use vst3_bindgen::{Class, ComRef, Steinberg::Vst::*, Steinberg::*};

use super::util::{copy_wstring, utf16_from_ptr};
use crate::bus::{Format, Layout};
use crate::param::{ParamInfo, Range};
use crate::{Config, ParamId, Plugin, PluginInfo, Processor};

fn format_to_speaker_arrangement(format: &Format) -> SpeakerArrangement {
    match format {
        Format::Mono => SpeakerArr::kStereo,
        Format::Stereo => SpeakerArr::kStereo,
    }
}

fn speaker_arrangement_to_format(speaker_arrangement: SpeakerArrangement) -> Option<Format> {
    match speaker_arrangement {
        SpeakerArr::kMono => Some(Format::Mono),
        SpeakerArr::kStereo => Some(Format::Stereo),
        _ => None,
    }
}

fn map_param(param: &ParamInfo, value: ParamValue) -> ParamValue {
    match param.range {
        Range::Continuous { min, max } => (1.0 - value) * min + value * max,
        Range::Discrete { steps } => value * steps as f64,
    }
}

fn unmap_param(param: &ParamInfo, value: ParamValue) -> ParamValue {
    match param.range {
        Range::Continuous { min, max } => (value - min) / (max - min),
        Range::Discrete { steps } => value / steps as f64,
    }
}

struct MainThreadState {
    layout: Layout,
}

struct ProcessState<P: Plugin> {
    inputs_active: Vec<bool>,
    outputs_active: Vec<bool>,
    sample_rate: f64,
    processor: Option<P::Processor>,
}

pub struct Component<P: Plugin> {
    info: Arc<PluginInfo>,
    param_map: HashMap<ParamId, usize>,
    layout_set: HashSet<Layout>,
    // References to MainThreadState may only be formed from the main thread.
    main_thread_state: UnsafeCell<MainThreadState>,
    // When the audio processor is *not* active, references to ProcessState may only be formed from
    // the main thread. When the audio processor *is* active, references to ProcessState may only
    // be formed from the audio thread.
    process_state: UnsafeCell<ProcessState<P>>,
    plugin: P,
}

impl<P: Plugin> Component<P> {
    pub fn new(info: &Arc<PluginInfo>) -> Component<P> {
        let mut param_map = HashMap::new();
        for (index, param) in info.params.iter().enumerate() {
            param_map.insert(param.id, index);
        }

        let layout_set = info.layouts.iter().cloned().collect::<HashSet<_>>();

        Component {
            info: info.clone(),
            param_map,
            layout_set,
            main_thread_state: UnsafeCell::new(MainThreadState {
                layout: info.layouts.first().unwrap().clone(),
            }),
            process_state: UnsafeCell::new(ProcessState {
                inputs_active: vec![true; info.inputs.len()],
                outputs_active: vec![true; info.outputs.len()],
                sample_rate: 0.0,
                processor: None,
            }),
            plugin: P::create(),
        }
    }
}

impl<P: Plugin> Class for Component<P> {
    type Interfaces = (
        IComponent,
        IAudioProcessor,
        IProcessContextRequirements,
        IEditController,
    );
}

impl<P: Plugin> IPluginBaseTrait for Component<P> {
    unsafe fn initialize(&self, _context: *mut FUnknown) -> tresult {
        kResultOk
    }

    unsafe fn terminate(&self) -> tresult {
        kResultOk
    }
}

impl<P: Plugin> IComponentTrait for Component<P> {
    unsafe fn getControllerClassId(&self, _classId: *mut TUID) -> tresult {
        kNotImplemented
    }

    unsafe fn setIoMode(&self, _mode: IoMode) -> tresult {
        kResultOk
    }

    unsafe fn getBusCount(&self, type_: MediaType, dir: BusDirection) -> int32 {
        match type_ as MediaTypes {
            MediaTypes_::kAudio => match dir as BusDirections {
                BusDirections_::kInput => self.info.inputs.len() as int32,
                BusDirections_::kOutput => self.info.outputs.len() as int32,
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
        let main_thread_state = &mut *self.main_thread_state.get();

        match type_ as MediaTypes {
            MediaTypes_::kAudio => {
                let (info, format) = match dir as BusDirections {
                    BusDirections_::kInput => {
                        let info = self.info.inputs.get(index as usize);
                        let format = main_thread_state.layout.inputs.get(index as usize);
                        (info, format)
                    }
                    BusDirections_::kOutput => {
                        let info = self.info.outputs.get(index as usize);
                        let format = main_thread_state.layout.outputs.get(index as usize);
                        (info, format)
                    }
                    _ => return kInvalidArgument,
                };

                if let (Some(info), Some(format)) = (info, format) {
                    let bus = &mut *bus;

                    bus.mediaType = type_;
                    bus.direction = dir;
                    bus.channelCount = format.channels() as int32;
                    copy_wstring(&info.name, &mut bus.name);
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
        _inInfo: *mut RoutingInfo,
        _outInfo: *mut RoutingInfo,
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
        let process_state = &mut *self.process_state.get();

        match type_ as MediaTypes {
            MediaTypes_::kAudio => match dir as BusDirections {
                BusDirections_::kInput => {
                    if let Some(active) = process_state.inputs_active.get_mut(index as usize) {
                        *active = state != 0;
                        return kResultOk;
                    }
                }
                BusDirections_::kOutput => {
                    if let Some(active) = process_state.outputs_active.get_mut(index as usize) {
                        *active = state != 0;
                        return kResultOk;
                    }
                }
                _ => {}
            },
            MediaTypes_::kEvent => {}
            _ => {}
        }

        kInvalidArgument
    }

    unsafe fn setActive(&self, state: TBool) -> tresult {
        let main_thread_state = &mut *self.main_thread_state.get();
        let process_state = &mut *self.process_state.get();

        if state == 0 {
            process_state.processor = None;
        } else {
            let config = Config {
                layout: main_thread_state.layout.clone(),
                sample_rate: process_state.sample_rate,
            };

            process_state.processor = Some(P::Processor::create(&self.plugin, config));
        }

        kResultOk
    }

    unsafe fn setState(&self, state: *mut IBStream) -> tresult {
        kResultOk
    }

    unsafe fn getState(&self, state: *mut IBStream) -> tresult {
        kResultOk
    }
}

impl<P: Plugin> IAudioProcessorTrait for Component<P> {
    unsafe fn setBusArrangements(
        &self,
        inputs: *mut SpeakerArrangement,
        numIns: int32,
        outputs: *mut SpeakerArrangement,
        numOuts: int32,
    ) -> tresult {
        if numIns as usize != self.info.inputs.len() || numOuts as usize != self.info.outputs.len()
        {
            return kInvalidArgument;
        }

        let mut candidate = Layout {
            inputs: Vec::new(),
            outputs: Vec::new(),
        };

        if numIns > 0 {
            let inputs = slice::from_raw_parts(inputs, numIns as usize);
            for input in inputs {
                if let Some(format) = speaker_arrangement_to_format(*input) {
                    candidate.inputs.push(format);
                } else {
                    return kResultFalse;
                }
            }
        }

        if numOuts > 0 {
            let outputs = slice::from_raw_parts(outputs, numOuts as usize);
            for output in outputs {
                if let Some(format) = speaker_arrangement_to_format(*output) {
                    candidate.outputs.push(format);
                } else {
                    return kResultFalse;
                }
            }
        }

        if self.layout_set.contains(&candidate) {
            let main_thread_state = &mut *self.main_thread_state.get();
            main_thread_state.layout = candidate;
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
        let main_thread_state = &mut *self.main_thread_state.get();

        match dir as BusDirections {
            BusDirections_::kInput => {
                if let Some(format) = main_thread_state.layout.inputs.get(index as usize) {
                    *arr = format_to_speaker_arrangement(format);
                    return kResultOk;
                }
            }
            BusDirections_::kOutput => {
                if let Some(format) = main_thread_state.layout.outputs.get(index as usize) {
                    *arr = format_to_speaker_arrangement(format);
                    return kResultOk;
                }
            }
            _ => {}
        };

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
        let process_state = &mut *self.process_state.get();

        let setup = &*setup;
        process_state.sample_rate = setup.sampleRate;

        kResultOk
    }

    unsafe fn setProcessing(&self, state: TBool) -> tresult {
        let process_state = &mut *self.process_state.get();

        if let Some(processor) = &mut process_state.processor {
            if state == 0 {
                processor.reset();
            }

            return kResultOk;
        }

        kNotInitialized
    }

    unsafe fn process(&self, data: *mut ProcessData) -> tresult {
        let process_state = &mut *self.process_state.get();

        let Some(processor) = &mut process_state.processor else {
            return kNotInitialized;
        };

        let data = &*data;
        if let Some(param_changes) = ComRef::from_raw(data.inputParameterChanges) {
            for index in 0..param_changes.getParameterCount() {
                let param_data = param_changes.getParameterData(index);
                let Some(param_data) = ComRef::from_raw(param_data) else {
                    continue;
                };

                let id = param_data.getParameterId();
                let point_count = param_data.getPointCount();

                let Some(&param_index) = self.param_map.get(&id) else {
                    continue;
                };
                let param = &self.info.params[param_index];

                for index in 0..point_count {
                    let mut offset = 0;
                    let mut value_normalized = 0.0;
                    let result = param_data.getPoint(index, &mut offset, &mut value_normalized);

                    if result != kResultOk {
                        continue;
                    }

                    let value = map_param(param, value_normalized);
                    self.plugin.set_param(id, value);
                }
            }
        }

        kResultOk
    }

    unsafe fn getTailSamples(&self) -> uint32 {
        kInfiniteTail
    }
}

impl<P: Plugin> IProcessContextRequirementsTrait for Component<P> {
    unsafe fn getProcessContextRequirements(&self) -> uint32 {
        0
    }
}

impl<P: Plugin> IEditControllerTrait for Component<P> {
    unsafe fn setComponentState(&self, _state: *mut IBStream) -> tresult {
        kResultOk
    }

    unsafe fn setState(&self, _state: *mut IBStream) -> tresult {
        kResultOk
    }

    unsafe fn getState(&self, _state: *mut IBStream) -> tresult {
        kResultOk
    }

    unsafe fn getParameterCount(&self) -> int32 {
        self.info.params.len() as int32
    }

    unsafe fn getParameterInfo(&self, paramIndex: int32, info: *mut ParameterInfo) -> tresult {
        if let Some(param) = self.info.params.get(paramIndex as usize) {
            let info = &mut *info;

            info.id = param.id as ParamID;
            copy_wstring(&param.name, &mut info.title);
            copy_wstring(&param.name, &mut info.shortTitle);
            copy_wstring("", &mut info.units);
            info.stepCount = match param.range {
                Range::Continuous { .. } => 0,
                Range::Discrete { steps } => (steps as int32 - 1).max(1),
            };
            info.defaultNormalizedValue = map_param(param, param.default);
            info.unitId = 0;
            info.flags = ParameterInfo_::ParameterFlags_::kCanAutomate as int32;

            return kResultOk;
        }

        kInvalidArgument
    }

    unsafe fn getParamStringByValue(
        &self,
        id: ParamID,
        valueNormalized: ParamValue,
        string: *mut String128,
    ) -> tresult {
        if let Some(&index) = self.param_map.get(&id) {
            let param = &self.info.params[index];

            let mut display = String::new();
            let value = map_param(param, valueNormalized);
            param.display.display(value, &mut display);
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
        if let Some(&index) = self.param_map.get(&id) {
            let param = &self.info.params[index];

            if let Ok(display) = String::from_utf16(utf16_from_ptr(string)) {
                if let Some(value) = param.display.parse(&display) {
                    *valueNormalized = unmap_param(param, value);
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
        if let Some(&index) = self.param_map.get(&id) {
            let param = &self.info.params[index];
            return map_param(param, valueNormalized);
        }

        0.0
    }

    unsafe fn plainParamToNormalized(&self, id: ParamID, plainValue: ParamValue) -> ParamValue {
        if let Some(&index) = self.param_map.get(&id) {
            let param = &self.info.params[index];
            return unmap_param(param, plainValue);
        }

        0.0
    }

    unsafe fn getParamNormalized(&self, id: ParamID) -> ParamValue {
        0.0
    }

    unsafe fn setParamNormalized(&self, id: ParamID, value: ParamValue) -> tresult {
        kResultOk
    }

    unsafe fn setComponentHandler(&self, handler: *mut IComponentHandler) -> tresult {
        kResultOk
    }

    unsafe fn createView(&self, name: FIDString) -> *mut IPlugView {
        ptr::null_mut()
    }
}

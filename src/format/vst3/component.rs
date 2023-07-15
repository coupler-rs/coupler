use std::cell::UnsafeCell;
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::ptr;
use std::sync::Arc;

use vst3_bindgen::{Class, ComRef, Steinberg::Vst::*, Steinberg::*};

use super::buffers::ScratchBuffers;
use super::util::{copy_wstring, slice_from_raw_parts_checked, utf16_from_ptr};
use crate::bus::{BusDir, Format, Layout};
use crate::events::{Data, Event, Events};
use crate::param::{ParamInfo, Range};
use crate::{Config, Host, ParamId, Plugin, PluginInfo, Processor};

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

struct MainThreadState<P> {
    config: Config,
    plugin: P,
}

struct ProcessState<P: Plugin> {
    config: Config,
    inputs_active: Vec<bool>,
    outputs_active: Vec<bool>,
    scratch_buffers: ScratchBuffers,
    events: Vec<Event>,
    processor: Option<P::Processor>,
}

pub struct Component<P: Plugin> {
    info: Arc<PluginInfo>,
    input_bus_map: Vec<usize>,
    output_bus_map: Vec<usize>,
    layout_set: HashSet<Layout>,
    param_map: HashMap<ParamId, usize>,
    // References to MainThreadState may only be formed from the main thread.
    main_thread_state: UnsafeCell<MainThreadState<P>>,
    // When the audio processor is *not* active, references to ProcessState may only be formed from
    // the main thread. When the audio processor *is* active, references to ProcessState may only
    // be formed from the audio thread.
    process_state: UnsafeCell<ProcessState<P>>,
}

impl<P: Plugin> Component<P> {
    pub fn new(info: &Arc<PluginInfo>) -> Component<P> {
        let mut input_bus_map = Vec::new();
        let mut output_bus_map = Vec::new();
        for (index, bus) in info.buses.iter().enumerate() {
            match bus.dir {
                BusDir::In => input_bus_map.push(index),
                BusDir::Out => output_bus_map.push(index),
            }
        }

        let layout_set = info.layouts.iter().cloned().collect::<HashSet<_>>();

        let mut param_map = HashMap::new();
        for (index, param) in info.params.iter().enumerate() {
            param_map.insert(param.id, index);
        }

        let config = Config {
            layout: info.layouts.first().unwrap().clone(),
            sample_rate: 0.0,
            max_buffer_size: 0,
        };

        let inputs_active = vec![true; input_bus_map.len()];
        let outputs_active = vec![true; output_bus_map.len()];

        Component {
            info: info.clone(),
            input_bus_map,
            output_bus_map,
            layout_set,
            param_map,
            main_thread_state: UnsafeCell::new(MainThreadState {
                config: config.clone(),
                plugin: P::new(Host {}),
            }),
            process_state: UnsafeCell::new(ProcessState {
                config,
                inputs_active,
                outputs_active,
                scratch_buffers: ScratchBuffers::new(),
                events: Vec::with_capacity(4096),
                processor: None,
            }),
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
                BusDirections_::kInput => self.input_bus_map.len() as int32,
                BusDirections_::kOutput => self.output_bus_map.len() as int32,
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
                let bus_index = match dir as BusDirections {
                    BusDirections_::kInput => self.input_bus_map.get(index as usize),
                    BusDirections_::kOutput => self.output_bus_map.get(index as usize),
                    _ => return kInvalidArgument,
                };

                if let Some(&bus_index) = bus_index {
                    let info = self.info.buses.get(bus_index);
                    let format = main_thread_state.config.layout.formats.get(bus_index);

                    if let (Some(info), Some(format)) = (info, format) {
                        let bus = &mut *bus;

                        bus.mediaType = type_;
                        bus.direction = dir;
                        bus.channelCount = format.channel_count() as int32;
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
            let config = main_thread_state.config.clone();
            process_state.config = config.clone();
            process_state.scratch_buffers.resize(&self.info.buses, &config);
            process_state.processor = Some(main_thread_state.plugin.processor(config));
        }

        kResultOk
    }

    unsafe fn setState(&self, state: *mut IBStream) -> tresult {
        use std::io::{Error, ErrorKind, Read, Result};

        struct StreamReader<'a>(ComRef<'a, IBStream>);

        impl<'a> Read for StreamReader<'a> {
            fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
                let ptr = buf.as_mut_ptr() as *mut c_void;
                let len = buf.len() as int32;
                let mut bytes: int32 = 0;
                let result = unsafe { self.0.read(ptr, len, &mut bytes) };

                if result == kResultOk {
                    Ok(bytes as usize)
                } else {
                    Err(Error::new(ErrorKind::Other, "failed to read from stream"))
                }
            }
        }

        if let Some(state) = ComRef::from_raw(state) {
            let main_thread_state = &mut *self.main_thread_state.get();

            if let Ok(_) = main_thread_state.plugin.load(&mut StreamReader(state)) {
                return kResultOk;
            }
        }

        kResultFalse
    }

    unsafe fn getState(&self, state: *mut IBStream) -> tresult {
        use std::io::{Error, ErrorKind, Result, Write};

        struct StreamWriter<'a>(ComRef<'a, IBStream>);

        impl<'a> Write for StreamWriter<'a> {
            fn write(&mut self, buf: &[u8]) -> Result<usize> {
                let ptr = buf.as_ptr() as *mut c_void;
                let len = buf.len() as int32;
                let mut bytes: int32 = 0;
                let result = unsafe { self.0.write(ptr, len, &mut bytes) };

                if result == kResultOk {
                    Ok(bytes as usize)
                } else {
                    Err(Error::new(ErrorKind::Other, "failed to write to stream"))
                }
            }

            fn flush(&mut self) -> Result<()> {
                Ok(())
            }
        }

        if let Some(state) = ComRef::from_raw(state) {
            let main_thread_state = &*self.main_thread_state.get();

            if let Ok(_) = main_thread_state.plugin.save(&mut StreamWriter(state)) {
                return kResultOk;
            }
        }

        kResultFalse
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
        let input_count = numIns as usize;
        let output_count = numOuts as usize;
        if input_count != self.input_bus_map.len() || output_count != self.output_bus_map.len() {
            return kInvalidArgument;
        }

        let mut candidate = Layout {
            formats: Vec::new(),
        };

        let mut inputs = slice_from_raw_parts_checked(inputs, input_count).into_iter();
        let mut outputs = slice_from_raw_parts_checked(outputs, output_count).into_iter();
        for bus in &self.info.buses {
            let arrangement = match bus.dir {
                BusDir::In => *inputs.next().unwrap(),
                BusDir::Out => *outputs.next().unwrap(),
            };

            if let Some(format) = speaker_arrangement_to_format(arrangement) {
                candidate.formats.push(format);
            } else {
                return kResultFalse;
            }
        }

        if self.layout_set.contains(&candidate) {
            let main_thread_state = &mut *self.main_thread_state.get();
            main_thread_state.config.layout = candidate;
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

        let bus_index = match dir as BusDirections {
            BusDirections_::kInput => self.input_bus_map.get(index as usize),
            BusDirections_::kOutput => self.output_bus_map.get(index as usize),
            _ => return kInvalidArgument,
        };

        if let Some(&bus_index) = bus_index {
            if let Some(format) = main_thread_state.config.layout.formats.get(bus_index as usize) {
                *arr = format_to_speaker_arrangement(format);
                return kResultOk;
            }
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
        let main_thread_state = &mut *self.main_thread_state.get();

        main_thread_state.plugin.latency(&main_thread_state.config) as uint32
    }

    unsafe fn setupProcessing(&self, setup: *mut ProcessSetup) -> tresult {
        let main_thread_state = &mut *self.main_thread_state.get();

        let setup = &*setup;
        main_thread_state.config.sample_rate = setup.sampleRate;
        main_thread_state.config.max_buffer_size = setup.maxSamplesPerBlock as usize;

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

        let Ok(buffers) = process_state.scratch_buffers.get_buffers(
            &self.info.buses,
            &self.input_bus_map,
            &self.output_bus_map,
            &process_state.config,
            &process_state.inputs_active,
            &process_state.outputs_active,
            &data,
        ) else {
            return kInvalidArgument;
        };

        process_state.events.clear();

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
                    process_state.events.push(Event {
                        time: offset as i64,
                        data: Data::ParamChange { id, value },
                    });
                }
            }
        }

        processor.process(buffers, Events::new(&process_state.events));

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
        let main_thread_state = &*self.main_thread_state.get();

        if let Some(&index) = self.param_map.get(&id) {
            let param = &self.info.params[index];
            let value = main_thread_state.plugin.get_param(id);
            return unmap_param(param, value);
        }

        0.0
    }

    unsafe fn setParamNormalized(&self, id: ParamID, value: ParamValue) -> tresult {
        let main_thread_state = &mut *self.main_thread_state.get();

        if let Some(&index) = self.param_map.get(&id) {
            let param = &self.info.params[index];
            let mapped = map_param(param, value);
            main_thread_state.plugin.set_param(id, mapped);
            return kResultOk;
        }

        kInvalidArgument
    }

    unsafe fn setComponentHandler(&self, _handler: *mut IComponentHandler) -> tresult {
        kResultOk
    }

    unsafe fn createView(&self, _name: FIDString) -> *mut IPlugView {
        ptr::null_mut()
    }
}

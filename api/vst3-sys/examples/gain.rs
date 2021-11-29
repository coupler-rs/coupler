use vst3_sys::*;

use std::ffi::{c_void, CString};
use std::os::raw::c_char;
use std::str::FromStr;
use std::sync::atomic;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::{mem, ptr, slice};

fn copy_cstring(src: &str, dst: &mut [c_char]) {
    let c_string = CString::new(src).unwrap_or_else(|_| CString::default());
    let bytes = c_string.as_bytes_with_nul();

    for (src, dst) in bytes.iter().zip(dst.iter_mut()) {
        *dst = *src as c_char;
    }

    if bytes.len() > dst.len() {
        if let Some(last) = dst.last_mut() {
            *last = 0;
        }
    }
}

fn copy_wstring(src: &str, dst: &mut [i16]) {
    let mut len = 0;
    for (src, dst) in src.encode_utf16().zip(dst.iter_mut()) {
        *dst = src as i16;
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

#[repr(C)]
struct GainProcessor {
    component: *const IComponent,
    audio_processor: *const IAudioProcessor,
    process_context_requirements: *const IProcessContextRequirements,
    count: AtomicU32,
    gain: AtomicU64,
}

impl GainProcessor {
    const CID: TUID = uid(0x367C3805, 0x446D40DA, 0x82E6BBB4, 0x900BC212);
    const NAME: &'static str = "Gain";

    const COMPONENT_OFFSET: isize = 0;
    const AUDIO_PROCESSOR_OFFSET: isize =
        Self::COMPONENT_OFFSET + mem::size_of::<*const IComponent>() as isize;
    const PROCESS_CONTEXT_REQUIREMENTS_OFFSET: isize =
        Self::AUDIO_PROCESSOR_OFFSET + mem::size_of::<*const IAudioProcessor>() as isize;

    fn create_instance() -> *mut GainProcessor {
        Box::into_raw(Box::new(GainProcessor {
            component: &COMPONENT_VTABLE,
            audio_processor: &AUDIO_PROCESSOR_VTABLE,
            process_context_requirements: &PROCESS_CONTEXT_REQUIREMENTS_VTABLE,
            count: AtomicU32::new(1),
            gain: AtomicU64::new(1.0f64.to_bits()),
        }))
    }

    unsafe fn query_interface(
        this: *mut c_void,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> TResult {
        let iid = *iid;

        if iid == FUnknown::IID || iid == IPluginBase::IID || iid == IComponent::IID {
            Self::add_ref(this);
            *obj = this.offset(Self::COMPONENT_OFFSET);
            return result::OK;
        }

        if iid == IAudioProcessor::IID {
            Self::add_ref(this);
            *obj = this.offset(Self::AUDIO_PROCESSOR_OFFSET);
            return result::OK;
        }

        if iid == IProcessContextRequirements::IID {
            Self::add_ref(this);
            *obj = this.offset(Self::PROCESS_CONTEXT_REQUIREMENTS_OFFSET);
            return result::OK;
        }

        result::NO_INTERFACE
    }

    unsafe fn add_ref(this: *mut c_void) -> u32 {
        (*(this as *const GainProcessor)).count.fetch_add(1, Ordering::Relaxed) + 1
    }

    unsafe fn release(this: *mut c_void) -> u32 {
        let count = (*(this as *const GainProcessor)).count.fetch_sub(1, Ordering::Release) - 1;

        if count == 0 {
            atomic::fence(Ordering::Acquire);
            drop(Box::from_raw(this as *mut GainProcessor));
        }

        count
    }

    unsafe extern "system" fn component_query_interface(
        this: *mut c_void,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> TResult {
        Self::query_interface(this.offset(-Self::COMPONENT_OFFSET), iid, obj)
    }

    unsafe extern "system" fn component_add_ref(this: *mut c_void) -> u32 {
        Self::add_ref(this.offset(-Self::COMPONENT_OFFSET))
    }

    unsafe extern "system" fn component_release(this: *mut c_void) -> u32 {
        Self::release(this.offset(-Self::COMPONENT_OFFSET))
    }

    unsafe extern "system" fn component_initialize(
        _this: *mut c_void,
        _context: *mut FUnknown,
    ) -> TResult {
        result::OK
    }

    unsafe extern "system" fn component_terminate(_this: *mut c_void) -> TResult {
        result::OK
    }

    unsafe extern "system" fn get_controller_class_id(
        _this: *mut c_void,
        class_id: *mut TUID,
    ) -> TResult {
        *class_id = GainController::CID;
        result::OK
    }

    unsafe extern "system" fn set_io_mode(_this: *mut c_void, _mode: IoMode) -> TResult {
        result::OK
    }

    unsafe extern "system" fn get_bus_count(
        _this: *mut c_void,
        media_type: MediaType,
        dir: BusDirection,
    ) -> i32 {
        match media_type {
            media_types::AUDIO => match dir {
                bus_directions::INPUT => 1,
                bus_directions::OUTPUT => 1,
                _ => 0,
            },
            media_types::EVENT => 0,
            _ => 0,
        }
    }

    unsafe extern "system" fn get_bus_info(
        _this: *mut c_void,
        media_type: MediaType,
        dir: BusDirection,
        index: i32,
        bus: *mut BusInfo,
    ) -> TResult {
        match media_type {
            media_types::AUDIO => match dir {
                bus_directions::INPUT => match index {
                    0 => {
                        let bus = &mut *bus;

                        bus.media_type = media_types::AUDIO;
                        bus.direction = bus_directions::INPUT;
                        bus.channel_count = 2;
                        copy_wstring("Input", &mut bus.name);
                        bus.bus_type = bus_types::MAIN;
                        bus.flags = BusInfo::DEFAULT_ACTIVE;

                        result::OK
                    }
                    _ => result::INVALID_ARGUMENT,
                },
                bus_directions::OUTPUT => match index {
                    0 => {
                        let bus = &mut *bus;

                        bus.media_type = media_types::AUDIO;
                        bus.direction = bus_directions::OUTPUT;
                        bus.channel_count = 2;
                        copy_wstring("Output", &mut bus.name);
                        bus.bus_type = bus_types::MAIN;
                        bus.flags = BusInfo::DEFAULT_ACTIVE;

                        result::OK
                    }
                    _ => result::INVALID_ARGUMENT,
                },
                _ => result::INVALID_ARGUMENT,
            },
            media_types::EVENT => result::INVALID_ARGUMENT,
            _ => result::INVALID_ARGUMENT,
        }
    }

    unsafe extern "system" fn get_routing_info(
        _this: *mut c_void,
        _in_info: *mut RoutingInfo,
        _out_info: *mut RoutingInfo,
    ) -> TResult {
        result::NOT_IMPLEMENTED
    }

    unsafe extern "system" fn activate_bus(
        _this: *mut c_void,
        _media_type: MediaType,
        _dir: BusDirection,
        _index: i32,
        _state: TBool,
    ) -> TResult {
        result::OK
    }

    unsafe extern "system" fn set_active(_this: *mut c_void, _state: TBool) -> TResult {
        result::OK
    }

    unsafe extern "system" fn set_state(
        _this: *mut c_void,
        _state: *mut *const IBStream,
    ) -> TResult {
        result::OK
    }

    unsafe extern "system" fn get_state(
        _this: *mut c_void,
        _state: *mut *const IBStream,
    ) -> TResult {
        result::OK
    }

    unsafe extern "system" fn audio_processor_query_interface(
        this: *mut c_void,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> TResult {
        Self::query_interface(this.offset(-Self::AUDIO_PROCESSOR_OFFSET), iid, obj)
    }

    unsafe extern "system" fn audio_processor_add_ref(this: *mut c_void) -> u32 {
        Self::add_ref(this.offset(-Self::AUDIO_PROCESSOR_OFFSET))
    }

    unsafe extern "system" fn audio_processor_release(this: *mut c_void) -> u32 {
        Self::release(this.offset(-Self::AUDIO_PROCESSOR_OFFSET))
    }

    unsafe extern "system" fn set_bus_arrangements(
        _this: *mut c_void,
        inputs: *const SpeakerArrangement,
        num_ins: i32,
        outputs: *const SpeakerArrangement,
        num_outs: i32,
    ) -> TResult {
        if num_ins != 1 || num_outs != 1 {
            return result::FALSE;
        }

        if *inputs != speaker_arrangements::STEREO || *outputs != speaker_arrangements::STEREO {
            return result::FALSE;
        }

        result::TRUE
    }

    unsafe extern "system" fn get_bus_arrangement(
        _this: *mut c_void,
        dir: BusDirection,
        index: i32,
        arr: *mut SpeakerArrangement,
    ) -> TResult {
        match dir {
            bus_directions::INPUT => {
                if index == 0 {
                    *arr = speaker_arrangements::STEREO;
                    result::OK
                } else {
                    result::INVALID_ARGUMENT
                }
            }
            bus_directions::OUTPUT => {
                if index == 0 {
                    *arr = speaker_arrangements::STEREO;
                    result::OK
                } else {
                    result::INVALID_ARGUMENT
                }
            }
            _ => result::INVALID_ARGUMENT,
        }
    }

    unsafe extern "system" fn can_process_sample_size(
        _this: *mut c_void,
        symbolic_sample_size: i32,
    ) -> TResult {
        match symbolic_sample_size {
            symbolic_sample_sizes::SAMPLE_32 => result::OK,
            symbolic_sample_sizes::SAMPLE_64 => result::NOT_IMPLEMENTED,
            _ => result::INVALID_ARGUMENT,
        }
    }

    unsafe extern "system" fn get_latency_samples(_this: *mut c_void) -> u32 {
        0
    }

    unsafe extern "system" fn setup_processing(
        _this: *mut c_void,
        _setup: *mut ProcessSetup,
    ) -> TResult {
        result::OK
    }

    unsafe extern "system" fn set_processing(_this: *mut c_void, _state: TBool) -> TResult {
        result::OK
    }

    unsafe extern "system" fn process(this: *mut c_void, data: *mut ProcessData) -> TResult {
        let processor = &mut *(this.offset(-Self::AUDIO_PROCESSOR_OFFSET) as *mut GainProcessor);

        let process_data = &*data;

        let param_changes = process_data.input_parameter_changes;
        if !param_changes.is_null() {
            let param_count =
                ((*(*param_changes)).get_parameter_count)(param_changes as *mut c_void);
            for param_index in 0..param_count {
                let param_queue = ((*(*param_changes)).get_parameter_data)(
                    param_changes as *mut c_void,
                    param_index,
                );

                if param_queue.is_null() {
                    continue;
                }

                let param_id = ((*(*param_queue)).get_parameter_id)(param_queue as *mut c_void);
                let point_count = ((*(*param_queue)).get_point_count)(param_queue as *mut c_void);

                match param_id {
                    0 => {
                        let mut sample_offset = 0;
                        let mut value = 0.0;
                        let result = ((*(*param_queue)).get_point)(
                            param_queue as *mut c_void,
                            point_count - 1,
                            &mut sample_offset,
                            &mut value,
                        );

                        if result == result::TRUE {
                            processor.gain.store(value.to_bits(), Ordering::Relaxed);
                        }
                    }
                    _ => {}
                }
            }
        }

        let gain = f64::from_bits(processor.gain.load(Ordering::Relaxed)) as f32;

        let num_samples = process_data.num_samples as usize;

        if process_data.num_inputs != 1 || process_data.num_outputs != 1 {
            return result::OK;
        }

        let input_buses =
            slice::from_raw_parts(process_data.inputs, process_data.num_inputs as usize);
        let output_buses =
            slice::from_raw_parts(process_data.outputs, process_data.num_outputs as usize);

        if input_buses[0].num_channels != 2 || output_buses[0].num_channels != 2 {
            return result::OK;
        }

        let input_channels = slice::from_raw_parts(
            input_buses[0].channel_buffers as *const *const f32,
            input_buses[0].num_channels as usize,
        );
        let output_channels = slice::from_raw_parts(
            output_buses[0].channel_buffers as *mut *mut f32,
            output_buses[0].num_channels as usize,
        );

        let input_l = slice::from_raw_parts(input_channels[0], num_samples);
        let input_r = slice::from_raw_parts(input_channels[1], num_samples);
        let output_l = slice::from_raw_parts_mut(output_channels[0], num_samples);
        let output_r = slice::from_raw_parts_mut(output_channels[1], num_samples);

        for i in 0..num_samples {
            output_l[i] = gain * input_l[i];
            output_r[i] = gain * input_r[i];
        }

        result::OK
    }

    unsafe extern "system" fn get_tail_samples(_this: *mut c_void) -> u32 {
        0
    }

    pub unsafe extern "system" fn process_context_requirements_query_interface(
        this: *mut c_void,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> TResult {
        Self::query_interface(this.offset(-Self::PROCESS_CONTEXT_REQUIREMENTS_OFFSET), iid, obj)
    }

    pub unsafe extern "system" fn process_context_requirements_add_ref(this: *mut c_void) -> u32 {
        Self::add_ref(this.offset(-Self::PROCESS_CONTEXT_REQUIREMENTS_OFFSET))
    }

    pub unsafe extern "system" fn process_context_requirements_release(this: *mut c_void) -> u32 {
        Self::release(this.offset(-Self::PROCESS_CONTEXT_REQUIREMENTS_OFFSET))
    }

    pub unsafe extern "system" fn get_process_context_requirements(_this: *mut c_void) -> u32 {
        0
    }
}

static COMPONENT_VTABLE: IComponent = IComponent {
    plugin_base: IPluginBase {
        unknown: FUnknown {
            query_interface: GainProcessor::component_query_interface,
            add_ref: GainProcessor::component_add_ref,
            release: GainProcessor::component_release,
        },
        initialize: GainProcessor::component_initialize,
        terminate: GainProcessor::component_terminate,
    },
    get_controller_class_id: GainProcessor::get_controller_class_id,
    set_io_mode: GainProcessor::set_io_mode,
    get_bus_count: GainProcessor::get_bus_count,
    get_bus_info: GainProcessor::get_bus_info,
    get_routing_info: GainProcessor::get_routing_info,
    activate_bus: GainProcessor::activate_bus,
    set_active: GainProcessor::set_active,
    set_state: GainProcessor::set_state,
    get_state: GainProcessor::get_state,
};

static AUDIO_PROCESSOR_VTABLE: IAudioProcessor = IAudioProcessor {
    unknown: FUnknown {
        query_interface: GainProcessor::audio_processor_query_interface,
        add_ref: GainProcessor::audio_processor_add_ref,
        release: GainProcessor::audio_processor_release,
    },
    set_bus_arrangements: GainProcessor::set_bus_arrangements,
    get_bus_arrangement: GainProcessor::get_bus_arrangement,
    can_process_sample_size: GainProcessor::can_process_sample_size,
    get_latency_samples: GainProcessor::get_latency_samples,
    setup_processing: GainProcessor::setup_processing,
    set_processing: GainProcessor::set_processing,
    process: GainProcessor::process,
    get_tail_samples: GainProcessor::get_tail_samples,
};

static PROCESS_CONTEXT_REQUIREMENTS_VTABLE: IProcessContextRequirements =
    IProcessContextRequirements {
        unknown: FUnknown {
            query_interface: GainProcessor::process_context_requirements_query_interface,
            add_ref: GainProcessor::process_context_requirements_add_ref,
            release: GainProcessor::process_context_requirements_release,
        },
        get_process_context_requirements: GainProcessor::get_process_context_requirements,
    };

#[repr(C)]
struct GainController {
    edit_controller: *const IEditController,
    count: AtomicU32,
    gain: f64,
}

impl GainController {
    const CID: TUID = uid(0xD93CC3FD, 0xDBFE459A, 0xAAE03612, 0xF9AF088E);
    const NAME: &'static str = "Gain Controller";

    const EDIT_CONTROLLER_OFFSET: isize = 0;

    fn create_instance() -> *mut GainController {
        Box::into_raw(Box::new(GainController {
            edit_controller: &EDIT_CONTROLLER_VTABLE,
            count: AtomicU32::new(1),
            gain: 1.0,
        }))
    }

    unsafe fn query_interface(
        this: *mut c_void,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> TResult {
        let iid = *iid;

        if iid == FUnknown::IID || iid == IEditController::IID {
            Self::add_ref(this);
            *obj = this.offset(Self::EDIT_CONTROLLER_OFFSET);
            return result::OK;
        }

        result::NO_INTERFACE
    }

    unsafe fn add_ref(this: *mut c_void) -> u32 {
        (*(this as *const GainController)).count.fetch_add(1, Ordering::Relaxed) + 1
    }

    unsafe fn release(this: *mut c_void) -> u32 {
        let count = (*(this as *const GainController)).count.fetch_sub(1, Ordering::Release) - 1;

        if count == 0 {
            atomic::fence(Ordering::Acquire);
            drop(Box::from_raw(this as *mut GainController));
        }

        count
    }

    pub unsafe extern "system" fn edit_controller_query_interface(
        this: *mut c_void,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> TResult {
        Self::query_interface(this.offset(-Self::EDIT_CONTROLLER_OFFSET), iid, obj)
    }

    pub unsafe extern "system" fn edit_controller_add_ref(this: *mut c_void) -> u32 {
        Self::add_ref(this.offset(-Self::EDIT_CONTROLLER_OFFSET))
    }

    pub unsafe extern "system" fn edit_controller_release(this: *mut c_void) -> u32 {
        Self::release(this.offset(-Self::EDIT_CONTROLLER_OFFSET))
    }

    pub unsafe extern "system" fn edit_controller_initialize(
        _this: *mut c_void,
        _context: *mut FUnknown,
    ) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn edit_controller_terminate(_this: *mut c_void) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn set_component_state(
        _this: *mut c_void,
        _state: *mut *const IBStream,
    ) -> TResult {
        result::NOT_IMPLEMENTED
    }

    pub unsafe extern "system" fn edit_controller_set_state(
        _this: *mut c_void,
        _state: *mut *const IBStream,
    ) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn edit_controller_get_state(
        _this: *mut c_void,
        _state: *mut *const IBStream,
    ) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn get_parameter_count(_this: *mut c_void) -> i32 {
        1
    }

    pub unsafe extern "system" fn get_parameter_info(
        _this: *mut c_void,
        param_index: i32,
        info: *mut ParameterInfo,
    ) -> TResult {
        match param_index {
            0 => {
                let info = &mut *info;

                info.id = 0;
                copy_wstring("Gain", &mut info.title);
                copy_wstring("Gain", &mut info.short_title);
                copy_wstring("", &mut info.units);
                info.step_count = 0;
                info.default_normalized_value = 1.0;
                info.unit_id = 0;
                info.flags = ParameterInfo::CAN_AUTOMATE;

                result::OK
            }
            _ => result::INVALID_ARGUMENT,
        }
    }

    pub unsafe extern "system" fn get_param_string_by_value(
        _this: *mut c_void,
        id: u32,
        value_normalized: f64,
        string: *mut String128,
    ) -> TResult {
        match id {
            0 => {
                let display = value_normalized.to_string();
                copy_wstring(&display, &mut *string);
                result::OK
            }
            _ => result::INVALID_ARGUMENT,
        }
    }

    pub unsafe extern "system" fn get_param_value_by_string(
        _this: *mut c_void,
        id: u32,
        string: *const TChar,
        value_normalized: *mut f64,
    ) -> TResult {
        match id {
            0 => {
                let len = len_wstring(string);
                if let Ok(string) =
                    String::from_utf16(slice::from_raw_parts(string as *const u16, len))
                {
                    if let Ok(value) = f64::from_str(&string) {
                        *value_normalized = value;
                        return result::OK;
                    }
                }
                result::INVALID_ARGUMENT
            }
            _ => result::INVALID_ARGUMENT,
        }
    }

    pub unsafe extern "system" fn normalized_param_to_plain(
        _this: *mut c_void,
        id: u32,
        value_normalized: f64,
    ) -> f64 {
        match id {
            0 => value_normalized,
            _ => 0.0,
        }
    }

    pub unsafe extern "system" fn plain_param_to_normalized(
        _this: *mut c_void,
        id: u32,
        plain_value: f64,
    ) -> f64 {
        match id {
            0 => plain_value,
            _ => 0.0,
        }
    }

    pub unsafe extern "system" fn get_param_normalized(this: *mut c_void, id: u32) -> f64 {
        let controller = &*(this.offset(-Self::EDIT_CONTROLLER_OFFSET) as *const GainController);
        match id {
            0 => controller.gain,
            _ => 0.0,
        }
    }

    pub unsafe extern "system" fn set_param_normalized(
        this: *mut c_void,
        id: u32,
        value: f64,
    ) -> TResult {
        let controller = &mut *(this.offset(-Self::EDIT_CONTROLLER_OFFSET) as *mut GainController);
        match id {
            0 => {
                controller.gain = value;
                result::OK
            }
            _ => result::INVALID_ARGUMENT,
        }
    }

    pub unsafe extern "system" fn set_component_handler(
        _this: *mut c_void,
        _handler: *mut *const IComponentHandler,
    ) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn create_view(
        _this: *mut c_void,
        _name: *const c_char,
    ) -> *mut *const IPlugView {
        ptr::null_mut()
    }
}

static EDIT_CONTROLLER_VTABLE: IEditController = IEditController {
    plugin_base: IPluginBase {
        unknown: FUnknown {
            query_interface: GainController::edit_controller_query_interface,
            add_ref: GainController::edit_controller_add_ref,
            release: GainController::edit_controller_release,
        },
        initialize: GainController::edit_controller_initialize,
        terminate: GainController::edit_controller_terminate,
    },
    set_component_state: GainController::set_component_state,
    set_state: GainController::edit_controller_set_state,
    get_state: GainController::edit_controller_get_state,
    get_parameter_count: GainController::get_parameter_count,
    get_parameter_info: GainController::get_parameter_info,
    get_param_string_by_value: GainController::get_param_string_by_value,
    get_param_value_by_string: GainController::get_param_value_by_string,
    normalized_param_to_plain: GainController::normalized_param_to_plain,
    plain_param_to_normalized: GainController::plain_param_to_normalized,
    get_param_normalized: GainController::get_param_normalized,
    set_param_normalized: GainController::set_param_normalized,
    set_component_handler: GainController::set_component_handler,
    create_view: GainController::create_view,
};

#[repr(C)]
struct Factory {
    plugin_factory_3: *const IPluginFactory3,
}

unsafe impl Sync for Factory {}

impl Factory {
    unsafe extern "system" fn query_interface(
        this: *mut c_void,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> TResult {
        let iid = *iid;

        if iid == FUnknown::IID
            || iid == IPluginFactory::IID
            || iid == IPluginFactory2::IID
            || iid == IPluginFactory3::IID
        {
            *obj = this;
            return result::OK;
        }

        result::NO_INTERFACE
    }

    unsafe extern "system" fn add_ref(_this: *mut c_void) -> u32 {
        1
    }

    unsafe extern "system" fn release(_this: *mut c_void) -> u32 {
        1
    }

    unsafe extern "system" fn get_factory_info(
        _this: *mut c_void,
        info: *mut PFactoryInfo,
    ) -> TResult {
        let info = &mut *info;

        copy_cstring("Vendor", &mut info.vendor);
        copy_cstring("https://example.com", &mut info.url);
        copy_cstring("someone@example.com", &mut info.email);
        info.flags = PFactoryInfo::UNICODE;

        result::OK
    }

    unsafe extern "system" fn count_classes(_this: *mut c_void) -> i32 {
        2
    }

    unsafe extern "system" fn get_class_info(
        _this: *mut c_void,
        index: i32,
        info: *mut PClassInfo,
    ) -> TResult {
        match index {
            0 => {
                let info = &mut *info;
                info.cid = GainProcessor::CID;
                info.cardinality = PClassInfo::MANY_INSTANCES;
                copy_cstring("Audio Module Class", &mut info.category);
                copy_cstring(GainProcessor::NAME, &mut info.name);
            }
            1 => {
                let info = &mut *info;
                info.cid = GainController::CID;
                info.cardinality = PClassInfo::MANY_INSTANCES;
                copy_cstring("Component Controller Class", &mut info.category);
                copy_cstring(GainController::NAME, &mut info.name);
            }
            _ => {
                return result::INVALID_ARGUMENT;
            }
        }

        result::OK
    }

    unsafe extern "system" fn create_instance(
        _this: *mut c_void,
        cid: *const c_char,
        iid: *const c_char,
        obj: *mut *mut c_void,
    ) -> TResult {
        let instance = match *(cid as *const TUID) {
            GainProcessor::CID => Some(GainProcessor::create_instance() as *mut *const FUnknown),
            GainController::CID => Some(GainController::create_instance() as *mut *const FUnknown),
            _ => None,
        };

        if let Some(instance) = instance {
            let result =
                ((*(*instance)).query_interface)(instance as *mut c_void, iid as *const TUID, obj);
            if result == result::OK {
                ((*(*instance)).release)(instance as *mut c_void);
                result::OK
            } else {
                ((*(*instance)).release)(instance as *mut c_void);
                result::NO_INTERFACE
            }
        } else {
            result::INVALID_ARGUMENT
        }
    }

    unsafe extern "system" fn get_class_info_2(
        _this: *mut c_void,
        index: i32,
        info: *mut PClassInfo2,
    ) -> TResult {
        match index {
            0 => {
                let info = &mut *info;
                info.cid = GainProcessor::CID;
                info.cardinality = PClassInfo::MANY_INSTANCES;
                copy_cstring("Audio Module Class", &mut info.category);
                copy_cstring(GainProcessor::NAME, &mut info.name);
                info.class_flags = component_flags::DISTRIBUTABLE;
                copy_cstring("Fx", &mut info.sub_categories);
                copy_cstring("Vendor", &mut info.vendor);
                copy_cstring("0.1.0", &mut info.version);
                copy_cstring("VST 3.7.1", &mut info.sdk_version);
            }
            1 => {
                let info = &mut *info;
                info.cid = GainController::CID;
                info.cardinality = PClassInfo::MANY_INSTANCES;
                copy_cstring("Component Controller Class", &mut info.category);
                copy_cstring(GainProcessor::NAME, &mut info.name);
                info.class_flags = 0;
                copy_cstring("Fx", &mut info.sub_categories);
                copy_cstring("Vendor", &mut info.vendor);
                copy_cstring("0.1.0", &mut info.version);
                copy_cstring("VST 3.7.1", &mut info.sdk_version);
            }
            _ => {
                return result::INVALID_ARGUMENT;
            }
        }

        result::OK
    }

    unsafe extern "system" fn get_class_info_unicode(
        _this: *mut c_void,
        index: i32,
        info: *mut PClassInfoW,
    ) -> TResult {
        match index {
            0 => {
                let info = &mut *info;
                info.cid = GainProcessor::CID;
                info.cardinality = PClassInfo::MANY_INSTANCES;
                copy_cstring("Audio Module Class", &mut info.category);
                copy_wstring(GainProcessor::NAME, &mut info.name);
                info.class_flags = component_flags::DISTRIBUTABLE;
                copy_cstring("Fx", &mut info.sub_categories);
                copy_wstring("Vendor", &mut info.vendor);
                copy_wstring("0.1.0", &mut info.version);
                copy_wstring("VST 3.7", &mut info.sdk_version);
            }
            1 => {
                let info = &mut *info;
                info.cid = GainController::CID;
                info.cardinality = PClassInfo::MANY_INSTANCES;
                copy_cstring("Component Controller Class", &mut info.category);
                copy_wstring(GainController::NAME, &mut info.name);
                info.class_flags = component_flags::DISTRIBUTABLE;
                copy_cstring("Fx", &mut info.sub_categories);
                copy_wstring("Vendor", &mut info.vendor);
                copy_wstring("0.1.0", &mut info.version);
                copy_wstring("VST 3.7", &mut info.sdk_version);
            }
            _ => {
                return result::INVALID_ARGUMENT;
            }
        }

        result::OK
    }

    unsafe extern "system" fn set_host_context(
        _this: *mut c_void,
        _context: *mut *const FUnknown,
    ) -> TResult {
        result::NOT_IMPLEMENTED
    }
}

static PLUGIN_FACTORY_3_VTABLE: IPluginFactory3 = IPluginFactory3 {
    plugin_factory_2: IPluginFactory2 {
        plugin_factory: IPluginFactory {
            unknown: FUnknown {
                query_interface: Factory::query_interface,
                add_ref: Factory::add_ref,
                release: Factory::release,
            },
            get_factory_info: Factory::get_factory_info,
            count_classes: Factory::count_classes,
            get_class_info: Factory::get_class_info,
            create_instance: Factory::create_instance,
        },
        get_class_info_2: Factory::get_class_info_2,
    },
    get_class_info_unicode: Factory::get_class_info_unicode,
    set_host_context: Factory::set_host_context,
};

static PLUGIN_FACTORY: Factory = Factory { plugin_factory_3: &PLUGIN_FACTORY_3_VTABLE };

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
extern "system" fn BundleEntry(_bundle_ref: *mut c_void) -> bool {
    true
}

#[cfg(target_os = "macos")]
#[no_mangle]
extern "system" fn BundleExit() -> bool {
    true
}

#[cfg(target_os = "linux")]
#[no_mangle]
extern "system" fn ModuleEntry(_library_handle: *mut c_void) -> bool {
    true
}

#[cfg(target_os = "linux")]
#[no_mangle]
extern "system" fn ModuleExit() -> bool {
    true
}

#[no_mangle]
extern "system" fn GetPluginFactory() -> *mut c_void {
    &PLUGIN_FACTORY as *const Factory as *mut c_void
}

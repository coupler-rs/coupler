use std::ffi::c_void;
use std::os::raw::c_char;

pub type TUID = [u8; 16];

#[cfg(target_os = "windows")]
pub const fn iid(a: u32, b: u32, c: u32, d: u32) -> TUID {
    [
        ((a & 0x000000FF) >> 0) as u8,
        ((a & 0x0000FF00) >> 8) as u8,
        ((a & 0x00FF0000) >> 16) as u8,
        ((a & 0xFF000000) >> 24) as u8,
        ((b & 0x00FF0000) >> 16) as u8,
        ((b & 0xFF000000) >> 24) as u8,
        ((b & 0x000000FF) >> 0) as u8,
        ((b & 0x0000FF00) >> 8) as u8,
        ((c & 0xFF000000) >> 24) as u8,
        ((c & 0x00FF0000) >> 16) as u8,
        ((c & 0x0000FF00) >> 8) as u8,
        ((c & 0x000000FF) >> 0) as u8,
        ((d & 0xFF000000) >> 24) as u8,
        ((d & 0x00FF0000) >> 16) as u8,
        ((d & 0x0000FF00) >> 8) as u8,
        ((d & 0x000000FF) >> 0) as u8,
    ]
}

#[cfg(not(target_os = "windows"))]
pub const fn iid(a: u32, b: u32, c: u32, d: u32) -> TUID {
    [
        ((a & 0xFF000000) >> 24) as u8,
        ((a & 0x00FF0000) >> 16) as u8,
        ((a & 0x0000FF00) >> 8) as u8,
        ((a & 0x000000FF) >> 0) as u8,
        ((b & 0xFF000000) >> 24) as u8,
        ((b & 0x00FF0000) >> 16) as u8,
        ((b & 0x0000FF00) >> 8) as u8,
        ((b & 0x000000FF) >> 0) as u8,
        ((c & 0xFF000000) >> 24) as u8,
        ((c & 0x00FF0000) >> 16) as u8,
        ((c & 0x0000FF00) >> 8) as u8,
        ((c & 0x000000FF) >> 0) as u8,
        ((d & 0xFF000000) >> 24) as u8,
        ((d & 0x00FF0000) >> 16) as u8,
        ((d & 0x0000FF00) >> 8) as u8,
        ((d & 0x000000FF) >> 0) as u8,
    ]
}

pub type TResult = i32;
pub type TBool = u8;

#[cfg(target_os = "windows")]
#[allow(overflowing_literals)]
pub mod result {
    use super::TResult;
    pub const NO_INTERFACE: TResult = 0x80004002;
    pub const OK: TResult = 0x00000000;
    pub const TRUE: TResult = 0x00000000;
    pub const FALSE: TResult = 0x00000001;
    pub const INVALID_ARGUMENT: TResult = 0x80070057;
    pub const NOT_IMPLEMENTED: TResult = 0x80004001;
    pub const INTERNAL_ERROR: TResult = 0x80004005;
    pub const NOT_INITIALIZED: TResult = 0x8000FFFF;
    pub const OUT_OF_MEMORY: TResult = 0x8007000E;
}

#[cfg(not(target_os = "windows"))]
#[allow(overflowing_literals)]
pub mod result {
    use super::TResult;
    pub const NO_INTERFACE: TResult = -1;
    pub const OK: TResult = 0;
    pub const TRUE: TResult = 1;
    pub const FALSE: TResult = 2;
    pub const INVALID_ARGUMENT: TResult = 3;
    pub const NOT_IMPLEMENTED: TResult = 4;
    pub const INTERNAL_ERROR: TResult = 5;
    pub const NOT_INITIALIZED: TResult = 6;
    pub const OUT_OF_MEMORY: TResult = 7;
}

#[repr(C)]
pub struct FUnknown {
    pub query_interface: unsafe extern "system" fn(
        this: *mut c_void,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> TResult,
    pub add_ref: unsafe extern "system" fn(this: *mut c_void) -> u32,
    pub release: unsafe extern "system" fn(this: *mut c_void) -> u32,
}

impl FUnknown {
    pub const IID: TUID = iid(0x00000000, 0x00000000, 0xC0000000, 0x00000046);
}

#[repr(C)]
pub struct IBStream {
    pub unknown: FUnknown,
    pub read: unsafe extern "system" fn(
        buffer: *mut c_void,
        num_bytes: i32,
        num_bytes_read: *mut i32,
    ) -> TResult,
    pub write: unsafe extern "system" fn(
        buffer: *const c_void,
        num_bytes: i32,
        num_bytes_written: *mut i32,
    ) -> TResult,
    pub seek: unsafe extern "system" fn(pos: i64, mode: i32, result: *mut i64) -> TResult,
    pub tell: unsafe extern "system" fn(pos: *mut i64) -> TResult,
}

impl IBStream {
    pub const IID: TUID = iid(0xC3BF6EA2, 0x30994752, 0x9B6BF990, 0x1EE33E9B);

    pub const SEEK_SET: i32 = 0;
    pub const SEEK_CUR: i32 = 1;
    pub const SEEK_END: i32 = 2;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PFactoryInfo {
    pub vendor: [c_char; 64],
    pub url: [c_char; 256],
    pub email: [c_char; 128],
    pub flags: i32,
}

impl PFactoryInfo {
    pub const NO_FLAGS: i32 = 0;
    pub const CLASSES_DISCARDABLE: i32 = 1 << 0;
    pub const LICENSE_CHECK: i32 = 1 << 1;
    pub const COMPONENT_NON_DISCARDABLE: i32 = 1 << 3;
    pub const UNICODE: i32 = 1 << 4;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PClassInfo {
    pub cid: TUID,
    pub cardinality: i32,
    pub category: [c_char; 32],
    pub name: [c_char; 64],
}

impl PClassInfo {
    pub const MANY_INSTANCES: i32 = 0x7FFFFFFF;
}

#[repr(C)]
pub struct IPluginFactory {
    pub unknown: FUnknown,
    pub get_factory_info:
        unsafe extern "system" fn(this: *mut c_void, info: *mut PFactoryInfo) -> TResult,
    pub count_classes: unsafe extern "system" fn(this: *mut c_void) -> i32,
    pub get_class_info:
        unsafe extern "system" fn(this: *mut c_void, index: i32, info: *mut PClassInfo) -> TResult,
    pub create_instance: unsafe extern "system" fn(
        this: *mut c_void,
        cid: *const c_char,
        iid: *const c_char,
        obj: *mut *mut c_void,
    ) -> TResult,
}

impl IPluginFactory {
    pub const IID: TUID = iid(0x7A4D811C, 0x52114A1F, 0xAED9D2EE, 0x0B43BF9F);
}

pub mod component_flags {
    pub const DISTRIBUTABLE: u32 = 1 << 0;
    pub const SIMPLE_MODE_SUPPORTED: u32 = 1 << 1;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PClassInfo2 {
    pub cid: TUID,
    pub cardinality: i32,
    pub category: [c_char; 32],
    pub name: [c_char; 64],
    pub class_flags: u32,
    pub sub_categories: [c_char; 128],
    pub vendor: [c_char; 64],
    pub version: [c_char; 64],
    pub sdk_version: [c_char; 64],
}

#[repr(C)]
pub struct IPluginFactory2 {
    pub plugin_factory: IPluginFactory,
    pub get_class_info_2:
        unsafe extern "system" fn(this: *mut c_void, index: i32, info: *mut PClassInfo2) -> TResult,
}

impl IPluginFactory2 {
    pub const IID: TUID = iid(0x0007B650, 0xF24B4C0B, 0xA464EDB9, 0xF00B2ABB);
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PClassInfoW {
    pub cid: TUID,
    pub cardinality: i32,
    pub category: [c_char; 32],
    pub name: [i16; 64],
    pub class_flags: u32,
    pub sub_categories: [c_char; 128],
    pub vendor: [i16; 64],
    pub version: [i16; 64],
    pub sdk_version: [i16; 64],
}

#[repr(C)]
pub struct IPluginFactory3 {
    pub plugin_factory_2: IPluginFactory2,
    pub get_class_info_unicode:
        unsafe extern "system" fn(this: *mut c_void, index: i32, info: *mut PClassInfoW) -> TResult,
    pub set_host_context:
        unsafe extern "system" fn(this: *mut c_void, context: *mut *const FUnknown) -> TResult,
}

impl IPluginFactory3 {
    pub const IID: TUID = iid(0x4555A2AB, 0xC1234E57, 0x9B122910, 0x36878931);
}

#[repr(C)]
pub struct IPluginBase {
    pub unknown: FUnknown,
    pub initialize: unsafe extern "system" fn(this: *mut c_void, context: *mut FUnknown) -> TResult,
    pub terminate: unsafe extern "system" fn(this: *mut c_void) -> TResult,
}

impl IPluginBase {
    pub const IID: TUID = iid(0x22888DDB, 0x156E45AE, 0x8358B348, 0x08190625);
}

pub type TChar = i16;
pub type String128 = [TChar; 128];

pub type MediaType = i32;

pub mod media_types {
    use super::MediaType;
    pub const AUDIO: MediaType = 0;
    pub const EVENT: MediaType = 1;
}

pub type BusDirection = i32;

pub mod bus_directions {
    use super::BusDirection;
    pub const INPUT: BusDirection = 0;
    pub const OUTPUT: BusDirection = 1;
}

pub type BusType = i32;

pub mod bus_types {
    use super::BusType;
    pub const MAIN: BusType = 0;
    pub const AUX: BusType = 0;
}

pub type IoMode = i32;

pub mod io_modes {
    use super::IoMode;
    pub const SIMPLE: IoMode = 0;
    pub const ADVANCED: IoMode = 0;
    pub const OFFLINE_PROCESSING: IoMode = 0;
}

pub type TQuarterNotes = f64;

pub type SpeakerArrangement = u64;
pub type Speaker = u64;

pub mod speakers {
    use super::Speaker;
    pub const L: Speaker = 1 << 0;
    pub const R: Speaker = 1 << 1;
    pub const M: Speaker = 1 << 19;
}

pub mod speaker_arrangements {
    use super::{speakers, SpeakerArrangement};
    pub const EMPTY: SpeakerArrangement = 0;
    pub const MONO: SpeakerArrangement = speakers::M;
    pub const STEREO: SpeakerArrangement = speakers::L | speakers::R;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BusInfo {
    pub media_type: MediaType,
    pub direction: BusDirection,
    pub channel_count: i32,
    pub name: String128,
    pub bus_type: BusType,
    pub flags: u32,
}

impl BusInfo {
    pub const DEFAULT_ACTIVE: u32 = 1 << 0;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct RoutingInfo {
    pub media_type: MediaType,
    pub bus_index: i32,
    pub channel: i32,
}

#[repr(C)]
pub struct IComponent {
    pub plugin_base: IPluginBase,
    pub get_controller_class_id:
        unsafe extern "system" fn(this: *mut c_void, class_id: *const TUID) -> TResult,
    pub set_io_mode: unsafe extern "system" fn(this: *mut c_void, mode: IoMode) -> TResult,
    pub get_bus_count: unsafe extern "system" fn(
        this: *mut c_void,
        media_type: MediaType,
        dir: BusDirection,
    ) -> i32,
    pub get_bus_info: unsafe extern "system" fn(
        this: *mut c_void,
        media_type: MediaType,
        dir: BusDirection,
        index: i32,
        bus: *mut BusInfo,
    ) -> TResult,
    pub get_routing_info: unsafe extern "system" fn(
        this: *mut c_void,
        in_info: *mut RoutingInfo,
        out_info: *mut RoutingInfo,
    ) -> TResult,
    pub activate_bus: unsafe extern "system" fn(
        this: *mut c_void,
        media_type: MediaType,
        dir: BusDirection,
        index: i32,
        state: TBool,
    ) -> TResult,
    pub set_active: unsafe extern "system" fn(this: *mut c_void, state: TBool) -> TResult,
    pub set_state: unsafe extern "system" fn(this: *mut c_void, state: *mut IBStream) -> TResult,
    pub get_state: unsafe extern "system" fn(this: *mut c_void, state: *mut IBStream) -> TResult,
}

impl IComponent {
    pub const IID: TUID = iid(0xE831FF31, 0xF2D54301, 0x928EBBEE, 0x25697802);
}

pub mod symbolic_sample_sizes {
    pub const SAMPLE_32: i32 = 0;
    pub const SAMPLE_64: i32 = 1;
}

pub mod process_modes {
    pub const REALTIME: i32 = 0;
    pub const PREFETCH: i32 = 1;
    pub const OFFLINE: i32 = 2;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ProcessSetup {
    pub process_mode: i32,
    pub symbolic_sample_size: i32,
    pub max_samples_per_block: i32,
    pub sample_rate: f64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct AudioBusBuffers {
    pub num_channels: i32,
    pub silence_flags: u64,
    pub channel_buffers: *mut *mut c_void,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FrameRate {
    pub frames_per_second: u32,
    pub flags: u32,
}

impl FrameRate {
    pub const PULL_DOWN_RATE: u32 = 1 << 0;
    pub const DROP_RATE: u32 = 1 << 1;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Chord {
    pub key_note: u8,
    pub root_note: u8,
    pub chord_mask: i16,
}

#[allow(overflowing_literals)]
impl Chord {
    pub const CHORD_MASK: i16 = 0x0FFF;
    pub const RESERVED_MASK: i16 = 0xF000;
}

pub mod process_states {
    pub const PLAYING: u32 = 1 << 1;
    pub const CYCLE_ACTIVE: u32 = 1 << 2;
    pub const RECORDING: u32 = 1 << 3;

    pub const SYSTEM_TIME_VALID: u32 = 1 << 8;
    pub const CONT_TIME_VALID: u32 = 1 << 17;

    pub const PROJECT_TIME_MUSIC_VALID: u32 = 1 << 9;
    pub const BAR_POSITION_VALID: u32 = 1 << 11;
    pub const CYCLE_VALID: u32 = 1 << 12;

    pub const TEMPO_VALID: u32 = 1 << 10;
    pub const TIME_SIG_VALID: u32 = 1 << 13;
    pub const CHORD_VALID: u32 = 1 << 18;

    pub const SMPTE_VALID: u32 = 1 << 14;
    pub const CLOCK_VALID: u32 = 1 << 15;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ProcessContext {
    pub state: u32,
    pub sample_rate: f64,
    pub project_time_samples: i64,
    pub system_time: i64,
    pub continuous_time_samples: i64,
    pub project_time_music: TQuarterNotes,
    pub bar_position_music: TQuarterNotes,
    pub cycle_start_music: TQuarterNotes,
    pub cycle_end_music: TQuarterNotes,
    pub tempo: f64,
    pub time_sig_numerator: i32,
    pub time_sig_denominator: i32,
    pub chord: Chord,
    pub smpte_offset_sub_frames: i32,
    pub frame_rate: FrameRate,
    pub samples_to_next_clock: i32,
}

#[repr(C)]
pub struct IParamValueQueue {
    pub unknown: FUnknown,
    pub get_parameter_id: unsafe extern "system" fn(this: *mut c_void) -> u32,
    pub get_point_count: unsafe extern "system" fn(this: *mut c_void) -> i32,
    pub get_point: unsafe extern "system" fn(
        this: *mut c_void,
        index: i32,
        sample_offset: *mut i32,
        value: *mut f64,
    ) -> TResult,
    pub add_point: unsafe extern "system" fn(
        this: *mut c_void,
        sample_offset: i32,
        value: f64,
        index: *mut i32,
    ) -> TResult,
}

impl IParamValueQueue {
    pub const IID: TUID = iid(0x01263A18, 0xED074F6F, 0x98C9D356, 0x4686F9BA);
}

#[repr(C)]
pub struct IParameterChanges {
    pub unknown: FUnknown,
    pub get_parameter_count: unsafe extern "system" fn(this: *mut c_void) -> i32,
    pub get_parameter_data:
        unsafe extern "system" fn(this: *mut c_void, index: i32) -> *mut *const IParamValueQueue,
    pub add_parameter_data: unsafe extern "system" fn(
        this: *mut c_void,
        id: *const u32,
        index: *mut i32,
    ) -> *mut *const IParamValueQueue,
}

impl IParameterChanges {
    pub const IID: TUID = iid(0xA4779663, 0x0BB64A56, 0xB44384A8, 0x466FEB9D);
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct NoteOnEvent {
    pub channel: i16,
    pub pitch: i16,
    pub tuning: f32,
    pub velocity: f32,
    pub length: i32,
    pub note_id: i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct NoteOffEvent {
    pub channel: i16,
    pub pitch: i16,
    pub velocity: f32,
    pub note_id: i32,
    pub tuning: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DataEvent {
    pub size: u32,
    pub data_type: u32,
    pub bytes: *const u8,
}

pub mod data_types {
    pub const MIDI_SYS_EX: u32 = 0;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PolyPressureEvent {
    pub channel: i16,
    pub pitch: u16,
    pub pressure: f32,
    pub note_id: i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct NoteExpressionValueEvent {
    pub type_id: u32,
    pub note_id: i32,
    pub value: f64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct NoteExpressionTextEvent {
    pub type_id: u32,
    pub note_id: i32,
    pub text_len: u32,
    pub text: *const TChar,
}

pub mod note_expression_type_ids {
    pub const VOLUME: u32 = 0;
    pub const PAN: u32 = 1;
    pub const TUNING: u32 = 2;
    pub const VIBRATO: u32 = 3;
    pub const EXPRESSION: u32 = 4;
    pub const BRIGHTNESS: u32 = 5;
    pub const TEXT: u32 = 6;
    pub const PHONEME: u32 = 7;
    pub const CUSTOM_START: u32 = 100000;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ChordEvent {
    pub root: i16,
    pub bass_note: i16,
    pub mask: i16,
    pub text_len: u16,
    pub text: *const TChar,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ScaleEvent {
    pub root: i16,
    pub mask: i16,
    pub text_len: u16,
    pub text: *const TChar,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LegacyMidiCCOutEvent {
    pub control_number: u8,
    pub channel: i8,
    pub value: i8,
    pub value2: i8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union EventData {
    pub note_on: NoteOnEvent,
    pub note_off: NoteOffEvent,
    pub data: DataEvent,
    pub poly_pressure: PolyPressureEvent,
    pub note_expression_value: NoteExpressionValueEvent,
    pub note_expression_text: NoteExpressionTextEvent,
    pub chord: ChordEvent,
    pub scale: ScaleEvent,
    pub legacy_midi_cc_out: LegacyMidiCCOutEvent,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Event {
    pub bus_index: i32,
    pub sample_offset: i32,
    pub ppq_position: TQuarterNotes,
    pub flags: u16,
    pub event_type: u16,
    pub data: EventData,
}

pub mod event_flags {
    pub const IS_LIVE: u16 = 1 << 0;
    pub const USER_RESERVED_1: u16 = 1 << 14;
    pub const USER_RESERVED_2: u16 = 1 << 15;
}

pub mod event_types {
    pub const NOTE_ON: u16 = 0;
    pub const NOTE_OFF: u16 = 1;
    pub const DATA: u16 = 2;
    pub const POLY_PRESSURE: u16 = 3;
    pub const NOTE_EXPRESSION_VALUE: u16 = 4;
    pub const NOTE_EXPRESSION_TEXT: u16 = 5;
    pub const CHORD: u16 = 6;
    pub const SCALE: u16 = 7;
    pub const LEGACY_MIDI_CC_OUT: u16 = 65535;
}

#[repr(C)]
pub struct IEventList {
    pub unknown: FUnknown,
    pub get_event_count: unsafe extern "system" fn(this: *mut c_void) -> i32,
    pub get_event:
        unsafe extern "system" fn(this: *mut c_void, index: i32, event: *mut Event) -> TResult,
    pub add_event: unsafe extern "system" fn(this: *mut c_void, event: *const Event) -> TResult,
}

impl IEventList {
    pub const IID: TUID = iid(0x3A2C4214, 0x346349FE, 0xB2C4F397, 0xB9695A44);
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ProcessData {
    pub process_mode: i32,
    pub symbolic_sample_size: i32,
    pub num_samples: i32,
    pub num_inputs: i32,
    pub num_outputs: i32,
    pub inputs: *mut AudioBusBuffers,
    pub outputs: *mut AudioBusBuffers,
    pub input_parameter_changes: *mut *const IParameterChanges,
    pub output_parameter_changes: *mut *const IParameterChanges,
    pub input_events: *mut *const IEventList,
    pub output_events: *mut *const IEventList,
    pub process_context: *mut ProcessContext,
}

#[repr(C)]
pub struct IAudioProcessor {
    pub unknown: FUnknown,
    pub set_bus_arrangements: unsafe extern "system" fn(
        this: *mut c_void,
        inputs: *const SpeakerArrangement,
        num_ins: i32,
        outputs: *const SpeakerArrangement,
        num_outs: i32,
    ) -> TResult,
    pub get_bus_arrangement: unsafe extern "system" fn(
        this: *mut c_void,
        dir: BusDirection,
        index: i32,
        arr: *mut SpeakerArrangement,
    ) -> TResult,
    pub can_process_sample_size:
        unsafe extern "system" fn(this: *mut c_void, symbolic_sample_size: i32) -> TResult,
    pub get_latency_samples: unsafe extern "system" fn(this: *mut c_void) -> u32,
    pub setup_processing:
        unsafe extern "system" fn(this: *mut c_void, setup: *mut ProcessSetup) -> TResult,
    pub set_processing: unsafe extern "system" fn(this: *mut c_void, state: TBool) -> TResult,
    pub process: unsafe extern "system" fn(this: *mut c_void, data: *mut ProcessData) -> TResult,
    pub get_tail_samples: unsafe extern "system" fn(this: *mut c_void) -> u32,
}

impl IAudioProcessor {
    pub const IID: TUID = iid(0x42043F99, 0xB7DA453C, 0xA569E79D, 0x9AAEC33D);
}

#[repr(C)]
pub struct IAudioPresentationLatency {
    pub unknown: FUnknown,
    pub set_audio_presentation_latency_samples: unsafe extern "system" fn(
        this: *mut c_void,
        dir: BusDirection,
        bus_index: i32,
        latency_in_samples: u32,
    ) -> TResult,
}

impl IAudioPresentationLatency {
    pub const IID: TUID = iid(0x309ECE78, 0xEB7D4fae, 0x8B2225D9, 0x09FD08B6);
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ParameterInfo {
    pub id: u32,
    pub title: String128,
    pub short_title: String128,
    pub units: String128,
    pub step_count: i32,
    pub default_normalized_value: f64,
    pub unit_id: i32,
    pub flags: i32,
}

impl ParameterInfo {
    pub const CAN_AUTOMATE: i32 = 1 << 0;
    pub const IS_READ_ONLY: i32 = 1 << 1;
    pub const IS_WRAP_AROUND: i32 = 1 << 2;
    pub const IS_LIST: i32 = 1 << 3;
    pub const IS_PROGRAM_CHANGE: i32 = 1 << 15;
    pub const IS_BYPASS: i32 = 1 << 16;
}

pub mod restart_flags {
    pub const RELOAD_COMPONENT: i32 = 1 << 0;
    pub const IO_CHANGED: i32 = 1 << 0;
    pub const PARAM_VALUES_CHANGED: i32 = 1 << 0;
    pub const LATENCY_CHANGED: i32 = 1 << 0;
    pub const PARAM_TITLES_CHANGED: i32 = 1 << 0;
    pub const MIDI_CC_ASSIGNMENT_CHANGED: i32 = 1 << 0;
    pub const NOTE_EXPRESSION_CHANGED: i32 = 1 << 0;
    pub const IO_TITLES_CHANGED: i32 = 1 << 0;
    pub const PREFETCHABLE_SUPPORT_CHANGED: i32 = 1 << 0;
    pub const ROUTING_INFO_CHANGED: i32 = 1 << 0;
}

#[repr(C)]
pub struct IComponentHandler {
    pub unknown: FUnknown,
    pub begin_edit: unsafe extern "system" fn(this: *mut c_void, id: u32) -> TResult,
    pub perform_edit:
        unsafe extern "system" fn(this: *mut c_void, id: u32, value_normalized: f64) -> TResult,
    pub end_edit: unsafe extern "system" fn(this: *mut c_void, id: u32) -> TResult,
    pub restart_component: unsafe extern "system" fn(this: *mut c_void, flags: i32) -> TResult,
}

impl IComponentHandler {
    pub const IID: TUID = iid(0x93A0BEA3, 0x0BD045DB, 0x8E890B0C, 0xC1E46AC6);
}

#[repr(C)]
pub struct IComponentHandler2 {
    pub unknown: FUnknown,
    pub set_dirty: unsafe extern "system" fn(this: *mut c_void, state: TBool) -> TResult,
    pub request_open_editor:
        unsafe extern "system" fn(this: *mut c_void, name: *const c_char) -> TResult,
    pub start_group_edit: unsafe extern "system" fn(this: *mut c_void) -> TResult,
    pub finish_group_edit: unsafe extern "system" fn(this: *mut c_void) -> TResult,
}

impl IComponentHandler2 {
    pub const IID: TUID = iid(0xF040B4B3, 0xA36045EC, 0xABCDC045, 0xB4D5A2CC);
}

#[repr(C)]
pub struct IEditController {
    pub plugin_base: IPluginBase,
    pub set_component_state:
        unsafe extern "system" fn(this: *mut c_void, state: *mut *const IBStream) -> TResult,
    pub set_state:
        unsafe extern "system" fn(this: *mut c_void, state: *mut *const IBStream) -> TResult,
    pub get_state:
        unsafe extern "system" fn(this: *mut c_void, state: *mut *const IBStream) -> TResult,
    pub get_parameter_count: unsafe extern "system" fn(this: *mut c_void) -> i32,
    pub get_parameter_info: unsafe extern "system" fn(
        this: *mut c_void,
        param_index: i32,
        info: *mut ParameterInfo,
    ) -> TResult,
    pub get_param_string_by_value: unsafe extern "system" fn(
        this: *mut c_void,
        id: u32,
        value_normalized: f64,
        string: *mut String128,
    ) -> TResult,
    pub get_param_value_by_string: unsafe extern "system" fn(
        this: *mut c_void,
        id: u32,
        string: *const TChar,
        value_normalized: *mut f64,
    ) -> TResult,
    pub normalized_param_to_plain:
        unsafe extern "system" fn(this: *mut c_void, id: u32, value_normalized: f64) -> f64,
    pub plain_param_to_normalized:
        unsafe extern "system" fn(this: *mut c_void, id: u32, plain_value: f64) -> f64,
    pub get_param_normalized: unsafe extern "system" fn(this: *mut c_void, id: u32) -> f64,
    pub set_param_normalized:
        unsafe extern "system" fn(this: *mut c_void, id: u32, value: f64) -> TResult,
    pub set_component_handler: unsafe extern "system" fn(
        this: *mut c_void,
        handler: *mut *const IComponentHandler,
    ) -> TResult,
    pub create_view:
        unsafe extern "system" fn(this: *mut c_void, name: *const c_char) -> *mut *const IPlugView,
}

impl IEditController {
    pub const IID: TUID = iid(0xDCD7BBE3, 0x7742448D, 0xA874AACC, 0x979C759E);
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ViewRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

pub mod platform_types {
    use std::os::raw::c_char;
    pub const HWND: *const c_char = b"HWND\0".as_ptr() as *const c_char;
    pub const HI_VIEW: *const c_char = b"HIView\0".as_ptr() as *const c_char;
    pub const NS_VIEW: *const c_char = b"NSView\0".as_ptr() as *const c_char;
    pub const UI_VIEW: *const c_char = b"UIView\0".as_ptr() as *const c_char;
    pub const X11_EMBED_WINDOW_ID: *const c_char = b"X11EmbedWindowID\0".as_ptr() as *const c_char;
}

#[repr(C)]
pub struct IPlugView {
    pub unknown: FUnknown,
    pub is_platform_type_supported:
        unsafe extern "system" fn(this: *mut c_void, platform_type: *const c_char) -> TResult,
    pub attached:
        unsafe extern "system" fn(this: *mut c_void, platform_type: *const c_char) -> TResult,
    pub removed: unsafe extern "system" fn(this: *mut c_void) -> TResult,
    pub on_wheel: unsafe extern "system" fn(this: *mut c_void, distance: f32) -> TResult,
    pub on_key_down: unsafe extern "system" fn(
        this: *mut c_void,
        key: i16,
        key_code: i16,
        modifiers: i16,
    ) -> TResult,
    pub on_key_up: unsafe extern "system" fn(
        this: *mut c_void,
        key: i16,
        key_code: i16,
        modifiers: i16,
    ) -> TResult,
    pub get_size: unsafe extern "system" fn(this: *mut c_void, size: *mut ViewRect) -> TResult,
    pub on_size: unsafe extern "system" fn(this: *mut c_void, new_size: *const ViewRect) -> TResult,
    pub on_focus: unsafe extern "system" fn(this: *mut c_void, state: TBool) -> TResult,
    pub set_frame:
        unsafe extern "system" fn(this: *mut c_void, frame: *mut *const IPlugFrame) -> TResult,
    pub can_resize: unsafe extern "system" fn(this: *mut c_void) -> TResult,
    pub check_size_constraint:
        unsafe extern "system" fn(this: *mut c_void, rect: *mut ViewRect) -> TResult,
}

impl IPlugView {
    pub const IID: TUID = iid(0x5BC32507, 0xD06049EA, 0xA6151B52, 0x2B755B29);
}

#[repr(C)]
pub struct IPlugFrame {
    pub unknown: FUnknown,
    pub resize_view: unsafe extern "system" fn(
        this: *mut c_void,
        view: *mut *const IPlugView,
        new_size: *mut ViewRect,
    ) -> TResult,
}

impl IPlugFrame {
    pub const IID: TUID = iid(0x367FAF01, 0xAFA94693, 0x8D4DA2A0, 0xED0882A3);
}

#[repr(C)]
pub struct IPlugViewContentScaleSupport {
    pub unknown: FUnknown,
    pub set_content_scale_factor:
        unsafe extern "system" fn(this: *mut c_void, factor: f32) -> TResult,
}

impl IPlugViewContentScaleSupport {
    pub const IID: TUID = iid(0x65ED9690, 0x8AC44525, 0x8AADEF7A, 0x72EA703F);
}

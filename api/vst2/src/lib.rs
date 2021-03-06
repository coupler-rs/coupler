use std::ffi::c_void;
use std::os::raw::c_char;

#[repr(C)]
pub struct AEffect {
    pub magic: i32,
    pub dispatcher: DispatcherProc,
    pub process: ProcessProc,
    pub set_parameter: SetParameterProc,
    pub get_parameter: GetParameterProc,
    pub num_programs: i32,
    pub num_params: i32,
    pub num_inputs: i32,
    pub num_outputs: i32,
    pub flags: i32,
    pub _reserved_1: isize,
    pub _reserved_2: isize,
    pub initial_delay: i32,
    pub real_qualities: i32,
    pub off_qualities: i32,
    pub io_ratio: f32,
    pub object: *mut c_void,
    pub user: *mut c_void,
    pub unique_id: i32,
    pub version: i32,
    pub process_replacing: ProcessProc,
    pub process_replacing_f64: ProcessF64Proc,
    pub _future: [u8; 56],
}

pub type HostCallbackProc = unsafe extern "C" fn(
    effect: *mut AEffect,
    opcode: i32,
    index: i32,
    value: isize,
    ptr: *mut c_void,
    opt: f32,
) -> isize;

pub type DispatcherProc = extern "C" fn(
    effect: *mut AEffect,
    opcode: i32,
    index: i32,
    value: isize,
    ptr: *mut c_void,
    opt: f32,
) -> isize;

pub type ProcessProc = extern "C" fn(
    effect: *mut AEffect,
    inputs: *const *const f32,
    outputs: *mut *mut f32,
    sample_frames: i32,
);

pub type ProcessF64Proc = extern "C" fn(
    effect: *mut AEffect,
    inputs: *const *const f64,
    outputs: *mut *mut f64,
    sample_frames: i32,
);

pub type SetParameterProc = extern "C" fn(effect: *mut AEffect, index: i32, parameter: f32);

pub type GetParameterProc = extern "C" fn(effect: *mut AEffect, index: i32) -> f32;

pub mod string_constants {
    pub const MAX_PROG_NAME_LEN: usize = 24;
    pub const MAX_PARAM_STR_LEN: usize = 8;
    pub const MAX_VENDOR_STR_LEN: usize = 64;
    pub const MAX_PRODUCT_STR_LEN: usize = 64;
    pub const MAX_EFFECT_NAME_LEN: usize = 32;
    pub const MAX_NAME_LEN: usize = 64;
    pub const MAX_LABEL_LEN: usize = 64;
    pub const MAX_SHORT_LABEL_LEN: usize = 8;
    pub const MAX_CATEGORY_LABEL_LEN: usize = 24;
    pub const MAX_FILE_NAME_LEN: usize = 100;
}

impl AEffect {
    pub const MAGIC: i32 =
        (('V' as i32) << 24) | (('s' as i32) << 16) | (('t' as i32) << 8) | (('P' as i32) << 0);
}

pub mod effect_flags {
    pub const HAS_EDITOR: i32 = 1 << 0;
    pub const CAN_REPLACING: i32 = 1 << 4;
    pub const PROGRAM_CHUNKS: i32 = 1 << 5;
    pub const IS_SYNTH: i32 = 1 << 8;
    pub const NO_SOUND_IN_STOP: i32 = 1 << 9;
    pub const CAN_DOUBLE_REPLACING: i32 = 1 << 12;
}

pub mod host_opcodes {
    pub const AUTOMATE: i32 = 0;
    pub const VERSION: i32 = 1;
    pub const CURRENT_ID: i32 = 2;
    pub const IDLE: i32 = 3;
    pub const PIN_CONNECTED: i32 = 4;
    pub const WANT_MIDI: i32 = 6;
    pub const GET_TIME: i32 = 7;
    pub const PROCESS_EVENTS: i32 = 8;
    pub const SET_TIME: i32 = 9;
    pub const TEMPO_AT: i32 = 10;
    pub const GET_NUM_AUTOMATABLE_PARAMETERS: i32 = 11;
    pub const GET_PARAMETER_QUANTIZATION: i32 = 12;
    pub const IO_CHANGED: i32 = 13;
    pub const NEED_IDLE: i32 = 14;
    pub const SIZE_WINDOW: i32 = 15;
    pub const GET_SAMPLE_RATE: i32 = 16;
    pub const GET_BLOCK_SIZE: i32 = 17;
    pub const GET_INPUT_LATENCY: i32 = 18;
    pub const GET_OUTPUT_LATENCY: i32 = 19;
    pub const GET_PREVIOUS_PLUGIN: i32 = 20;
    pub const GET_NEXT_PLUGIN: i32 = 21;
    pub const WILL_REPLACE_OR_ACCUMULATE: i32 = 22;
    pub const GET_CURRENT_PROCESS_LEVEL: i32 = 23;
    pub const GET_AUTOMATION_STATE: i32 = 24;
    pub const OFFLINE_START: i32 = 25;
    pub const OFFLINE_READ: i32 = 26;
    pub const OFFLINE_WRITE: i32 = 27;
    pub const GET_CURRENT_PASS: i32 = 28;
    pub const GET_CURRENT_META_PASS: i32 = 29;
    pub const SET_OUTPUT_SAMPLE_RATE: i32 = 30;
    pub const GET_OUTPUT_SPEAKER_ARRANGEMENT: i32 = 31;
    pub const GET_VENDOR_STRING: i32 = 32;
    pub const GET_PRODUCT_STRING: i32 = 33;
    pub const GET_VENDOR_VERSION: i32 = 34;
    pub const VENDOR_SPECIFIC: i32 = 35;
    pub const SET_ICON: i32 = 36;
    pub const CAN_DO: i32 = 37;
    pub const GET_LANGUAGE: i32 = 38;
    pub const OPEN_WINDOW: i32 = 39;
    pub const CLOSE_WINDOW: i32 = 40;
    pub const GET_DIRECTORY: i32 = 41;
    pub const UPDATE_DISPLAY: i32 = 42;
    pub const BEGIN_EDIT: i32 = 43;
    pub const END_EDIT: i32 = 44;
    pub const OPEN_FILE_SELECTOR: i32 = 45;
    pub const CLOSE_FILE_SELECTOR: i32 = 46;
    pub const EDIT_FILE: i32 = 47;
    pub const GET_CHUNK_FILE: i32 = 48;
    pub const GET_INPUT_SPEAKER_ARRANGEMENT: i32 = 49;
}

pub mod effect_opcodes {
    pub const OPEN: i32 = 0;
    pub const CLOSE: i32 = 1;
    pub const SET_PROGRAM: i32 = 2;
    pub const GET_PROGRAM: i32 = 3;
    pub const SET_PROGRAM_NAME: i32 = 4;
    pub const GET_PROGRAM_NAME: i32 = 5;
    pub const GET_PARAM_LABEL: i32 = 6;
    pub const GET_PARAM_DISPLAY: i32 = 7;
    pub const GET_PARAM_NAME: i32 = 8;
    pub const GET_VU: i32 = 9;
    pub const SET_SAMPLE_RATE: i32 = 10;
    pub const SET_BLOCK_SIZE: i32 = 11;
    pub const MAINS_CHANGED: i32 = 12;
    pub const EDIT_GET_RECT: i32 = 13;
    pub const EDIT_OPEN: i32 = 14;
    pub const EDIT_CLOSE: i32 = 15;
    pub const EDIT_DRAW: i32 = 16;
    pub const EDIT_MOUSE: i32 = 17;
    pub const EDIT_KEY: i32 = 18;
    pub const EDIT_IDLE: i32 = 19;
    pub const EDIT_TOP: i32 = 20;
    pub const EDIT_SLEEP: i32 = 21;
    pub const IDENTIFY: i32 = 22;
    pub const GET_CHUNK: i32 = 23;
    pub const SET_CHUNK: i32 = 24;
    pub const PROCESS_EVENTS: i32 = 25;
    pub const CAN_BE_AUTOMATED: i32 = 26;
    pub const STRING_TO_PARAMETER: i32 = 27;
    pub const GET_NUM_PROGRAM_CATEGORIES: i32 = 28;
    pub const GET_PROGRAM_NAME_INDEXED: i32 = 29;
    pub const COPY_PROGRAM: i32 = 30;
    pub const CONNECT_INPUT: i32 = 31;
    pub const CONNECT_OUTPUT: i32 = 32;
    pub const GET_INPUT_PROPERTIES: i32 = 33;
    pub const GET_OUTPUT_PROPERTIES: i32 = 34;
    pub const GET_PLUGIN_CATEGORY: i32 = 35;
    pub const GET_CURRENT_POSITION: i32 = 36;
    pub const GET_DESTINATION_BUFFER: i32 = 37;
    pub const OFFLINE_NOTIFY: i32 = 38;
    pub const OFFLINE_PREPARE: i32 = 39;
    pub const OFFLINE_RUN: i32 = 40;
    pub const PROCESS_VARIABLE_IO: i32 = 41;
    pub const SET_SPEAKER_ARRANGEMENT: i32 = 42;
    pub const SET_BLOCK_SIZE_AND_SAMPLE_RATE: i32 = 43;
    pub const SET_BYPASS: i32 = 44;
    pub const GET_EFFECT_NAME: i32 = 45;
    pub const GET_ERROR_TEXT: i32 = 46;
    pub const GET_VENDOR_STRING: i32 = 47;
    pub const GET_PRODUCT_STRING: i32 = 48;
    pub const GET_VENDOR_VERSION: i32 = 49;
    pub const VENDOR_SPECIFIC: i32 = 50;
    pub const CAN_DO: i32 = 51;
    pub const GET_TAIL_SIZE: i32 = 52;
    pub const IDLE: i32 = 53;
    pub const GET_ICON: i32 = 54;
    pub const SET_VIEW_POSITION: i32 = 55;
    pub const GET_PARAMETER_PROPERTIES: i32 = 56;
    pub const KEYS_REQUIRED: i32 = 57;
    pub const GET_VST_VERSION: i32 = 58;
    pub const EDIT_KEY_DOWN: i32 = 59;
    pub const EDIT_KEY_UP: i32 = 60;
    pub const SET_EDIT_KNOB_MODE: i32 = 61;
    pub const GET_MIDI_PROGRAM_NAME: i32 = 62;
    pub const GET_CURRENT_MIDI_PROGRAM: i32 = 63;
    pub const GET_MIDI_PROGRAM_CATEGORY: i32 = 64;
    pub const HAS_MIDI_PROGRAMS_CHANGED: i32 = 65;
    pub const GET_MIDI_KEY_NAME: i32 = 66;
    pub const BEGIN_SET_PROGRAM: i32 = 67;
    pub const END_SET_PROGRAM: i32 = 68;
    pub const GET_SPEAKER_ARRANGEMENT: i32 = 69;
    pub const SHELL_GET_NEXT_PLUGIN: i32 = 70;
    pub const START_PROCESS: i32 = 71;
    pub const STOP_PROCESS: i32 = 72;
    pub const SET_TOTAL_SAMPLE_TO_PROCESS: i32 = 73;
    pub const SET_PAN_LAW: i32 = 74;
    pub const BEGIN_LOAD_BANK: i32 = 75;
    pub const BEGIN_LOAD_PROGRAM: i32 = 76;
    pub const SET_PROCESS_PRECISION: i32 = 77;
    pub const GET_NUM_MIDI_INPUT_CHANNELS: i32 = 78;
    pub const GET_NUM_MIDI_OUTPUT_CHANNELS: i32 = 79;
}

#[repr(C)]
pub struct Rect {
    pub top: i16,
    pub left: i16,
    pub bottom: i16,
    pub right: i16,
}

#[repr(C)]
pub struct Event {
    pub event_type: i32,
    pub byte_size: i32,
    pub delta_frames: i32,
    pub flags: i32,
    pub data: [u8; 16],
}

pub mod event_types {
    pub const MIDI: i32 = 1;
    pub const SYSEX: i32 = 6;
}

#[repr(C)]
pub struct Events {
    pub num_events: i32,
    pub _reserved: isize,
    pub events: [*const Event; 2],
}

#[repr(C)]
pub struct MidiEvent {
    pub event_type: i32,
    pub byte_size: i32,
    pub delta_frames: i32,
    pub flags: i32,
    pub note_length: i32,
    pub note_offset: i32,
    pub midi_data: [u8; 4],
    pub detune: i8,
    pub note_off_velocity: u8,
    pub _reserved_1: u8,
    pub _reserved_2: u8,
}

pub mod midi_event_flags {
    pub const IS_REALTIME: i32 = 1 << 0;
}

#[repr(C)]
pub struct MidiSysexEvent {
    pub event_type: i32,
    pub byte_size: i32,
    pub delta_frames: i32,
    pub flags: i32,
    pub dump_bytes: i32,
    pub reserved1: isize,
    pub sysex_dump: *const u8,
    pub reserved2: isize,
}

#[repr(C)]
pub struct TimeInfo {
    pub sample_pos: f64,
    pub sample_rate: f64,
    pub nanoseconds: f64,
    pub ppq_pos: f64,
    pub tempo: f64,
    pub bar_start_pos: f64,
    pub cycle_start_pos: f64,
    pub cycle_end_pos: f64,
    pub time_sig_numerator: i32,
    pub time_sig_denominator: i32,
    pub smpte_offset: i32,
    pub smpte_frame_rate: i32,
    pub samples_to_next_clock: i32,
    pub flags: i32,
}

pub mod time_info_flags {
    pub const TRANSPORT_CHANGED: i32 = 1;
    pub const TRANSPORT_PLAYING: i32 = 1 << 1;
    pub const TRANSPORT_CYCLE_ACTIVE: i32 = 1 << 2;
    pub const TRANSPORT_RECORDING: i32 = 1 << 3;
    pub const AUTOMATION_WRITING: i32 = 1 << 6;
    pub const AUTOMATION_READING: i32 = 1 << 7;
    pub const NANOS_VALID: i32 = 1 << 8;
    pub const PPQ_POS_VALID: i32 = 1 << 9;
    pub const TEMPO_VALID: i32 = 1 << 10;
    pub const BARS_VALID: i32 = 1 << 11;
    pub const CYCLE_POS_VALID: i32 = 1 << 12;
    pub const TIME_SIG_VALID: i32 = 1 << 13;
    pub const SMPTE_VALID: i32 = 1 << 14;
    pub const CLOCK_VALID: i32 = 1 << 15;
}

pub mod smpte_frame_rates {
    pub const SMPTE_24_FPS: i32 = 0;
    pub const SMPTE_25_FPS: i32 = 1;
    pub const SMPTE_2997_FPS: i32 = 2;
    pub const SMPTE_30_FPS: i32 = 3;
    pub const SMPTE_2997_DROP_FPS: i32 = 4;
    pub const SMPTE_30_DROP_FPS: i32 = 5;
    pub const SMPTE_FILM_16_MM: i32 = 6;
    pub const SMPTE_FILM_35_MM: i32 = 7;
    pub const SMPTE_239_FPS: i32 = 10;
    pub const SMPTE_249_FPS: i32 = 11;
    pub const SMPTE_599_FPS: i32 = 12;
    pub const SMPTE_60_FPS: i32 = 13;
}

#[repr(C)]
pub struct VariableIo {
    pub inputs: *const *const f32,
    pub outputs: *mut *mut f32,
    pub num_samples_input: i32,
    pub num_samples_output: i32,
    pub num_samples_input_processed: *const i32,
    pub num_samples_output_processed: *mut i32,
}

pub mod host_language {
    pub const ENGLISH: i32 = 1;
    pub const GERMAN: i32 = 2;
    pub const FRENCH: i32 = 3;
    pub const ITALIAN: i32 = 4;
    pub const SPANISH: i32 = 5;
    pub const JAPANESE: i32 = 6;
}

pub mod process_precision {
    pub const F32: i32 = 0;
    pub const F64: i32 = 1;
}

#[repr(C)]
pub struct ParameterProperties {
    pub step_float: f32,
    pub small_step_float: f32,
    pub large_step_float: f32,
    pub label: [c_char; string_constants::MAX_LABEL_LEN],
    pub flags: i32,
    pub min_integer: i32,
    pub max_integer: i32,
    pub step_integer: i32,
    pub large_step_integer: i32,
    pub short_label: [c_char; string_constants::MAX_SHORT_LABEL_LEN],
    pub display_index: i16,
    pub category: i16,
    pub num_parameters_in_category: i16,
    pub _reserved: i16,
    pub category_label: [c_char; string_constants::MAX_CATEGORY_LABEL_LEN],
    pub _future: [u8; 16],
}

pub mod parameter_flags {
    pub const IS_SWITCH: i32 = 1 << 0;
    pub const USES_INTEGER_MIN_MAX: i32 = 1 << 1;
    pub const USES_FLOAT_STEP: i32 = 1 << 2;
    pub const USES_INT_STEP: i32 = 1 << 3;
    pub const SUPPORTS_DISPLAY_INDEX: i32 = 1 << 4;
    pub const SUPPORTS_DISPLAY_CATEGORY: i32 = 1 << 5;
    pub const CAN_RAMP: i32 = 1 << 6;
}

#[repr(C)]
pub struct PinProperties {
    pub label: [c_char; string_constants::MAX_LABEL_LEN],
    pub flags: i32,
    pub arrangement_type: i32,
    pub short_label: [c_char; string_constants::MAX_SHORT_LABEL_LEN],
    pub _future: [u8; 48],
}

pub mod pin_flags {
    pub const IS_ACTIVE: i32 = 1 << 0;
    pub const IS_STEREO: i32 = 1 << 1;
    pub const USE_SPEAKER: i32 = 1 << 2;
}

pub mod plugin_categories {
    pub const UNKNOWN: i32 = 0;
    pub const EFFECT: i32 = 1;
    pub const SYNTH: i32 = 2;
    pub const ANALYSIS: i32 = 3;
    pub const MASTERING: i32 = 4;
    pub const SPACIALIZER: i32 = 5;
    pub const ROOM_FX: i32 = 6;
    pub const SURROUND_FX: i32 = 7;
    pub const RESTORATION: i32 = 8;
    pub const OFFLINE_PROCESS: i32 = 9;
    pub const SHELL: i32 = 10;
    pub const GENERATOR: i32 = 11;
}

#[repr(C)]
pub struct MidiProgramName {
    pub this_program_index: i32,
    pub name: [c_char; string_constants::MAX_NAME_LEN],
    pub midi_program: i8,
    pub midi_bank_msb: i8,
    pub midi_bank_lsb: i8,
    pub _reserved: u8,
    pub parent_category_index: i32,
    pub flags: i32,
}

#[repr(C)]
pub struct MidiProgramCategory {
    pub this_category_index: i32,
    pub name: [c_char; string_constants::MAX_NAME_LEN],
    pub parent_category_index: i32,
    pub flags: i32,
}

#[repr(C)]
pub struct MidiKeyName {
    pub this_program_index: i32,
    pub this_key_number: i32,
    pub key_name: [c_char; string_constants::MAX_NAME_LEN],
    pub _reserved: i32,
    pub flags: i32,
}

#[repr(C)]
pub struct SpeakerArrangement {
    pub speaker_arrangement_type: i32,
    pub num_channels: i32,
    pub speakers: [SpeakerProperties; 8],
}

#[repr(C)]
pub struct SpeakerProperties {
    pub azimuth: f32,
    pub elevation: f32,
    pub radius: f32,
    pub _reserved: f32,
    pub name: [c_char; string_constants::MAX_NAME_LEN],
    pub speaker_type: i32,
    pub _future: [u8; 28],
}

pub mod speaker_types {
    pub const UNDEFINED: i32 = 0x7fffffff;
    pub const MONO: i32 = 0;
    pub const LEFT: i32 = 1;
    pub const RIGHT: i32 = 2;
    pub const CENTER: i32 = 3;
    pub const LFE: i32 = 4;
    pub const LEFT_SURROUND: i32 = 5;
    pub const RIGHT_SURROUND: i32 = 6;
    pub const LEFT_OF_CENTER: i32 = 7;
    pub const RIGHT_OF_CENTER: i32 = 8;
    pub const SURROUND: i32 = 9;
    pub const CENTER_OF_SURROUND: i32 = 10;
    pub const SIDE_LEFT: i32 = 11;
    pub const SIDE_RIGHT: i32 = 12;
    pub const TOP_MIDDLE: i32 = 13;
    pub const TOP_FRONT_LEFT: i32 = 14;
    pub const TOP_FRONT_CENTER: i32 = 15;
    pub const TOP_FRONT_RIGHT: i32 = 16;
    pub const TOP_REAR_LEFT: i32 = 17;
    pub const TOP_REAR_CENTER: i32 = 18;
    pub const TOP_REAR_RIGHT: i32 = 19;
    pub const LFE_2: i32 = 20;
    pub const USER_32: i32 = -32;
    pub const USER_31: i32 = -31;
    pub const USER_30: i32 = -30;
    pub const USER_29: i32 = -29;
    pub const USER_28: i32 = -28;
    pub const USER_27: i32 = -27;
    pub const USER_26: i32 = -26;
    pub const USER_25: i32 = -25;
    pub const USER_24: i32 = -24;
    pub const USER_23: i32 = -23;
    pub const USER_22: i32 = -22;
    pub const USER_21: i32 = -21;
    pub const USER_20: i32 = -20;
    pub const USER_19: i32 = -19;
    pub const USER_18: i32 = -18;
    pub const USER_17: i32 = -17;
    pub const USER_16: i32 = -16;
    pub const USER_15: i32 = -15;
    pub const USER_14: i32 = -14;
    pub const USER_13: i32 = -13;
    pub const USER_12: i32 = -12;
    pub const USER_11: i32 = -11;
    pub const USER_10: i32 = -10;
    pub const USER_9: i32 = -9;
    pub const USER_8: i32 = -8;
    pub const USER_7: i32 = -7;
    pub const USER_6: i32 = -6;
    pub const USER_5: i32 = -5;
    pub const USER_4: i32 = -4;
    pub const USER_3: i32 = -3;
    pub const USER_2: i32 = -2;
    pub const USER_1: i32 = -1;
}

pub mod speaker_arrangement_types {
    pub const USER_DEFINED: i32 = -2;
    pub const EMPTY: i32 = -1;
    pub const MONO: i32 = 0;
    pub const STEREO: i32 = 1;
    pub const STEREO_SURROUND: i32 = 2;
    pub const STEREO_CENTER: i32 = 3;
    pub const STEREO_SIDE: i32 = 4;
    pub const STEREO_CENTER_LFE: i32 = 5;
    pub const SURROUND_3_0_CINE: i32 = 6;
    pub const SURROUND_3_0_MUSIC: i32 = 7;
    pub const SURROUND_3_1_CINE: i32 = 8;
    pub const SURROUND_3_1_MUSIC: i32 = 9;
    pub const SURROUND_4_0_CINE: i32 = 10;
    pub const SURROUND_4_0_MUSIC: i32 = 11;
    pub const SURROUND_4_1_CINE: i32 = 12;
    pub const SURROUND_4_1_MUSIC: i32 = 13;
    pub const SURROUND_5_0: i32 = 14;
    pub const SURROUND_5_1: i32 = 15;
    pub const SURROUND_6_0_CINE: i32 = 16;
    pub const SURROUND_6_0_MUSIC: i32 = 17;
    pub const SURROUND_6_1_CINE: i32 = 18;
    pub const SURROUND_6_1_MUSIC: i32 = 19;
    pub const SURROUND_7_0_CINE: i32 = 20;
    pub const SURROUND_7_0_MUSIC: i32 = 21;
    pub const SURROUND_7_1_CINE: i32 = 22;
    pub const SURROUND_7_1_MUSIC: i32 = 23;
    pub const SURROUND_8_0_CINE: i32 = 24;
    pub const SURROUND_8_0_MUSIC: i32 = 25;
    pub const SURROUND_8_1_CINE: i32 = 26;
    pub const SURROUND_8_1_MUSIC: i32 = 27;
    pub const SURROUND_10_2: i32 = 28;
}

#[repr(C)]
pub struct OfflineTask {
    pub process_name: [c_char; 96],
    pub read_position: f64,
    pub write_position: f64,
    pub read_count: i32,
    pub write_count: i32,
    pub size_input_buffer: i32,
    pub size_output_buffer: i32,
    pub input_buffer: *const c_void,
    pub output_buffer: *mut c_void,
    pub position_to_process_from: f64,
    pub num_frames_to_process: f64,
    pub max_frames_to_write: f64,
    pub extra_buffer: *mut c_void,
    pub value: i32,
    pub index: i32,
    pub num_frames_in_source_file: f64,
    pub source_sample_rate: f64,
    pub destination_sample_rate: f64,
    pub num_source_channels: i32,
    pub num_destination_channels: i32,
    pub source_format: i32,
    pub destination_format: i32,
    pub output_text: [c_char; 512],
    pub progress: f64,
    pub progress_mode: i32,
    pub progress_text: [c_char; 100],
    pub flags: i32,
    pub return_value: i32,
    pub host_owned: *const c_void,
    pub plug_owned: *mut c_void,
    pub _future: [u8; 1024],
}

pub mod offline_task_flags {
    pub const INVALID_PARAMETER: i32 = 1 << 0;
    pub const NEW_FILE: i32 = 1 << 1;
    pub const PLUGIN_ERROR: i32 = 1 << 10;
    pub const INTERLEAVED_AUDIO: i32 = 1 << 11;
    pub const TEMP_OUTPUT_FILE: i32 = 1 << 12;
    pub const FLOAT_OUTPUT_FILE: i32 = 1 << 13;
    pub const RANDOM_WRITE: i32 = 1 << 14;
    pub const STRETCH: i32 = 1 << 15;
    pub const NO_THREAD: i32 = 1 << 16;
}

pub mod offline_options {
    pub const AUDIO: i32 = 0;
    pub const PEAKS: i32 = 1;
    pub const PARAMETER: i32 = 2;
    pub const MARKER: i32 = 3;
    pub const CURSOR: i32 = 4;
    pub const SELECTION: i32 = 5;
    pub const QUERY_FILES: i32 = 6;
}

#[repr(C)]
pub struct AudioFile {
    pub flags: i32,
    pub host_owned: *const c_void,
    pub plugin_owned: *mut c_void,
    pub name: [c_char; string_constants::MAX_FILE_NAME_LEN],
    pub unique_id: i32,
    pub sample_rate: f64,
    pub num_channels: i32,
    pub num_frames: f64,
    pub format: i32,
    pub edit_cursor_position: f64,
    pub selection_start: f64,
    pub selection_size: f64,
    pub selected_channels_mask: i32,
    pub num_markers: i32,
    pub time_ruler_unit: i32,
    pub time_ruler_offset: f64,
    pub tempo: f64,
    pub time_sig_numerator: i32,
    pub time_sig_denominator: i32,
    pub ticks_per_black_note: i32,
    pub smpte_frame_rate: i32,
    pub _future: [u8; 64],
}

pub mod audio_file_flags {
    pub const READ_ONLY: i32 = 1 << 0;
    pub const NO_RATE_CONVERSION: i32 = 1 << 1;
    pub const NO_CHANNEL_CHANGE: i32 = 1 << 2;
    pub const CAN_PROCESS_SELECTION: i32 = 1 << 10;
    pub const NO_CROSS_FADE: i32 = 1 << 11;
    pub const WANT_READ: i32 = 1 << 12;
    pub const WANT_WRITE: i32 = 1 << 13;
    pub const WANT_WRITE_MARKER: i32 = 1 << 14;
    pub const WANT_MOVE_CURSOR: i32 = 1 << 15;
    pub const WANT_SELECT: i32 = 1 << 1;
}

#[repr(C)]
pub struct AudioFileMarker {
    pub position: f64,
    pub name: [c_char; 32],
    pub file_marker_type: i32,
    pub id: i32,
    pub _reserved: i32,
}

#[repr(C)]
pub struct _Window {
    pub title: [c_char; 128],
    pub x_pos: i16,
    pub y_pos: i16,
    pub width: i16,
    pub height: i16,
    pub style: i32,
    pub parent: *mut c_void,
    pub user_handle: *mut c_void,
    pub win_handle: *mut c_void,
    pub _future: [u8; 104],
}

#[repr(C)]
pub struct KeyCode {
    pub character: i32,
    pub virtual_key: u8,
    pub modifier: u8,
}

pub mod virtual_keys {
    pub const BACK: i32 = 1;
    pub const TAB: i32 = 2;
    pub const CLEAR: i32 = 3;
    pub const RETURN: i32 = 4;
    pub const PAUSE: i32 = 5;
    pub const ESCAPE: i32 = 6;
    pub const SPACE: i32 = 7;
    pub const NEXT: i32 = 8;
    pub const END: i32 = 9;
    pub const HOME: i32 = 10;
    pub const LEFT: i32 = 11;
    pub const UP: i32 = 12;
    pub const RIGHT: i32 = 13;
    pub const DOWN: i32 = 14;
    pub const PAGE_UP: i32 = 15;
    pub const PAGE_DOWN: i32 = 16;
    pub const SELECT: i32 = 17;
    pub const PRINT: i32 = 18;
    pub const ENTER: i32 = 19;
    pub const SNAPSHOT: i32 = 20;
    pub const INSERT: i32 = 21;
    pub const DELETE: i32 = 22;
    pub const HELP: i32 = 23;
    pub const NUMPAD_0: i32 = 24;
    pub const NUMPAD_1: i32 = 25;
    pub const NUMPAD_2: i32 = 26;
    pub const NUMPAD_3: i32 = 27;
    pub const NUMPAD_4: i32 = 28;
    pub const NUMPAD_5: i32 = 29;
    pub const NUMPAD_6: i32 = 30;
    pub const NUMPAD_7: i32 = 31;
    pub const NUMPAD_8: i32 = 32;
    pub const NUMPAD_9: i32 = 33;
    pub const MULTIPLY: i32 = 34;
    pub const ADD: i32 = 35;
    pub const SEPARATOR: i32 = 36;
    pub const SUBTRACT: i32 = 37;
    pub const DECIMAL: i32 = 38;
    pub const DIVIDE: i32 = 39;
    pub const F1: i32 = 40;
    pub const F2: i32 = 41;
    pub const F3: i32 = 42;
    pub const F4: i32 = 43;
    pub const F5: i32 = 44;
    pub const F6: i32 = 45;
    pub const F7: i32 = 46;
    pub const F8: i32 = 47;
    pub const F9: i32 = 48;
    pub const F10: i32 = 49;
    pub const F11: i32 = 50;
    pub const F12: i32 = 51;
    pub const NUM_LOCK: i32 = 52;
    pub const SCROLL: i32 = 53;
    pub const SHIFT: i32 = 54;
    pub const CONTROL: i32 = 55;
    pub const ALT: i32 = 56;
    pub const EQUALS: i32 = 57;
}

pub mod modifier_keys {
    pub const SHIFT: i32 = 1 << 0;
    pub const ALTERNATE: i32 = 1 << 1;
    pub const COMMAND: i32 = 1 << 2;
    pub const CONTROL: i32 = 1 << 3;
}

#[repr(C)]
pub struct FileSelect {
    pub command: i32,
    pub file_select_type: i32,
    pub mac_creator: i32,
    pub num_file_types: i32,
    pub file_types: *const FileType,
    pub title: [c_char; 1024],
    pub initial_path: *const c_char,
    pub return_path: *const c_char,
    pub size_return_path: i32,
    pub return_multiple_paths: *const *const c_char,
    pub num_return_paths: i32,
    pub _reserved: isize,
    pub _future: [u8; 116],
}

#[repr(C)]
pub struct FileType {
    pub name: [c_char; 128],
    pub mac_type: [c_char; 8],
    pub dos_type: [c_char; 8],
    pub unix_type: [c_char; 8],
    pub mime_type_1: [c_char; 128],
    pub mime_type_2: [c_char; 128],
}

pub mod file_select_commands {
    pub const FILE_LOAD: i32 = 0;
    pub const FILE_SAVE: i32 = 1;
    pub const MULTIPLE_FILES_LOAD: i32 = 2;
    pub const DIRECTORY_SELECT: i32 = 3;
}

pub mod file_select_types {
    pub const FILE: i32 = 0;
}

#[repr(C)]
pub struct PatchChunkInfo {
    pub version: i32,
    pub plugin_unique_id: i32,
    pub plugin_version: i32,
    pub num_elements: i32,
    pub _future: [u8; 48],
}

pub mod pan_law_type {
    pub const LINEAR: i32 = 0;
    pub const EQUAL_POWER: i32 = 0;
}

pub mod process_levels {
    pub const UNKNOWN: i32 = 0;
    pub const USER: i32 = 1;
    pub const REALTIME: i32 = 2;
    pub const PREFETCH: i32 = 3;
    pub const OFFLINE: i32 = 4;
}

pub mod automation_states {
    pub const UNSUPPORTED: i32 = 0;
    pub const OFF: i32 = 1;
    pub const READ: i32 = 2;
    pub const WRITE: i32 = 3;
    pub const READ_WRITE: i32 = 4;
}

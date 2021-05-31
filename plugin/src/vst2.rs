use crate::Plugin;

use std::os::raw::c_char;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{ffi, ptr};

pub use vst2;
use vst2::*;

unsafe fn copy_cstring(string: &str, dst: *mut c_char, len: usize) {
    let name = ffi::CString::new(string).unwrap_or_else(|_| ffi::CString::default());
    ptr::copy_nonoverlapping(
        name.as_ptr(),
        dst as *mut c_char,
        name.as_bytes().len().min(len),
    );
}

#[repr(C)]
struct Wrapper {
    effect: AEffect,
    params: Vec<AtomicU64>,
}

extern "C" fn dispatcher<P: Plugin>(
    effect: *mut AEffect,
    opcode: i32,
    index: i32,
    _value: isize,
    ptr: *mut std::ffi::c_void,
    _opt: f32,
) -> isize {
    unsafe {
        use effect_opcodes::*;

        let wrapper = &*(effect as *const Wrapper);

        match opcode {
            OPEN => {}
            CLOSE => {
                drop(Box::from_raw(effect));
            },
            SET_PROGRAM => {}
            GET_PROGRAM => {}
            SET_PROGRAM_NAME => {}
            GET_PROGRAM_NAME => {}
            GET_PARAM_LABEL => {
                if let Some(param) = P::PARAMS.get(index as usize) {
                    copy_cstring(param.label, ptr as *mut c_char, string_constants::MAX_PARAM_STR_LEN);
                }
                return 0;
            }
            GET_PARAM_DISPLAY => {
                if let Some(param) = wrapper.params.get(index as usize) {
                    let display = format!("{}", f64::from_bits(param.load(Ordering::Relaxed)));
                    copy_cstring(&display, ptr as *mut c_char, string_constants::MAX_PARAM_STR_LEN);
                }
                return 0;
            }
            GET_PARAM_NAME => {
                if let Some(param) = P::PARAMS.get(index as usize) {
                    copy_cstring(param.name, ptr as *mut c_char, string_constants::MAX_PARAM_STR_LEN);
                }
                return 0;
            }
            SET_SAMPLE_RATE => {}
            SET_BLOCK_SIZE => {}
            MAINS_CHANGED => {}
            EDIT_GET_RECT => {}
            EDIT_OPEN => {}
            EDIT_CLOSE => {}
            EDIT_IDLE => {}
            GET_CHUNK => {}
            SET_CHUNK => {}
            PROCESS_EVENTS => {}
            CAN_BE_AUTOMATED => {
                if let Some(_) = P::PARAMS.get(index as usize) {
                    return 1;
                } else {
                    return 0;
                }
            }
            STRING_TO_PARAMETER => {}
            GET_PROGRAM_NAME_INDEXED => {}
            GET_INPUT_PROPERTIES => {}
            GET_OUTPUT_PROPERTIES => {}
            GET_PLUGIN_CATEGORY => {}
            OFFLINE_NOTIFY => {}
            OFFLINE_PREPARE => {}
            OFFLINE_RUN => {}
            PROCESS_VARIABLE_IO => {}
            SET_SPEAKER_ARRANGEMENT => {}
            SET_BYPASS => {}
            GET_EFFECT_NAME => {
                copy_cstring(P::INFO.name, ptr as *mut c_char, string_constants::MAX_EFFECT_NAME_LEN);
                return 1;
            }
            GET_VENDOR_STRING => {
                copy_cstring(P::INFO.vendor, ptr as *mut c_char, string_constants::MAX_VENDOR_STR_LEN);
                return 1;
            }
            GET_PRODUCT_STRING => {
                copy_cstring(P::INFO.name, ptr as *mut c_char, string_constants::MAX_PRODUCT_STR_LEN);
                return 1;
            }
            GET_VENDOR_VERSION => {}
            VENDOR_SPECIFIC => {}
            CAN_DO => {}
            GET_TAIL_SIZE => {}
            GET_PARAMETER_PROPERTIES => {}
            GET_VST_VERSION => {
                return 2400;
            }
            EDIT_KEY_DOWN => {}
            EDIT_KEY_UP => {}
            SET_EDIT_KNOB_MODE => {}
            GET_MIDI_PROGRAM_NAME => {}
            GET_CURRENT_MIDI_PROGRAM => {}
            GET_MIDI_PROGRAM_CATEGORY => {}
            HAS_MIDI_PROGRAMS_CHANGED => {}
            GET_MIDI_KEY_NAME => {}
            BEGIN_SET_PROGRAM => {}
            END_SET_PROGRAM => {}
            GET_SPEAKER_ARRANGEMENT => {}
            SHELL_GET_NEXT_PLUGIN => {}
            START_PROCESS => {}
            STOP_PROCESS => {}
            SET_TOTAL_SAMPLE_TO_PROCESS => {}
            SET_PAN_LAW => {}
            BEGIN_LOAD_BANK => {}
            BEGIN_LOAD_PROGRAM => {}
            SET_PROCESS_PRECISION => {}
            GET_NUM_MIDI_INPUT_CHANNELS => {}
            GET_NUM_MIDI_OUTPUT_CHANNELS => {}
            _ => {}
        }

        0
    }
}

pub extern "C" fn process(
    _effect: *mut AEffect,
    _inputs: *const *const f32,
    _outputs: *mut *mut f32,
    _sample_frames: i32,
) {
}

pub extern "C" fn process_f64(
    _effect: *mut AEffect,
    _inputs: *const *const f64,
    _outputs: *mut *mut f64,
    _sample_frames: i32,
) {
}

pub extern "C" fn set_parameter<P: Plugin>(effect: *mut AEffect, index: i32, parameter: f32) {
    unsafe {
        let wrapper = &*(effect as *const Wrapper);
        if let Some(param) = wrapper.params.get(index as usize) {
            param.store((parameter as f64).to_bits(), Ordering::Relaxed);
        }
    }
}

pub extern "C" fn get_parameter<P: Plugin>(effect: *mut AEffect, index: i32) -> f32 {
    unsafe {
        let wrapper = &*(effect as *const Wrapper);
        if let Some(param) = wrapper.params.get(index as usize) {
            f64::from_bits(param.load(Ordering::Relaxed)) as f32
        } else {
            0.0
        }
    }
}

pub fn plugin_main<P: Plugin>(_host_callback: HostCallbackProc) -> *mut AEffect {
    let mut params = Vec::with_capacity(P::PARAMS.len());
    for _ in 0..P::PARAMS.len() {
        params.push(AtomicU64::new(0f64.to_bits()));
    }

    Box::into_raw(Box::new(Wrapper {
        effect: AEffect {
            magic: AEffect::MAGIC,
            dispatcher: dispatcher::<P>,
            process,
            set_parameter: set_parameter::<P>,
            get_parameter: get_parameter::<P>,
            num_programs: 0,
            num_params: P::PARAMS.len() as i32,
            num_inputs: 2,
            num_outputs: 2,
            flags: 0,
            _reserved_1: 0,
            _reserved_2: 0,
            initial_delay: 0,
            real_qualities: 0,
            off_qualities: 0,
            io_ratio: 0.0,
            object: std::ptr::null_mut(),
            user: std::ptr::null_mut(),
            unique_id: cconst(P::INFO.unique_id[0], P::INFO.unique_id[1], P::INFO.unique_id[2], P::INFO.unique_id[3]),
            version: 0,
            process_replacing: process,
            process_replacing_f64: process_f64,
            _future: [0; 56],
        },
        params,
    })) as *mut AEffect
}

#[macro_export]
macro_rules! vst2 {
    ($plugin:ty) => {
        #[cfg(not(test))]
        #[no_mangle]
        pub extern "C" fn main(
            host_callback: $crate::vst2::vst2::HostCallbackProc,
        ) -> *mut $crate::vst2::vst2::AEffect {
            $crate::vst2::plugin_main::<$plugin>(host_callback)
        }

        #[no_mangle]
        pub extern "C" fn VSTPluginMain(
            host_callback: $crate::vst2::vst2::HostCallbackProc,
        ) -> *mut $crate::vst2::vst2::AEffect {
            $crate::vst2::plugin_main::<$plugin>(host_callback)
        }
    };
}

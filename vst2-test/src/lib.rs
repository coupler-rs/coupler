use vst2::*;

extern "C" fn dispatcher(
    effect: *mut AEffect,
    opcode: i32,
    index: i32,
    value: isize,
    ptr: *mut std::ffi::c_void,
    opt: f32,
) -> isize {
    0
}

pub extern "C" fn process(
    effect: *mut AEffect,
    inputs: *const *const f32,
    outputs: *mut *mut f32,
    sample_frames: i32,
) {
}

pub extern "C" fn process_f64(
    effect: *mut AEffect,
    inputs: *const *const f64,
    outputs: *mut *mut f64,
    sample_frames: i32,
) {
}

pub extern "C" fn set_parameter(effect: *mut AEffect, index: i32, parameter: f32) {}

pub extern "C" fn get_parameter(effect: *mut AEffect, index: i32) -> f32 {
    0.0
}

#[no_mangle]
pub extern "C" fn main(host_callback: HostCallbackProc) -> *mut AEffect {
    VSTPluginMain(host_callback)
}

#[no_mangle]
pub extern "C" fn VSTPluginMain(host_callback: HostCallbackProc) -> *mut AEffect {
    Box::into_raw(Box::new(AEffect {
        magic: AEffect::MAGIC,
        dispatcher,
        process,
        set_parameter,
        get_parameter,
        num_programs: 0,
        num_params: 0,
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
        unique_id: 0,
        version: 0,
        process_replacing: process,
        process_replacing_f64: process_f64,
        _future: [0; 56],
    }))
}

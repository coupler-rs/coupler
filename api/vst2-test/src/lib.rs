use vst2::*;

extern "C" fn dispatcher(
    _effect: *mut AEffect,
    _opcode: i32,
    _index: i32,
    _value: isize,
    _ptr: *mut std::ffi::c_void,
    _opt: f32,
) -> isize {
    0
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

pub extern "C" fn set_parameter(_effect: *mut AEffect, _index: i32, _parameter: f32) {}

pub extern "C" fn get_parameter(_effect: *mut AEffect, _index: i32) -> f32 {
    0.0
}

#[no_mangle]
pub extern "C" fn main(host_callback: HostCallbackProc) -> *mut AEffect {
    VSTPluginMain(host_callback)
}

#[no_mangle]
pub extern "C" fn VSTPluginMain(_host_callback: HostCallbackProc) -> *mut AEffect {
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

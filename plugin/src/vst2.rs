use crate::{Editor, EditorContext, ParamInfo, ParamValues, ParentWindow, Plugin};

use std::cell::{Cell, UnsafeCell};
use std::os::raw::c_char;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::{ffi, ptr, slice};

use raw_window_handle::RawWindowHandle;

pub use vst2 as vst2_api;
use vst2::*;

unsafe fn copy_cstring(string: &str, dst: *mut c_char, len: usize) {
    let name = ffi::CString::new(string).unwrap_or_else(|_| ffi::CString::default());
    ptr::copy_nonoverlapping(
        name.as_ptr(),
        dst as *mut c_char,
        name.as_bytes_with_nul().len().min(len),
    );
}

#[repr(C)]
struct Wrapper<P: Plugin> {
    effect: AEffect,
    params: Arc<Vec<AtomicU64>>,
    plugin_state: UnsafeCell<PluginState<P>>,
    editor_state: UnsafeCell<EditorState<P>>,
}

struct PluginState<P: Plugin> {
    params: Vec<f64>,
    plugin: P,
}

struct EditorState<P: Plugin> {
    rect: Rect,
    context: Rc<Vst2EditorContext>,
    editor: P::Editor,
}

struct Vst2EditorContext {
    alive: Cell<bool>,
    host_callback: HostCallbackProc,
    effect: Cell<*mut AEffect>,
    params: Arc<Vec<AtomicU64>>,
}

impl EditorContext for Vst2EditorContext {
    fn get(&self, param_info: &ParamInfo) -> f64 {
        if let Some(param) = self.params.get(param_info.id as usize) {
            f64::from_bits(param.load(Ordering::Relaxed))
        } else {
            0.0
        }
    }

    fn set(&self, param_info: &ParamInfo, value: f64) {
        if let Some(param) = self.params.get(param_info.id as usize) {
            param.store(value.to_bits(), Ordering::Relaxed);

            if self.alive.get() {
                unsafe {
                    (self.host_callback)(
                        self.effect.get(),
                        host_opcodes::AUTOMATE,
                        param_info.id as i32,
                        0,
                        ptr::null_mut(),
                        value as f32,
                    );
                }
            }
        }
    }

    fn begin_edit(&self, param_info: &ParamInfo) {
        if let Some(_param) = self.params.get(param_info.id as usize) {
            if self.alive.get() {
                unsafe {
                    (self.host_callback)(
                        self.effect.get(),
                        host_opcodes::BEGIN_EDIT,
                        param_info.id as i32,
                        0,
                        ptr::null_mut(),
                        0.0,
                    );
                }
            }
        }
    }

    fn end_edit(&self, param_info: &ParamInfo) {
        if let Some(_param) = self.params.get(param_info.id as usize) {
            if self.alive.get() {
                unsafe {
                    (self.host_callback)(
                        self.effect.get(),
                        host_opcodes::END_EDIT,
                        param_info.id as i32,
                        0,
                        ptr::null_mut(),
                        0.0,
                    );
                }
            }
        }
    }
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

        let wrapper_ptr = effect as *mut Wrapper<P>;

        match opcode {
            OPEN => {}
            CLOSE => {
                {
                    let wrapper = &*wrapper_ptr;
                    let editor_state = &mut *wrapper.editor_state.get();
                    editor_state.context.alive.set(false);
                }
                drop(Box::from_raw(wrapper_ptr));
            }
            SET_PROGRAM => {}
            GET_PROGRAM => {}
            SET_PROGRAM_NAME => {}
            GET_PROGRAM_NAME => {}
            GET_PARAM_LABEL => {
                if let Some(param_info) = P::PARAMS.get(index as usize) {
                    copy_cstring(
                        param_info.label,
                        ptr as *mut c_char,
                        string_constants::MAX_PARAM_STR_LEN,
                    );
                }
                return 0;
            }
            GET_PARAM_DISPLAY => {
                let wrapper = &*wrapper_ptr;
                let param = wrapper.params.get(index as usize);
                let param_info = P::PARAMS.get(index as usize);
                if let (Some(param), Some(param_info)) = (param, param_info) {
                    let value = f64::from_bits(param.load(Ordering::Relaxed));
                    let display = (param_info.to_string)(value);
                    copy_cstring(&display, ptr as *mut c_char, string_constants::MAX_PARAM_STR_LEN);
                }
                return 0;
            }
            GET_PARAM_NAME => {
                if let Some(param_info) = P::PARAMS.get(index as usize) {
                    copy_cstring(
                        param_info.name,
                        ptr as *mut c_char,
                        string_constants::MAX_PARAM_STR_LEN,
                    );
                }
                return 0;
            }
            SET_SAMPLE_RATE => {}
            SET_BLOCK_SIZE => {}
            MAINS_CHANGED => {}
            EDIT_GET_RECT => {
                let wrapper = &*wrapper_ptr;
                let editor_state = &mut *wrapper.editor_state.get();

                let (width, height) = editor_state.editor.size();
                let rect = &mut editor_state.rect;
                rect.right = width.round() as i16;
                rect.bottom = height.round() as i16;
                ptr::write(ptr as *mut *const Rect, &mut editor_state.rect);

                return 1;
            }
            EDIT_OPEN => {
                let wrapper = &*wrapper_ptr;
                let editor_state = &mut *wrapper.editor_state.get();

                #[cfg(target_os = "macos")]
                let parent = {
                    use raw_window_handle::macos::MacOSHandle;
                    RawWindowHandle::MacOS(MacOSHandle {
                        ns_view: ptr as *mut ::std::ffi::c_void,
                        ..MacOSHandle::empty()
                    })
                };

                #[cfg(target_os = "windows")]
                let parent = {
                    use raw_window_handle::windows::WindowsHandle;
                    RawWindowHandle::Windows(WindowsHandle { hwnd: ptr, ..WindowsHandle::empty() })
                };

                #[cfg(target_os = "linux")]
                let parent = {
                    use raw_window_handle::unix::XcbHandle;
                    RawWindowHandle::Xcb(XcbHandle { window: ptr as u32, ..XcbHandle::empty() })
                };

                editor_state.editor.open(Some(&ParentWindow(parent)));

                return 1;
            }
            EDIT_CLOSE => {
                let wrapper = &*wrapper_ptr;
                let editor_state = &mut *wrapper.editor_state.get();

                editor_state.editor.close();

                return 1;
            }
            EDIT_IDLE => {
                #[cfg(target_os = "linux")]
                {
                    let wrapper = &*wrapper_ptr;
                    let editor_state = &mut *wrapper.editor_state.get();

                    editor_state.editor.poll();
                }
                return 1;
            }
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
            STRING_TO_PARAMETER => {
                let wrapper = &*wrapper_ptr;
                let param = wrapper.params.get(index as usize);
                let param_info = P::PARAMS.get(index as usize);
                if let (Some(param), Some(param_info)) = (param, param_info) {
                    if !ptr.is_null() {
                        let c_str = ffi::CStr::from_ptr(ptr as *const c_char);
                        if let Ok(string) = c_str.to_str() {
                            let value = (param_info.from_string)(string);
                            param.store(value.to_bits(), Ordering::Relaxed);
                        }
                    }
                    return 1;
                }
            }
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
                copy_cstring(
                    P::INFO.name,
                    ptr as *mut c_char,
                    string_constants::MAX_EFFECT_NAME_LEN,
                );
                return 1;
            }
            GET_VENDOR_STRING => {
                copy_cstring(
                    P::INFO.vendor,
                    ptr as *mut c_char,
                    string_constants::MAX_VENDOR_STR_LEN,
                );
                return 1;
            }
            GET_PRODUCT_STRING => {
                copy_cstring(
                    P::INFO.name,
                    ptr as *mut c_char,
                    string_constants::MAX_PRODUCT_STR_LEN,
                );
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

extern "C" fn process(
    _effect: *mut AEffect,
    _inputs: *const *const f32,
    _outputs: *mut *mut f32,
    _sample_frames: i32,
) {
}

extern "C" fn set_parameter<P: Plugin>(effect: *mut AEffect, index: i32, parameter: f32) {
    unsafe {
        let wrapper = &*(effect as *const Wrapper<P>);
        let param = wrapper.params.get(index as usize);
        let param_info = P::PARAMS.get(index as usize);
        if let (Some(param), Some(param_info)) = (param, param_info) {
            let value = (param_info.from_normal)(parameter as f64);
            param.store(value.to_bits(), Ordering::Relaxed);
        }
    }
}

extern "C" fn get_parameter<P: Plugin>(effect: *mut AEffect, index: i32) -> f32 {
    unsafe {
        let wrapper = &*(effect as *const Wrapper<P>);
        let param = wrapper.params.get(index as usize);
        let param_info = P::PARAMS.get(index as usize);
        if let (Some(param), Some(param_info)) = (param, param_info) {
            let value = f64::from_bits(param.load(Ordering::Relaxed));
            (param_info.to_normal)(value) as f32
        } else {
            0.0
        }
    }
}

extern "C" fn process_replacing<P: Plugin>(
    effect: *mut AEffect,
    inputs: *const *const f32,
    outputs: *mut *mut f32,
    sample_frames: i32,
) {
    unsafe {
        let wrapper = &*(effect as *const Wrapper<P>);
        let plugin_state = &mut *wrapper.plugin_state.get();

        for (i, param) in wrapper.params.iter().enumerate() {
            plugin_state.params[i] = f64::from_bits(param.load(Ordering::Relaxed));
        }

        let input_ptrs = slice::from_raw_parts(inputs, 2);
        let input_slices = &[
            slice::from_raw_parts(input_ptrs[0], sample_frames as usize),
            slice::from_raw_parts(input_ptrs[1], sample_frames as usize),
        ];

        let output_ptrs = slice::from_raw_parts(outputs, 2);
        let output_slices = &mut [
            slice::from_raw_parts_mut(output_ptrs[0], sample_frames as usize),
            slice::from_raw_parts_mut(output_ptrs[1], sample_frames as usize),
        ];

        plugin_state.plugin.process(
            &ParamValues { values: &plugin_state.params },
            input_slices,
            output_slices,
        );
    }
}

extern "C" fn process_replacing_f64(
    _effect: *mut AEffect,
    _inputs: *const *const f64,
    _outputs: *mut *mut f64,
    _sample_frames: i32,
) {
}

pub fn plugin_main<P: Plugin>(host_callback: HostCallbackProc) -> *mut AEffect {
    let mut params = Vec::with_capacity(P::PARAMS.len());
    for param_info in P::PARAMS {
        params.push(AtomicU64::new(param_info.default.to_bits()));
    }
    let params = Arc::new(params);

    let editor_context = Rc::new(Vst2EditorContext {
        alive: Cell::new(true),
        host_callback,
        effect: Cell::new(ptr::null_mut()),
        params: params.clone(),
    });

    let (plugin, editor) = P::create(editor_context.clone());

    let mut flags = effect_flags::CAN_REPLACING;
    if P::INFO.has_editor {
        flags |= effect_flags::HAS_EDITOR;
    }

    let effect = Box::new(Wrapper {
        effect: AEffect {
            magic: AEffect::MAGIC,
            dispatcher: dispatcher::<P>,
            process,
            set_parameter: set_parameter::<P>,
            get_parameter: get_parameter::<P>,
            num_programs: 1,
            num_params: P::PARAMS.len() as i32,
            num_inputs: 2,
            num_outputs: 2,
            flags,
            _reserved_1: 0,
            _reserved_2: 0,
            initial_delay: 0,
            real_qualities: 0,
            off_qualities: 0,
            io_ratio: 0.0,
            object: std::ptr::null_mut(),
            user: std::ptr::null_mut(),
            unique_id: cconst(
                P::INFO.unique_id[0],
                P::INFO.unique_id[1],
                P::INFO.unique_id[2],
                P::INFO.unique_id[3],
            ),
            version: 0,
            process_replacing: process_replacing::<P>,
            process_replacing_f64,
            _future: [0; 56],
        },
        params,
        plugin_state: UnsafeCell::new(PluginState { params: vec![0.0; P::PARAMS.len()], plugin }),
        editor_state: UnsafeCell::new(EditorState {
            rect: Rect { top: 0, left: 0, bottom: 0, right: 0 },
            context: editor_context,
            editor,
        }),
    });

    let editor_state = unsafe { &*effect.editor_state.get() };
    editor_state.context.effect.set(&*effect as *const Wrapper<P> as *mut AEffect);

    Box::into_raw(effect) as *mut AEffect
}

#[macro_export]
macro_rules! vst2 {
    ($plugin:ty) => {
        mod vst2_impl {
            use $crate::vst2::vst2_api::*;
            use $crate::vst2::*;

            #[cfg(not(test))]
            #[no_mangle]
            extern "C" fn main(host_callback: HostCallbackProc) -> *mut AEffect {
                plugin_main::<$plugin>(host_callback)
            }

            #[no_mangle]
            extern "C" fn VSTPluginMain(host_callback: HostCallbackProc) -> *mut AEffect {
                plugin_main::<$plugin>(host_callback)
            }
        }
    };
}

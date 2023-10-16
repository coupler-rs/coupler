use std::cell::UnsafeCell;
use std::ffi::{c_char, c_void};
use std::ptr;
use std::sync::Arc;

use clap_sys::{plugin::*, process::*};

use crate::{Host, Plugin, PluginInfo};

#[repr(C)]
pub struct Instance<P> {
    #[allow(unused)]
    clap_plugin: clap_plugin,
    info: Arc<PluginInfo>,
    plugin: UnsafeCell<P>,
}

unsafe impl<P> Sync for Instance<P> {}

impl<P: Plugin> Instance<P> {
    pub fn new(desc: *const clap_plugin_descriptor, info: &Arc<PluginInfo>) -> Self {
        Instance {
            clap_plugin: clap_plugin {
                desc,
                plugin_data: ptr::null_mut(),
                init: Some(Self::init),
                destroy: Some(Self::destroy),
                activate: Some(Self::activate),
                deactivate: Some(Self::deactivate),
                start_processing: Some(Self::start_processing),
                stop_processing: Some(Self::stop_processing),
                reset: Some(Self::reset),
                process: Some(Self::process),
                get_extension: Some(Self::get_extension),
                on_main_thread: Some(Self::on_main_thread),
            },
            info: info.clone(),
            plugin: UnsafeCell::new(P::new(Host {})),
        }
    }

    unsafe extern "C" fn init(_plugin: *const clap_plugin) -> bool {
        true
    }

    unsafe extern "C" fn destroy(plugin: *const clap_plugin) {
        drop(Box::from_raw(plugin as *mut Self));
    }

    unsafe extern "C" fn activate(
        _plugin: *const clap_plugin,
        _sample_rate: f64,
        _min_frames_count: u32,
        _max_frames_count: u32,
    ) -> bool {
        true
    }

    unsafe extern "C" fn deactivate(_plugin: *const clap_plugin) {}

    unsafe extern "C" fn start_processing(_plugin: *const clap_plugin) -> bool {
        true
    }

    unsafe extern "C" fn stop_processing(_plugin: *const clap_plugin) {}

    unsafe extern "C" fn reset(_plugin: *const clap_plugin) {}

    unsafe extern "C" fn process(
        _plugin: *const clap_plugin,
        _process: *const clap_process,
    ) -> clap_process_status {
        CLAP_PROCESS_CONTINUE
    }

    unsafe extern "C" fn get_extension(
        _plugin: *const clap_plugin,
        _id: *const c_char,
    ) -> *const c_void {
        ptr::null()
    }

    unsafe extern "C" fn on_main_thread(_plugin: *const clap_plugin) {}
}

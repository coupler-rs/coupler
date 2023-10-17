use std::cell::UnsafeCell;
use std::ffi::{c_char, c_void, CStr};
use std::ptr;
use std::sync::Arc;

use clap_sys::ext::{audio_ports::*, audio_ports_config::*};
use clap_sys::{id::*, plugin::*, process::*};

use crate::bus::{BusDir, Format};
use crate::util::copy_cstring;
use crate::{Host, Plugin, PluginInfo};

fn port_type_from_format(format: &Format) -> &'static CStr {
    match format {
        Format::Mono => CLAP_PORT_MONO,
        Format::Stereo => CLAP_PORT_STEREO,
    }
}

struct MainThreadState<P> {
    layout_index: usize,
    plugin: P,
}

#[repr(C)]
pub struct Instance<P> {
    #[allow(unused)]
    clap_plugin: clap_plugin,
    info: Arc<PluginInfo>,
    input_bus_map: Vec<usize>,
    output_bus_map: Vec<usize>,
    main_thread_state: UnsafeCell<MainThreadState<P>>,
}

unsafe impl<P> Sync for Instance<P> {}

impl<P: Plugin> Instance<P> {
    pub fn new(desc: *const clap_plugin_descriptor, info: &Arc<PluginInfo>) -> Self {
        let mut input_bus_map = Vec::new();
        let mut output_bus_map = Vec::new();
        for (index, bus) in info.buses.iter().enumerate() {
            match bus.dir {
                BusDir::In => input_bus_map.push(index),
                BusDir::Out => output_bus_map.push(index),
                BusDir::InOut => {
                    input_bus_map.push(index);
                    output_bus_map.push(index);
                }
            }
        }

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
            input_bus_map,
            output_bus_map,
            main_thread_state: UnsafeCell::new(MainThreadState {
                layout_index: 0,
                plugin: P::new(Host {}),
            }),
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
        id: *const c_char,
    ) -> *const c_void {
        let id = CStr::from_ptr(id);

        if id == CLAP_EXT_AUDIO_PORTS {
            return &Self::AUDIO_PORTS as *const _ as *const c_void;
        }

        if id == CLAP_EXT_AUDIO_PORTS_CONFIG {
            return &Self::AUDIO_PORTS_CONFIG as *const _ as *const c_void;
        }

        ptr::null()
    }

    unsafe extern "C" fn on_main_thread(_plugin: *const clap_plugin) {}
}

impl<P: Plugin> Instance<P> {
    const AUDIO_PORTS: clap_plugin_audio_ports = clap_plugin_audio_ports {
        count: Some(Self::audio_ports_count),
        get: Some(Self::audio_ports_get),
    };

    unsafe extern "C" fn audio_ports_count(plugin: *const clap_plugin, is_input: bool) -> u32 {
        let instance = &*(plugin as *const Self);

        if is_input {
            instance.input_bus_map.len() as u32
        } else {
            instance.output_bus_map.len() as u32
        }
    }

    unsafe extern "C" fn audio_ports_get(
        plugin: *const clap_plugin,
        index: u32,
        is_input: bool,
        info: *mut clap_audio_port_info,
    ) -> bool {
        let instance = &*(plugin as *const Self);
        let main_thread_state = &mut *instance.main_thread_state.get();

        let bus_index = if is_input {
            instance.input_bus_map.get(index as usize)
        } else {
            instance.output_bus_map.get(index as usize)
        };

        if let Some(&bus_index) = bus_index {
            let bus_info = instance.info.buses.get(bus_index);

            let layout = &instance.info.layouts[main_thread_state.layout_index];
            let format = layout.formats.get(bus_index);

            if let (Some(bus_info), Some(format)) = (bus_info, format) {
                let port_info = &mut *info;

                port_info.id = index;
                copy_cstring(&bus_info.name, &mut port_info.name);
                port_info.flags = if index == 0 {
                    CLAP_AUDIO_PORT_IS_MAIN
                } else {
                    0
                };
                port_info.channel_count = format.channel_count() as u32;
                port_info.port_type = port_type_from_format(format).as_ptr();
                port_info.in_place_pair = if bus_info.dir == BusDir::InOut {
                    // Find the other half of this input-output pair
                    let bus_map = if is_input {
                        &instance.output_bus_map
                    } else {
                        &instance.input_bus_map
                    };

                    bus_map.iter().position(|&i| i == bus_index).unwrap() as clap_id
                } else {
                    CLAP_INVALID_ID
                };

                return true;
            }
        }

        false
    }
}

impl<P: Plugin> Instance<P> {
    const AUDIO_PORTS_CONFIG: clap_plugin_audio_ports_config = clap_plugin_audio_ports_config {
        count: Some(Self::audio_ports_config_count),
        get: Some(Self::audio_ports_config_get),
        select: Some(Self::audio_ports_config_select),
    };

    unsafe extern "C" fn audio_ports_config_count(plugin: *const clap_plugin) -> u32 {
        let instance = &*(plugin as *const Self);

        instance.info.layouts.len() as u32
    }

    unsafe extern "C" fn audio_ports_config_get(
        plugin: *const clap_plugin,
        index: u32,
        config: *mut clap_audio_ports_config,
    ) -> bool {
        let instance = &*(plugin as *const Self);

        if let Some(layout) = instance.info.layouts.get(index as usize) {
            let mut config = &mut *config;

            config.id = index;
            copy_cstring("", &mut config.name);
            config.input_port_count = instance.input_bus_map.len() as u32;
            config.output_port_count = instance.output_bus_map.len() as u32;

            if let Some(&bus_index) = instance.input_bus_map.first() {
                config.has_main_input = true;

                let format = &layout.formats[bus_index];
                config.main_input_channel_count = format.channel_count() as u32;
                config.main_input_port_type = port_type_from_format(format).as_ptr();
            } else {
                config.has_main_input = false;
                config.main_input_channel_count = 0;
                config.main_input_port_type = ptr::null();
            }

            if let Some(&bus_index) = instance.output_bus_map.first() {
                config.has_main_output = true;

                let format = &layout.formats[bus_index];
                config.main_output_channel_count = format.channel_count() as u32;
                config.main_output_port_type = port_type_from_format(format).as_ptr();
            } else {
                config.has_main_output = false;
                config.main_output_channel_count = 0;
                config.main_output_port_type = ptr::null();
            }

            return true;
        }

        false
    }

    unsafe extern "C" fn audio_ports_config_select(
        plugin: *const clap_plugin,
        config_id: clap_id,
    ) -> bool {
        let instance = &*(plugin as *const Self);
        let main_thread_state = &mut *instance.main_thread_state.get();

        if instance.info.layouts.get(config_id as usize).is_some() {
            main_thread_state.layout_index = config_id as usize;
            return true;
        }

        false
    }
}

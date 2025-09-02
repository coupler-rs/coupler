use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr};
use std::iter::zip;
use std::ptr::NonNull;
use std::sync::Arc;
use std::{io, mem, ptr, slice};

use clap_sys::ext::{audio_ports::*, audio_ports_config::*, gui::*, params::*, state::*};
use clap_sys::{events::*, host::*, id::*, plugin::*, process::*, stream::*};

use super::host::ClapHost;
use crate::buffers::{BufferData, BufferType, Buffers};
use crate::bus::{BusDir, Format};
use crate::engine::{Config, Engine};
use crate::events::{Data, Event, Events};
use crate::host::Host;
use crate::params::{ParamId, ParamInfo, ParamValue};
use crate::plugin::{Plugin, PluginInfo};
use crate::sync::param_gestures::{GestureStates, GestureUpdate, ParamGestures};
use crate::sync::params::ParamValues;
use crate::util::{copy_cstring, slice_from_raw_parts_checked, DisplayParam};
use crate::view::View;

fn port_type_from_format(format: &Format) -> &'static CStr {
    match format {
        Format::Mono => CLAP_PORT_MONO,
        Format::Stereo => CLAP_PORT_STEREO,
    }
}

fn map_param_in(param: &ParamInfo, value: f64) -> ParamValue {
    if let Some(steps) = param.steps {
        (value + 0.5) / steps as f64
    } else {
        value
    }
}

fn map_param_out(param: &ParamInfo, value: ParamValue) -> f64 {
    if let Some(steps) = param.steps {
        (value * steps as f64).floor()
    } else {
        value
    }
}

pub struct MainThreadState<P: Plugin> {
    pub host_params: Option<*const clap_host_params>,
    pub layout_index: usize,
    pub plugin: P,
    pub view: Option<P::View>,
}

pub struct ProcessState<P: Plugin> {
    gesture_states: GestureStates,
    buffer_data: Vec<BufferData>,
    buffer_ptrs: Vec<*mut f32>,
    events: Vec<Event>,
    engine: Option<P::Engine>,
}

#[repr(C)]
pub struct Instance<P: Plugin> {
    #[allow(unused)]
    pub clap_plugin: clap_plugin,
    pub host: *const clap_host,
    pub info: Arc<PluginInfo>,
    pub input_bus_map: Vec<usize>,
    pub output_bus_map: Vec<usize>,
    pub param_map: Arc<HashMap<ParamId, usize>>,
    // Engine -> plugin parameter changes
    pub plugin_params: ParamValues,
    // Plugin -> engine parameter changes
    pub engine_params: ParamValues,
    pub param_gestures: Arc<ParamGestures>,
    pub main_thread_state: UnsafeCell<MainThreadState<P>>,
    pub process_state: UnsafeCell<ProcessState<P>>,
}

unsafe impl<P: Plugin> Sync for Instance<P> {}

impl<P: Plugin> Instance<P> {
    pub fn new(
        desc: *const clap_plugin_descriptor,
        info: &Arc<PluginInfo>,
        host: *const clap_host,
    ) -> Self {
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

        let mut param_map = HashMap::new();
        for (index, param) in info.params.iter().enumerate() {
            param_map.insert(param.id, index);
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
            host,
            info: info.clone(),
            input_bus_map,
            output_bus_map,
            param_map: Arc::new(param_map),
            plugin_params: ParamValues::with_count(info.params.len()),
            engine_params: ParamValues::with_count(info.params.len()),
            param_gestures: Arc::new(ParamGestures::with_count(info.params.len())),
            main_thread_state: UnsafeCell::new(MainThreadState {
                host_params: None,
                layout_index: 0,
                plugin: P::new(Host::from_inner(Arc::new(ClapHost {}))),
                view: None,
            }),
            process_state: UnsafeCell::new(ProcessState {
                gesture_states: GestureStates::with_count(info.params.len()),
                buffer_data: Vec::new(),
                buffer_ptrs: Vec::new(),
                events: Vec::with_capacity(4096),
                engine: None,
            }),
        }
    }

    fn sync_plugin(&self, main_thread_state: &mut MainThreadState<P>) {
        for (index, value) in self.plugin_params.poll() {
            let id = self.info.params[index].id;
            main_thread_state.plugin.set_param(id, value);

            if let Some(view) = &mut main_thread_state.view {
                view.param_changed(id, value);
            }
        }
    }

    fn sync_engine(&self, events: &mut Vec<Event>) {
        for (index, value) in self.engine_params.poll() {
            events.push(Event {
                time: 0,
                data: Data::ParamChange {
                    id: self.info.params[index].id,
                    value,
                },
            });
        }
    }

    unsafe fn process_param_events(
        &self,
        in_events: *const clap_input_events,
        events: &mut Vec<Event>,
    ) {
        let mut params_changed = false;

        let size = (*in_events).size.unwrap()(in_events);
        for i in 0..size {
            let event = (*in_events).get.unwrap()(in_events, i);

            if (*event).space_id == CLAP_CORE_EVENT_SPACE_ID
                && (*event).type_ == CLAP_EVENT_PARAM_VALUE
            {
                let event = &*(event as *const clap_event_param_value);

                if let Some(&index) = self.param_map.get(&event.param_id) {
                    let value = map_param_in(&self.info.params[index], event.value);

                    events.push(Event {
                        time: event.header.time as i64,
                        data: Data::ParamChange {
                            id: event.param_id,
                            value,
                        },
                    });

                    self.plugin_params.set(index, value);

                    params_changed = true;
                }
            }
        }

        if params_changed {
            (*self.host).request_callback.unwrap()(self.host);
        }
    }

    unsafe fn process_gestures(
        &self,
        gesture_states: &mut GestureStates,
        events: &mut Vec<Event>,
        out_events: *const clap_output_events,
        time: u32,
    ) {
        for update in self.param_gestures.poll(gesture_states) {
            let param = &self.info.params[update.index];

            if let Some(value) = update.set_value {
                events.push(Event {
                    time: time as i64,
                    data: Data::ParamChange {
                        id: param.id,
                        value,
                    },
                });

                self.plugin_params.set(update.index, value);
            }

            self.send_gesture_events(&update, out_events, time);
        }
    }

    unsafe fn send_gesture_events(
        &self,
        update: &GestureUpdate,
        out_events: *const clap_output_events,
        time: u32,
    ) {
        let param = &self.info.params[update.index];

        if update.begin_gesture {
            let event = clap_event_param_gesture {
                header: clap_event_header {
                    size: mem::size_of::<clap_event_param_gesture>() as u32,
                    time,
                    space_id: CLAP_CORE_EVENT_SPACE_ID,
                    type_: CLAP_EVENT_PARAM_GESTURE_BEGIN,
                    flags: CLAP_EVENT_IS_LIVE,
                },
                param_id: param.id,
            };

            (*out_events).try_push.unwrap()(
                out_events,
                &event as *const clap_event_param_gesture as *const clap_event_header,
            );
        }

        if let Some(value) = update.set_value {
            let event = clap_event_param_value {
                header: clap_event_header {
                    size: mem::size_of::<clap_event_param_value>() as u32,
                    time,
                    space_id: CLAP_CORE_EVENT_SPACE_ID,
                    type_: CLAP_EVENT_PARAM_VALUE,
                    flags: CLAP_EVENT_IS_LIVE,
                },
                param_id: param.id,
                cookie: ptr::null_mut(),
                note_id: -1,
                port_index: -1,
                channel: -1,
                key: -1,
                value: map_param_out(param, value),
            };

            (*out_events).try_push.unwrap()(
                out_events,
                &event as *const clap_event_param_value as *const clap_event_header,
            );
        }

        if update.end_gesture {
            let event = clap_event_param_gesture {
                header: clap_event_header {
                    size: mem::size_of::<clap_event_param_gesture>() as u32,
                    time,
                    space_id: CLAP_CORE_EVENT_SPACE_ID,
                    type_: CLAP_EVENT_PARAM_GESTURE_END,
                    flags: CLAP_EVENT_IS_LIVE,
                },
                param_id: param.id,
            };

            (*out_events).try_push.unwrap()(
                out_events,
                &event as *const clap_event_param_gesture as *const clap_event_header,
            );
        }
    }
}

impl<P: Plugin> Instance<P> {
    unsafe extern "C" fn init(plugin: *const clap_plugin) -> bool {
        let instance = &*(plugin as *const Self);
        let main_thread_state = &mut *instance.main_thread_state.get();

        let host_params =
            (*instance.host).get_extension.unwrap()(instance.host, CLAP_EXT_PARAMS.as_ptr());
        if !host_params.is_null() {
            main_thread_state.host_params = Some(host_params as *const clap_host_params);
        }

        true
    }

    unsafe extern "C" fn destroy(plugin: *const clap_plugin) {
        drop(Box::from_raw(plugin as *mut Self));
    }

    unsafe extern "C" fn activate(
        plugin: *const clap_plugin,
        sample_rate: f64,
        _min_frames_count: u32,
        max_frames_count: u32,
    ) -> bool {
        let instance = &*(plugin as *const Self);
        let main_thread_state = &mut *instance.main_thread_state.get();
        let process_state = &mut *instance.process_state.get();

        let layout = &instance.info.layouts[main_thread_state.layout_index];

        process_state.buffer_data.clear();
        let mut total_channels = 0;
        for (info, format) in zip(&instance.info.buses, &layout.formats) {
            let buffer_type = match info.dir {
                BusDir::In => BufferType::Const,
                BusDir::Out | BusDir::InOut => BufferType::Mut,
            };
            let channel_count = format.channel_count();

            process_state.buffer_data.push(BufferData {
                buffer_type,
                start: total_channels,
                end: total_channels + channel_count,
            });

            total_channels += channel_count;
        }

        process_state.buffer_ptrs.resize(total_channels, NonNull::dangling().as_ptr());

        let config = Config {
            layout: layout.clone(),
            sample_rate,
            max_buffer_size: max_frames_count as usize,
        };

        // Discard any pending plugin -> engine parameter changes, since they will already be
        // reflected in the initial state of the engine.
        for _ in instance.engine_params.poll() {}

        process_state.engine = Some(main_thread_state.plugin.engine(config));

        true
    }

    unsafe extern "C" fn deactivate(plugin: *const clap_plugin) {
        let instance = &*(plugin as *const Self);
        let main_thread_state = &mut *instance.main_thread_state.get();
        let process_state = &mut *instance.process_state.get();

        // Apply any remaining engine -> plugin parameter changes. There won't be any more until
        // the next call to `activate`.
        instance.sync_plugin(main_thread_state);

        process_state.engine = None;
    }

    unsafe extern "C" fn start_processing(_plugin: *const clap_plugin) -> bool {
        true
    }

    unsafe extern "C" fn stop_processing(_plugin: *const clap_plugin) {}

    unsafe extern "C" fn reset(plugin: *const clap_plugin) {
        let instance = &*(plugin as *const Self);
        let process_state = &mut *instance.process_state.get();

        if let Some(engine) = &mut process_state.engine {
            // Flush plugin -> engine parameter changes
            process_state.events.clear();
            instance.sync_engine(&mut process_state.events);

            if !process_state.events.is_empty() {
                engine.flush(Events::new(&process_state.events));
            }

            engine.reset();
        }
    }

    unsafe extern "C" fn process(
        plugin: *const clap_plugin,
        process: *const clap_process,
    ) -> clap_process_status {
        let instance = &*(plugin as *const Self);
        let process_state = &mut *instance.process_state.get();

        let Some(engine) = &mut process_state.engine else {
            return CLAP_PROCESS_ERROR;
        };

        let process = &*process;

        let len = process.frames_count as usize;

        let input_count = process.audio_inputs_count as usize;
        let output_count = process.audio_outputs_count as usize;
        if input_count != instance.input_bus_map.len()
            || output_count != instance.output_bus_map.len()
        {
            return CLAP_PROCESS_ERROR;
        }

        let inputs = slice_from_raw_parts_checked(process.audio_inputs, input_count);
        let outputs = slice_from_raw_parts_checked(process.audio_outputs, output_count);

        for (&bus_index, output) in zip(&instance.output_bus_map, outputs) {
            let data = &process_state.buffer_data[bus_index];

            let channel_count = output.channel_count as usize;
            if channel_count != data.end - data.start {
                return CLAP_PROCESS_ERROR;
            }

            let channels =
                slice_from_raw_parts_checked(output.data32 as *const *mut f32, channel_count);
            process_state.buffer_ptrs[data.start..data.end].copy_from_slice(channels);
        }

        for (&bus_index, input) in zip(&instance.input_bus_map, inputs) {
            let data = &process_state.buffer_data[bus_index];
            let bus_info = &instance.info.buses[bus_index];

            let channel_count = input.channel_count as usize;
            if channel_count != data.end - data.start {
                return CLAP_PROCESS_ERROR;
            }

            let channels =
                slice_from_raw_parts_checked(input.data32 as *const *mut f32, channel_count);
            let ptrs = &mut process_state.buffer_ptrs[data.start..data.end];

            match bus_info.dir {
                BusDir::In => {
                    ptrs.copy_from_slice(channels);
                }
                BusDir::InOut => {
                    for (&src, &mut dst) in zip(channels, ptrs) {
                        if src != dst {
                            let src = slice::from_raw_parts(src, len);
                            let dst = slice::from_raw_parts_mut(dst, len);
                            dst.copy_from_slice(src);
                        }
                    }
                }
                BusDir::Out => unreachable!(),
            }
        }

        process_state.events.clear();
        instance.sync_engine(&mut process_state.events);
        instance.process_param_events(process.in_events, &mut process_state.events);

        let last_sample = process.frames_count.saturating_sub(1);
        instance.process_gestures(
            &mut process_state.gesture_states,
            &mut process_state.events,
            process.out_events,
            last_sample,
        );

        engine.process(
            Buffers::from_raw_parts(
                &process_state.buffer_data,
                &process_state.buffer_ptrs,
                0,
                len,
            ),
            Events::new(&process_state.events),
        );

        CLAP_PROCESS_CONTINUE
    }

    unsafe extern "C" fn get_extension(
        plugin: *const clap_plugin,
        id: *const c_char,
    ) -> *const c_void {
        let id = CStr::from_ptr(id);

        if id == CLAP_EXT_AUDIO_PORTS {
            return &Self::AUDIO_PORTS as *const _ as *const c_void;
        }

        if id == CLAP_EXT_AUDIO_PORTS_CONFIG {
            return &Self::AUDIO_PORTS_CONFIG as *const _ as *const c_void;
        }

        if id == CLAP_EXT_PARAMS {
            return &Self::PARAMS as *const _ as *const c_void;
        }

        if id == CLAP_EXT_STATE {
            return &Self::STATE as *const _ as *const c_void;
        }

        if id == CLAP_EXT_GUI {
            let instance = &*(plugin as *const Self);
            if instance.info.has_view {
                return &Self::GUI as *const _ as *const c_void;
            }
        }

        ptr::null()
    }

    unsafe extern "C" fn on_main_thread(plugin: *const clap_plugin) {
        let instance = &*(plugin as *const Self);
        let main_thread_state = &mut *instance.main_thread_state.get();

        instance.sync_plugin(main_thread_state);
    }
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
            let config = &mut *config;

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

impl<P: Plugin> Instance<P> {
    const PARAMS: clap_plugin_params = clap_plugin_params {
        count: Some(Self::params_count),
        get_info: Some(Self::params_get_info),
        get_value: Some(Self::params_get_value),
        value_to_text: Some(Self::params_value_to_text),
        text_to_value: Some(Self::params_text_to_value),
        flush: Some(Self::params_flush),
    };

    unsafe extern "C" fn params_count(plugin: *const clap_plugin) -> u32 {
        let instance = &*(plugin as *const Self);

        instance.info.params.len() as u32
    }

    unsafe extern "C" fn params_get_info(
        plugin: *const clap_plugin,
        param_index: u32,
        param_info: *mut clap_param_info,
    ) -> bool {
        let instance = &*(plugin as *const Self);

        if let Some(param) = instance.info.params.get(param_index as usize) {
            let param_info = &mut *param_info;

            param_info.id = param.id;
            param_info.flags = CLAP_PARAM_IS_AUTOMATABLE;
            param_info.cookie = ptr::null_mut();
            copy_cstring(&param.name, &mut param_info.name);
            copy_cstring("", &mut param_info.module);
            if let Some(steps) = param.steps {
                param_info.flags |= CLAP_PARAM_IS_STEPPED;
                param_info.min_value = 0.0;
                param_info.max_value = (steps.max(2) - 1) as f64;
            } else {
                param_info.min_value = 0.0;
                param_info.max_value = 1.0;
            }
            param_info.default_value = map_param_out(param, param.default);

            return true;
        }

        false
    }

    unsafe extern "C" fn params_get_value(
        plugin: *const clap_plugin,
        param_id: clap_id,
        value: *mut f64,
    ) -> bool {
        let instance = &*(plugin as *const Self);
        let main_thread_state = &mut *instance.main_thread_state.get();

        if let Some(&index) = instance.param_map.get(&param_id) {
            instance.sync_plugin(main_thread_state);

            let param = &instance.info.params[index];
            *value = map_param_out(param, main_thread_state.plugin.get_param(param_id));
            return true;
        }

        false
    }

    unsafe extern "C" fn params_value_to_text(
        plugin: *const clap_plugin,
        param_id: clap_id,
        value: f64,
        display: *mut c_char,
        size: u32,
    ) -> bool {
        let instance = &*(plugin as *const Self);
        let main_thread_state = &mut *instance.main_thread_state.get();

        if let Some(&index) = instance.param_map.get(&param_id) {
            let param = &instance.info.params[index];

            let text = format!(
                "{}",
                DisplayParam::new(
                    &main_thread_state.plugin,
                    param_id,
                    map_param_in(param, value)
                )
            );

            let dst = slice::from_raw_parts_mut(display, size as usize);
            copy_cstring(&text, dst);

            return true;
        }

        false
    }

    unsafe extern "C" fn params_text_to_value(
        plugin: *const clap_plugin,
        param_id: clap_id,
        display: *const c_char,
        value: *mut f64,
    ) -> bool {
        let instance = &*(plugin as *const Self);
        let main_thread_state = &mut *instance.main_thread_state.get();

        if let Some(&index) = instance.param_map.get(&param_id) {
            if let Ok(text) = CStr::from_ptr(display).to_str() {
                let param = &instance.info.params[index];
                if let Some(out) = main_thread_state.plugin.parse_param(param_id, text) {
                    *value = map_param_out(param, out);
                    return true;
                }
            }

            return true;
        }

        false
    }

    unsafe extern "C" fn params_flush(
        plugin: *const clap_plugin,
        in_: *const clap_input_events,
        out: *const clap_output_events,
    ) {
        let instance = &*(plugin as *const Self);
        let process_state = &mut *instance.process_state.get();

        // If we are in the active state, flush will be called on the audio thread.
        if let Some(engine) = &mut process_state.engine {
            process_state.events.clear();
            instance.sync_engine(&mut process_state.events);
            instance.process_param_events(in_, &mut process_state.events);
            instance.process_gestures(
                &mut process_state.gesture_states,
                &mut process_state.events,
                out,
                0,
            );

            engine.flush(Events::new(&process_state.events));
        }
        // Otherwise, flush will be called on the main thread.
        else {
            let main_thread_state = &mut *instance.main_thread_state.get();

            let size = (*in_).size.unwrap()(in_);
            for i in 0..size {
                let event = (*in_).get.unwrap()(in_, i);

                if (*event).space_id == CLAP_CORE_EVENT_SPACE_ID
                    && (*event).type_ == CLAP_EVENT_PARAM_VALUE
                {
                    let event = &*(event as *const clap_event_param_value);

                    if let Some(&index) = instance.param_map.get(&event.param_id) {
                        let value = map_param_in(&instance.info.params[index], event.value);
                        main_thread_state.plugin.set_param(event.param_id, value);

                        if let Some(view) = &mut main_thread_state.view {
                            view.param_changed(event.param_id, value);
                        }
                    }
                }
            }

            for update in instance.param_gestures.poll(&mut process_state.gesture_states) {
                let param = &instance.info.params[update.index];

                if let Some(value) = update.set_value {
                    main_thread_state.plugin.set_param(param.id, value);

                    if let Some(view) = &mut main_thread_state.view {
                        view.param_changed(param.id, value);
                    }
                }

                instance.send_gesture_events(&update, out, 0);
            }
        }
    }
}

impl<P: Plugin> Instance<P> {
    const STATE: clap_plugin_state = clap_plugin_state {
        save: Some(Self::state_save),
        load: Some(Self::state_load),
    };

    unsafe extern "C" fn state_save(
        plugin: *const clap_plugin,
        stream: *const clap_ostream,
    ) -> bool {
        struct StreamWriter(*const clap_ostream);

        impl io::Write for StreamWriter {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                let result = unsafe {
                    (*self.0).write.unwrap()(
                        self.0,
                        buf.as_ptr() as *const c_void,
                        buf.len() as u64,
                    )
                };

                if result == -1 {
                    Err(io::Error::new(
                        io::ErrorKind::Other,
                        "failed to write to stream",
                    ))
                } else {
                    io::Result::Ok(result as usize)
                }
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let instance = &*(plugin as *const Self);
        let main_thread_state = &mut *instance.main_thread_state.get();

        instance.sync_plugin(main_thread_state);
        let result = main_thread_state.plugin.save(&mut StreamWriter(stream));
        result.is_ok()
    }

    unsafe extern "C" fn state_load(
        plugin: *const clap_plugin,
        stream: *const clap_istream,
    ) -> bool {
        struct StreamReader(*const clap_istream);

        impl io::Read for StreamReader {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                let result = unsafe {
                    (*self.0).read.unwrap()(
                        self.0,
                        buf.as_mut_ptr() as *mut c_void,
                        buf.len() as u64,
                    )
                };

                if result == -1 {
                    Err(io::Error::new(
                        io::ErrorKind::Other,
                        "failed to read from stream",
                    ))
                } else {
                    io::Result::Ok(result as usize)
                }
            }
        }

        let instance = &*(plugin as *const Self);
        let main_thread_state = &mut *instance.main_thread_state.get();

        instance.sync_plugin(main_thread_state);
        if main_thread_state.plugin.load(&mut StreamReader(stream)).is_ok() {
            for (index, param) in instance.info.params.iter().enumerate() {
                let value = main_thread_state.plugin.get_param(param.id);
                instance.engine_params.set(index, value);

                if let Some(view) = &mut main_thread_state.view {
                    view.param_changed(param.id, value);
                }
            }

            return true;
        }

        false
    }
}

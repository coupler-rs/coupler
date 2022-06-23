use super::util::{self, copy_cstring};
use crate::{bus::*, editor::*, param::*, plugin::*, process::*};

use clap_sys::ext::{audio_ports::*, audio_ports_config::*, gui::*, params::*, timer_support::*};
use clap_sys::{
    entry::*, events::*, host::*, id::*, plugin::*, plugin_factory::*, process::*, version::*,
};

use raw_window_handle::RawWindowHandle;

use std::cell::UnsafeCell;
use std::ffi::{c_void, CStr, CString};
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::ptr;
use std::rc::Rc;
use std::slice;

fn bus_format_to_port_type(bus_format: &BusFormat) -> *const c_char {
    match bus_format {
        BusFormat::Stereo => CLAP_PORT_STEREO,
    }
}

struct ClapEditorContext {}

impl<P> EditorContextHandler<P> for ClapEditorContext {
    fn begin_edit(&self, _id: ParamId) {
        unimplemented!();
    }

    fn perform_edit(&self, _id: ParamId, _value: f64) {
        unimplemented!();
    }

    fn end_edit(&self, _id: ParamId) {
        unimplemented!();
    }

    fn poll_params(&self) -> PollParams<P> {
        unimplemented!();
    }
}

struct BusStates {
    inputs: Vec<BusState>,
    outputs: Vec<BusState>,
}

struct ProcessorState<P: Plugin> {
    sample_rate: f64,
    max_buffer_size: usize,
    // This is safe to live in ProcessorState since all audio-ports and audio-ports-config
    // methods can only be called from the main thread while the plugin is deactivated.
    bus_states: BusStates,
    processor: Option<P::Processor>,
}

struct EditorState<P: Plugin> {
    #[cfg_attr(not(target_os = "linux"), allow(unused))]
    timer_id: Option<clap_id>,
    editor: Option<P::Editor>,
}

struct HostExtensions {
    timer_support: Option<*const clap_host_timer_support>,
}

#[repr(C)]
struct Wrapper<P: Plugin> {
    #[allow(unused)]
    clap_plugin: clap_plugin,
    clap_host: *const clap_host,
    // Safety: We only form an &mut in init(), which must be called before any other methods
    host_extensions: UnsafeCell<HostExtensions>,
    has_editor: bool,
    bus_list: BusList,
    bus_config_list: BusConfigList,
    plugin: PluginHandle<P>,
    processor_state: UnsafeCell<ProcessorState<P>>,
    editor_state: UnsafeCell<EditorState<P>>,
}

unsafe impl<P: Plugin> Sync for Wrapper<P> {}

impl<P: Plugin> Wrapper<P> {
    const AUDIO_PORTS: clap_plugin_audio_ports = clap_plugin_audio_ports {
        count: Self::audio_ports_count,
        get: Self::audio_ports_get,
    };

    const AUDIO_PORTS_CONFIG: clap_plugin_audio_ports_config = clap_plugin_audio_ports_config {
        count: Self::audio_ports_config_count,
        get: Self::audio_ports_config_get,
        select: Self::audio_ports_config_select,
    };

    const PARAMS: clap_plugin_params = clap_plugin_params {
        count: Self::params_count,
        get_info: Self::params_get_info,
        get_value: Self::params_get_value,
        value_to_text: Self::params_value_to_text,
        text_to_value: Self::params_text_to_value,
        flush: Self::params_flush,
    };

    const GUI: clap_plugin_gui = clap_plugin_gui {
        is_api_supported: Self::gui_is_api_supported,
        get_preferred_api: Self::gui_get_preferred_api,
        create: Self::gui_create,
        destroy: Self::gui_destroy,
        set_scale: Self::gui_set_scale,
        get_size: Self::gui_get_size,
        can_resize: Self::gui_can_resize,
        get_resize_hints: Self::gui_get_resize_hints,
        adjust_size: Self::gui_adjust_size,
        set_size: Self::gui_set_size,
        set_parent: Self::gui_set_parent,
        set_transient: Self::gui_set_transient,
        suggest_title: Self::gui_suggest_title,
        show: Self::gui_show,
        hide: Self::gui_hide,
    };

    #[cfg(target_os = "linux")]
    const TIMER_SUPPORT: clap_plugin_timer_support = clap_plugin_timer_support {
        on_timer: Self::timer_support_on_timer,
    };

    pub fn create(
        info: &PluginInfo,
        desc: *const clap_plugin_descriptor,
        host: *const clap_host,
    ) -> *mut Wrapper<P> {
        let bus_list = P::buses();
        let bus_config_list = P::bus_configs();

        util::validate_bus_configs(&bus_list, &bus_config_list);

        let default_config = bus_config_list.get_default().unwrap();

        let mut inputs = Vec::with_capacity(bus_list.get_inputs().len());
        for format in default_config.get_inputs() {
            inputs.push(BusState::new(format.clone(), true));
        }

        let mut outputs = Vec::with_capacity(bus_list.get_outputs().len());
        for format in default_config.get_outputs() {
            outputs.push(BusState::new(format.clone(), true));
        }

        let bus_states = BusStates { inputs, outputs };

        Box::into_raw(Box::new(Wrapper {
            clap_plugin: clap_plugin {
                desc,
                plugin_data: ptr::null_mut(),
                init: Self::init,
                destroy: Self::destroy,
                activate: Self::activate,
                deactivate: Self::deactivate,
                start_processing: Self::start_processing,
                stop_processing: Self::stop_processing,
                reset: Self::reset,
                process: Self::process,
                get_extension: Self::get_extension,
                on_main_thread: Self::on_main_thread,
            },
            clap_host: host,
            host_extensions: UnsafeCell::new(HostExtensions {
                timer_support: None,
            }),
            has_editor: info.get_has_editor(),
            bus_list,
            bus_config_list,
            plugin: PluginHandle::new(),
            processor_state: UnsafeCell::new(ProcessorState {
                sample_rate: 0.0,
                max_buffer_size: 0,
                bus_states,
                processor: None,
            }),
            editor_state: UnsafeCell::new(EditorState {
                timer_id: None,
                editor: None,
            }),
        }))
    }

    unsafe extern "C" fn init(plugin: *const clap_plugin) -> bool {
        // Query for host extensions here since calls to clap_host methods are not allowed
        // in create_plugin().

        let wrapper = &*(plugin as *mut Wrapper<P>);
        let host_extensions = &mut *wrapper.host_extensions.get();

        let timer_support =
            ((*wrapper.clap_host).get_extension)(wrapper.clap_host, CLAP_EXT_TIMER_SUPPORT);
        if !timer_support.is_null() {
            host_extensions.timer_support = Some(timer_support as *const clap_host_timer_support);
        }

        true
    }

    unsafe extern "C" fn destroy(plugin: *const clap_plugin) {
        drop(Box::from_raw(plugin as *mut Wrapper<P>));
    }

    unsafe extern "C" fn activate(
        plugin: *const clap_plugin,
        sample_rate: f64,
        _min_frames_count: u32,
        max_frames_count: u32,
    ) -> bool {
        let wrapper = &*(plugin as *mut Wrapper<P>);
        let processor_state = &mut *wrapper.processor_state.get();

        processor_state.sample_rate = sample_rate;
        processor_state.max_buffer_size = max_frames_count as usize;

        let context = ProcessContext::new(
            processor_state.sample_rate,
            processor_state.max_buffer_size,
            &[],
            &[],
        );
        processor_state.processor = Some(P::Processor::create(wrapper.plugin.clone(), &context));

        true
    }

    unsafe extern "C" fn deactivate(plugin: *const clap_plugin) {
        let wrapper = &*(plugin as *mut Wrapper<P>);
        let processor_state = &mut *wrapper.processor_state.get();

        processor_state.processor = None;
    }

    unsafe extern "C" fn start_processing(_plugin: *const clap_plugin) -> bool {
        true
    }

    unsafe extern "C" fn stop_processing(_plugin: *const clap_plugin) {}

    unsafe extern "C" fn reset(plugin: *const clap_plugin) {
        let wrapper = &*(plugin as *mut Wrapper<P>);
        let processor_state = &mut *wrapper.processor_state.get();

        if let Some(processor) = &mut processor_state.processor {
            let context = ProcessContext::new(
                processor_state.sample_rate,
                processor_state.max_buffer_size,
                &[],
                &[],
            );
            processor.reset(&context);
        }
    }

    unsafe extern "C" fn process(
        _plugin: *const clap_plugin,
        _process: *const clap_process,
    ) -> clap_process_status {
        CLAP_PROCESS_CONTINUE
    }

    unsafe extern "C" fn get_extension(
        plugin: *const clap_plugin,
        id: *const c_char,
    ) -> *const c_void {
        let wrapper = &*(plugin as *mut Wrapper<P>);

        if CStr::from_ptr(id) == CStr::from_ptr(CLAP_EXT_AUDIO_PORTS) {
            return &Self::AUDIO_PORTS as *const clap_plugin_audio_ports as *const c_void;
        }

        if CStr::from_ptr(id) == CStr::from_ptr(CLAP_EXT_AUDIO_PORTS_CONFIG) {
            return &Self::AUDIO_PORTS_CONFIG as *const clap_plugin_audio_ports_config
                as *const c_void;
        }

        if CStr::from_ptr(id) == CStr::from_ptr(CLAP_EXT_PARAMS) {
            return &Self::PARAMS as *const clap_plugin_params as *const c_void;
        }

        if wrapper.has_editor {
            if CStr::from_ptr(id) == CStr::from_ptr(CLAP_EXT_GUI) {
                return &Self::GUI as *const clap_plugin_gui as *const c_void;
            }

            #[cfg(target_os = "linux")]
            if CStr::from_ptr(id) == CStr::from_ptr(CLAP_EXT_TIMER_SUPPORT) {
                return &Self::TIMER_SUPPORT as *const clap_plugin_timer_support as *const c_void;
            }
        }

        ptr::null()
    }

    unsafe extern "C" fn on_main_thread(_plugin: *const clap_plugin) {}

    unsafe extern "C" fn audio_ports_count(plugin: *const clap_plugin, is_input: bool) -> u32 {
        let wrapper = &*(plugin as *mut Wrapper<P>);

        if is_input {
            wrapper.bus_list.get_inputs().len() as u32
        } else {
            wrapper.bus_list.get_outputs().len() as u32
        }
    }

    unsafe extern "C" fn audio_ports_get(
        plugin: *const clap_plugin,
        index: u32,
        is_input: bool,
        info: *mut clap_audio_port_info,
    ) -> bool {
        let wrapper = &*(plugin as *mut Wrapper<P>);
        let processor_state = &mut *wrapper.processor_state.get();

        let bus_info = if is_input {
            wrapper.bus_list.get_inputs().get(index as usize)
        } else {
            wrapper.bus_list.get_outputs().get(index as usize)
        };

        let bus_state = if is_input {
            processor_state.bus_states.inputs.get(index as usize)
        } else {
            processor_state.bus_states.outputs.get(index as usize)
        };

        if let (Some(bus_info), Some(bus_state)) = (bus_info, bus_state) {
            let info = &mut *info;

            info.id = index;
            copy_cstring(bus_info.get_name(), &mut info.name);
            info.flags = if index == 0 {
                CLAP_AUDIO_PORT_IS_MAIN
            } else {
                0
            };
            info.channel_count = bus_state.format().channels() as u32;
            info.port_type = bus_format_to_port_type(bus_state.format());
            info.in_place_pair = CLAP_INVALID_ID;

            return true;
        }

        false
    }

    unsafe extern "C" fn audio_ports_config_count(plugin: *const clap_plugin) -> u32 {
        let wrapper = &*(plugin as *mut Wrapper<P>);

        wrapper.bus_config_list.get_configs().len() as u32
    }

    unsafe extern "C" fn audio_ports_config_get(
        plugin: *const clap_plugin,
        index: u32,
        config: *mut clap_audio_ports_config,
    ) -> bool {
        let wrapper = &*(plugin as *mut Wrapper<P>);

        if let Some(bus_config) = wrapper.bus_config_list.get_configs().get(index as usize) {
            let config = &mut *config;

            config.id = index;
            copy_cstring("", &mut config.name); // TODO: Generate a name for the bus config
            config.input_port_count = bus_config.get_inputs().len() as u32;
            config.output_port_count = bus_config.get_outputs().len() as u32;

            config.has_main_input = !bus_config.get_inputs().is_empty();
            if let Some(bus_format) = bus_config.get_inputs().first() {
                config.main_input_channel_count = bus_format.channels() as u32;
                config.main_input_port_type = bus_format_to_port_type(bus_format);
            } else {
                config.main_input_channel_count = 0;
                config.main_input_port_type = ptr::null();
            }

            config.has_main_output = !bus_config.get_outputs().is_empty();
            if let Some(bus_format) = bus_config.get_outputs().first() {
                config.main_output_channel_count = bus_format.channels() as u32;
                config.main_output_port_type = bus_format_to_port_type(bus_format);
            } else {
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
        let wrapper = &*(plugin as *mut Wrapper<P>);
        let processor_state = &mut *wrapper.processor_state.get();

        if let Some(bus_config) = wrapper
            .bus_config_list
            .get_configs()
            .get(config_id as usize)
        {
            for (input, bus_state) in bus_config
                .get_inputs()
                .iter()
                .zip(processor_state.bus_states.inputs.iter_mut())
            {
                bus_state.set_format(input.clone());
            }

            for (output, bus_state) in bus_config
                .get_outputs()
                .iter()
                .zip(processor_state.bus_states.outputs.iter_mut())
            {
                bus_state.set_format(output.clone());
            }

            return true;
        }

        false
    }

    unsafe extern "C" fn params_count(plugin: *const clap_plugin) -> u32 {
        let wrapper = &*(plugin as *mut Wrapper<P>);

        PluginHandle::params(&wrapper.plugin).params().len() as u32
    }

    unsafe extern "C" fn params_get_info(
        plugin: *const clap_plugin,
        param_index: u32,
        param_info: *mut clap_param_info,
    ) -> bool {
        let wrapper = &*(plugin as *mut Wrapper<P>);

        let info = &mut *param_info;

        if let Some(param_info) = PluginHandle::params(&wrapper.plugin)
            .params()
            .get(param_index as usize)
        {
            info.id = param_info.get_id();
            info.flags = CLAP_PARAM_IS_AUTOMATABLE;
            info.cookie = ptr::null_mut();
            copy_cstring(param_info.get_name(), &mut info.name);
            copy_cstring("", &mut info.module);
            info.default_value = param_info.get_default();

            if let Some(steps) = param_info.get_steps() {
                info.flags |= CLAP_PARAM_IS_STEPPED;
                info.min_value = 0.0;
                info.max_value = (steps.max(2) - 1) as f64;
            } else {
                info.min_value = 0.0;
                info.max_value = 1.0;
            }

            return true;
        }

        false
    }

    unsafe extern "C" fn params_get_value(
        plugin: *const clap_plugin,
        param_id: clap_id,
        value: *mut f64,
    ) -> bool {
        let wrapper = &*(plugin as *mut Wrapper<P>);

        if let Some(param_info) = PluginHandle::params(&wrapper.plugin).get(param_id) {
            let value_mapped = param_info.get_accessor().get(&wrapper.plugin);
            *value = param_info.get_mapping().unmap(value_mapped);

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
        let wrapper = &*(plugin as *mut Wrapper<P>);

        if let Some(param_info) = PluginHandle::params(&wrapper.plugin).get(param_id) {
            let mut string = String::new();
            let value_mapped = param_info.get_mapping().map(value);
            param_info.get_format().display(value_mapped, &mut string);

            if size == 0 {
                return false;
            }

            let display = slice::from_raw_parts_mut(display, size as usize);
            copy_cstring(&string, display);

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
        let wrapper = &*(plugin as *mut Wrapper<P>);

        if let Some(param_info) = PluginHandle::params(&wrapper.plugin).get(param_id) {
            if let Ok(display) = CStr::from_ptr(display).to_str() {
                if let Ok(value_mapped) = param_info.get_format().parse(display) {
                    *value = param_info.get_mapping().unmap(value_mapped);
                    return true;
                }
            }
        }

        false
    }

    unsafe extern "C" fn params_flush(
        plugin: *const clap_plugin,
        in_: *const clap_input_events,
        _out: *const clap_output_events,
    ) {
        let wrapper = &*(plugin as *mut Wrapper<P>);

        let size = ((*in_).size)(in_);
        for i in 0..size {
            let event = ((*in_).get)(in_, i);

            if (*event).type_ == CLAP_EVENT_PARAM_VALUE {
                let event = &*(event as *const clap_event_param_value);

                if let Some(param_info) = PluginHandle::params(&wrapper.plugin).get(event.param_id)
                {
                    let value = param_info.get_mapping().map(event.value);
                    param_info.get_accessor().set(&wrapper.plugin, value);
                }
            }
        }
    }

    unsafe extern "C" fn gui_is_api_supported(
        _plugin: *const clap_plugin,
        api: *const c_char,
        is_floating: bool,
    ) -> bool {
        if is_floating {
            return false;
        }

        #[cfg(target_os = "windows")]
        if CStr::from_ptr(api) == CStr::from_ptr(CLAP_WINDOW_API_WIN32) {
            return true;
        }

        #[cfg(target_os = "macos")]
        if CStr::from_ptr(api) == CStr::from_ptr(CLAP_WINDOW_API_COCOA) {
            return true;
        }

        #[cfg(target_os = "linux")]
        if CStr::from_ptr(api) == CStr::from_ptr(CLAP_WINDOW_API_X11) {
            return true;
        }

        false
    }

    unsafe extern "C" fn gui_get_preferred_api(
        _plugin: *const clap_plugin,
        api: *mut *const c_char,
        is_floating: *mut bool,
    ) -> bool {
        *is_floating = false;

        #[cfg(target_os = "windows")]
        {
            *api = CLAP_WINDOW_API_WIN32;
            return true;
        }

        #[cfg(target_os = "macos")]
        {
            *api = CLAP_WINDOW_API_COCOA;
            return true;
        }

        #[cfg(target_os = "linux")]
        {
            *api = CLAP_WINDOW_API_X11;
            return true;
        }

        #[allow(unreachable_code)]
        false
    }

    unsafe extern "C" fn gui_create(
        plugin: *const clap_plugin,
        api: *const c_char,
        is_floating: bool,
    ) -> bool {
        if !Self::gui_is_api_supported(plugin, api, is_floating) {
            return false;
        }

        true
    }

    unsafe extern "C" fn gui_destroy(plugin: *const clap_plugin) {
        let wrapper = &*(plugin as *mut Wrapper<P>);
        let editor_state = &mut *wrapper.editor_state.get();

        #[cfg(target_os = "linux")]
        {
            if let Some(timer_support) = (*wrapper.host_extensions.get()).timer_support {
                if let Some(timer_id) = editor_state.timer_id.take() {
                    ((*timer_support).unregister_timer)(wrapper.clap_host, timer_id);
                }
            }
        }

        if let Some(editor) = &mut editor_state.editor {
            editor.close();
        }

        editor_state.editor = None;
    }

    unsafe extern "C" fn gui_set_scale(_plugin: *const clap_plugin, _scale: f64) -> bool {
        false
    }

    unsafe extern "C" fn gui_get_size(
        _plugin: *const clap_plugin,
        width: *mut u32,
        height: *mut u32,
    ) -> bool {
        let (editor_width, editor_height) = P::Editor::size();

        *width = editor_width.round() as u32;
        *height = editor_height.round() as u32;

        true
    }

    unsafe extern "C" fn gui_can_resize(_plugin: *const clap_plugin) -> bool {
        false
    }

    unsafe extern "C" fn gui_get_resize_hints(
        _plugin: *const clap_plugin,
        _hints: *mut clap_gui_resize_hints,
    ) -> bool {
        false
    }

    unsafe extern "C" fn gui_adjust_size(
        _plugin: *const clap_plugin,
        _width: *mut u32,
        _height: *mut u32,
    ) -> bool {
        false
    }

    unsafe extern "C" fn gui_set_size(
        _plugin: *const clap_plugin,
        _width: u32,
        _height: u32,
    ) -> bool {
        false
    }

    unsafe extern "C" fn gui_set_parent(
        plugin: *const clap_plugin,
        window: *const clap_window,
    ) -> bool {
        let wrapper = &*(plugin as *mut Wrapper<P>);
        let editor_state = &mut *wrapper.editor_state.get();

        let window = &*window;

        #[cfg(target_os = "macos")]
        let parent = {
            if CStr::from_ptr(window.api) != CStr::from_ptr(CLAP_WINDOW_API_COCOA) {
                return false;
            }

            use raw_window_handle::macos::MacOSHandle;
            RawWindowHandle::MacOS(MacOSHandle {
                ns_view: window.specific.cocoa,
                ..MacOSHandle::empty()
            })
        };

        #[cfg(target_os = "windows")]
        let parent = {
            if CStr::from_ptr(window.api) != CStr::from_ptr(CLAP_WINDOW_API_WIN32) {
                return false;
            }

            use raw_window_handle::windows::WindowsHandle;
            RawWindowHandle::Windows(WindowsHandle {
                hwnd: window.specific.win32,
                ..WindowsHandle::empty()
            })
        };

        #[cfg(target_os = "linux")]
        let parent = {
            if CStr::from_ptr(window.api) != CStr::from_ptr(CLAP_WINDOW_API_X11) {
                return false;
            }

            use raw_window_handle::unix::XcbHandle;
            RawWindowHandle::Xcb(XcbHandle {
                window: window.specific.x11 as u32,
                ..XcbHandle::empty()
            })
        };

        #[cfg(target_os = "linux")]
        {
            const TIMER_PERIOD_MS: u32 = 16;

            let timer_support =
                if let Some(timer_support) = (*wrapper.host_extensions.get()).timer_support {
                    timer_support
                } else {
                    return false;
                };

            let mut timer_id = CLAP_INVALID_ID;
            if !((*timer_support).register_timer)(
                wrapper.clap_host,
                TIMER_PERIOD_MS,
                &mut timer_id,
            ) {
                return false;
            }

            editor_state.timer_id = Some(timer_id);
        }

        let context = EditorContext::<P>::new(Rc::new(ClapEditorContext {}));
        let editor = P::Editor::open(wrapper.plugin.clone(), context, Some(&ParentWindow(parent)));
        editor_state.editor = Some(editor);

        true
    }

    unsafe extern "C" fn gui_set_transient(
        _plugin: *const clap_plugin,
        _window: *const clap_window,
    ) -> bool {
        false
    }

    unsafe extern "C" fn gui_suggest_title(_plugin: *const clap_plugin, _title: *const c_char) {}

    unsafe extern "C" fn gui_show(_plugin: *const clap_plugin) -> bool {
        false
    }

    unsafe extern "C" fn gui_hide(_plugin: *const clap_plugin) -> bool {
        false
    }

    #[cfg(target_os = "linux")]
    unsafe extern "C" fn timer_support_on_timer(plugin: *const clap_plugin, timer_id: clap_id) {
        let wrapper = &*(plugin as *mut Wrapper<P>);
        let editor_state = &mut *wrapper.editor_state.get();

        if let Some(id) = editor_state.timer_id {
            if let Some(editor) = &mut editor_state.editor {
                if timer_id == id {
                    editor.poll();
                }
            }
        }
    }
}

struct DescriptorBufs {
    id: CString,
    name: CString,
    vendor: CString,
    url: CString,
    manual_url: CString,
    support_url: CString,
    version: CString,
    description: CString,
    #[allow(unused)]
    features: Vec<CString>,
    feature_ptrs: Vec<*const c_char>,
}

#[doc(hidden)]
#[repr(C)]
pub struct Factory<P> {
    #[allow(unused)]
    factory: clap_plugin_factory,
    info: PluginInfo,
    descriptor_bufs: DescriptorBufs,
    descriptor: clap_plugin_descriptor,
    phantom: PhantomData<P>,
}

impl<P: Plugin + ClapPlugin> Factory<P> {
    fn new() -> Factory<P> {
        let info = P::info();
        let clap_info = P::clap_info();

        let features: Vec<CString> = Vec::new();
        let mut feature_ptrs = Vec::with_capacity(features.len() + 1);
        for feature in features.iter() {
            feature_ptrs.push(feature.as_ptr());
        }
        feature_ptrs.push(ptr::null());

        let descriptor_bufs = DescriptorBufs {
            id: CString::new(clap_info.get_id()).unwrap(),
            name: CString::new(info.get_name()).unwrap(),
            vendor: CString::new(info.get_vendor()).unwrap(),
            url: CString::new(info.get_url()).unwrap(),
            manual_url: CString::new("").unwrap(),
            support_url: CString::new("").unwrap(),
            version: CString::new("").unwrap(),
            description: CString::new("").unwrap(),
            features,
            feature_ptrs,
        };

        let descriptor = clap_plugin_descriptor {
            clap_version: CLAP_VERSION,
            id: descriptor_bufs.id.as_ptr(),
            name: descriptor_bufs.name.as_ptr(),
            vendor: descriptor_bufs.vendor.as_ptr(),
            url: descriptor_bufs.url.as_ptr(),
            manual_url: descriptor_bufs.manual_url.as_ptr(),
            support_url: descriptor_bufs.support_url.as_ptr(),
            version: descriptor_bufs.version.as_ptr(),
            description: descriptor_bufs.description.as_ptr(),
            features: descriptor_bufs.feature_ptrs.as_ptr(),
        };

        Factory {
            factory: clap_plugin_factory {
                get_plugin_count: Self::get_plugin_count,
                get_plugin_descriptor: Self::get_plugin_descriptor,
                create_plugin: Self::create_plugin,
            },
            info,
            descriptor_bufs,
            descriptor,
            phantom: PhantomData,
        }
    }

    unsafe extern "C" fn get_plugin_count(_factory: *const clap_plugin_factory) -> u32 {
        1
    }

    unsafe extern "C" fn get_plugin_descriptor(
        factory: *const clap_plugin_factory,
        index: u32,
    ) -> *const clap_plugin_descriptor {
        let factory = &*(factory as *const Self);

        if index == 0 {
            &factory.descriptor
        } else {
            ptr::null()
        }
    }

    unsafe extern "C" fn create_plugin(
        factory: *const clap_plugin_factory,
        host: *const clap_host,
        plugin_id: *const c_char,
    ) -> *const clap_plugin {
        let factory = &*(factory as *const Self);

        if CStr::from_ptr(plugin_id) == factory.descriptor_bufs.id.as_c_str() {
            return Wrapper::<P>::create(&factory.info, &factory.descriptor, host)
                as *const clap_plugin;
        }

        ptr::null()
    }
}

#[doc(hidden)]
#[repr(transparent)]
pub struct EntryPoint<P> {
    #[allow(unused)]
    entry_point: clap_plugin_entry,
    phantom: std::marker::PhantomData<P>,
}

impl<P: Plugin + ClapPlugin> EntryPoint<P> {
    pub const fn new(
        init: unsafe extern "C" fn(plugin_path: *const c_char) -> bool,
        deinit: unsafe extern "C" fn(),
        get_factory: unsafe extern "C" fn(factory_id: *const c_char) -> *const c_void,
    ) -> EntryPoint<P> {
        EntryPoint {
            entry_point: clap_plugin_entry {
                clap_version: CLAP_VERSION,
                init,
                deinit,
                get_factory,
            },
            phantom: PhantomData,
        }
    }

    pub unsafe extern "C" fn init(
        _plugin_path: *const c_char,
        factory: &mut Option<Factory<P>>,
    ) -> bool {
        *factory = Some(Factory::new());

        true
    }

    pub unsafe extern "C" fn deinit(factory: &mut Option<Factory<P>>) {
        *factory = None;
    }

    pub unsafe extern "C" fn get_factory(
        factory_id: *const c_char,
        factory: &Option<Factory<P>>,
    ) -> *const c_void {
        if CStr::from_ptr(factory_id) == CStr::from_ptr(CLAP_PLUGIN_FACTORY_ID) {
            if let Some(factory) = factory {
                return factory as *const Factory<P> as *const c_void;
            }
        }

        ptr::null()
    }
}

pub struct ClapInfo {
    id: String,
}

impl ClapInfo {
    #[inline]
    pub fn with_id(id: &str) -> ClapInfo {
        ClapInfo { id: id.to_string() }
    }

    #[inline]
    pub fn id(mut self, id: &str) -> ClapInfo {
        self.id = id.to_string();
        self
    }

    #[inline]
    pub fn get_id(&self) -> &str {
        &self.id
    }
}

pub trait ClapPlugin {
    fn clap_info() -> ClapInfo;
}

#[macro_export]
macro_rules! clap {
    ($plugin:ty) => {
        #[allow(non_upper_case_globals)]
        #[no_mangle]
        static clap_entry: ::coupler::format::clap::EntryPoint<$plugin> = {
            // Safety: We only form an &mut in init() and deinit(), which can't be called
            // concurrently with anything else according to the CLAP headers.
            static mut FACTORY: Option<::coupler::format::clap::Factory<$plugin>> = None;

            unsafe extern "C" fn init(plugin_path: *const ::std::os::raw::c_char) -> bool {
                ::coupler::format::clap::EntryPoint::<$plugin>::init(plugin_path, &mut FACTORY)
            }

            unsafe extern "C" fn deinit() {
                ::coupler::format::clap::EntryPoint::<$plugin>::deinit(&mut FACTORY)
            }

            unsafe extern "C" fn get_factory(
                factory_id: *const ::std::os::raw::c_char,
            ) -> *const ::std::ffi::c_void {
                ::coupler::format::clap::EntryPoint::<$plugin>::get_factory(factory_id, &FACTORY)
            }

            ::coupler::format::clap::EntryPoint::new(init, deinit, get_factory)
        };
    };
}

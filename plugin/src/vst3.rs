use crate::{
    atomic::AtomicF64, buffer::*, bus::*, editor::*, params::*, plugin::*, process::ProcessContext,
    process::*,
};

use std::cell::{Cell, UnsafeCell};
use std::ffi::{c_void, CString};
use std::marker::PhantomData;
use std::os::raw::{c_char, c_int};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{ffi, io, mem, ptr, slice};

use raw_window_handle::RawWindowHandle;

use vst3_sys::{BusInfo, *};

fn copy_cstring(src: &str, dst: &mut [c_char]) {
    let c_string = CString::new(src).unwrap_or_else(|_| CString::default());
    let bytes = c_string.as_bytes_with_nul();

    for (src, dst) in bytes.iter().zip(dst.iter_mut()) {
        *dst = *src as c_char;
    }

    if bytes.len() > dst.len() {
        if let Some(last) = dst.last_mut() {
            *last = 0;
        }
    }
}

fn copy_wstring(src: &str, dst: &mut [i16]) {
    let mut len = 0;
    for (src, dst) in src.encode_utf16().zip(dst.iter_mut()) {
        *dst = src as i16;
        len += 1;
    }

    if len < dst.len() {
        dst[len] = 0;
    } else if let Some(last) = dst.last_mut() {
        *last = 0;
    }
}

unsafe fn len_wstring(string: *const i16) -> usize {
    let mut len = 0;

    while *string.offset(len) != 0 {
        len += 1;
    }

    len as usize
}

fn bus_layout_to_speaker_arrangement(bus_layout: &BusLayout) -> SpeakerArrangement {
    match bus_layout {
        BusLayout::Stereo => speaker_arrangements::STEREO,
    }
}

fn speaker_arrangement_to_bus_layout(speaker_arrangement: SpeakerArrangement) -> Option<BusLayout> {
    match speaker_arrangement {
        speaker_arrangements::STEREO => Some(BusLayout::Stereo),
        _ => None,
    }
}

#[repr(C)]
pub struct Factory<P> {
    plugin_factory_3: *const IPluginFactory3,
    uid: TUID,
    plugin_info: PluginInfo,
    phantom: PhantomData<P>,
}

unsafe impl<P> Sync for Factory<P> {}

impl<P: Plugin> Factory<P> {
    const PLUGIN_FACTORY_3_VTABLE: IPluginFactory3 = IPluginFactory3 {
        plugin_factory_2: IPluginFactory2 {
            plugin_factory: IPluginFactory {
                unknown: FUnknown {
                    query_interface: Self::query_interface,
                    add_ref: Self::add_ref,
                    release: Self::release,
                },
                get_factory_info: Self::get_factory_info,
                count_classes: Self::count_classes,
                get_class_info: Self::get_class_info,
                create_instance: Self::create_instance,
            },
            get_class_info_2: Self::get_class_info_2,
        },
        get_class_info_unicode: Self::get_class_info_unicode,
        set_host_context: Self::set_host_context,
    };

    pub fn create(plugin_uid: [u32; 4]) -> *mut Factory<P> {
        Arc::into_raw(Arc::new(Factory::<P> {
            plugin_factory_3: &Self::PLUGIN_FACTORY_3_VTABLE as *const _,
            uid: uid(plugin_uid[0], plugin_uid[1], plugin_uid[2], plugin_uid[3]),
            plugin_info: P::info(),
            phantom: PhantomData,
        })) as *mut Factory<P>
    }

    unsafe extern "system" fn query_interface(
        this: *mut c_void,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> TResult {
        let iid = *iid;

        if iid == FUnknown::IID
            || iid == IPluginFactory::IID
            || iid == IPluginFactory2::IID
            || iid == IPluginFactory3::IID
        {
            Self::add_ref(this);
            *obj = this;
            return result::OK;
        }

        result::NO_INTERFACE
    }

    unsafe extern "system" fn add_ref(this: *mut c_void) -> u32 {
        let factory = Arc::from_raw(this as *const Factory<P>);
        let _ = Arc::into_raw(factory.clone());
        let count = Arc::strong_count(&factory);
        let _ = Arc::into_raw(factory);

        count as u32
    }

    unsafe extern "system" fn release(this: *mut c_void) -> u32 {
        let factory = Arc::from_raw(this as *const Factory<P>);
        let count = Arc::strong_count(&factory) - 1;
        drop(factory);

        count as u32
    }

    unsafe extern "system" fn get_factory_info(
        this: *mut c_void,
        info: *mut PFactoryInfo,
    ) -> TResult {
        let factory = &*(this as *const Factory<P>);

        let info = &mut *info;

        copy_cstring(&factory.plugin_info.vendor, &mut info.vendor);
        copy_cstring(&factory.plugin_info.url, &mut info.url);
        copy_cstring(&factory.plugin_info.email, &mut info.email);
        info.flags = PFactoryInfo::UNICODE;

        result::OK
    }

    unsafe extern "system" fn count_classes(_this: *mut c_void) -> i32 {
        1
    }

    unsafe extern "system" fn get_class_info(
        this: *mut c_void,
        index: i32,
        info: *mut PClassInfo,
    ) -> TResult {
        let factory = &*(this as *const Factory<P>);

        if index != 0 {
            return result::INVALID_ARGUMENT;
        }

        let info = &mut *info;

        info.cid = factory.uid;
        info.cardinality = PClassInfo::MANY_INSTANCES;
        copy_cstring("Audio Module Class", &mut info.category);
        copy_cstring(&factory.plugin_info.name, &mut info.name);

        result::OK
    }

    unsafe extern "system" fn create_instance(
        this: *mut c_void,
        cid: *const c_char,
        iid: *const c_char,
        obj: *mut *mut c_void,
    ) -> TResult {
        let factory = &*(this as *const Factory<P>);

        let cid = *(cid as *const TUID);
        let iid = *(iid as *const TUID);
        if cid != factory.uid || iid != IComponent::IID {
            return result::INVALID_ARGUMENT;
        }

        *obj = Wrapper::<P>::create(&factory.plugin_info) as *mut c_void;

        result::OK
    }

    unsafe extern "system" fn get_class_info_2(
        this: *mut c_void,
        index: i32,
        info: *mut PClassInfo2,
    ) -> TResult {
        let factory = &*(this as *const Factory<P>);

        if index != 0 {
            return result::INVALID_ARGUMENT;
        }

        let info = &mut *info;

        info.cid = factory.uid;
        info.cardinality = PClassInfo::MANY_INSTANCES;
        copy_cstring("Audio Module Class", &mut info.category);
        copy_cstring(&factory.plugin_info.name, &mut info.name);
        info.class_flags = 0;
        copy_cstring("Fx", &mut info.sub_categories);
        copy_cstring(&factory.plugin_info.vendor, &mut info.vendor);
        copy_cstring("", &mut info.version);
        copy_cstring("VST 3.7", &mut info.sdk_version);

        result::OK
    }

    unsafe extern "system" fn get_class_info_unicode(
        this: *mut c_void,
        index: i32,
        info: *mut PClassInfoW,
    ) -> TResult {
        let factory = &*(this as *const Factory<P>);

        if index != 0 {
            return result::INVALID_ARGUMENT;
        }

        let info = &mut *info;

        info.cid = factory.uid;
        info.cardinality = PClassInfo::MANY_INSTANCES;
        copy_cstring("Audio Module Class", &mut info.category);
        copy_wstring(&factory.plugin_info.name, &mut info.name);
        info.class_flags = 0;
        copy_cstring("Fx", &mut info.sub_categories);
        copy_wstring(&factory.plugin_info.vendor, &mut info.vendor);
        copy_wstring("", &mut info.version);
        copy_wstring("VST 3.7", &mut info.sdk_version);

        result::OK
    }

    unsafe extern "system" fn set_host_context(
        _this: *mut c_void,
        _context: *mut *const FUnknown,
    ) -> TResult {
        result::NOT_IMPLEMENTED
    }
}

struct Vst3EditorContext {
    alive: Cell<bool>,
    component_handler: Cell<*mut *const IComponentHandler>,
    param_list: Arc<ParamList>,
    param_values: Vec<Cell<f64>>,
}

impl EditorContextInner for Vst3EditorContext {
    fn get_param(&self, id: ParamId) -> f64 {
        let index = self.param_list.indices[&id];
        self.param_values[index].get()
    }

    fn begin_edit(&self, id: ParamId) {
        let component_handler = self.component_handler.get();
        if self.alive.get() && !component_handler.is_null() {
            unsafe {
                ((*(*component_handler)).begin_edit)(component_handler as *mut c_void, id);
            }
        }
    }

    fn perform_edit(&self, id: ParamId, value: f64) {
        let index = self.param_list.indices[&id];
        self.param_values[index].set(value);

        let unmapped = self.param_list.params[index].map.unmap(value);

        let component_handler = self.component_handler.get();
        if self.alive.get() && !component_handler.is_null() {
            unsafe {
                ((*(*component_handler)).perform_edit)(
                    component_handler as *mut c_void,
                    id,
                    unmapped,
                );
            }
        }
    }

    fn end_edit(&self, id: ParamId) {
        let component_handler = self.component_handler.get();
        if self.alive.get() && !component_handler.is_null() {
            unsafe {
                ((*(*component_handler)).end_edit)(component_handler as *mut c_void, id);
            }
        }
    }
}

#[repr(C)]
struct Wrapper<P: Plugin> {
    component: *const IComponent,
    audio_processor: *const IAudioProcessor,
    process_context_requirements: *const IProcessContextRequirements,
    edit_controller: *const IEditController,
    plug_view: *const IPlugView,
    event_handler: *const IEventHandler,
    timer_handler: *const ITimerHandler,
    has_editor: bool,
    bus_list: BusList,
    // We only form an &mut to bus_states in set_bus_arrangements and
    // activate_bus, which aren't called concurrently with any other methods on
    // IComponent or IAudioProcessor per the spec.
    bus_states: UnsafeCell<BusStates>,
    param_list: Arc<ParamList>,
    param_values: Vec<AtomicF64>,
    params_dirty: AtomicBool,
    plugin: P,
    processor_state: UnsafeCell<ProcessorState<P>>,
    editor_state: UnsafeCell<EditorState<P>>,
}

struct BusStates {
    input_layouts: Vec<BusLayout>,
    inputs_enabled: Vec<bool>,
    output_layouts: Vec<BusLayout>,
    outputs_enabled: Vec<bool>,
}

struct ProcessorState<P: Plugin> {
    input_buses: Vec<AudioBus<'static>>,
    output_buses: Vec<AudioBus<'static>>,
    sample_rate: f64,
    param_values: Vec<f64>,
    param_changes: Vec<ParamChange>,
    processor: Option<P::Processor>,
}

struct EditorState<P: Plugin> {
    plug_frame: *mut *const IPlugFrame,
    context: Rc<Vst3EditorContext>,
    editor: Option<P::Editor>,
}

unsafe impl<P: Plugin> Sync for Wrapper<P> {}

impl<P: Plugin> Wrapper<P> {
    const COMPONENT_VTABLE: IComponent = IComponent {
        plugin_base: IPluginBase {
            unknown: FUnknown {
                query_interface: Self::component_query_interface,
                add_ref: Self::component_add_ref,
                release: Self::component_release,
            },
            initialize: Self::component_initialize,
            terminate: Self::component_terminate,
        },
        get_controller_class_id: Self::get_controller_class_id,
        set_io_mode: Self::set_io_mode,
        get_bus_count: Self::get_bus_count,
        get_bus_info: Self::get_bus_info,
        get_routing_info: Self::get_routing_info,
        activate_bus: Self::activate_bus,
        set_active: Self::set_active,
        set_state: Self::component_set_state,
        get_state: Self::component_get_state,
    };

    const AUDIO_PROCESSOR_VTABLE: IAudioProcessor = IAudioProcessor {
        unknown: FUnknown {
            query_interface: Self::audio_processor_query_interface,
            add_ref: Self::audio_processor_add_ref,
            release: Self::audio_processor_release,
        },
        set_bus_arrangements: Self::set_bus_arrangements,
        get_bus_arrangement: Self::get_bus_arrangement,
        can_process_sample_size: Self::can_process_sample_size,
        get_latency_samples: Self::get_latency_samples,
        setup_processing: Self::setup_processing,
        set_processing: Self::set_processing,
        process: Self::process,
        get_tail_samples: Self::get_tail_samples,
    };

    const PROCESS_CONTEXT_REQUIREMENTS_VTABLE: IProcessContextRequirements =
        IProcessContextRequirements {
            unknown: FUnknown {
                query_interface: Self::process_context_requirements_query_interface,
                add_ref: Self::process_context_requirements_add_ref,
                release: Self::process_context_requirements_release,
            },
            get_process_context_requirements: Self::get_process_context_requirements,
        };

    const EDIT_CONTROLLER_VTABLE: IEditController = IEditController {
        plugin_base: IPluginBase {
            unknown: FUnknown {
                query_interface: Self::edit_controller_query_interface,
                add_ref: Self::edit_controller_add_ref,
                release: Self::edit_controller_release,
            },
            initialize: Self::edit_controller_initialize,
            terminate: Self::edit_controller_terminate,
        },
        set_component_state: Self::set_component_state,
        set_state: Self::edit_controller_set_state,
        get_state: Self::edit_controller_get_state,
        get_parameter_count: Self::get_parameter_count,
        get_parameter_info: Self::get_parameter_info,
        get_param_string_by_value: Self::get_param_string_by_value,
        get_param_value_by_string: Self::get_param_value_by_string,
        normalized_param_to_plain: Self::normalized_param_to_plain,
        plain_param_to_normalized: Self::plain_param_to_normalized,
        get_param_normalized: Self::get_param_normalized,
        set_param_normalized: Self::set_param_normalized,
        set_component_handler: Self::set_component_handler,
        create_view: Self::create_view,
    };

    const PLUG_VIEW_VTABLE: IPlugView = IPlugView {
        unknown: FUnknown {
            query_interface: Self::plug_view_query_interface,
            add_ref: Self::plug_view_add_ref,
            release: Self::plug_view_release,
        },
        is_platform_type_supported: Self::is_platform_type_supported,
        attached: Self::attached,
        removed: Self::removed,
        on_wheel: Self::on_wheel,
        on_key_down: Self::on_key_down,
        on_key_up: Self::on_key_up,
        get_size: Self::get_size,
        on_size: Self::on_size,
        on_focus: Self::on_focus,
        set_frame: Self::set_frame,
        can_resize: Self::can_resize,
        check_size_constraint: Self::check_size_constraint,
    };

    const EVENT_HANDLER_VTABLE: IEventHandler = IEventHandler {
        unknown: FUnknown {
            query_interface: Self::event_handler_query_interface,
            add_ref: Self::event_handler_add_ref,
            release: Self::event_handler_release,
        },
        on_fd_is_set: Self::on_fd_is_set,
    };

    const TIMER_HANDLER_VTABLE: ITimerHandler = ITimerHandler {
        unknown: FUnknown {
            query_interface: Self::timer_handler_query_interface,
            add_ref: Self::timer_handler_add_ref,
            release: Self::timer_handler_release,
        },
        on_timer: Self::on_timer,
    };

    const COMPONENT_OFFSET: isize = 0;
    const AUDIO_PROCESSOR_OFFSET: isize =
        Self::COMPONENT_OFFSET + mem::size_of::<*const IComponent>() as isize;
    const PROCESS_CONTEXT_REQUIREMENTS_OFFSET: isize =
        Self::AUDIO_PROCESSOR_OFFSET + mem::size_of::<*const IAudioProcessor>() as isize;
    const EDIT_CONTROLLER_OFFSET: isize = Self::PROCESS_CONTEXT_REQUIREMENTS_OFFSET
        + mem::size_of::<*const IProcessContextRequirements>() as isize;
    const PLUG_VIEW_OFFSET: isize =
        Self::EDIT_CONTROLLER_OFFSET + mem::size_of::<*const IEditController>() as isize;
    const EVENT_HANDLER_OFFSET: isize =
        Self::PLUG_VIEW_OFFSET + mem::size_of::<*const IPlugView>() as isize;
    const TIMER_HANDLER_OFFSET: isize =
        Self::EVENT_HANDLER_OFFSET + mem::size_of::<*const IEventHandler>() as isize;

    pub fn create(plugin_info: &PluginInfo) -> *mut Wrapper<P> {
        let bus_list = P::buses();

        let mut input_layouts = Vec::with_capacity(bus_list.inputs().len());
        let mut inputs_enabled = Vec::with_capacity(bus_list.inputs().len());
        for bus_info in bus_list.inputs() {
            input_layouts.push(bus_info.default_layout.clone());
            inputs_enabled.push(true);
        }

        let mut output_layouts = Vec::with_capacity(bus_list.outputs().len());
        let mut outputs_enabled = Vec::with_capacity(bus_list.outputs().len());
        for bus_info in bus_list.outputs() {
            output_layouts.push(bus_info.default_layout.clone());
            outputs_enabled.push(true);
        }

        let plugin = P::create();

        let param_list = Arc::new(plugin.params());

        let mut plugin_param_values = Vec::with_capacity(param_list.params.len());
        let mut processor_param_values = Vec::with_capacity(param_list.params.len());
        let mut editor_param_values = Vec::with_capacity(param_list.params.len());
        for param in param_list.params.iter() {
            plugin_param_values.push(AtomicF64::new(param.default));
            processor_param_values.push(param.default);
            editor_param_values.push(Cell::new(param.default));
        }

        let processor_state = UnsafeCell::new(ProcessorState {
            input_buses: Vec::with_capacity(bus_list.inputs().len()),
            output_buses: Vec::with_capacity(bus_list.outputs().len()),
            sample_rate: 44_100.0,
            // We can't know the maximum number of param changes in a
            // block, so make a reasonable guess and hope we don't have to
            // allocate more
            param_changes: Vec::with_capacity(4 * param_list.params.len()),
            param_values: processor_param_values,
            processor: None,
        });

        let editor_context = Rc::new(Vst3EditorContext {
            alive: Cell::new(true),
            component_handler: Cell::new(ptr::null_mut()),
            param_list: param_list.clone(),
            param_values: editor_param_values,
        });

        let editor_state = UnsafeCell::new(EditorState {
            plug_frame: ptr::null_mut(),
            context: editor_context,
            editor: None,
        });

        Arc::into_raw(Arc::new(Wrapper {
            component: &Wrapper::<P>::COMPONENT_VTABLE as *const _,
            audio_processor: &Wrapper::<P>::AUDIO_PROCESSOR_VTABLE as *const _,
            process_context_requirements: &Wrapper::<P>::PROCESS_CONTEXT_REQUIREMENTS_VTABLE
                as *const _,
            edit_controller: &Wrapper::<P>::EDIT_CONTROLLER_VTABLE as *const _,
            plug_view: &Wrapper::<P>::PLUG_VIEW_VTABLE as *const _,
            event_handler: &Wrapper::<P>::EVENT_HANDLER_VTABLE as *const _,
            timer_handler: &Wrapper::<P>::TIMER_HANDLER_VTABLE as *const _,
            bus_list,
            has_editor: plugin_info.has_editor,
            bus_states: UnsafeCell::new(BusStates {
                input_layouts,
                output_layouts,
                inputs_enabled,
                outputs_enabled,
            }),
            param_list,
            param_values: plugin_param_values,
            params_dirty: AtomicBool::new(false),
            plugin,
            processor_state,
            editor_state,
        })) as *mut Wrapper<P>
    }

    unsafe fn query_interface(
        this: *mut c_void,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> TResult {
        let wrapper = &*(this as *const Wrapper<P>);

        let iid = *iid;

        if iid == FUnknown::IID || iid == IComponent::IID {
            Self::add_ref(this);
            *obj = this.offset(Self::COMPONENT_OFFSET);
            return result::OK;
        }

        if iid == IAudioProcessor::IID {
            Self::add_ref(this);
            *obj = this.offset(Self::AUDIO_PROCESSOR_OFFSET);
            return result::OK;
        }

        if iid == IProcessContextRequirements::IID {
            Self::add_ref(this);
            *obj = this.offset(Self::PROCESS_CONTEXT_REQUIREMENTS_OFFSET);
            return result::OK;
        }

        if iid == IEditController::IID {
            Self::add_ref(this);
            *obj = this.offset(Self::EDIT_CONTROLLER_OFFSET);
            return result::OK;
        }

        if iid == IPlugView::IID && wrapper.has_editor {
            Self::add_ref(this);
            *obj = this.offset(Self::PLUG_VIEW_OFFSET);
            return result::OK;
        }

        if iid == IEventHandler::IID {
            Self::add_ref(this);
            *obj = this.offset(Self::EVENT_HANDLER_OFFSET);
            return result::OK;
        }

        if iid == ITimerHandler::IID {
            Self::add_ref(this);
            *obj = this.offset(Self::TIMER_HANDLER_OFFSET);
            return result::OK;
        }

        result::NO_INTERFACE
    }

    unsafe fn add_ref(this: *mut c_void) -> u32 {
        let wrapper = Arc::from_raw(this as *const Wrapper<P>);
        let _ = Arc::into_raw(wrapper.clone());
        let count = Arc::strong_count(&wrapper);
        let _ = Arc::into_raw(wrapper);

        count as u32
    }

    unsafe fn release(this: *mut c_void) -> u32 {
        let wrapper = Arc::from_raw(this as *const Wrapper<P>);
        let count = Arc::strong_count(&wrapper) - 1;
        drop(wrapper);

        count as u32
    }

    unsafe extern "system" fn component_query_interface(
        this: *mut c_void,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> TResult {
        Self::query_interface(this.offset(-Self::COMPONENT_OFFSET), iid, obj)
    }

    unsafe extern "system" fn component_add_ref(this: *mut c_void) -> u32 {
        Self::add_ref(this.offset(-Self::COMPONENT_OFFSET))
    }

    unsafe extern "system" fn component_release(this: *mut c_void) -> u32 {
        Self::release(this.offset(-Self::COMPONENT_OFFSET))
    }

    unsafe extern "system" fn component_initialize(
        _this: *mut c_void,
        _context: *mut FUnknown,
    ) -> TResult {
        result::OK
    }

    unsafe extern "system" fn component_terminate(_this: *mut c_void) -> TResult {
        result::OK
    }

    unsafe extern "system" fn get_controller_class_id(
        _this: *mut c_void,
        _class_id: *mut TUID,
    ) -> TResult {
        result::NOT_IMPLEMENTED
    }

    unsafe extern "system" fn set_io_mode(_this: *mut c_void, _mode: IoMode) -> TResult {
        result::OK
    }

    unsafe extern "system" fn get_bus_count(
        this: *mut c_void,
        media_type: MediaType,
        dir: BusDirection,
    ) -> i32 {
        let wrapper = &*(this.offset(-Self::COMPONENT_OFFSET) as *const Wrapper<P>);

        match media_type {
            media_types::AUDIO => match dir {
                bus_directions::INPUT => wrapper.bus_list.inputs().len() as i32,
                bus_directions::OUTPUT => wrapper.bus_list.outputs().len() as i32,
                _ => 0,
            },
            media_types::EVENT => 0,
            _ => 0,
        }
    }

    unsafe extern "system" fn get_bus_info(
        this: *mut c_void,
        media_type: MediaType,
        dir: BusDirection,
        index: i32,
        bus: *mut BusInfo,
    ) -> TResult {
        let wrapper = &*(this.offset(-Self::COMPONENT_OFFSET) as *const Wrapper<P>);
        let bus_states = &*wrapper.bus_states.get();

        match media_type {
            media_types::AUDIO => {
                let bus_info = match dir {
                    bus_directions::INPUT => wrapper.bus_list.inputs().get(index as usize),
                    bus_directions::OUTPUT => wrapper.bus_list.outputs().get(index as usize),
                    _ => None,
                };

                let bus_layout = match dir {
                    bus_directions::INPUT => bus_states.input_layouts.get(index as usize),
                    bus_directions::OUTPUT => bus_states.output_layouts.get(index as usize),
                    _ => None,
                };

                if let (Some(bus_info), Some(bus_layout)) = (bus_info, bus_layout) {
                    let bus = &mut *bus;

                    bus.media_type = media_types::AUDIO;
                    bus.direction = dir;
                    bus.channel_count = bus_layout.channels() as i32;
                    copy_wstring(&bus_info.name, &mut bus.name);
                    bus.bus_type = if index == 0 { bus_types::MAIN } else { bus_types::AUX };
                    bus.flags = BusInfo::DEFAULT_ACTIVE;

                    return result::OK;
                }
            }
            media_types::EVENT => {}
            _ => {}
        }

        result::INVALID_ARGUMENT
    }

    unsafe extern "system" fn get_routing_info(
        _this: *mut c_void,
        _in_info: *mut RoutingInfo,
        _out_info: *mut RoutingInfo,
    ) -> TResult {
        result::NOT_IMPLEMENTED
    }

    unsafe extern "system" fn activate_bus(
        this: *mut c_void,
        media_type: MediaType,
        dir: BusDirection,
        index: i32,
        state: TBool,
    ) -> TResult {
        let wrapper = &*(this.offset(-Self::COMPONENT_OFFSET) as *const Wrapper<P>);
        let bus_states = &mut *wrapper.bus_states.get();

        match media_type {
            media_types::AUDIO => {
                let bus_enabled = match dir {
                    bus_directions::INPUT => bus_states.inputs_enabled.get_mut(index as usize),
                    bus_directions::OUTPUT => bus_states.outputs_enabled.get_mut(index as usize),
                    _ => None,
                };

                if let Some(bus_enabled) = bus_enabled {
                    *bus_enabled = if state == 0 { false } else { true };
                    return result::OK;
                }
            }
            media_types::EVENT => {}
            _ => {}
        }

        result::INVALID_ARGUMENT
    }

    unsafe extern "system" fn set_active(this: *mut c_void, state: TBool) -> TResult {
        let wrapper = &*(this.offset(-Self::COMPONENT_OFFSET) as *const Wrapper<P>);
        let bus_states = &mut *wrapper.bus_states.get();
        let processor_state = &mut *wrapper.processor_state.get();

        match state {
            0 => {
                processor_state.processor = None;
            }
            _ => {
                let context = ProcessContext {
                    sample_rate: processor_state.sample_rate,
                    input_layouts: &bus_states.input_layouts[..],
                    output_layouts: &bus_states.output_layouts[..],
                    param_list: &wrapper.param_list,
                    param_values: &processor_state.param_values,
                };

                processor_state.processor = Some(wrapper.plugin.processor(&context));
            }
        }

        result::OK
    }

    unsafe extern "system" fn component_set_state(
        this: *mut c_void,
        state: *mut *const IBStream,
    ) -> TResult {
        struct StreamReader(*mut *const IBStream);

        impl io::Read for StreamReader {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                let mut bytes: i32 = 0;
                let result = unsafe {
                    ((*(*self.0)).read)(
                        self.0 as *mut c_void,
                        buf.as_mut_ptr() as *mut c_void,
                        buf.len() as i32,
                        &mut bytes,
                    )
                };

                if result == result::OK {
                    Ok(bytes as usize)
                } else {
                    Err(io::Error::new(io::ErrorKind::Other, "Failed to read from stream"))
                }
            }
        }

        let wrapper = &*(this.offset(-Self::COMPONENT_OFFSET) as *const Wrapper<P>);

        let mut param_values =
            ParamValues { param_list: &wrapper.param_list, values: &wrapper.param_values };
        match wrapper.plugin.deserialize(&mut param_values, &mut StreamReader(state)) {
            Ok(_) => {
                wrapper.params_dirty.store(true, Ordering::Release);
                result::OK
            }
            Err(_) => result::FALSE,
        }
    }

    unsafe extern "system" fn component_get_state(
        this: *mut c_void,
        state: *mut *const IBStream,
    ) -> TResult {
        struct StreamWriter(*mut *const IBStream);

        impl io::Write for StreamWriter {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                let mut bytes: i32 = 0;
                let result = unsafe {
                    ((*(*self.0)).write)(
                        self.0 as *mut c_void,
                        buf.as_ptr() as *mut c_void,
                        buf.len() as i32,
                        &mut bytes,
                    )
                };

                if result == result::OK {
                    Ok(bytes as usize)
                } else {
                    Err(io::Error::new(io::ErrorKind::Other, "Failed to write to stream"))
                }
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let wrapper = &*(this.offset(-Self::COMPONENT_OFFSET) as *const Wrapper<P>);

        let param_values =
            ParamValues { param_list: &wrapper.param_list, values: &wrapper.param_values };
        match wrapper.plugin.serialize(&param_values, &mut StreamWriter(state)) {
            Ok(_) => result::OK,
            Err(_) => result::FALSE,
        }
    }

    unsafe extern "system" fn audio_processor_query_interface(
        this: *mut c_void,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> TResult {
        Self::query_interface(this.offset(-Self::AUDIO_PROCESSOR_OFFSET), iid, obj)
    }

    unsafe extern "system" fn audio_processor_add_ref(this: *mut c_void) -> u32 {
        Self::add_ref(this.offset(-Self::AUDIO_PROCESSOR_OFFSET))
    }

    unsafe extern "system" fn audio_processor_release(this: *mut c_void) -> u32 {
        Self::release(this.offset(-Self::AUDIO_PROCESSOR_OFFSET))
    }

    unsafe extern "system" fn set_bus_arrangements(
        this: *mut c_void,
        inputs: *const SpeakerArrangement,
        num_ins: i32,
        outputs: *const SpeakerArrangement,
        num_outs: i32,
    ) -> TResult {
        let wrapper = &*(this.offset(-Self::AUDIO_PROCESSOR_OFFSET) as *const Wrapper<P>);
        let bus_states = &mut *wrapper.bus_states.get();

        if num_ins as usize != wrapper.bus_list.inputs().len()
            || num_outs as usize != wrapper.bus_list.outputs().len()
        {
            return result::FALSE;
        }

        // Don't use from_raw_parts for zero-length inputs, since the pointer
        // may be null or unaligned
        let inputs =
            if num_ins > 0 { slice::from_raw_parts(inputs, num_ins as usize) } else { &[] };
        let mut candidate_inputs = Vec::with_capacity(num_ins as usize);
        for input in inputs {
            if let Some(bus_layout) = speaker_arrangement_to_bus_layout(*input) {
                candidate_inputs.push(bus_layout);
            } else {
                return result::FALSE;
            }
        }

        // Don't use from_raw_parts for zero-length inputs, since the pointer
        // may be null or unaligned
        let outputs =
            if num_outs > 0 { slice::from_raw_parts(outputs, num_outs as usize) } else { &[] };
        let mut candidate_outputs = Vec::with_capacity(num_outs as usize);
        for output in outputs {
            if let Some(bus_layout) = speaker_arrangement_to_bus_layout(*output) {
                candidate_outputs.push(bus_layout);
            } else {
                return result::FALSE;
            }
        }

        if P::supports_layout(&candidate_inputs[..], &candidate_outputs[..]) {
            for (i, input) in candidate_inputs.into_iter().enumerate() {
                bus_states.input_layouts[i] = input;
            }
            for (i, output) in candidate_outputs.into_iter().enumerate() {
                bus_states.output_layouts[i] = output;
            }
        }

        result::TRUE
    }

    unsafe extern "system" fn get_bus_arrangement(
        this: *mut c_void,
        dir: BusDirection,
        index: i32,
        arr: *mut SpeakerArrangement,
    ) -> TResult {
        let wrapper = &*(this.offset(-Self::AUDIO_PROCESSOR_OFFSET) as *const Wrapper<P>);
        let bus_states = &*wrapper.bus_states.get();

        let bus_layout = match dir {
            bus_directions::INPUT => bus_states.input_layouts.get(index as usize),
            bus_directions::OUTPUT => bus_states.output_layouts.get(index as usize),
            _ => None,
        };

        if let Some(bus_layout) = bus_layout {
            *arr = bus_layout_to_speaker_arrangement(&bus_layout);
            return result::OK;
        }

        result::INVALID_ARGUMENT
    }

    unsafe extern "system" fn can_process_sample_size(
        _this: *mut c_void,
        symbolic_sample_size: i32,
    ) -> TResult {
        match symbolic_sample_size {
            symbolic_sample_sizes::SAMPLE_32 => result::OK,
            symbolic_sample_sizes::SAMPLE_64 => result::NOT_IMPLEMENTED,
            _ => result::INVALID_ARGUMENT,
        }
    }

    unsafe extern "system" fn get_latency_samples(_this: *mut c_void) -> u32 {
        0
    }

    unsafe extern "system" fn setup_processing(
        this: *mut c_void,
        setup: *mut ProcessSetup,
    ) -> TResult {
        let wrapper = &*(this.offset(-Self::AUDIO_PROCESSOR_OFFSET) as *const Wrapper<P>);
        let processor_state = &mut *wrapper.processor_state.get();

        let setup = &*setup;

        processor_state.sample_rate = setup.sample_rate;

        result::OK
    }

    unsafe extern "system" fn set_processing(this: *mut c_void, state: TBool) -> TResult {
        let wrapper = &*(this.offset(-Self::AUDIO_PROCESSOR_OFFSET) as *const Wrapper<P>);
        let bus_states = &*wrapper.bus_states.get();
        let processor_state = &mut *wrapper.processor_state.get();

        if processor_state.processor.is_none() {
            return result::NOT_INITIALIZED;
        }

        if state == 0 {
            let context = ProcessContext {
                sample_rate: processor_state.sample_rate,
                input_layouts: &bus_states.input_layouts[..],
                output_layouts: &bus_states.output_layouts[..],
                param_list: &wrapper.param_list,
                param_values: &processor_state.param_values,
            };

            if let Some(processor) = &mut processor_state.processor {
                processor.reset(&context);
            }
        }

        result::OK
    }

    unsafe extern "system" fn process(this: *mut c_void, data: *mut ProcessData) -> TResult {
        let wrapper = &*(this.offset(-Self::AUDIO_PROCESSOR_OFFSET) as *const Wrapper<P>);
        let bus_states = &*wrapper.bus_states.get();
        let processor_state = &mut *wrapper.processor_state.get();

        if processor_state.processor.is_none() {
            return result::NOT_INITIALIZED;
        }

        processor_state.param_changes.clear();

        // If params have been deserialized, read them out from the shared copy
        if wrapper.params_dirty.swap(false, Ordering::Acquire) {
            for (index, param) in wrapper.param_list.params.iter().enumerate() {
                let value = wrapper.param_values[index].load();
                processor_state.param_changes.push(ParamChange { id: param.id, offset: 0, value });
                processor_state.param_values[index] = value;
            }
        }

        let process_data = &*data;

        let param_changes = process_data.input_parameter_changes;
        if !param_changes.is_null() {
            let param_count =
                ((*(*param_changes)).get_parameter_count)(param_changes as *mut c_void);
            for param_index in 0..param_count {
                let param_data = ((*(*param_changes)).get_parameter_data)(
                    param_changes as *mut c_void,
                    param_index,
                );

                if param_data.is_null() {
                    continue;
                }

                let param_id = ((*(*param_data)).get_parameter_id)(param_data as *mut c_void);
                let point_count = ((*(*param_data)).get_point_count)(param_data as *mut c_void);

                for index in 0..point_count {
                    let mut offset = 0;
                    let mut value = 0.0;
                    let result = ((*(*param_data)).get_point)(
                        param_data as *mut c_void,
                        index,
                        &mut offset,
                        &mut value,
                    );

                    if result != result::OK {
                        continue;
                    }

                    if let Some(&index) = wrapper.param_list.indices.get(&param_id) {
                        let mapped = wrapper.param_list.params[index].map.map(value);

                        processor_state.param_changes.push(ParamChange {
                            id: param_id,
                            offset: offset as usize,
                            value: mapped,
                        });

                        processor_state.param_values[index] = mapped;

                        wrapper.param_values[index].store(mapped);
                    }
                }
            }
        }

        processor_state.param_changes.sort_by_key(|param_change| param_change.offset);

        processor_state.input_buses.clear();

        if process_data.num_inputs > 0 {
            if process_data.num_inputs as usize != wrapper.bus_list.inputs().len() {
                return result::INVALID_ARGUMENT;
            }

            let inputs =
                slice::from_raw_parts(process_data.inputs, process_data.num_inputs as usize);

            for (index, input) in inputs.iter().enumerate() {
                let bus_layout = &bus_states.input_layouts[index];
                let bus_enabled = bus_states.inputs_enabled[index];

                let channels = if !bus_enabled {
                    None
                } else if input.num_channels as usize == bus_layout.channels() {
                    Some(if input.num_channels > 0 {
                        slice::from_raw_parts(
                            input.channel_buffers as *const *mut f32,
                            input.num_channels as usize,
                        )
                    } else {
                        &[]
                    })
                } else if input.num_channels == 0 {
                    None
                } else {
                    return result::INVALID_ARGUMENT;
                };

                processor_state.input_buses.push(AudioBus {
                    layout: bus_layout,
                    samples: process_data.num_samples as usize,
                    channels,
                });
            }
        }

        processor_state.output_buses.clear();

        if process_data.num_outputs > 0 {
            if process_data.num_outputs as usize != wrapper.bus_list.outputs().len() {
                return result::INVALID_ARGUMENT;
            }

            let outputs =
                slice::from_raw_parts(process_data.outputs, process_data.num_outputs as usize);

            for (index, output) in outputs.iter().enumerate() {
                let bus_layout = &bus_states.output_layouts[index];
                let bus_enabled = bus_states.outputs_enabled[index];

                let channels = if !bus_enabled {
                    None
                } else if output.num_channels as usize == bus_layout.channels() {
                    Some(if output.num_channels > 0 {
                        slice::from_raw_parts(
                            output.channel_buffers as *const *mut f32,
                            output.num_channels as usize,
                        )
                    } else {
                        &[]
                    })
                } else if output.num_channels == 0 {
                    None
                } else {
                    return result::INVALID_ARGUMENT;
                };

                processor_state.output_buses.push(AudioBus {
                    layout: bus_layout,
                    samples: process_data.num_samples as usize,
                    channels,
                });
            }
        }

        let samples = process_data.num_samples as usize;
        let mut audio_buses = AudioBuffers {
            samples,
            inputs: AudioBuses { samples, buses: &mut processor_state.input_buses },
            outputs: AudioBuses { samples, buses: &mut processor_state.output_buses },
        };

        if !process_data.process_context.is_null() {
            processor_state.sample_rate = (*process_data.process_context).sample_rate;
        }

        let context = ProcessContext {
            sample_rate: processor_state.sample_rate,
            input_layouts: &bus_states.input_layouts[..],
            output_layouts: &bus_states.output_layouts[..],
            param_list: &wrapper.param_list,
            param_values: &processor_state.param_values,
        };

        if let Some(processor) = &mut processor_state.processor {
            processor.process(&context, &mut audio_buses, &processor_state.param_changes[..]);
        }

        result::OK
    }

    unsafe extern "system" fn get_tail_samples(_this: *mut c_void) -> u32 {
        0
    }

    unsafe extern "system" fn process_context_requirements_query_interface(
        this: *mut c_void,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> TResult {
        Self::query_interface(this.offset(-Self::PROCESS_CONTEXT_REQUIREMENTS_OFFSET), iid, obj)
    }

    unsafe extern "system" fn process_context_requirements_add_ref(this: *mut c_void) -> u32 {
        Self::add_ref(this.offset(-Self::PROCESS_CONTEXT_REQUIREMENTS_OFFSET))
    }

    unsafe extern "system" fn process_context_requirements_release(this: *mut c_void) -> u32 {
        Self::release(this.offset(-Self::PROCESS_CONTEXT_REQUIREMENTS_OFFSET))
    }

    unsafe extern "system" fn get_process_context_requirements(_this: *mut c_void) -> u32 {
        0
    }

    unsafe extern "system" fn edit_controller_query_interface(
        this: *mut c_void,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> TResult {
        Self::query_interface(this.offset(-Self::EDIT_CONTROLLER_OFFSET), iid, obj)
    }

    unsafe extern "system" fn edit_controller_add_ref(this: *mut c_void) -> u32 {
        Self::add_ref(this.offset(-Self::EDIT_CONTROLLER_OFFSET))
    }

    unsafe extern "system" fn edit_controller_release(this: *mut c_void) -> u32 {
        Self::release(this.offset(-Self::EDIT_CONTROLLER_OFFSET))
    }

    unsafe extern "system" fn edit_controller_initialize(
        _this: *mut c_void,
        _context: *mut FUnknown,
    ) -> TResult {
        result::OK
    }

    unsafe extern "system" fn edit_controller_terminate(this: *mut c_void) -> TResult {
        let wrapper = &*(this.offset(-Self::EDIT_CONTROLLER_OFFSET) as *const Wrapper<P>);
        let editor_state = &mut *wrapper.editor_state.get();

        editor_state.context.alive.set(false);
        let component_handler = editor_state.context.component_handler.get();
        if !component_handler.is_null() {
            ((*(*component_handler)).unknown.release)(component_handler as *mut c_void);
            editor_state.context.component_handler.set(ptr::null_mut());
        }

        if !editor_state.plug_frame.is_null() {
            ((*(*editor_state.plug_frame)).unknown.release)(editor_state.plug_frame as *mut c_void);
            editor_state.plug_frame = ptr::null_mut();
        }

        result::OK
    }

    unsafe extern "system" fn set_component_state(
        this: *mut c_void,
        _state: *mut *const IBStream,
    ) -> TResult {
        // Since we implement a single-component effect, we don't need to call
        // Plugin::deserialize again; we just use this method as a notification
        // that deserialization has happened and we need to read back parameter
        // values from the shared copy.

        let wrapper = &*(this.offset(-Self::EDIT_CONTROLLER_OFFSET) as *const Wrapper<P>);
        let editor_state = &mut *wrapper.editor_state.get();

        for (index, value) in wrapper.param_values.iter().enumerate() {
            editor_state.context.param_values[index].set(value.load());
        }

        result::OK
    }

    unsafe extern "system" fn edit_controller_set_state(
        _this: *mut c_void,
        _state: *mut *const IBStream,
    ) -> TResult {
        result::OK
    }

    unsafe extern "system" fn edit_controller_get_state(
        _this: *mut c_void,
        _state: *mut *const IBStream,
    ) -> TResult {
        result::OK
    }

    unsafe extern "system" fn get_parameter_count(this: *mut c_void) -> i32 {
        let wrapper = &*(this.offset(-Self::EDIT_CONTROLLER_OFFSET) as *const Wrapper<P>);

        wrapper.param_list.params.len() as i32
    }

    unsafe extern "system" fn get_parameter_info(
        this: *mut c_void,
        param_index: i32,
        info: *mut ParameterInfo,
    ) -> TResult {
        let wrapper = &*(this.offset(-Self::EDIT_CONTROLLER_OFFSET) as *const Wrapper<P>);

        if let Some(param_info) = wrapper.param_list.params.get(param_index as usize) {
            let info = &mut *info;

            info.id = param_info.id;
            copy_wstring(&param_info.name, &mut info.title);
            copy_wstring(&param_info.name, &mut info.short_title);
            copy_wstring(&param_info.units, &mut info.units);
            info.step_count =
                if let Some(steps) = param_info.steps { steps.saturating_sub(1) as i32 } else { 0 };
            info.default_normalized_value = param_info.map.unmap(param_info.default);
            info.unit_id = 0;
            info.flags = ParameterInfo::CAN_AUTOMATE;

            result::OK
        } else {
            result::INVALID_ARGUMENT
        }
    }

    unsafe extern "system" fn get_param_string_by_value(
        this: *mut c_void,
        id: u32,
        value_normalized: f64,
        string: *mut String128,
    ) -> TResult {
        let wrapper = &*(this.offset(-Self::EDIT_CONTROLLER_OFFSET) as *const Wrapper<P>);

        if let Some(&index) = wrapper.param_list.indices.get(&id) {
            let param_info = &wrapper.param_list.params[index];

            let mut display = String::new();
            let mapped = param_info.map.map(value_normalized);
            param_info.format.display(mapped, &mut display);
            copy_wstring(&display, &mut *string);

            return result::OK;
        }

        result::INVALID_ARGUMENT
    }

    unsafe extern "system" fn get_param_value_by_string(
        this: *mut c_void,
        id: u32,
        string: *const TChar,
        value_normalized: *mut f64,
    ) -> TResult {
        let wrapper = &*(this.offset(-Self::EDIT_CONTROLLER_OFFSET) as *const Wrapper<P>);

        if let Some(&index) = wrapper.param_list.indices.get(&id) {
            let param_info = &wrapper.param_list.params[index];

            let len = len_wstring(string);
            if let Ok(string) = String::from_utf16(slice::from_raw_parts(string as *const u16, len))
            {
                if let Ok(value) = param_info.format.parse(&string) {
                    let unmapped = param_info.map.unmap(value);
                    *value_normalized = unmapped;
                    return result::OK;
                }
            }
        }

        result::INVALID_ARGUMENT
    }

    unsafe extern "system" fn normalized_param_to_plain(
        this: *mut c_void,
        id: u32,
        value_normalized: f64,
    ) -> f64 {
        let wrapper = &*(this.offset(-Self::EDIT_CONTROLLER_OFFSET) as *const Wrapper<P>);

        if let Some(&index) = wrapper.param_list.indices.get(&id) {
            return wrapper.param_list.params[index].map.map(value_normalized);
        }

        0.0
    }

    unsafe extern "system" fn plain_param_to_normalized(
        this: *mut c_void,
        id: u32,
        plain_value: f64,
    ) -> f64 {
        let wrapper = &*(this.offset(-Self::EDIT_CONTROLLER_OFFSET) as *const Wrapper<P>);

        if let Some(&index) = wrapper.param_list.indices.get(&id) {
            return wrapper.param_list.params[index].map.unmap(plain_value);
        }

        0.0
    }

    unsafe extern "system" fn get_param_normalized(this: *mut c_void, id: u32) -> f64 {
        let wrapper = &*(this.offset(-Self::EDIT_CONTROLLER_OFFSET) as *const Wrapper<P>);
        let editor_state = &*wrapper.editor_state.get();

        if let Some(&index) = wrapper.param_list.indices.get(&id) {
            let value = editor_state.context.param_values[index].get();
            return wrapper.param_list.params[index].map.unmap(value);
        }

        0.0
    }

    unsafe extern "system" fn set_param_normalized(
        this: *mut c_void,
        id: u32,
        value: f64,
    ) -> TResult {
        let wrapper = &*(this.offset(-Self::EDIT_CONTROLLER_OFFSET) as *const Wrapper<P>);
        let editor_state = &*wrapper.editor_state.get();

        if let Some(&index) = wrapper.param_list.indices.get(&id) {
            let mapped = wrapper.param_list.params[index].map.map(value);
            editor_state.context.param_values[index].set(mapped);

            return result::OK;
        }

        result::INVALID_ARGUMENT
    }

    unsafe extern "system" fn set_component_handler(
        this: *mut c_void,
        handler: *mut *const IComponentHandler,
    ) -> TResult {
        let wrapper = &*(this.offset(-Self::EDIT_CONTROLLER_OFFSET) as *const Wrapper<P>);
        let editor_state = &*wrapper.editor_state.get();

        if !handler.is_null() {
            ((*(*handler)).unknown.add_ref)(handler as *mut c_void);
        }
        editor_state.context.component_handler.set(handler);

        result::OK
    }

    unsafe extern "system" fn create_view(
        this: *mut c_void,
        name: *const c_char,
    ) -> *mut *const IPlugView {
        let wrapper = &*(this.offset(-Self::EDIT_CONTROLLER_OFFSET) as *const Wrapper<P>);

        if !wrapper.has_editor {
            return ptr::null_mut();
        }

        if ffi::CStr::from_ptr(name) == ffi::CStr::from_ptr(view_types::EDITOR) {
            Self::add_ref(this.offset(-Self::EDIT_CONTROLLER_OFFSET));
            return this.offset(-Self::EDIT_CONTROLLER_OFFSET + Self::PLUG_VIEW_OFFSET)
                as *mut *const IPlugView;
        }

        ptr::null_mut()
    }

    unsafe extern "system" fn plug_view_query_interface(
        this: *mut c_void,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> TResult {
        Self::query_interface(this.offset(-Self::PLUG_VIEW_OFFSET), iid, obj)
    }

    unsafe extern "system" fn plug_view_add_ref(this: *mut c_void) -> u32 {
        Self::add_ref(this.offset(-Self::PLUG_VIEW_OFFSET))
    }

    unsafe extern "system" fn plug_view_release(this: *mut c_void) -> u32 {
        Self::release(this.offset(-Self::PLUG_VIEW_OFFSET))
    }

    unsafe extern "system" fn is_platform_type_supported(
        _this: *mut c_void,
        platform_type: *const c_char,
    ) -> TResult {
        #[cfg(target_os = "windows")]
        if ffi::CStr::from_ptr(platform_type) == ffi::CStr::from_ptr(platform_types::HWND) {
            return result::TRUE;
        }

        #[cfg(target_os = "macos")]
        if ffi::CStr::from_ptr(platform_type) == ffi::CStr::from_ptr(platform_types::NS_VIEW) {
            return result::TRUE;
        }

        #[cfg(target_os = "linux")]
        if ffi::CStr::from_ptr(platform_type)
            == ffi::CStr::from_ptr(platform_types::X11_EMBED_WINDOW_ID)
        {
            return result::TRUE;
        }

        result::FALSE
    }

    unsafe extern "system" fn attached(
        this: *mut c_void,
        parent: *mut c_void,
        platform_type: *const c_char,
    ) -> TResult {
        if Self::is_platform_type_supported(this, platform_type) != result::TRUE {
            return result::NOT_IMPLEMENTED;
        }

        let wrapper = &*(this.offset(-Self::PLUG_VIEW_OFFSET) as *const Wrapper<P>);
        let editor_state = &mut *wrapper.editor_state.get();

        #[cfg(target_os = "macos")]
        let parent = {
            use raw_window_handle::macos::MacOSHandle;
            RawWindowHandle::MacOS(MacOSHandle { ns_view: parent, ..MacOSHandle::empty() })
        };

        #[cfg(target_os = "windows")]
        let parent = {
            use raw_window_handle::windows::WindowsHandle;
            RawWindowHandle::Windows(WindowsHandle { hwnd: parent, ..WindowsHandle::empty() })
        };

        #[cfg(target_os = "linux")]
        let parent = {
            use raw_window_handle::unix::XcbHandle;
            RawWindowHandle::Xcb(XcbHandle { window: parent as u32, ..XcbHandle::empty() })
        };

        let editor_context = EditorContext { inner: editor_state.context.clone() };
        let editor = wrapper.plugin.editor(editor_context, Some(&ParentWindow(parent)));

        #[cfg(target_os = "linux")]
        if let Some(file_descriptor) = editor.file_descriptor() {
            if !editor_state.plug_frame.is_null() {
                let mut obj = ptr::null_mut();
                let result = ((*(*editor_state.plug_frame)).unknown.query_interface)(
                    editor_state.plug_frame as *mut c_void,
                    &IRunLoop::IID,
                    &mut obj,
                );
                if result == result::OK {
                    let run_loop = obj as *mut *const IRunLoop;

                    let event_handler = this
                        .offset(-Self::PLUG_VIEW_OFFSET + Self::EVENT_HANDLER_OFFSET)
                        as *mut *const IEventHandler;
                    ((*(*run_loop)).register_event_handler)(
                        run_loop as *mut c_void,
                        event_handler,
                        file_descriptor,
                    );

                    let timer_handler = this
                        .offset(-Self::PLUG_VIEW_OFFSET + Self::TIMER_HANDLER_OFFSET)
                        as *mut *const ITimerHandler;
                    ((*(*run_loop)).register_timer)(run_loop as *mut c_void, timer_handler, 16);

                    ((*(*run_loop)).unknown.release)(run_loop as *mut c_void);
                }
            }
        }

        editor_state.editor = Some(editor);

        result::OK
    }

    unsafe extern "system" fn removed(this: *mut c_void) -> TResult {
        let wrapper = &*(this.offset(-Self::PLUG_VIEW_OFFSET) as *const Wrapper<P>);
        let editor_state = &mut *wrapper.editor_state.get();

        if let Some(mut editor) = editor_state.editor.take() {
            editor.close();
        }

        #[cfg(target_os = "linux")]
        {
            if !editor_state.plug_frame.is_null() {
                let mut obj = ptr::null_mut();
                let result = ((*(*editor_state.plug_frame)).unknown.query_interface)(
                    editor_state.plug_frame as *mut c_void,
                    &IRunLoop::IID,
                    &mut obj,
                );
                if result == result::OK {
                    let run_loop = obj as *mut *const IRunLoop;

                    let event_handler = this
                        .offset(-Self::PLUG_VIEW_OFFSET + Self::EVENT_HANDLER_OFFSET)
                        as *mut *const IEventHandler;
                    ((*(*run_loop)).unregister_event_handler)(
                        run_loop as *mut c_void,
                        event_handler,
                    );

                    let timer_handler = this
                        .offset(-Self::PLUG_VIEW_OFFSET + Self::TIMER_HANDLER_OFFSET)
                        as *mut *const ITimerHandler;
                    ((*(*run_loop)).unregister_timer)(run_loop as *mut c_void, timer_handler);

                    ((*(*run_loop)).unknown.release)(run_loop as *mut c_void);
                }
            }
        }

        result::OK
    }

    unsafe extern "system" fn on_wheel(_this: *mut c_void, _distance: f32) -> TResult {
        result::NOT_IMPLEMENTED
    }

    unsafe extern "system" fn on_key_down(
        _this: *mut c_void,
        _key: i16,
        _key_code: i16,
        _modifiers: i16,
    ) -> TResult {
        result::NOT_IMPLEMENTED
    }

    unsafe extern "system" fn on_key_up(
        _this: *mut c_void,
        _key: i16,
        _key_code: i16,
        _modifiers: i16,
    ) -> TResult {
        result::NOT_IMPLEMENTED
    }

    unsafe extern "system" fn get_size(_this: *mut c_void, size: *mut ViewRect) -> TResult {
        let (width, height) = P::Editor::initial_size();

        let size = &mut *size;
        size.left = 0;
        size.top = 0;
        size.right = width.round() as i32;
        size.bottom = height.round() as i32;

        result::OK
    }

    unsafe extern "system" fn on_size(
        _this: *mut c_void,
        _new_size: *const ViewRect,
    ) -> TResult {
        result::NOT_IMPLEMENTED
    }

    unsafe extern "system" fn on_focus(_this: *mut c_void, _state: TBool) -> TResult {
        result::NOT_IMPLEMENTED
    }

    unsafe extern "system" fn set_frame(
        this: *mut c_void,
        frame: *mut *const IPlugFrame,
    ) -> TResult {
        let wrapper = &*(this.offset(-Self::PLUG_VIEW_OFFSET) as *const Wrapper<P>);
        let editor_state = &mut *wrapper.editor_state.get();

        if !editor_state.plug_frame.is_null() {
            ((*(*editor_state.plug_frame)).unknown.release)(editor_state.plug_frame as *mut c_void);
        }
        if !frame.is_null() {
            ((*(*frame)).unknown.add_ref)(frame as *mut c_void);
        }
        editor_state.plug_frame = frame;

        result::OK
    }

    unsafe extern "system" fn can_resize(_this: *mut c_void) -> TResult {
        result::FALSE
    }

    unsafe extern "system" fn check_size_constraint(
        _this: *mut c_void,
        _rect: *mut ViewRect,
    ) -> TResult {
        result::NOT_IMPLEMENTED
    }

    unsafe extern "system" fn event_handler_query_interface(
        this: *mut c_void,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> TResult {
        Self::query_interface(this.offset(-Self::EVENT_HANDLER_OFFSET), iid, obj)
    }

    unsafe extern "system" fn event_handler_add_ref(this: *mut c_void) -> u32 {
        Self::add_ref(this.offset(-Self::EVENT_HANDLER_OFFSET))
    }

    unsafe extern "system" fn event_handler_release(this: *mut c_void) -> u32 {
        Self::release(this.offset(-Self::EVENT_HANDLER_OFFSET))
    }

    #[cfg(target_os = "linux")]
    unsafe extern "system" fn on_fd_is_set(this: *mut c_void, _fd: c_int) {
        let wrapper = &*(this.offset(-Self::EVENT_HANDLER_OFFSET) as *const Wrapper<P>);
        let editor_state = &mut *wrapper.editor_state.get();

        if let Some(editor) = &mut editor_state.editor {
            editor.poll();
        }
    }

    #[cfg(not(target_os = "linux"))]
    unsafe extern "system" fn on_fd_is_set(_this: *mut c_void, _fd: c_int) {}

    unsafe extern "system" fn timer_handler_query_interface(
        this: *mut c_void,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> TResult {
        Self::query_interface(this.offset(-Self::TIMER_HANDLER_OFFSET), iid, obj)
    }

    unsafe extern "system" fn timer_handler_add_ref(this: *mut c_void) -> u32 {
        Self::add_ref(this.offset(-Self::TIMER_HANDLER_OFFSET))
    }

    unsafe extern "system" fn timer_handler_release(this: *mut c_void) -> u32 {
        Self::release(this.offset(-Self::TIMER_HANDLER_OFFSET))
    }

    #[cfg(target_os = "linux")]
    unsafe extern "system" fn on_timer(this: *mut c_void) {
        let wrapper = &*(this.offset(-Self::TIMER_HANDLER_OFFSET) as *const Wrapper<P>);
        let editor_state = &mut *wrapper.editor_state.get();

        if let Some(editor) = &mut editor_state.editor {
            editor.poll();
        }
    }

    #[cfg(not(target_os = "linux"))]
    unsafe extern "system" fn on_timer(_this: *mut c_void) {}
}

#[macro_export]
macro_rules! vst3 {
    ($plugin:ty, $uid:expr) => {
        mod vst3_impl {
            use std::ffi::c_void;
            use std::sync::Arc;

            use $crate::plugin::*;
            use $crate::vst3::*;

            #[cfg(target_os = "windows")]
            #[no_mangle]
            extern "system" fn InitDll() -> bool {
                true
            }

            #[cfg(target_os = "windows")]
            #[no_mangle]
            extern "system" fn ExitDll() -> bool {
                true
            }

            #[cfg(target_os = "macos")]
            #[no_mangle]
            extern "system" fn BundleEntry(_bundle_ref: *mut c_void) -> bool {
                true
            }

            #[cfg(target_os = "macos")]
            #[no_mangle]
            extern "system" fn BundleExit() -> bool {
                true
            }

            #[cfg(target_os = "linux")]
            #[no_mangle]
            extern "system" fn ModuleEntry(_library_handle: *mut c_void) -> bool {
                true
            }

            #[cfg(target_os = "linux")]
            #[no_mangle]
            extern "system" fn ModuleExit() -> bool {
                true
            }

            #[no_mangle]
            extern "system" fn GetPluginFactory() -> *mut c_void {
                Factory::<$plugin>::create($uid) as *mut c_void
            }
        }
    };
}

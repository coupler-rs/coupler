use crate::Plugin;

use std::cell::UnsafeCell;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::{ffi, ptr};

#[cfg(target_os = "macos")]
pub use core_foundation;

pub use vst3 as vst3_api;
use vst3::*;

fn copy_cstring(src: &str, dst: &mut [c_char]) {
    let c_string = ffi::CString::new(src).unwrap_or_else(|_| ffi::CString::default());
    for (src, dst) in c_string.as_bytes_with_nul().iter().zip(dst.iter_mut()) {
        *dst = *src as c_char;
    }
}

fn copy_wstring(src: &str, dst: &mut [i16]) {
    for (src, dst) in src.encode_utf16().zip(dst.iter_mut()) {
        *dst = src as i16;
    }
}

#[repr(C)]
pub struct Factory<P> {
    pub plugin_factory_3: *const IPluginFactory3,
    pub phantom: PhantomData<P>,
}

unsafe impl<P> Sync for Factory<P> {}

impl<P: Plugin> Factory<P> {
    pub unsafe extern "system" fn query_interface(
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
            *obj = this;
            return result::OK;
        }

        result::NO_INTERFACE
    }

    pub unsafe extern "system" fn add_ref(_this: *mut c_void) -> u32 {
        1
    }

    pub unsafe extern "system" fn release(_this: *mut c_void) -> u32 {
        1
    }

    pub unsafe extern "system" fn get_factory_info(
        _this: *mut c_void,
        info: *mut PFactoryInfo,
    ) -> TResult {
        let info = &mut *info;

        copy_cstring(P::INFO.vendor, &mut info.vendor);
        copy_cstring(P::INFO.url, &mut info.url);
        copy_cstring(P::INFO.email, &mut info.email);
        info.flags = PFactoryInfo::UNICODE;

        result::OK
    }

    pub unsafe extern "system" fn count_classes(_this: *mut c_void) -> i32 {
        1
    }

    pub unsafe extern "system" fn get_class_info(
        _this: *mut c_void,
        index: i32,
        info: *mut PClassInfo,
    ) -> TResult {
        if index != 0 {
            return result::INVALID_ARGUMENT;
        }

        let info = &mut *info;

        info.cid = iid(P::INFO.uid[0], P::INFO.uid[1], P::INFO.uid[2], P::INFO.uid[3]);
        info.cardinality = PClassInfo::MANY_INSTANCES;
        copy_cstring("Audio Module Class", &mut info.category);
        copy_cstring(P::INFO.name, &mut info.name);

        result::OK
    }

    pub unsafe extern "system" fn create_instance(
        _this: *mut c_void,
        _cid: *const c_char,
        _iid: *const c_char,
        _obj: *mut *mut c_void,
    ) -> TResult {
        result::NOT_IMPLEMENTED
    }

    pub unsafe extern "system" fn get_class_info_2(
        this: *mut c_void,
        index: i32,
        info: *mut PClassInfo2,
    ) -> TResult {
        if index != 0 {
            return result::INVALID_ARGUMENT;
        }

        let info = &mut *info;

        info.cid = iid(P::INFO.uid[0], P::INFO.uid[1], P::INFO.uid[2], P::INFO.uid[3]);
        info.cardinality = PClassInfo::MANY_INSTANCES;
        copy_cstring("Audio Module Class", &mut info.category);
        copy_cstring(P::INFO.name, &mut info.name);
        info.class_flags = 0;
        copy_cstring("Fx", &mut info.sub_categories);
        copy_cstring(P::INFO.vendor, &mut info.vendor);
        copy_cstring("", &mut info.version);
        copy_cstring("VST 3.7", &mut info.sdk_version);

        result::OK
    }

    pub unsafe extern "system" fn get_class_info_unicode(
        this: *mut c_void,
        index: i32,
        info: *mut PClassInfoW,
    ) -> TResult {
        if index != 0 {
            return result::INVALID_ARGUMENT;
        }

        let info = &mut *info;

        info.cid = iid(P::INFO.uid[0], P::INFO.uid[1], P::INFO.uid[2], P::INFO.uid[3]);
        info.cardinality = PClassInfo::MANY_INSTANCES;
        copy_cstring("Audio Module Class", &mut info.category);
        copy_wstring(P::INFO.name, &mut info.name);
        info.class_flags = 0;
        copy_cstring("Fx", &mut info.sub_categories);
        copy_wstring(P::INFO.vendor, &mut info.vendor);
        copy_wstring("", &mut info.version);
        copy_wstring("VST 3.7", &mut info.sdk_version);

        result::OK
    }

    pub unsafe extern "system" fn set_host_context(
        this: *mut c_void,
        context: *mut *const FUnknown,
    ) -> TResult {
        result::OK
    }
}

#[repr(C)]
pub struct Wrapper<P> {
    pub component: *const IComponent,
    pub audio_processor: *const IAudioProcessor,
    pub edit_controller: *const IEditController,
    pub plugin: UnsafeCell<P>,
}

impl<P: Plugin> Wrapper<P> {
    pub unsafe extern "system" fn component_query_interface(
        this: *mut c_void,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> TResult {
        result::NO_INTERFACE
    }

    pub unsafe extern "system" fn component_add_ref(_this: *mut c_void) -> u32 {
        1
    }

    pub unsafe extern "system" fn component_release(_this: *mut c_void) -> u32 {
        1
    }

    pub unsafe extern "system" fn component_initialize(
        this: *mut c_void,
        context: *mut FUnknown,
    ) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn component_terminate(this: *mut c_void) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn get_controller_class_id(
        this: *mut c_void,
        class_id: *const TUID,
    ) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn set_io_mode(this: *mut c_void, mode: IoMode) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn get_bus_count(
        this: *mut c_void,
        media_type: MediaType,
        dir: BusDirection,
    ) -> i32 {
        0
    }

    pub unsafe extern "system" fn get_bus_info(
        this: *mut c_void,
        media_type: MediaType,
        dir: BusDirection,
        index: i32,
        bus: *mut BusInfo,
    ) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn get_routing_info(
        this: *mut c_void,
        in_info: *mut RoutingInfo,
        out_info: *mut RoutingInfo,
    ) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn activate_bus(
        this: *mut c_void,
        media_type: MediaType,
        dir: BusDirection,
        index: i32,
        state: TBool,
    ) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn set_active(this: *mut c_void, state: TBool) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn component_set_state(
        this: *mut c_void,
        state: *mut IBStream,
    ) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn component_get_state(
        this: *mut c_void,
        state: *mut IBStream,
    ) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn audio_processor_query_interface(
        this: *mut c_void,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> TResult {
        result::NO_INTERFACE
    }

    pub unsafe extern "system" fn audio_processor_add_ref(_this: *mut c_void) -> u32 {
        1
    }

    pub unsafe extern "system" fn audio_processor_release(_this: *mut c_void) -> u32 {
        1
    }

    pub unsafe extern "system" fn set_bus_arrangements(
        this: *mut c_void,
        inputs: *const SpeakerArrangement,
        num_ins: i32,
        outputs: *const SpeakerArrangement,
        num_outs: i32,
    ) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn get_bus_arrangement(
        this: *mut c_void,
        dir: BusDirection,
        index: i32,
        arr: *mut SpeakerArrangement,
    ) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn can_process_sample_size(
        this: *mut c_void,
        symbolic_sample_size: i32,
    ) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn get_latency_samples(this: *mut c_void) -> u32 {
        0
    }

    pub unsafe extern "system" fn setup_processing(
        this: *mut c_void,
        setup: *mut ProcessSetup,
    ) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn set_processing(this: *mut c_void, state: TBool) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn process(this: *mut c_void, data: *mut ProcessData) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn get_tail_samples(this: *mut c_void) -> u32 {
        0
    }

    pub unsafe extern "system" fn edit_controller_query_interface(
        this: *mut c_void,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> TResult {
        result::NO_INTERFACE
    }

    pub unsafe extern "system" fn edit_controller_add_ref(_this: *mut c_void) -> u32 {
        1
    }

    pub unsafe extern "system" fn edit_controller_release(_this: *mut c_void) -> u32 {
        1
    }

    pub unsafe extern "system" fn edit_controller_initialize(
        this: *mut c_void,
        context: *mut FUnknown,
    ) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn edit_controller_terminate(this: *mut c_void) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn set_component_state(
        this: *mut c_void,
        state: *mut *const IBStream,
    ) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn edit_controller_set_state(
        this: *mut c_void,
        state: *mut *const IBStream,
    ) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn edit_controller_get_state(
        this: *mut c_void,
        state: *mut *const IBStream,
    ) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn get_parameter_count(this: *mut c_void) -> i32 {
        0
    }

    pub unsafe extern "system" fn get_parameter_info(
        this: *mut c_void,
        param_index: i32,
        info: *mut ParameterInfo,
    ) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn get_param_string_by_value(
        this: *mut c_void,
        id: u32,
        value_normalized: f64,
        string: *mut String128,
    ) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn get_param_value_by_string(
        this: *mut c_void,
        id: u32,
        string: *const TChar,
        value_normalized: *mut f64,
    ) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn normalized_param_to_plain(
        this: *mut c_void,
        id: u32,
        value_normalized: f64,
    ) -> f64 {
        0.0
    }

    pub unsafe extern "system" fn plain_param_to_normalized(
        this: *mut c_void,
        id: u32,
        plain_value: f64,
    ) -> f64 {
        0.0
    }

    pub unsafe extern "system" fn get_param_normalized(this: *mut c_void, id: u32) -> f64 {
        0.0
    }

    pub unsafe extern "system" fn set_param_normalized(
        this: *mut c_void,
        id: u32,
        value: f64,
    ) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn set_component_handler(
        this: *mut c_void,
        handler: *mut *const IComponentHandler,
    ) -> TResult {
        result::OK
    }

    pub unsafe extern "system" fn create_view(
        this: *mut c_void,
        name: *const c_char,
    ) -> *mut *const IPlugView {
        ptr::null_mut()
    }
}

#[macro_export]
macro_rules! vst3 {
    ($plugin:ty) => {
        mod vst3_impl {
            use std::ffi::c_void;
            use std::marker::PhantomData;

            use $crate::vst3::vst3_api::*;
            use $crate::vst3::*;

            static PLUGIN_FACTORY_3_VTABLE: IPluginFactory3 = IPluginFactory3 {
                plugin_factory_2: IPluginFactory2 {
                    plugin_factory: IPluginFactory {
                        unknown: FUnknown {
                            query_interface: Factory::<$plugin>::query_interface,
                            add_ref: Factory::<$plugin>::add_ref,
                            release: Factory::<$plugin>::release,
                        },
                        get_factory_info: Factory::<$plugin>::get_factory_info,
                        count_classes: Factory::<$plugin>::count_classes,
                        get_class_info: Factory::<$plugin>::get_class_info,
                        create_instance: Factory::<$plugin>::create_instance,
                    },
                    get_class_info_2: Factory::<$plugin>::get_class_info_2,
                },
                get_class_info_unicode: Factory::<$plugin>::get_class_info_unicode,
                set_host_context: Factory::<$plugin>::set_host_context,
            };

            static PLUGIN_FACTORY: Factory<$plugin> =
                Factory { plugin_factory_3: &PLUGIN_FACTORY_3_VTABLE, phantom: PhantomData };

            static COMPONENT_VTABLE: IComponent = IComponent {
                plugin_base: IPluginBase {
                    unknown: FUnknown {
                        query_interface: Wrapper::<$plugin>::component_query_interface,
                        add_ref: Wrapper::<$plugin>::component_add_ref,
                        release: Wrapper::<$plugin>::component_release,
                    },
                    initialize: Wrapper::<$plugin>::component_initialize,
                    terminate: Wrapper::<$plugin>::component_terminate,
                },
                get_controller_class_id: Wrapper::<$plugin>::get_controller_class_id,
                set_io_mode: Wrapper::<$plugin>::set_io_mode,
                get_bus_count: Wrapper::<$plugin>::get_bus_count,
                get_bus_info: Wrapper::<$plugin>::get_bus_info,
                get_routing_info: Wrapper::<$plugin>::get_routing_info,
                activate_bus: Wrapper::<$plugin>::activate_bus,
                set_active: Wrapper::<$plugin>::set_active,
                set_state: Wrapper::<$plugin>::component_set_state,
                get_state: Wrapper::<$plugin>::component_get_state,
            };

            static AUDIO_PROCESSOR_VTABLE: IAudioProcessor = IAudioProcessor {
                unknown: FUnknown {
                    query_interface: Wrapper::<$plugin>::audio_processor_query_interface,
                    add_ref: Wrapper::<$plugin>::audio_processor_add_ref,
                    release: Wrapper::<$plugin>::audio_processor_release,
                },
                set_bus_arrangements: Wrapper::<$plugin>::set_bus_arrangements,
                get_bus_arrangement: Wrapper::<$plugin>::get_bus_arrangement,
                can_process_sample_size: Wrapper::<$plugin>::can_process_sample_size,
                get_latency_samples: Wrapper::<$plugin>::get_latency_samples,
                setup_processing: Wrapper::<$plugin>::setup_processing,
                set_processing: Wrapper::<$plugin>::set_processing,
                process: Wrapper::<$plugin>::process,
                get_tail_samples: Wrapper::<$plugin>::get_tail_samples,
            };

            static EDIT_CONTROLLER_VTABLE: IEditController = IEditController {
                plugin_base: IPluginBase {
                    unknown: FUnknown {
                        query_interface: Wrapper::<$plugin>::edit_controller_query_interface,
                        add_ref: Wrapper::<$plugin>::edit_controller_add_ref,
                        release: Wrapper::<$plugin>::edit_controller_release,
                    },
                    initialize: Wrapper::<$plugin>::edit_controller_initialize,
                    terminate: Wrapper::<$plugin>::edit_controller_terminate,
                },
                set_component_state: Wrapper::<$plugin>::set_component_state,
                set_state: Wrapper::<$plugin>::edit_controller_set_state,
                get_state: Wrapper::<$plugin>::edit_controller_get_state,
                get_parameter_count: Wrapper::<$plugin>::get_parameter_count,
                get_parameter_info: Wrapper::<$plugin>::get_parameter_info,
                get_param_string_by_value: Wrapper::<$plugin>::get_param_string_by_value,
                get_param_value_by_string: Wrapper::<$plugin>::get_param_value_by_string,
                normalized_param_to_plain: Wrapper::<$plugin>::normalized_param_to_plain,
                plain_param_to_normalized: Wrapper::<$plugin>::plain_param_to_normalized,
                get_param_normalized: Wrapper::<$plugin>::get_param_normalized,
                set_param_normalized: Wrapper::<$plugin>::set_param_normalized,
                set_component_handler: Wrapper::<$plugin>::set_component_handler,
                create_view: Wrapper::<$plugin>::create_view,
            };

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
            extern "system" fn BundleEntry(
                _bundle_ref: $crate::vst3::core_foundation::bundle::CFBundleRef,
            ) -> bool {
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
                &PLUGIN_FACTORY as *const Factory<$plugin> as *mut c_void
            }
        }
    };
}

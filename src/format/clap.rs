use crate::{plugin::*, process::*};

use clap_sys::ext::params::*;
use clap_sys::{
    entry::*, events::*, host::*, id::*, plugin::*, plugin_factory::*, process::*, version::*,
};

use std::cell::UnsafeCell;
use std::ffi::{c_void, CStr, CString};
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::ptr;

struct ProcessorState<P: Plugin> {
    sample_rate: f64,
    max_buffer_size: usize,
    processor: Option<P::Processor>,
}

struct Wrapper<P: Plugin> {
    #[allow(unused)]
    clap_plugin: clap_plugin,
    plugin: PluginHandle<P>,
    processor_state: UnsafeCell<ProcessorState<P>>,
}

unsafe impl<P: Plugin> Sync for Wrapper<P> {}

impl<P: Plugin> Wrapper<P> {
    const PARAMS: clap_plugin_params = clap_plugin_params {
        count: Self::count,
        get_info: Self::get_info,
        get_value: Self::get_value,
        value_to_text: Self::value_to_text,
        text_to_value: Self::text_to_value,
        flush: Self::flush,
    };

    pub fn create(desc: *const clap_plugin_descriptor) -> *mut Wrapper<P> {
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
            plugin: PluginHandle::new(),
            processor_state: UnsafeCell::new(ProcessorState {
                sample_rate: 0.0,
                max_buffer_size: 0,
                processor: None,
            }),
        }))
    }

    unsafe extern "C" fn init(_plugin: *const clap_plugin) -> bool {
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
        _plugin: *const clap_plugin,
        id: *const c_char,
    ) -> *const c_void {
        if CStr::from_ptr(id) == CStr::from_ptr(CLAP_EXT_PARAMS) {
            return &Self::PARAMS as *const clap_plugin_params as *const c_void;
        }

        ptr::null()
    }

    unsafe extern "C" fn on_main_thread(_plugin: *const clap_plugin) {}

    unsafe extern "C" fn count(_plugin: *const clap_plugin) -> u32 {
        0
    }

    unsafe extern "C" fn get_info(
        _plugin: *const clap_plugin,
        _param_index: u32,
        _param_info: *mut clap_param_info,
    ) -> bool {
        false
    }

    unsafe extern "C" fn get_value(
        _plugin: *const clap_plugin,
        _param_id: clap_id,
        _value: *mut f64,
    ) -> bool {
        false
    }

    unsafe extern "C" fn value_to_text(
        _plugin: *const clap_plugin,
        _param_id: clap_id,
        _value: f64,
        _display: *mut c_char,
        _size: u32,
    ) -> bool {
        false
    }

    unsafe extern "C" fn text_to_value(
        _plugin: *const clap_plugin,
        _param_id: clap_id,
        _display: *const c_char,
        _value: *mut f64,
    ) -> bool {
        false
    }

    unsafe extern "C" fn flush(
        _plugin: *const clap_plugin,
        _in_: *const clap_input_events,
        _out: *const clap_output_events,
    ) {
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
            id: CString::new(&clap_info.id[..]).unwrap(),
            name: CString::new(&info.name[..]).unwrap(),
            vendor: CString::new(&info.vendor[..]).unwrap(),
            url: CString::new(&info.url[..]).unwrap(),
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
        _host: *const clap_host,
        plugin_id: *const c_char,
    ) -> *const clap_plugin {
        let factory = &*(factory as *const Self);

        if CStr::from_ptr(plugin_id) == factory.descriptor_bufs.id.as_c_str() {
            return Wrapper::<P>::create(&factory.descriptor) as *const clap_plugin;
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
    pub id: String,
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
            // Safety: The CLAP headers specify that init must be called before get_factory or
            // deinit, init must not be called more than once, and none of the three may be called
            // after deinit.
            //
            // This means that init and deinit can safely form exclusive &mut references to
            // FACTORY, and that these will not overlap with any & references formed by
            // get_factory.

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

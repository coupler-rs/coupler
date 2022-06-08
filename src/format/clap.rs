use crate::plugin::*;

use clap_sys::{entry::*, host::*, plugin::*, plugin_factory::*, version::*};

use std::ffi::{c_void, CStr};
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::ptr;

#[doc(hidden)]
pub struct Factory<P> {
    #[allow(unused)]
    factory: clap_plugin_factory,
    descriptor: clap_plugin_descriptor,
    phantom: PhantomData<P>,
}

impl<P: Plugin> Factory<P> {
    fn new() -> Factory<P> {
        Factory {
            factory: clap_plugin_factory {
                get_plugin_count: Self::get_plugin_count,
                get_plugin_descriptor: Self::get_plugin_descriptor,
                create_plugin: Self::create_plugin,
            },
            descriptor: clap_plugin_descriptor {
                clap_version: CLAP_VERSION,
                id: b"\0".as_ptr() as *const c_char,
                name: b"\0".as_ptr() as *const c_char,
                vendor: b"\0".as_ptr() as *const c_char,
                url: b"\0".as_ptr() as *const c_char,
                manual_url: b"\0".as_ptr() as *const c_char,
                support_url: b"\0".as_ptr() as *const c_char,
                version: b"\0".as_ptr() as *const c_char,
                description: b"\0".as_ptr() as *const c_char,
                features: [ptr::null()].as_ptr(),
            },
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
        let this = &*(factory as *const Self);

        if index == 0 {
            &this.descriptor
        } else {
            ptr::null()
        }
    }

    unsafe extern "C" fn create_plugin(
        _factory: *const clap_plugin_factory,
        _host: *const clap_host,
        _plugin_id: *const c_char,
    ) -> *const clap_plugin {
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

impl<P: Plugin> EntryPoint<P> {
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

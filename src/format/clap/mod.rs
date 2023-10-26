use std::ffi::{c_char, c_void};

use clap_sys::{entry::*, version::*};

mod factory;
mod gui;
mod instance;

#[doc(hidden)]
pub use factory::Factory;

pub struct ClapInfo {
    pub id: String,
}

pub trait ClapPlugin {
    fn clap_info() -> ClapInfo;
}

#[doc(hidden)]
#[repr(transparent)]
pub struct EntryPoint {
    #[allow(unused)]
    entry: clap_plugin_entry,
}

impl EntryPoint {
    pub const fn new(
        init: unsafe extern "C" fn(_plugin_path: *const c_char) -> bool,
        deinit: unsafe extern "C" fn(),
        get_factory: unsafe extern "C" fn(factory_id: *const c_char) -> *const c_void,
    ) -> EntryPoint {
        EntryPoint {
            entry: clap_plugin_entry {
                clap_version: CLAP_VERSION,
                init: Some(init),
                deinit: Some(deinit),
                get_factory: Some(get_factory),
            },
        }
    }
}

#[macro_export]
macro_rules! clap {
    ($plugin:ty) => {
        #[allow(non_upper_case_globals)]
        #[no_mangle]
        static clap_entry: ::coupler::format::clap::EntryPoint = {
            static FACTORY: ::coupler::format::clap::Factory<$plugin> =
                ::coupler::format::clap::Factory::new();

            unsafe extern "C" fn init(_plugin_path: *const ::std::ffi::c_char) -> bool {
                FACTORY.init()
            }

            unsafe extern "C" fn deinit() {
                FACTORY.deinit();
            }

            unsafe extern "C" fn get_factory(
                factory_id: *const ::std::ffi::c_char,
            ) -> *const ::std::ffi::c_void {
                FACTORY.get(factory_id)
            }

            ::coupler::format::clap::EntryPoint::new(init, deinit, get_factory)
        };
    };
}

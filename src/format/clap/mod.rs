use std::ffi::{c_char, c_void};

use clap_sys::{entry::*, version::*};

mod factory;
mod gui;
mod host;
mod instance;

#[cfg(test)]
mod tests;

#[doc(hidden)]
pub use factory::Factory;

pub struct ClapInfo<'a> {
    pub id: &'a str,
}

pub trait BuildClapInfo {
    fn info(self, info: ClapInfo);
}

pub(crate) fn with_clap_info<P, F>(f: F)
where
    P: ClapPlugin,
    F: FnOnce(ClapInfo),
{
    struct BuildClapInfoFn<F>(F);

    impl<F> BuildClapInfo for BuildClapInfoFn<F>
    where
        F: FnOnce(ClapInfo),
    {
        fn info(self, info: ClapInfo) {
            self.0(info)
        }
    }

    P::clap_info(BuildClapInfoFn(f))
}

pub trait ClapPlugin {
    fn clap_info(build: impl BuildClapInfo);
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
        #[unsafe(no_mangle)]
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

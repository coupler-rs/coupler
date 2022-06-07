use crate::plugin::*;

use clap_sys::{entry::*, version::*};

use std::ffi::c_void;
use std::os::raw::c_char;

#[repr(transparent)]
pub struct EntryPoint<P> {
    #[allow(unused)]
    entry_point: clap_plugin_entry,
    phantom: std::marker::PhantomData<P>,
}

impl<P: Plugin> EntryPoint<P> {
    pub const ENTRY_POINT: EntryPoint<P> = EntryPoint {
        entry_point: clap_plugin_entry {
            clap_version: CLAP_VERSION,
            init: Self::init,
            deinit: Self::deinit,
            get_factory: Self::get_factory,
        },
        phantom: std::marker::PhantomData,
    };

    unsafe extern "C" fn init(_plugin_path: *const c_char) -> bool {
        true
    }

    unsafe extern "C" fn deinit() {}

    unsafe extern "C" fn get_factory(_factory_id: *const c_char) -> *const c_void {
        std::ptr::null_mut()
    }
}

#[macro_export]
macro_rules! clap {
    ($plugin:ty) => {
        #[allow(non_upper_case_globals)]
        #[no_mangle]
        static clap_plugin_entry: ::coupler::format::clap::EntryPoint<$plugin> =
            ::coupler::format::clap::EntryPoint::ENTRY_POINT;
    };
}

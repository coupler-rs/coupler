use std::cell::UnsafeCell;
use std::ffi::{c_char, c_void, CStr, CString};
use std::marker::PhantomData;
use std::ptr;

use clap_sys::{host::*, plugin::*, plugin_factory::*, version::*};

use super::ClapPlugin;
use crate::Plugin;

#[doc(hidden)]
#[repr(C)]
pub struct Factory<P> {
    #[allow(unused)]
    factory: clap_plugin_factory,
    descriptor: UnsafeCell<Option<clap_plugin_descriptor>>,
    _marker: PhantomData<P>,
}

unsafe impl<P> Sync for Factory<P> {}

impl<P: Plugin + ClapPlugin> Factory<P> {
    pub const fn new() -> Self {
        Factory {
            factory: clap_plugin_factory {
                get_plugin_count: Some(Self::get_plugin_count),
                get_plugin_descriptor: Some(Self::get_plugin_descriptor),
                create_plugin: Some(Self::create_plugin),
            },
            descriptor: UnsafeCell::new(None),
            _marker: PhantomData,
        }
    }

    pub unsafe fn init(&self) -> bool {
        let clap_info = P::clap_info();
        let id = CString::new(&*clap_info.id).unwrap().into_raw();

        let info = P::info();
        let name = CString::new(&*info.name).unwrap().into_raw();
        let vendor = CString::new(&*info.vendor).unwrap().into_raw();
        let url = CString::new(&*info.url).unwrap().into_raw();

        const EMPTY: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"\0") };
        const FEATURES: &'static [*const c_char] = &[ptr::null()];

        *self.descriptor.get() = Some(clap_plugin_descriptor {
            clap_version: CLAP_VERSION,
            id,
            name,
            vendor,
            url,
            manual_url: EMPTY.as_ptr(),
            support_url: EMPTY.as_ptr(),
            version: EMPTY.as_ptr(),
            description: EMPTY.as_ptr(),
            features: FEATURES.as_ptr(),
        });

        true
    }

    pub unsafe fn deinit(&self) {
        if let Some(descriptor) = (*self.descriptor.get()).take() {
            drop(CString::from_raw(descriptor.id as *mut c_char));
            drop(CString::from_raw(descriptor.name as *mut c_char));
            drop(CString::from_raw(descriptor.vendor as *mut c_char));
            drop(CString::from_raw(descriptor.url as *mut c_char));
        }
    }

    pub unsafe fn get(&self, factory_id: *const c_char) -> *const c_void {
        if CStr::from_ptr(factory_id) == CLAP_PLUGIN_FACTORY_ID {
            return self as *const Self as *const c_void;
        }

        ptr::null()
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
            if let Some(descriptor) = &*factory.descriptor.get() {
                return descriptor;
            }
        }

        ptr::null()
    }

    unsafe extern "C" fn create_plugin(
        _factory: *const clap_plugin_factory,
        _host: *const clap_host,
        _plugin_id: *const c_char,
    ) -> *const clap_plugin {
        ptr::null()
    }
}

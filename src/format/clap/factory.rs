use std::cell::UnsafeCell;
use std::ffi::{c_char, c_void, CStr, CString};
use std::marker::PhantomData;
use std::ptr;
use std::sync::Arc;

use clap_sys::{host::*, plugin::*, plugin_factory::*, version::*};

use super::instance::Instance;
use super::ClapPlugin;
use crate::plugin::{Plugin, PluginInfo};

struct FactoryState {
    descriptor: clap_plugin_descriptor,
    info: Arc<PluginInfo>,
}

#[doc(hidden)]
#[repr(C)]
pub struct Factory<P> {
    #[allow(unused)]
    factory: clap_plugin_factory,
    state: UnsafeCell<Option<FactoryState>>,
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
            state: UnsafeCell::new(None),
            _marker: PhantomData,
        }
    }

    pub unsafe fn init(&self) -> bool {
        let info = Arc::new(P::info());
        let clap_info = P::clap_info();

        let id = CString::new(&*clap_info.id).unwrap().into_raw();
        let name = CString::new(&*info.name).unwrap().into_raw();
        let vendor = CString::new(&*info.vendor).unwrap().into_raw();
        let url = CString::new(&*info.url).unwrap().into_raw();
        let version = CString::new(&*info.version).unwrap().into_raw();

        const EMPTY: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"\0") };
        const FEATURES: &'static [*const c_char] = &[ptr::null()];

        *self.state.get() = Some(FactoryState {
            descriptor: clap_plugin_descriptor {
                clap_version: CLAP_VERSION,
                id,
                name,
                vendor,
                url,
                manual_url: EMPTY.as_ptr(),
                support_url: EMPTY.as_ptr(),
                version,
                description: EMPTY.as_ptr(),
                features: FEATURES.as_ptr(),
            },
            info,
        });

        true
    }

    pub unsafe fn deinit(&self) {
        if let Some(state) = (*self.state.get()).take() {
            drop(CString::from_raw(state.descriptor.id as *mut c_char));
            drop(CString::from_raw(state.descriptor.name as *mut c_char));
            drop(CString::from_raw(state.descriptor.vendor as *mut c_char));
            drop(CString::from_raw(state.descriptor.url as *mut c_char));
            drop(CString::from_raw(state.descriptor.version as *mut c_char));
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
            if let Some(state) = &*factory.state.get() {
                return &state.descriptor;
            }
        }

        ptr::null()
    }

    unsafe extern "C" fn create_plugin(
        factory: *const clap_plugin_factory,
        _host: *const clap_host,
        plugin_id: *const c_char,
    ) -> *const clap_plugin {
        let factory = &*(factory as *const Self);

        if let Some(state) = &*factory.state.get() {
            if CStr::from_ptr(plugin_id) == CStr::from_ptr(state.descriptor.id) {
                let instance = Box::new(Instance::<P>::new(&state.descriptor, &state.info));
                return Box::into_raw(instance) as *const clap_plugin;
            }
        }

        ptr::null()
    }
}

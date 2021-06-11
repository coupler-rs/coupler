use crate::Plugin;

use std::ffi::c_void;
use std::marker::PhantomData;
use std::os::raw::c_char;
use std::{ffi, ptr};

pub use vst3 as vst3_api;
use vst3::*;

unsafe fn copy_cstring(string: &str, dst: *mut c_char, len: usize) {
    let name = ffi::CString::new(string).unwrap_or_else(|_| ffi::CString::default());
    ptr::copy_nonoverlapping(name.as_ptr(), dst as *mut c_char, name.as_bytes().len().min(len));
}

#[repr(C)]
pub struct Factory<P> {
    pub vtable: *const IPluginFactory,
    pub phantom: PhantomData<P>,
}

unsafe impl<P> Sync for Factory<P> {}

impl<P: Plugin> Factory<P> {
    pub extern "system" fn query_interface(
        this: *mut c_void,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> TResult {
        unsafe {
            let iid = *iid;

            if iid == FUnknown::IID || iid == IPluginFactory::IID {
                *obj = this;
                return result::OK;
            }

            result::NO_INTERFACE
        }
    }

    pub extern "system" fn add_ref(_this: *mut c_void) -> u32 {
        1
    }

    pub extern "system" fn release(_this: *mut c_void) -> u32 {
        1
    }

    pub extern "system" fn get_factory_info(
        _this: *mut c_void,
        info: *mut PFactoryInfo,
    ) -> TResult {
        unsafe {
            let info = &mut *info;

            copy_cstring(P::INFO.vendor, info.vendor.as_mut_ptr(), info.vendor.len());
            copy_cstring(P::INFO.url, info.url.as_mut_ptr(), info.url.len());
            copy_cstring(P::INFO.email, info.email.as_mut_ptr(), info.email.len());
            info.flags = PFactoryInfo::NO_FLAGS;

            result::OK
        }
    }

    pub extern "system" fn count_classes(_this: *mut c_void) -> i32 {
        1
    }

    pub extern "system" fn get_class_info(
        _this: *mut c_void,
        index: i32,
        info: *mut PClassInfo,
    ) -> TResult {
        unsafe {
            if index != 0 {
                return result::INVALID_ARGUMENT;
            }

            let info = &mut *info;

            info.cid = iid(P::INFO.uid[0], P::INFO.uid[1], P::INFO.uid[2], P::INFO.uid[3]);
            info.cardinality = PClassInfo::MANY_INSTANCES;
            copy_cstring("Audio Module Class", info.category.as_mut_ptr(), info.category.len());
            copy_cstring(P::INFO.name, info.name.as_mut_ptr(), info.name.len());

            result::OK
        }
    }

    pub extern "system" fn create_instance(
        _this: *mut c_void,
        _cid: *const c_char,
        _iid: *const c_char,
        _obj: *mut *mut c_void,
    ) -> TResult {
        result::NOT_IMPLEMENTED
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

            static FACTORY_VTABLE: IPluginFactory = IPluginFactory {
                unknown: FUnknown {
                    query_interface: Factory::<$plugin>::query_interface,
                    add_ref: Factory::<$plugin>::add_ref,
                    release: Factory::<$plugin>::release,
                },
                get_factory_info: Factory::<$plugin>::get_factory_info,
                count_classes: Factory::<$plugin>::count_classes,
                get_class_info: Factory::<$plugin>::get_class_info,
                create_instance: Factory::<$plugin>::create_instance,
            };

            static FACTORY: Factory<$plugin> =
                Factory { vtable: &FACTORY_VTABLE, phantom: PhantomData };

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
            extern "system" fn BundleEntry(bundle_ref: _core_foundation::CFBundleRef) -> bool {
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
                &FACTORY as *const Factory<$plugin> as *mut c_void
            }
        }
    };
}

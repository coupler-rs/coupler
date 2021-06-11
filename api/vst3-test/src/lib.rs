use std::ffi::c_void;
use std::os::raw::c_char;

use vst3::*;

#[repr(C)]
struct Factory {
    vtable: *const IPluginFactory,
}

unsafe impl Sync for Factory {}

static FACTORY_VTABLE: IPluginFactory = {
    extern "system" fn query_interface(
        this: *mut c_void,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> TResult {
        let iid = unsafe { *iid };

        if iid == FUnknown::IID || iid == IPluginFactory::IID {
            unsafe { *obj = this };
            return result::OK;
        }

        result::NO_INTERFACE
    }

    extern "system" fn add_ref(_this: *mut c_void) -> u32 {
        1
    }

    extern "system" fn release(_this: *mut c_void) -> u32 {
        0
    }

    extern "system" fn get_factory_info(_this: *mut c_void, info: *mut PFactoryInfo) -> TResult {
        let info = unsafe { &mut *info };

        let vendor = b"vendor\0";
        unsafe {
            std::ptr::copy(vendor.as_ptr() as *const c_char, info.vendor.as_mut_ptr(), vendor.len())
        };

        let url = b"https://example.com/\0";
        unsafe { std::ptr::copy(url.as_ptr() as *const c_char, info.url.as_mut_ptr(), url.len()) };

        let email = b"webmaster@example.com\0";
        unsafe {
            std::ptr::copy(email.as_ptr() as *const c_char, info.email.as_mut_ptr(), email.len())
        };

        info.flags = PFactoryInfo::NO_FLAGS;

        result::OK
    }

    extern "system" fn count_classes(_this: *mut c_void) -> i32 {
        1
    }

    extern "system" fn get_class_info(
        _this: *mut c_void,
        index: i32,
        info: *mut PClassInfo,
    ) -> TResult {
        if index != 0 {
            return result::INVALID_ARGUMENT;
        }

        let info = unsafe { &mut *info };

        info.cid = iid(0x1A4F6893, 0x11460191, 0x0000B439, 0xE5648ADA);

        info.cardinality = PClassInfo::MANY_INSTANCES;

        let category = b"Audio Module Class\0";
        unsafe {
            std::ptr::copy(
                category.as_ptr() as *const c_char,
                info.category.as_mut_ptr(),
                category.len(),
            )
        };

        let name = b"vst3 test\0";
        unsafe {
            std::ptr::copy(name.as_ptr() as *const c_char, info.name.as_mut_ptr(), name.len())
        };

        result::OK
    }

    extern "system" fn create_instance(
        _this: *mut c_void,
        _cid: *const c_char,
        _iid: *const c_char,
        _obj: *mut *mut c_void,
    ) -> TResult {
        result::OK
    }

    IPluginFactory {
        unknown: FUnknown { query_interface, add_ref, release },
        get_factory_info,
        count_classes,
        get_class_info,
        create_instance,
    }
};

static FACTORY: Factory = Factory { vtable: &FACTORY_VTABLE };

#[cfg(target_os = "windows")]
#[no_mangle]
pub extern "system" fn InitDll() -> bool {
    true
}

#[cfg(target_os = "windows")]
#[no_mangle]
pub extern "system" fn ExitDll() -> bool {
    true
}

#[cfg(target_os = "macos")]
#[no_mangle]
pub extern "system" fn BundleEntry(_bundle_ref: core_foundation::CFBundleRef) -> bool {
    true
}

#[cfg(target_os = "macos")]
#[no_mangle]
pub extern "system" fn BundleExit() -> bool {
    true
}

#[cfg(target_os = "linux")]
#[no_mangle]
pub extern "system" fn ModuleEntry(_library_handle: *mut c_void) -> bool {
    true
}

#[cfg(target_os = "linux")]
#[no_mangle]
pub extern "system" fn ModuleExit() -> bool {
    true
}

#[no_mangle]
pub extern "system" fn GetPluginFactory() -> *mut c_void {
    &FACTORY as *const Factory as *mut c_void
}

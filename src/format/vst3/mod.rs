use std::ffi::c_void;

use vst3_bindgen::{ComWrapper, Steinberg::IPluginFactory};

mod factory;
mod util;

use crate::Plugin;
use factory::Factory;

pub struct Uuid(pub u32, pub u32, pub u32, pub u32);

pub struct Vst3Info {
    pub class_id: Uuid,
}

pub trait Vst3Plugin {
    fn vst3_info() -> Vst3Info;
}

#[doc(hidden)]
pub fn get_plugin_factory<P: Plugin + Vst3Plugin>() -> *mut c_void {
    ComWrapper::new(Factory::<P>::new())
        .to_com_ptr::<IPluginFactory>()
        .unwrap()
        .into_raw() as *mut c_void
}

#[macro_export]
macro_rules! vst3 {
    ($plugin:ty) => {
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
        extern "system" fn BundleEntry(_bundle_ref: *mut ::std::ffi::c_void) -> bool {
            true
        }

        #[cfg(target_os = "macos")]
        #[no_mangle]
        extern "system" fn BundleExit() -> bool {
            true
        }

        #[cfg(target_os = "linux")]
        #[no_mangle]
        extern "system" fn ModuleEntry(_library_handle: *mut ::std::ffi::c_void) -> bool {
            true
        }

        #[cfg(target_os = "linux")]
        #[no_mangle]
        extern "system" fn ModuleExit() -> bool {
            true
        }

        #[no_mangle]
        extern "system" fn GetPluginFactory() -> *mut ::std::ffi::c_void {
            ::coupler::format::vst3::get_plugin_factory::<$plugin>()
        }
    };
}

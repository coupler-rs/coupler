#![allow(non_snake_case)]

use std::ffi::c_void;

use vst3::{ComWrapper, Steinberg::IPluginFactory};

mod buffers;
mod component;
mod factory;
mod host;
mod util;
mod view;

#[cfg(test)]
mod tests;

use crate::plugin::Plugin;
use factory::Factory;

pub struct Uuid(pub u32, pub u32, pub u32, pub u32);

impl Uuid {
    pub fn from_name(name: &str) -> Uuid {
        const NAMESPACE_COUPLER: uuid::Uuid = uuid::Uuid::from_bytes([
            0xad, 0xf0, 0x07, 0x9f, 0x40, 0x7f, 0x49, 0xcc, 0xb1, 0x0d, 0x2d, 0x37, 0x63, 0x36,
            0x57, 0x58,
        ]);

        let uuid = uuid::Uuid::new_v5(&NAMESPACE_COUPLER, name.as_bytes());
        let bytes = uuid.as_bytes();
        Uuid(
            u32::from_be_bytes(bytes[0..4].try_into().unwrap()),
            u32::from_be_bytes(bytes[4..8].try_into().unwrap()),
            u32::from_be_bytes(bytes[8..12].try_into().unwrap()),
            u32::from_be_bytes(bytes[12..16].try_into().unwrap()),
        )
    }
}

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

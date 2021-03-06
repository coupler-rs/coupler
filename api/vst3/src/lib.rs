use std::ffi::c_void;
use std::os::raw::c_char;

pub type TUID = [u8; 16];

#[cfg(target_os = "windows")]
pub const fn iid(a: u32, b: u32, c: u32, d: u32) -> TUID {
    [
        ((a & 0x000000FF) >> 0) as u8,
        ((a & 0x0000FF00) >> 8) as u8,
        ((a & 0x00FF0000) >> 16) as u8,
        ((a & 0xFF000000) >> 24) as u8,
        ((b & 0x00FF0000) >> 16) as u8,
        ((b & 0xFF000000) >> 24) as u8,
        ((b & 0x000000FF) >> 0) as u8,
        ((b & 0x0000FF00) >> 8) as u8,
        ((c & 0xFF000000) >> 24) as u8,
        ((c & 0x00FF0000) >> 16) as u8,
        ((c & 0x0000FF00) >> 8) as u8,
        ((c & 0x000000FF) >> 0) as u8,
        ((d & 0xFF000000) >> 24) as u8,
        ((d & 0x00FF0000) >> 16) as u8,
        ((d & 0x0000FF00) >> 8) as u8,
        ((d & 0x000000FF) >> 0) as u8,
    ]
}

#[cfg(not(target_os = "windows"))]
pub const fn iid(a: u32, b: u32, c: u32, d: u32) -> TUID {
    [
        ((a & 0xFF000000) >> 24) as u8,
        ((a & 0x00FF0000) >> 16) as u8,
        ((a & 0x0000FF00) >> 8) as u8,
        ((a & 0x000000FF) >> 0) as u8,
        ((b & 0xFF000000) >> 24) as u8,
        ((b & 0x00FF0000) >> 16) as u8,
        ((b & 0x0000FF00) >> 8) as u8,
        ((b & 0x000000FF) >> 0) as u8,
        ((c & 0xFF000000) >> 24) as u8,
        ((c & 0x00FF0000) >> 16) as u8,
        ((c & 0x0000FF00) >> 8) as u8,
        ((c & 0x000000FF) >> 0) as u8,
        ((d & 0xFF000000) >> 24) as u8,
        ((d & 0x00FF0000) >> 16) as u8,
        ((d & 0x0000FF00) >> 8) as u8,
        ((d & 0x000000FF) >> 0) as u8,
    ]
}

pub type TResult = i32;

#[cfg(target_os = "windows")]
#[allow(overflowing_literals)]
pub mod result {
    use crate::TResult;
    pub const NO_INTERFACE: TResult = 0x80004002;
    pub const OK: TResult = 0x00000000;
    pub const TRUE: TResult = 0x00000000;
    pub const FALSE: TResult = 0x00000001;
    pub const INVALID_ARGUMENT: TResult = 0x80070057;
    pub const NOT_IMPLEMENTED: TResult = 0x80004001;
    pub const INTERNAL_ERROR: TResult = 0x80004005;
    pub const NOT_INITIALIZED: TResult = 0x8000FFFF;
    pub const OUT_OF_MEMORY: TResult = 0x8007000E;
}

#[cfg(not(target_os = "windows"))]
#[allow(overflowing_literals)]
pub mod result {
    use crate::TResult;
    pub const NO_INTERFACE: TResult = -1;
    pub const OK: TResult = 0;
    pub const TRUE: TResult = 1;
    pub const FALSE: TResult = 2;
    pub const INVALID_ARGUMENT: TResult = 3;
    pub const NOT_IMPLEMENTED: TResult = 4;
    pub const INTERNAL_ERROR: TResult = 5;
    pub const NOT_INITIALIZED: TResult = 6;
    pub const OUT_OF_MEMORY: TResult = 7;
}

#[repr(C)]
pub struct FUnknown {
    pub query_interface: unsafe extern "system" fn(
        this: *mut c_void,
        iid: *const TUID,
        obj: *mut *mut c_void,
    ) -> TResult,
    pub add_ref: unsafe extern "system" fn(this: *mut c_void) -> u32,
    pub release: unsafe extern "system" fn(this: *mut c_void) -> u32,
}

impl FUnknown {
    pub const IID: TUID = iid(0x00000000, 0x00000000, 0xC0000000, 0x00000046);
}

#[repr(C)]
pub struct IPluginFactory {
    pub parent: FUnknown,
    pub get_factory_info:
        unsafe extern "system" fn(this: *mut c_void, info: *mut PFactoryInfo) -> TResult,
    pub count_classes: unsafe extern "system" fn(this: *mut c_void) -> i32,
    pub get_class_info:
        unsafe extern "system" fn(this: *mut c_void, index: i32, info: *mut PClassInfo) -> TResult,
    pub create_instance: unsafe extern "system" fn(
        this: *mut c_void,
        cid: *const c_char,
        iid: *const c_char,
        obj: *mut *mut c_void,
    ) -> TResult,
}

impl IPluginFactory {
    pub const IID: TUID = iid(0x7A4D811C, 0x52114A1F, 0xAED9D2EE, 0x0B43BF9F);
}

#[repr(C)]
pub struct PFactoryInfo {
    pub vendor: [c_char; 64],
    pub url: [c_char; 256],
    pub email: [c_char; 128],
    pub flags: i32,
}

impl PFactoryInfo {
    pub const NO_FLAGS: i32 = 0;
    pub const CLASSES_DISCARDABLE: i32 = 1 << 0;
    pub const LICENSE_CHECK: i32 = 1 << 1;
    pub const COMPONENT_NON_DISCARDABLE: i32 = 1 << 3;
    pub const UNICODE: i32 = 1 << 4;
}

#[repr(C)]
pub struct PClassInfo {
    pub cid: TUID,
    pub cardinality: i32,
    pub category: [c_char; 32],
    pub name: [c_char; 64],
}

impl PClassInfo {
    pub const MANY_INSTANCES: i32 = 0x7FFFFFFF;
}

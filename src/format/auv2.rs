use std::ffi::c_void;
use std::ptr;

#[doc(hidden)]
pub unsafe fn auv2_factory(_in_desc: *mut c_void) -> *mut c_void {
    ptr::null_mut()
}

#[macro_export]
macro_rules! auv2 {
    ($plugin:ty) => {
        #[no_mangle]
        unsafe extern "C" fn AUFactory(inDesc: *mut ::std::ffi::c_void) -> *mut ::std::ffi::c_void {
            ::coupler::format::auv2::auv2_factory(inDesc)
        }
    };
}

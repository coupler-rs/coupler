use std::ffi::c_void;
use std::ptr;

use auv2_sys::*;

struct Wrapper {
    #[allow(unused)]
    interface: AudioComponentPlugInInterface,
}

#[allow(non_snake_case)]
impl Wrapper {
    fn create() -> *mut Wrapper {
        Box::into_raw(Box::new(Wrapper {
            interface: AudioComponentPlugInInterface {
                Open: Some(Self::Open),
                Close: Some(Self::Close),
                Lookup: Some(Self::Lookup),
                reserved: ptr::null_mut(),
            },
        }))
    }

    pub unsafe extern "C" fn Open(
        _self_: *mut c_void,
        _mInstance: AudioComponentInstance,
    ) -> OSStatus {
        noErr
    }

    pub unsafe extern "C" fn Close(self_: *mut c_void) -> OSStatus {
        drop(Box::from_raw(self_ as *mut Wrapper));

        noErr
    }

    pub unsafe extern "C" fn Lookup(_selector: SInt16) -> AudioComponentMethod {
        None
    }
}

#[doc(hidden)]
pub unsafe fn auv2_factory(_in_desc: *mut c_void) -> *mut c_void {
    Wrapper::create() as *mut c_void
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

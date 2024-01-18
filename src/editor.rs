use std::ffi::{c_ulong, c_void};

use crate::params::{ParamId, ParamValue};

#[derive(Copy, Clone)]
pub enum RawParent {
    Win32(*mut c_void),
    Cocoa(*mut c_void),
    X11(c_ulong),
}

pub struct Parent {
    parent: RawParent,
}

impl Parent {
    pub unsafe fn from_raw(parent: RawParent) -> Parent {
        Parent { parent }
    }

    pub fn as_raw(&self) -> RawParent {
        self.parent
    }
}

pub struct Size {
    pub width: f64,
    pub height: f64,
}

pub trait Editor: Sized + 'static {
    fn exists() -> bool {
        true
    }

    fn size(&self) -> Size;
    fn set_param(&mut self, id: ParamId, value: ParamValue);
}

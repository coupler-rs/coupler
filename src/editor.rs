use std::ffi::{c_ulong, c_void};
use std::marker::PhantomData;
use std::rc::Rc;

use crate::params::{ParamId, ParamValue};

pub trait EditorHostInner {
    fn begin_gesture(&self, id: ParamId);
    fn end_gesture(&self, id: ParamId);
    fn set_param(&self, id: ParamId, value: ParamValue);
}

#[derive(Clone)]
pub struct EditorHost {
    inner: Rc<dyn EditorHostInner>,
    // Ensure !Send and !Sync
    _marker: PhantomData<*mut ()>,
}

impl EditorHost {
    pub fn from_inner(inner: Rc<dyn EditorHostInner>) -> EditorHost {
        EditorHost {
            inner,
            _marker: PhantomData,
        }
    }

    pub fn begin_gesture(&self, id: ParamId) {
        self.inner.begin_gesture(id);
    }

    pub fn end_gesture(&self, id: ParamId) {
        self.inner.end_gesture(id);
    }

    pub fn set_param(&self, id: ParamId, value: ParamValue) {
        self.inner.set_param(id, value);
    }
}

#[derive(Copy, Clone)]
pub enum RawParent {
    Win32(*mut c_void),
    Cocoa(*mut c_void),
    X11(c_ulong),
}

pub struct ParentWindow {
    parent: RawParent,
}

impl ParentWindow {
    pub unsafe fn from_raw(parent: RawParent) -> ParentWindow {
        ParentWindow { parent }
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
    fn size(&self) -> Size;
    fn param_changed(&mut self, id: ParamId, value: ParamValue);
}

pub struct NoEditor;

impl Editor for NoEditor {
    fn size(&self) -> Size {
        Size {
            width: 0.0,
            height: 0.0,
        }
    }

    fn param_changed(&mut self, _id: ParamId, _value: ParamValue) {}
}

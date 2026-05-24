use std::ffi::{c_ulong, c_void};
use std::marker::PhantomData;
use std::rc::Rc;

pub trait EditorHostInner {
    fn begin_gesture(&self, index: usize);
    fn end_gesture(&self, index: usize);
    fn set_param(&self, index: usize, value: f64);
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

    pub fn begin_gesture(&self, index: usize) {
        self.inner.begin_gesture(index);
    }

    pub fn end_gesture(&self, index: usize) {
        self.inner.end_gesture(index);
    }

    pub fn set_param(&self, index: usize, value: f64) {
        self.inner.set_param(index, value);
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
    fn param_changed(&mut self, index: usize, value: f64);
}

pub struct NoEditor;

impl Editor for NoEditor {
    fn size(&self) -> Size {
        Size {
            width: 0.0,
            height: 0.0,
        }
    }

    fn param_changed(&mut self, _index: usize, _value: f64) {}
}

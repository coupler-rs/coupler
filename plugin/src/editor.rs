use crate::params::*;

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use std::rc::Rc;

pub trait EditorContextInner {
    fn get_param(&self, param_id: ParamId) -> f64;
    fn begin_edit(&self, param_id: ParamId);
    fn perform_edit(&self, param_id: ParamId, value: f64);
    fn end_edit(&self, param_id: ParamId);
}

#[derive(Clone)]
pub struct EditorContext {
    pub(crate) inner: Rc<dyn EditorContextInner>,
}

impl EditorContext {
    pub fn get_param(&self, id: ParamId) -> f64 {
        self.inner.get_param(id)
    }

    pub fn begin_edit(&self, id: ParamId) {
        self.inner.begin_edit(id);
    }

    pub fn perform_edit(&self, id: ParamId, value: f64) {
        self.inner.perform_edit(id, value);
    }

    pub fn end_edit(&self, id: ParamId) {
        self.inner.end_edit(id);
    }
}

pub struct ParentWindow(pub(crate) RawWindowHandle);

unsafe impl HasRawWindowHandle for ParentWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.0
    }
}

pub trait Editor: Sized {
    fn initial_size() -> (f64, f64);
    fn close(&mut self);
    fn raw_window_handle(&self) -> Option<RawWindowHandle>;

    #[cfg(target_os = "linux")]
    fn file_descriptor(&self) -> Option<std::os::raw::c_int>;
    #[cfg(target_os = "linux")]
    fn poll(&mut self);
}

pub struct NoEditor;

impl Editor for NoEditor {
    fn initial_size() -> (f64, f64) {
        (0.0, 0.0)
    }
    fn close(&mut self) {}
    fn raw_window_handle(&self) -> Option<RawWindowHandle> {
        None
    }

    #[cfg(target_os = "linux")]
    fn file_descriptor(&self) -> Option<std::os::raw::c_int> {
        None
    }
    #[cfg(target_os = "linux")]
    fn poll(&mut self) {}
}

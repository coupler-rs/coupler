use crate::params::*;

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use std::rc::Rc;
use std::sync::Arc;

pub trait EditorContextInner {
    fn get_param(&self, param_id: ParamId) -> f64;
    fn begin_edit(&self, param_id: ParamId);
    fn perform_edit(&self, param_id: ParamId, value: f64);
    fn end_edit(&self, param_id: ParamId);
}

#[derive(Clone)]
pub struct EditorContext {
    pub(crate) param_list: Arc<ParamList>,
    pub(crate) inner: Rc<dyn EditorContextInner>,
}

impl EditorContext {
    pub fn get_param<P: Param + 'static>(&self, key: ParamKey<P>) -> P::Value {
        let index = self.param_list.indices[&key.id()];
        let param = self.param_list.params[index].param.as_any().downcast_ref::<P>().unwrap();
        param.decode(self.inner.get_param(key.id()))
    }

    pub fn begin_edit<P: Param + 'static>(&self, key: ParamKey<P>) {
        self.inner.begin_edit(key.id());
    }

    pub fn perform_edit<P: Param + 'static>(&self, key: ParamKey<P>, value: P::Value) {
        let index = self.param_list.indices[&key.id()];
        let param = self.param_list.params[index].param.as_any().downcast_ref::<P>().unwrap();
        self.inner.perform_edit(key.id(), param.encode(value));
    }

    pub fn end_edit<P: Param + 'static>(&self, key: ParamKey<P>) {
        self.inner.end_edit(key.id());
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

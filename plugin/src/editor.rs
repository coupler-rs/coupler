use crate::{param::*, plugin::*};

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone)]
pub struct EditorContext {
    param_states: Arc<ParamStates>,
    handler: Rc<dyn EditorContextHandler>,
}

impl EditorContext {
    pub fn new(
        param_states: Arc<ParamStates>,
        handler: Rc<dyn EditorContextHandler>,
    ) -> EditorContext {
        EditorContext { param_states, handler }
    }

    #[inline]
    pub fn get_param<P: Param + 'static>(&self, key: ParamKey<P>) -> P::Value {
        self.param_states.get_param(key)
    }

    pub fn begin_edit<P: Param + 'static>(&self, key: ParamKey<P>) {
        let param_info = self.param_states.list.get_param(key.id).expect("Invalid parameter key");
        let _ = param_info.param.downcast_ref::<P>().expect("Incorrect parameter type");
        self.handler.begin_edit(key.id);
    }

    pub fn perform_edit<P: Param + 'static>(&self, key: ParamKey<P>, value: P::Value) {
        let param_info = self.param_states.list.get_param(key.id).expect("Invalid parameter key");
        let param = param_info.param.downcast_ref::<P>().expect("Incorrect parameter type");
        self.handler.perform_edit(key.id, param.to_normalized(value));
    }

    pub fn end_edit<P: Param + 'static>(&self, key: ParamKey<P>) {
        let param_info = self.param_states.list.get_param(key.id).expect("Invalid parameter key");
        let _ = param_info.param.downcast_ref::<P>().expect("Incorrect parameter type");
        self.handler.end_edit(key.id);
    }
}

pub trait EditorContextHandler {
    fn begin_edit(&self, param_id: ParamId);
    fn perform_edit(&self, param_id: ParamId, value: f64);
    fn end_edit(&self, param_id: ParamId);
}

pub struct ParentWindow(pub(crate) RawWindowHandle);

unsafe impl HasRawWindowHandle for ParentWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.0
    }
}

pub trait Editor: Sized {
    type Plugin: Plugin;

    fn open(plugin: &Self::Plugin, context: EditorContext, parent: Option<&ParentWindow>) -> Self;
    fn close(&mut self);
    fn size() -> (f64, f64);
    fn raw_window_handle(&self) -> Option<RawWindowHandle>;

    #[cfg(target_os = "linux")]
    fn file_descriptor(&self) -> Option<std::os::raw::c_int>;
    #[cfg(target_os = "linux")]
    fn poll(&mut self);
}

pub struct NoEditor<P> {
    phantom: PhantomData<P>,
}

impl<P: Plugin> Editor for NoEditor<P> {
    type Plugin = P;

    fn open(
        _plugin: &Self::Plugin,
        _context: EditorContext,
        _parent: Option<&ParentWindow>,
    ) -> Self {
        NoEditor { phantom: PhantomData }
    }

    fn close(&mut self) {}

    fn size() -> (f64, f64) {
        (0.0, 0.0)
    }

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

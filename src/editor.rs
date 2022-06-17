use crate::atomic::DrainIndices;
use crate::{param::*, plugin::*};

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use std::marker::PhantomData;
use std::rc::Rc;

pub struct EditorContext {
    handler: Rc<dyn EditorContextHandler>,
}

impl EditorContext {
    pub(crate) fn new(handler: Rc<dyn EditorContextHandler>) -> EditorContext {
        EditorContext { handler }
    }

    pub fn begin_edit(&self, id: ParamId) {
        self.handler.begin_edit(id);
    }

    pub fn perform_edit(&self, id: ParamId, value_normalized: f64) {
        self.handler.perform_edit(id, value_normalized);
    }

    pub fn end_edit(&self, id: ParamId) {
        self.handler.end_edit(id);
    }

    pub fn poll_params(&self) -> PollParams {
        self.handler.poll_params()
    }
}

pub trait EditorContextHandler {
    fn begin_edit(&self, param_id: ParamId);
    fn perform_edit(&self, param_id: ParamId, value: f64);
    fn end_edit(&self, param_id: ParamId);
    fn poll_params(&self) -> PollParams;
}

pub struct PollParams<'a> {
    pub(crate) iter: DrainIndices<'a>,
    pub(crate) param_states: &'a ParamStates,
}

impl<'a> Iterator for PollParams<'a> {
    type Item = ParamId;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.iter.next() {
            Some(self.param_states.info[index].id)
        } else {
            None
        }
    }
}

pub struct ParentWindow(pub(crate) RawWindowHandle);

unsafe impl HasRawWindowHandle for ParentWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.0
    }
}

pub trait Editor: Sized {
    type Plugin: Plugin;

    fn open(
        plugin: PluginHandle<Self::Plugin>,
        context: EditorContext,
        parent: Option<&ParentWindow>,
    ) -> Self;
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
        _plugin: PluginHandle<Self::Plugin>,
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

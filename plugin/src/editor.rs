use crate::atomic::DrainIndices;
use crate::{param::*, plugin::*};

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub struct EditorContext {
    param_states: Arc<ParamStates>,
    handler: Rc<dyn EditorContextHandler>,
}

impl EditorContext {
    pub(crate) fn new(
        param_states: Arc<ParamStates>,
        handler: Rc<dyn EditorContextHandler>,
    ) -> EditorContext {
        EditorContext { param_states, handler }
    }

    pub fn begin_edit(&self, id: ParamId) {
        let _ = self.param_states.index.get(&id).expect("Invalid parameter id");
        self.handler.begin_edit(id);
    }

    pub fn perform_edit(&self, id: ParamId, value_normalized: f64) {
        let param_index = *self.param_states.index.get(&id).expect("Invalid parameter id");
        self.handler.perform_edit(id, value_normalized);
        self.param_states.dirty_processor.set(param_index, Ordering::Release);
    }

    pub fn end_edit(&self, id: ParamId) {
        let _ = self.param_states.index.get(&id).expect("Invalid parameter id");
        self.handler.end_edit(id);
    }

    pub fn poll_params(&self) -> PollParams {
        PollParams {
            iter: self.param_states.dirty_editor.drain_indices(Ordering::Acquire),
            param_states: &self.param_states,
        }
    }
}

pub trait EditorContextHandler {
    fn begin_edit(&self, param_id: ParamId);
    fn perform_edit(&self, param_id: ParamId, value: f64);
    fn end_edit(&self, param_id: ParamId);
}

pub struct PollParams<'a> {
    iter: DrainIndices<'a>,
    param_states: &'a ParamStates,
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

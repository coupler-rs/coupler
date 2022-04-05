use crate::atomic::DrainIndices;
use crate::internal::param_states::*;
use crate::{param::*, plugin::*};

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::atomic::Ordering;
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

    pub fn param_list(&self) -> &ParamList {
        &self.param_states.list
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
        let index = self.param_states.list.get_param_index(key.id).expect("Invalid parameter key");
        let param_info = &self.param_states.list.params()[index];
        let param = param_info.param.downcast_ref::<P>().expect("Incorrect parameter type");

        let value_normalized = param.to_normalized(value);
        self.param_states.values[index].store(value_normalized);
        self.param_states.dirty_processor.set(index, Ordering::Release);

        self.handler.perform_edit(key.id, value_normalized);
    }

    pub fn end_edit<P: Param + 'static>(&self, key: ParamKey<P>) {
        let param_info = self.param_states.list.get_param(key.id).expect("Invalid parameter key");
        let _ = param_info.param.downcast_ref::<P>().expect("Incorrect parameter type");

        self.handler.end_edit(key.id);
    }

    #[inline]
    pub fn read_change<P: Param + 'static>(
        &self,
        key: ParamKey<P>,
        change: ParamChange,
    ) -> Option<P::Value> {
        self.param_states.list.read_change(key, change)
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
    type Item = ParamChange;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.iter.next() {
            let id = self.param_states.list.params()[index].id;
            let value = self.param_states.values[index].load();

            Some(ParamChange { id, value })
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

use crate::atomic::DrainIndices;
use crate::{param::*, plugin::*};

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use std::rc::Rc;

pub struct EditorContext<P> {
    handler: Rc<dyn EditorContextHandler<P>>,
}

impl<P> EditorContext<P> {
    pub fn new(handler: Rc<dyn EditorContextHandler<P>>) -> EditorContext<P> {
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

    pub fn poll_params(&self) -> PollParams<P> {
        self.handler.poll_params()
    }
}

pub trait EditorContextHandler<P> {
    fn begin_edit(&self, param_id: ParamId);
    fn perform_edit(&self, param_id: ParamId, value: f64);
    fn end_edit(&self, param_id: ParamId);
    fn poll_params(&self) -> PollParams<P>;
}

pub struct PollParams<'a, P> {
    pub(crate) iter: DrainIndices<'a>,
    pub(crate) param_list: &'a ParamList<P>,
}

impl<'a, P> Iterator for PollParams<'a, P> {
    type Item = ParamId;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.iter.next() {
            Some(self.param_list.params()[index].get_id())
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

pub trait Editor<P>: Sized + 'static {
    fn open(plugin: &P, context: EditorContext<P>, parent: Option<&ParentWindow>) -> Self;
    fn close(&mut self);
    fn size() -> (f64, f64);
    fn raw_window_handle(&self) -> Option<RawWindowHandle>;

    #[cfg(target_os = "linux")]
    fn file_descriptor(&self) -> Option<std::os::raw::c_int>;
    #[cfg(target_os = "linux")]
    fn poll(&mut self);
}

pub struct NoEditor;

impl<P: Plugin> Editor<P> for NoEditor {
    fn open(_plugin: &P, _context: EditorContext<P>, _parent: Option<&ParentWindow>) -> Self {
        NoEditor
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

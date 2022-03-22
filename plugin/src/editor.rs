use crate::{param::*, plugin::*};

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use std::marker::PhantomData;
use std::rc::Rc;

pub trait EditorContext {
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

    fn open(
        plugin: &Self::Plugin,
        context: &Rc<dyn EditorContext>,
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
        _plugin: &Self::Plugin,
        _context: &Rc<dyn EditorContext>,
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

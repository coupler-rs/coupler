mod atomic;
mod audio_buffers;
mod buses;
mod params;
pub mod vst3;

pub use atomic::*;
pub use audio_buffers::*;
pub use buses::*;
pub use params::*;

use std::rc::Rc;
use std::sync::Arc;

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

pub struct PluginInfo {
    pub name: String,
    pub vendor: String,
    pub url: String,
    pub email: String,
    pub has_editor: bool,
}

pub struct ParamValues<'a> {
    param_list: &'a ParamList,
    values: &'a [AtomicF64],
}

impl<'a> ParamValues<'a> {
    pub fn get_param<P: Param + 'static>(&self, key: ParamKey<P>) -> P::Value {
        let index = self.param_list.indices[&key.id()];
        let param = self.param_list.params[index].param.as_any().downcast_ref::<P>().unwrap();
        param.decode(self.values[index].load())
    }

    pub fn set_param<P: Param + 'static>(&mut self, key: ParamKey<P>, value: P::Value) {
        let index = self.param_list.indices[&key.id()];
        let param = self.param_list.params[index].param.as_any().downcast_ref::<P>().unwrap();
        self.values[index].store(param.encode(value));
    }
}

pub struct ParamChange {
    pub id: ParamId,
    pub offset: usize,
    pub value: f64,
}

pub struct ProcessContext<'a> {
    sample_rate: f64,
    input_layouts: &'a [BusLayout],
    output_layouts: &'a [BusLayout],
    param_list: &'a ParamList,
    param_values: &'a [f64],
}

impl<'a> ProcessContext<'a> {
    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }

    pub fn input_layouts(&self) -> &[BusLayout] {
        self.input_layouts
    }

    pub fn output_layouts(&self) -> &[BusLayout] {
        self.output_layouts
    }

    pub fn get_param<P: Param + 'static>(&self, key: ParamKey<P>) -> P::Value {
        let index = self.param_list.indices[&key.id()];
        let param = self.param_list.params[index].param.as_any().downcast_ref::<P>().unwrap();
        param.decode(self.param_values[index])
    }
}

trait EditorContextInner {
    fn get_param(&self, param_id: ParamId) -> f64;
    fn begin_edit(&self, param_id: ParamId);
    fn perform_edit(&self, param_id: ParamId, value: f64);
    fn end_edit(&self, param_id: ParamId);
}

#[derive(Clone)]
pub struct EditorContext {
    param_list: Arc<ParamList>,
    inner: Rc<dyn EditorContextInner>,
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

pub struct ParentWindow(RawWindowHandle);

unsafe impl HasRawWindowHandle for ParentWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.0
    }
}

pub trait Plugin: Send + Sync + Sized {
    type Processor: Processor;
    type Editor: Editor;

    fn info() -> PluginInfo;

    fn create() -> Self;
    fn processor(&self, context: &ProcessContext) -> Self::Processor;
    fn editor(&self, context: EditorContext, parent: Option<&ParentWindow>) -> Self::Editor;

    fn buses() -> BusList;
    fn supports_layout(inputs: &[BusLayout], outputs: &[BusLayout]) -> bool;

    fn params(&self) -> ParamList;
    fn serialize(&self, params: &ParamValues, write: &mut impl std::io::Write) -> Result<(), ()>;
    fn deserialize(
        &self,
        params: &mut ParamValues,
        read: &mut impl std::io::Read,
    ) -> Result<(), ()>;
}

pub trait Processor: Send + Sized {
    fn process(
        &mut self,
        context: &ProcessContext,
        buffers: &mut AudioBuffers,
        param_changes: &[ParamChange],
    );
    fn reset(&mut self, context: &ProcessContext);
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

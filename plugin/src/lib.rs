mod atomic;
mod audio_buses;
pub mod vst3;

pub use atomic::*;
pub use audio_buses::*;

use std::rc::Rc;

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

pub struct PluginInfo {
    pub name: String,
    pub vendor: String,
    pub url: String,
    pub email: String,
    pub has_editor: bool,
}

pub struct BusList {
    inputs: Vec<BusInfo>,
    outputs: Vec<BusInfo>,
}

impl BusList {
    pub fn new() -> BusList {
        BusList { inputs: Vec::new(), outputs: Vec::new() }
    }

    pub fn add_input(mut self, bus: BusInfo) -> BusList {
        self.inputs.push(bus);
        self
    }

    pub fn add_output(mut self, bus: BusInfo) -> BusList {
        self.outputs.push(bus);
        self
    }

    pub fn inputs(&self) -> &[BusInfo] {
        &self.inputs
    }

    pub fn outputs(&self) -> &[BusInfo] {
        &self.outputs
    }
}

pub struct BusInfo {
    pub name: String,
    pub default_layout: BusLayout,
}

pub type ParamId = u32;

pub struct ParamList {
    params: Vec<ParamInfo>,
}

impl ParamList {
    pub fn new() -> ParamList {
        ParamList { params: Vec::new() }
    }

    pub fn add(mut self, param: ParamInfo) -> ParamList {
        self.params.push(param);
        self
    }

    pub fn params(&self) -> &[ParamInfo] {
        &self.params
    }
}

pub struct ParamInfo {
    pub id: ParamId,
    pub name: String,
    pub label: String,
    pub steps: Option<u32>,
    pub default: f64,
    pub format: Box<dyn ParamFormat>,
}

pub trait ParamFormat: Send + Sync {
    fn display(&self, value: f64, write: &mut dyn std::fmt::Write);
    fn parse(&self, string: &str) -> Result<f64, ()>;
    fn normalize(&self, value: f64) -> f64;
    fn denormalize(&self, value: f64) -> f64;
}

pub struct ParamChange {
    pub id: ParamId,
    pub offset: usize,
    pub value: f64,
}

pub struct ProcessContext<'a> {
    pub sample_rate: f64,
    pub input_layouts: &'a [BusLayout],
    pub output_layouts: &'a [BusLayout],
}

trait EditorContextInner {
    fn begin_edit(&self, param_id: ParamId);
    fn perform_edit(&self, param_id: ParamId, value: f64);
    fn end_edit(&self, param_id: ParamId);
}

#[derive(Clone)]
pub struct EditorContext(Rc<dyn EditorContextInner>);

impl EditorContext {
    pub fn begin_edit(&self, param_id: ParamId) {
        self.0.begin_edit(param_id);
    }

    pub fn perform_edit(&self, param_id: ParamId, value: f64) {
        self.0.perform_edit(param_id, value);
    }

    pub fn end_edit(&self, param_id: ParamId) {
        self.0.end_edit(param_id);
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

    fn buses() -> BusList;
    fn supports_layout(inputs: &[BusLayout], outputs: &[BusLayout]) -> bool;

    fn create() -> Self;
    fn processor(&self, context: &ProcessContext) -> Self::Processor;
    fn editor(&self, context: EditorContext, parent: Option<&ParentWindow>) -> Self::Editor;

    fn params(&self) -> ParamList;
    fn get_param(&self, id: ParamId) -> f64;
    fn set_param(&self, id: ParamId, value: f64);

    fn serialize(&self, write: &mut impl std::io::Write) -> Result<(), ()>;
    fn deserialize(&self, read: &mut impl std::io::Read) -> Result<(), ()>;
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

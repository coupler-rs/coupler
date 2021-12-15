mod audio_buses;
pub mod vst3;

pub use audio_buses::*;

use std::rc::Rc;

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

pub struct PluginDesc {
    pub name: String,
    pub vendor: String,
    pub url: String,
    pub email: String,
    pub has_editor: bool,
}

impl Default for PluginDesc {
    fn default() -> PluginDesc {
        PluginDesc {
            name: String::new(),
            vendor: String::new(),
            url: String::new(),
            email: String::new(),
            has_editor: false,
        }
    }
}

pub type ParamId = u32;

pub struct ParamInfo {
    pub id: ParamId,
    pub name: &'static str,
    pub label: &'static str,
    pub steps: Option<u32>,
    pub default: f64,
}

pub struct ParamChange {
    pub id: ParamId,
    pub offset: usize,
    pub value: f64,
}

pub struct BusInfo {
    pub name: &'static str,
    pub default_layout: BusLayout,
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
    const PARAMS: &'static [ParamInfo];
    const INPUTS: &'static [BusInfo];
    const OUTPUTS: &'static [BusInfo];

    type Processor: Processor;
    type Editor: Editor;

    fn describe(desc: &mut PluginDesc);

    fn create() -> Self;
    fn processor(&self, context: &ProcessContext) -> Self::Processor;
    fn editor(&self, context: EditorContext, parent: Option<&ParentWindow>) -> Self::Editor;

    fn get_param(&self, id: ParamId) -> f64;
    fn set_param(&self, id: ParamId, value: f64);
    fn display_param(&self, id: ParamId, value: f64, write: &mut impl std::fmt::Write);
    fn parse_param(&self, id: ParamId, string: &str) -> Result<f64, ()>;
    fn normalize_param(&self, id: ParamId, value: f64) -> f64;
    fn denormalize_param(&self, id: ParamId, value: f64) -> f64;

    fn serialize(&self, write: &mut impl std::io::Write) -> Result<(), ()>;
    fn deserialize(&self, read: &mut impl std::io::Read) -> Result<(), ()>;

    fn supports_layout(inputs: &[BusLayout], outputs: &[BusLayout]) -> bool;
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

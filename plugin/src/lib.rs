mod audio_buses;
pub mod vst3;

pub use audio_buses::*;

use std::rc::Rc;

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

pub struct PluginInfo {
    pub name: &'static str,
    pub vendor: &'static str,
    pub url: &'static str,
    pub email: &'static str,
    pub has_editor: bool,
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
    const INFO: PluginInfo;
    const PARAMS: &'static [ParamInfo];
    const INPUTS: &'static [BusInfo];
    const OUTPUTS: &'static [BusInfo];

    type Processor: Processor;
    type Editor: Editor;

    fn create() -> Self;
    fn processor(&self) -> Self::Processor;
    fn editor(&self, context: EditorContext) -> Self::Editor;

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
    fn process(&mut self, audio_buses: &mut AudioBuses, param_changes: &[ParamChange]);
}

pub trait Editor: Sized {
    fn size(&self) -> (f64, f64);
    fn open(&mut self, parent: Option<&ParentWindow>);
    fn close(&mut self);
    fn raw_window_handle(&self) -> Option<RawWindowHandle>;

    #[cfg(target_os = "linux")]
    fn file_descriptor(&self) -> Option<std::os::raw::c_int>;
    #[cfg(target_os = "linux")]
    fn poll(&mut self);
}

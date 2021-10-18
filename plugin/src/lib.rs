pub mod vst2;
pub mod vst3;

use std::rc::Rc;

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

pub struct PluginInfo {
    pub name: &'static str,
    pub vendor: &'static str,
    pub url: &'static str,
    pub email: &'static str,
    pub unique_id: [u8; 4],
    pub uid: [u32; 4],
    pub has_editor: bool,
}

pub type ParamId = u32;

pub struct ParamInfo {
    pub id: ParamId,
    pub name: &'static str,
    pub label: &'static str,
    pub steps: Option<u32>,
    pub default: f64,
    pub to_normal: fn(f64) -> f64,
    pub from_normal: fn(f64) -> f64,
    pub to_string: fn(f64) -> String,
    pub from_string: fn(&str) -> f64,
}

pub struct ParamChange {
    pub id: ParamId,
    pub offset: usize,
    pub value: f64,
}

trait EditorContextInner {
    fn begin_edit(&self, param_id: ParamId);
    fn perform_edit(&self, param_id: ParamId, value: f64);
    fn end_edit(&self, param_id: ParamId);
}

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

    type Processor: Processor;
    type Editor: Editor;

    fn create() -> Self;

    fn get_param(&self, id: ParamId) -> f64;
    fn set_param(&self, id: ParamId, value: f64);

    fn processor(&self) -> Self::Processor;

    fn editor_size(&self) -> (f64, f64);
    fn editor(&self, editor_context: EditorContext, parent: Option<&ParentWindow>) -> Self::Editor;
}

pub trait Processor: Send + Sized {
    fn process(
        &mut self,
        inputs: &[&[f32]],
        outputs: &mut [&mut [f32]],
        param_changes: &[ParamChange],
    );
}

pub trait Editor: Sized {
    fn poll(&mut self);
    fn raw_window_handle(&self) -> Option<RawWindowHandle>;
    fn file_descriptor(&self) -> Option<std::os::raw::c_int>;
}

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

pub struct ParamInfo {
    pub id: u32,
    pub name: &'static str,
    pub label: &'static str,
    pub steps: Option<u32>,
    pub default: f64,
    pub to_normal: fn(f64) -> f64,
    pub from_normal: fn(f64) -> f64,
    pub to_string: fn(f64) -> String,
    pub from_string: fn(&str) -> f64,
}

#[allow(unused_variables)]
pub trait Plugin: Send + Sized {
    const INFO: PluginInfo;
    const PARAMS: &'static [ParamInfo];

    type Editor: Editor;

    fn create(editor_context: Rc<dyn EditorContext>) -> (Self, Self::Editor);
    fn destroy(&mut self, editor: &mut Self::Editor) {}
    fn process(&mut self, params: &dyn ParamValues, inputs: &[&[f32]], outputs: &mut [&mut [f32]]) {
    }
}

#[allow(unused_variables)]
pub trait Editor: Sized {
    fn size(&self) -> (f64, f64) {
        (0.0, 0.0)
    }
    fn open(&mut self, parent: Option<&ParentWindow>) {}
    fn close(&mut self) {}
    fn poll(&mut self) {}
    fn raw_window_handle(&self) -> Option<RawWindowHandle> {
        None
    }
    fn file_descriptor(&self) -> Option<std::os::raw::c_int> {
        None
    }
    fn param_change(&mut self, param_info: &ParamInfo, value: f64) {}
}

pub trait EditorContext {
    fn get(&self, param_info: &ParamInfo) -> f64;
    fn set(&self, param_info: &ParamInfo, value: f64);
    fn begin_edit(&self, param_info: &ParamInfo);
    fn end_edit(&self, param_info: &ParamInfo);
}

pub trait ParamValues {
    fn get(&self, param_info: &ParamInfo) -> f64;
}

pub struct ParentWindow(RawWindowHandle);

unsafe impl HasRawWindowHandle for ParentWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.0
    }
}

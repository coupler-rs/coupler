use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

pub mod vst2;
pub mod vst3;

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

pub trait Plugin: Send + Sync + Sized {
    const INFO: PluginInfo;
    const PARAMS: &'static [ParamInfo];

    type Processor: Processor;
    type Editor: Editor;

    fn create() -> (Self, Self::Processor, Self::Editor);
    fn destroy(&mut self, processor: &mut Self::Processor, editor: &mut Self::Editor) {}
}

pub trait Processor: Send + Sized {
    fn process(&mut self, params: &Params, inputs: &[&[f32]], outputs: &mut [&mut [f32]]) {}
}

pub trait Editor: Sized {
    fn size(&self) -> (f64, f64) {
        (0.0, 0.0)
    }
    fn open(&mut self, parent: Option<ParentWindow>) {}
    fn close(&mut self) {}
    fn poll(&mut self) {}
    fn raw_window_handle(&self) -> Option<RawWindowHandle> {
        None
    }
    fn file_descriptor(&self) -> Option<i32> {
        None
    }
}

pub struct Params<'a> {
    inner: &'a dyn ParamsInner,
}

trait ParamsInner {
    fn get(&self, param: &ParamInfo) -> f64;
}

impl<'a> Params<'a> {
    pub fn get(&self, param: &ParamInfo) -> f64 {
        self.inner.get(param)
    }
}

pub struct ParentWindow(RawWindowHandle);

unsafe impl HasRawWindowHandle for ParentWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.0
    }
}

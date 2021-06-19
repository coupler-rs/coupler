pub mod vst2;
pub mod vst3;

pub struct PluginInfo {
    pub name: &'static str,
    pub vendor: &'static str,
    pub url: &'static str,
    pub email: &'static str,
    pub unique_id: [u8; 4],
    pub uid: [u32; 4],
}

pub struct Param {
    pub id: usize,
    pub name: &'static str,
    pub label: &'static str,
}

pub struct Params<'a> {
    inner: &'a dyn ParamsInner,
}

trait ParamsInner {
    fn get(&self, param: &Param) -> f64;
    fn set(&self, param: &Param, value: f64);
}

impl<'a> Params<'a> {
    pub fn get(&self, param: &Param) -> f64 {
        self.inner.get(param)
    }

    pub fn set(&self, param: &Param, value: f64) {
        self.inner.set(param, value);
    }
}

pub trait Plugin: Send + Sync + Sized {
    const INFO: PluginInfo;
    const PARAMS: &'static [&'static Param];

    type Processor: Processor;
    type Editor: Editor;

    fn create() -> (Self, Self::Processor, Self::Editor);
}

pub trait Processor: Send + Sized {
    fn process(&mut self, params: &Params, inputs: &[&[f32]], outputs: &mut [&mut [f32]]);
}

pub trait Editor: Sized {}

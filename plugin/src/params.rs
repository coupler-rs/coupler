use std::collections::HashMap;

use crate::atomic::AtomicF64;

pub type ParamId = u32;

pub struct ParamInfo {
    pub id: ParamId,
    pub name: String,
    pub units: String,
    pub default: f64,
    pub format: Box<dyn ParamFormat>,
}

pub trait ParamFormat {
    fn steps(&self) -> Option<usize>;
    fn map(&self, value: f64) -> f64;
    fn unmap(&self, value: f64) -> f64;
    fn display(&self, value: f64, write: &mut dyn std::fmt::Write);
    fn parse(&self, string: &str) -> Result<f64, ()>;
}

pub struct ParamList {
    pub(crate) indices: HashMap<ParamId, usize>,
    pub(crate) params: Vec<ParamInfo>,
}

impl ParamList {
    pub fn new() -> ParamList {
        ParamList { indices: HashMap::new(), params: Vec::new() }
    }

    pub fn add(mut self, param: ParamInfo) -> ParamList {
        self.indices.insert(param.id, self.params.len());
        self.params.push(param);
        self
    }
}

pub struct ParamValues<'a> {
    pub(crate) param_list: &'a ParamList,
    pub(crate) values: &'a [AtomicF64],
}

impl<'a> ParamValues<'a> {
    pub fn get_param(&self, id: ParamId) -> f64 {
        self.values[self.param_list.indices[&id]].load()
    }

    pub fn set_param(&mut self, id: ParamId, value: f64) {
        self.values[self.param_list.indices[&id]].store(value)
    }
}

pub struct FloatParam {
    min: f64,
    max: f64,
}

impl FloatParam {
    pub fn new(min: f64, max: f64) -> FloatParam {
        FloatParam { min, max }
    }
}

impl ParamFormat for FloatParam {
    fn steps(&self) -> Option<usize> {
        None
    }

    fn map(&self, value: f64) -> f64 {
        self.min + (self.max - self.min) * value
    }

    fn unmap(&self, value: f64) -> f64 {
        (value - self.min) / (self.max - self.min)
    }

    fn display(&self, value: f64, write: &mut dyn std::fmt::Write) {
        let _ = write!(write, "{}", value);
    }

    fn parse(&self, string: &str) -> Result<f64, ()> {
        string.parse().map_err(|_| ())
    }
}

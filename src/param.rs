use std::fmt::Write;

use crate::{ParamId, ParamValue};

pub struct ParamInfo {
    pub id: ParamId,
    pub name: String,
    pub default: ParamValue,
    pub range: Range,
    pub display: Box<dyn Display + Send + Sync>,
}

pub enum Range {
    Continuous { min: f64, max: f64 },
    Discrete { steps: u64 },
}

pub trait Display {
    fn parse(&self, string: &str) -> Option<ParamValue>;
    fn display(&self, value: ParamValue, output: &mut dyn Write);
}

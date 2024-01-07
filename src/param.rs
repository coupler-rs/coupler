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
    Discrete { min: i32, max: i32 },
}

pub trait Display {
    fn parse(&self, string: &str) -> Option<ParamValue>;
    fn display(&self, value: ParamValue, output: &mut dyn Write);
}

pub struct Float;

impl Display for Float {
    fn parse(&self, string: &str) -> Option<ParamValue> {
        string.parse().ok()
    }

    fn display(&self, value: ParamValue, output: &mut dyn Write) {
        let _ = write!(output, "{:.2}", value);
    }
}

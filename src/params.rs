use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

#[cfg(feature = "derive")]
pub use coupler_derive::{Enum, Params};

mod range;

pub use range::{Encode, Log, Range};

pub type ParamId = u32;
pub type ParamValue = f64;

pub struct ParamInfo {
    pub id: ParamId,
    pub name: String,
    pub default: ParamValue,
    pub steps: Option<u32>,
}

pub trait Params {
    fn params() -> Vec<ParamInfo>;
    fn set_param(&mut self, id: ParamId, value: ParamValue);
    fn get_param(&self, id: ParamId) -> ParamValue;
    fn parse_param(&self, id: ParamId, text: &str) -> Option<ParamValue>;
    fn display_param(
        &self,
        id: ParamId,
        value: ParamValue,
        fmt: &mut Formatter,
    ) -> Result<(), fmt::Error>;
}

pub trait Enum: Encode + FromStr + Display {}

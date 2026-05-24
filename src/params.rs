use std::fmt::{self, Display};
use std::str::FromStr;

#[cfg(feature = "derive")]
pub use coupler_derive::{Enum, Params};

mod format;
mod range;

pub use format::{DefaultFormat, Format};
pub use range::{DefaultRange, Encode, Log, Range};

pub type ParamId = u32;
pub type ParamValue = f64;

pub struct ParamInfo<'a> {
    pub id: ParamId,
    pub name: &'a str,
    pub default: ParamValue,
    pub steps: Option<u32>,
}

pub trait BuildParams {
    fn param(self, info: ParamInfo) -> Self;
}

pub trait Params {
    fn params(&self, build: impl BuildParams);
    fn set_param(&mut self, id: ParamId, value: ParamValue);
    fn get_param(&self, id: ParamId) -> ParamValue;
    fn parse_param(&self, id: ParamId, text: &str) -> Option<ParamValue>;
    fn display_param(
        &self,
        id: ParamId,
        value: ParamValue,
        write: impl fmt::Write,
    ) -> Result<(), fmt::Error>;
}

pub trait Enum: Encode + FromStr + Display {}

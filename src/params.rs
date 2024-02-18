use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

#[cfg(feature = "derive")]
pub use coupler_derive::{Enum, Params};

mod range;

pub use range::{Encode, Log, Range};

pub type ParamId = u32;
pub type ParamValue = f64;

pub type ParseFn = dyn Fn(&str) -> Option<ParamValue> + Send + Sync;
pub type DisplayFn = dyn Fn(ParamValue, &mut Formatter) -> Result<(), fmt::Error> + Send + Sync;

pub struct ParamInfo {
    pub id: ParamId,
    pub name: String,
    pub default: ParamValue,
    pub steps: Option<u32>,
    pub parse: Box<ParseFn>,
    pub display: Box<DisplayFn>,
}

pub trait Params {
    fn params() -> Vec<ParamInfo>;
    fn set_param(&mut self, id: ParamId, value: ParamValue);
    fn get_param(&self, id: ParamId) -> ParamValue;
}

pub trait Enum: Encode + FromStr + Display {}

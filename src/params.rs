use std::fmt::{self, Display};
use std::str::FromStr;

#[cfg(feature = "derive")]
pub use coupler_derive::{Enum, Params};

mod format;
mod range;

pub use format::{DefaultFormat, Format};
pub use range::{DefaultRange, Encode, Log, Range};

pub struct ParamInfo<'a> {
    pub id: u32,
    pub name: &'a str,
    pub default: f64,
    pub steps: Option<u32>,
}

pub trait BuildParams {
    fn param(self, info: ParamInfo) -> Self;
}

pub trait Params {
    fn params(&self, build: impl BuildParams);
    fn set_param(&mut self, index: usize, value: f64);
    fn get_param(&self, index: usize) -> f64;
    fn parse_param(&self, index: usize, text: &str) -> Option<f64>;
    fn display_param(
        &self,
        index: usize,
        value: f64,
        write: impl fmt::Write,
    ) -> Result<(), fmt::Error>;
}

pub trait Enum: Encode + FromStr + Display {}

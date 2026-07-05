use std::fmt::{self, Display};
use std::str::FromStr;

use crate::plugin::Plugin;

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

pub(crate) struct OwnedParamInfo {
    pub id: u32,
    pub name: String,
    pub default: f64,
    pub steps: Option<u32>,
}

pub(crate) fn collect_params<P: Plugin>(plugin: &P) -> Vec<OwnedParamInfo> {
    struct CollectParams<'a>(&'a mut Vec<OwnedParamInfo>);

    impl<'a> BuildParams for CollectParams<'a> {
        fn param(self, param: ParamInfo) -> Self {
            self.0.push(OwnedParamInfo {
                id: param.id,
                name: param.name.to_string(),
                default: param.default,
                steps: param.steps,
            });
            self
        }
    }

    let mut params = Vec::new();
    plugin.params(CollectParams(&mut params));
    params
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

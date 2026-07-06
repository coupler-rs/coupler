use std::fmt::{self, Display};
use std::str::FromStr;

use crate::key::{Key, KeyList};
use crate::plugin::Plugin;

#[cfg(feature = "derive")]
pub use coupler_derive::{Enum, Params};

mod format;
mod range;

pub use format::{DefaultFormat, Format};
pub use range::{DefaultRange, Encode, Log, Range};

pub struct ParamInfo<'a> {
    pub name: &'a str,
    pub default: f64,
    pub steps: Option<u32>,
}

pub trait BuildParams {
    fn param<'k>(self, key: impl Into<Key<'k>>, param: ParamInfo) -> Self;
    fn reserve<'k>(self, key: impl Into<Key<'k>>) -> Self;
}

pub(crate) struct OwnedParamInfo {
    pub name: String,
    pub default: f64,
    pub steps: Option<u32>,
}

pub(crate) fn collect_params<P: Plugin>(plugin: &P) -> (Vec<u32>, Vec<OwnedParamInfo>) {
    struct CollectParams<'a> {
        keys: &'a mut KeyList,
        params: &'a mut Vec<OwnedParamInfo>,
    }

    impl<'a> BuildParams for CollectParams<'a> {
        fn param<'k>(self, key: impl Into<Key<'k>>, param: ParamInfo) -> Self {
            self.keys.key(key);
            self.params.push(OwnedParamInfo {
                name: param.name.to_string(),
                default: param.default,
                steps: param.steps,
            });
            self
        }

        fn reserve<'k>(self, key: impl Into<Key<'k>>) -> Self {
            self.keys.reserve(key);
            self
        }
    }

    let mut keys = KeyList::new();
    let mut params = Vec::new();
    plugin.params(CollectParams {
        keys: &mut keys,
        params: &mut params,
    });

    (keys.into_ids(), params)
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

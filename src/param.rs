use std::fmt::Write;

use crate::{ParamId, ParamValue};

type ParseFn = dyn Fn(&str) -> Option<ParamValue> + Send + Sync;
type DisplayFn = dyn Fn(ParamValue, &mut dyn Write) + Send + Sync;

pub struct ParamInfo {
    pub id: ParamId,
    pub name: String,
    pub default: ParamValue,
    pub steps: Option<u32>,
    pub parse: Box<ParseFn>,
    pub display: Box<DisplayFn>,
}

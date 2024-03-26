use crate::host::HostInner;
use crate::params::{ParamId, ParamValue};

pub struct Vst3Host {}

impl HostInner for Vst3Host {
    fn begin_gesture(&self, _id: ParamId) {}
    fn end_gesture(&self, _id: ParamId) {}
    fn set_param(&self, _id: ParamId, _value: ParamValue) {}
}

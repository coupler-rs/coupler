use crate::host::HostInner;
use crate::params::{ParamId, ParamValue};

pub struct Vst3Host {}

impl HostInner for Vst3Host {
    fn begin_gesture(&self, id: ParamId) {}
    fn end_gesture(&self, id: ParamId) {}
    fn set_param(&self, id: ParamId, value: ParamValue) {}
}

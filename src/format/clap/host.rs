use crate::host::HostInner;
use crate::params::{ParamId, ParamValue};

pub struct ClapHost {}

impl HostInner for ClapHost {
    fn begin_gesture(&self, _id: ParamId) {}
    fn end_gesture(&self, _id: ParamId) {}
    fn set_param(&self, _id: ParamId, _value: ParamValue) {}
}

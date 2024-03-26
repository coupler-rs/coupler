use crate::host::HostInner;
use crate::params::{ParamId, ParamValue};

pub struct ClapHost {}

impl HostInner for ClapHost {
    fn begin_gesture(&self, id: ParamId) {}
    fn end_gesture(&self, id: ParamId) {}
    fn set_param(&self, id: ParamId, value: ParamValue) {}
}

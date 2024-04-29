use std::sync::Arc;

use crate::params::{ParamId, ParamValue};

pub trait HostInner {
    fn begin_gesture(&self, id: ParamId);
    fn end_gesture(&self, id: ParamId);
    fn set_param(&self, id: ParamId, value: ParamValue);
}

#[derive(Clone)]
pub struct Host {
    inner: Arc<dyn HostInner + Send + Sync>,
}

impl Host {
    pub fn from_inner(inner: Arc<dyn HostInner + Send + Sync>) -> Host {
        Host { inner }
    }

    pub fn begin_gesture(&self, id: ParamId) {
        self.inner.begin_gesture(id);
    }

    pub fn end_gesture(&self, id: ParamId) {
        self.inner.end_gesture(id);
    }

    pub fn set_param(&self, id: ParamId, value: ParamValue) {
        self.inner.set_param(id, value);
    }
}

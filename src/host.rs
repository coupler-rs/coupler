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

    pub fn edit_param(&self, id: ParamId) -> Gesture {
        self.inner.begin_gesture(id);

        Gesture {
            inner: Arc::clone(&self.inner),
            id,
        }
    }
}

pub struct Gesture {
    inner: Arc<dyn HostInner + Send + Sync>,
    id: ParamId,
}

impl Gesture {
    pub fn set_value(&self, value: ParamValue) {
        self.inner.set_param(self.id, value);
    }
}

impl Drop for Gesture {
    fn drop(&mut self) {
        self.inner.end_gesture(self.id);
    }
}

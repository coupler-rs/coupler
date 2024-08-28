use std::sync::Arc;

pub trait HostInner {}

#[derive(Clone)]
pub struct Host {
    _inner: Arc<dyn HostInner + Send + Sync>,
}

impl Host {
    pub fn from_inner(inner: Arc<dyn HostInner + Send + Sync>) -> Host {
        Host { _inner: inner }
    }
}

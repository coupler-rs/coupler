use crate::host::HostInner;

pub struct Vst3Host {}

impl Vst3Host {
    pub fn new() -> Vst3Host {
        Vst3Host {}
    }
}

impl HostInner for Vst3Host {}

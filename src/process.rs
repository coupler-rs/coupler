use crate::bus::Layout;

pub struct ProcessInfo {
    pub latency: u64,
}

pub struct Config {
    pub layout: Layout,
    pub sample_rate: f64,
}

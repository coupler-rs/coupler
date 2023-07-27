use std::sync::atomic::{AtomicU64, Ordering};

pub struct AtomicF64(AtomicU64);

impl AtomicF64 {
    pub fn new(value: f64) -> AtomicF64 {
        AtomicF64(AtomicU64::new(value.to_bits()))
    }

    pub fn load(&self, ordering: Ordering) -> f64 {
        f64::from_bits(self.0.load(ordering))
    }

    pub fn store(&self, value: f64, ordering: Ordering) {
        self.0.store(value.to_bits(), ordering)
    }
}

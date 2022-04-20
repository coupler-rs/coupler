use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

pub struct AtomicF64(AtomicU64);

impl AtomicF64 {
    pub fn new(value: f64) -> AtomicF64 {
        AtomicF64(AtomicU64::new(value.to_bits()))
    }

    pub fn load(&self) -> f64 {
        f64::from_bits(self.0.load(Ordering::Relaxed))
    }

    pub fn store(&self, value: f64) {
        self.0.store(value.to_bits(), Ordering::Relaxed)
    }
}

pub struct AtomicF32(AtomicU32);

impl AtomicF32 {
    pub fn new(value: f32) -> AtomicF32 {
        AtomicF32(AtomicU32::new(value.to_bits()))
    }

    pub fn load(&self) -> f32 {
        f32::from_bits(self.0.load(Ordering::Relaxed))
    }

    pub fn store(&self, value: f32) {
        self.0.store(value.to_bits(), Ordering::Relaxed)
    }
}

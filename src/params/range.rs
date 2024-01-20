use super::ParamValue;

pub trait Range<T> {
    fn steps(&self) -> Option<u32>;
    fn encode(&self, value: T) -> ParamValue;
    fn decode(&self, value: ParamValue) -> T;
}

pub trait DefaultRange: Sized {
    type Range: Range<Self>;

    fn default_range() -> Self::Range;
}

impl Range<f32> for std::ops::Range<f32> {
    fn steps(&self) -> Option<u32> {
        None
    }

    fn encode(&self, value: f32) -> ParamValue {
        ((value - self.start) / (self.end - self.start)) as f64
    }

    fn decode(&self, value: ParamValue) -> f32 {
        (1.0 - value as f32) * self.start + value as f32 * self.end
    }
}

impl Range<f64> for std::ops::Range<f64> {
    fn steps(&self) -> Option<u32> {
        None
    }

    fn encode(&self, value: f64) -> ParamValue {
        ((value - self.start) / (self.end - self.start)) as f64
    }

    fn decode(&self, value: ParamValue) -> f64 {
        (1.0 - value as f64) * self.start + value as f64 * self.end
    }
}

impl DefaultRange for f32 {
    type Range = std::ops::Range<f32>;

    fn default_range() -> Self::Range {
        0.0..1.0
    }
}

impl DefaultRange for f64 {
    type Range = std::ops::Range<f64>;

    fn default_range() -> Self::Range {
        0.0..1.0
    }
}

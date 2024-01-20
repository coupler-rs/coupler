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

macro_rules! float_range {
    ($float:ty) => {
        impl Range<$float> for std::ops::Range<$float> {
            #[inline]
            fn steps(&self) -> Option<u32> {
                None
            }

            #[inline]
            fn encode(&self, value: $float) -> ParamValue {
                ((value - self.start) / (self.end - self.start)) as f64
            }

            #[inline]
            fn decode(&self, value: ParamValue) -> $float {
                (1.0 - value as $float) * self.start + value as $float * self.end
            }
        }

        impl Range<$float> for std::ops::RangeInclusive<$float> {
            #[inline]
            fn steps(&self) -> Option<u32> {
                None
            }

            #[inline]
            fn encode(&self, value: $float) -> ParamValue {
                ((value - self.start()) / (self.end() - self.start())) as f64
            }

            #[inline]
            fn decode(&self, value: ParamValue) -> $float {
                (1.0 - value as $float) * self.start() + value as $float * self.end()
            }
        }

        impl DefaultRange for $float {
            type Range = std::ops::Range<$float>;

            #[inline]
            fn default_range() -> Self::Range {
                0.0..1.0
            }
        }
    };
}

float_range!(f32);
float_range!(f64);

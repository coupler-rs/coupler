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

macro_rules! int_range {
    ($int:ty) => {
        impl Range<$int> for std::ops::Range<$int> {
            #[inline]
            fn steps(&self) -> Option<u32> {
                Some(self.end.abs_diff(self.start) as u32)
            }

            #[inline]
            fn encode(&self, value: $int) -> ParamValue {
                let steps_recip = 1.0 / (self.end as f64 - self.start as f64);
                (value as f64 - self.start as f64 + 0.5) * steps_recip
            }

            #[inline]
            fn decode(&self, value: ParamValue) -> $int {
                let steps = self.end as f64 - self.start as f64;
                (self.start as f64 + value * steps) as $int
            }
        }

        impl Range<$int> for std::ops::RangeInclusive<$int> {
            #[inline]
            fn steps(&self) -> Option<u32> {
                Some(self.end().abs_diff(*self.start()).saturating_add(1) as u32)
            }

            #[inline]
            fn encode(&self, value: $int) -> ParamValue {
                let steps_recip = 1.0 / (*self.end() as f64 + 1.0 - *self.start() as f64);
                (value as f64 - *self.start() as f64 + 0.5) * steps_recip
            }

            #[inline]
            fn decode(&self, value: ParamValue) -> $int {
                let steps = *self.end() as f64 + 1.0 - *self.start() as f64;
                (*self.start() as f64 + value * steps) as $int
            }
        }

        impl DefaultRange for $int {
            type Range = std::ops::Range<$int>;

            #[inline]
            fn default_range() -> Self::Range {
                0..1
            }
        }
    };
}

int_range!(u8);
int_range!(u16);
int_range!(u32);
int_range!(u64);

int_range!(i8);
int_range!(i16);
int_range!(i32);
int_range!(i64);

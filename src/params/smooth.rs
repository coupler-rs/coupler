use super::{ParamId, ParamValue, Params};

#[cfg(feature = "derive")]
pub use coupler_derive::Smooth;

pub trait Smooth: Params {
    type Smoothed: SmoothParams;

    fn smoothed(&self, sample_rate: f64) -> Self::Smoothed;
}

pub type Smoothed<P> = <P as Smooth>::Smoothed;

pub trait SmoothParams {
    fn set_param(&mut self, id: ParamId, value: ParamValue);
    fn reset(&mut self);
}

pub trait BuildSmoother<T> {
    type Smoother: Smoother<T>;

    fn build(self, value: T, sample_rate: f64) -> Self::Smoother;
}

pub trait Smoother<T> {
    type Value;

    fn reset(&mut self);
    fn set(&mut self, value: T);
    fn get(&self) -> Self::Value;
    fn next(&mut self) -> Self::Value;

    #[inline]
    fn fill(&mut self, slice: &mut [Self::Value]) {
        for out in slice {
            *out = self.next();
        }
    }

    #[inline]
    fn is_active(&self) -> bool {
        true
    }
}

const EPSILON: f64 = 1e-3;

pub struct Exp {
    ms: f64,
}

impl Default for Exp {
    fn default() -> Exp {
        Exp { ms: 10.0 }
    }
}

impl Exp {
    pub fn ms(self, ms: f64) -> Self {
        Exp { ms, ..self }
    }
}

pub struct ExpSmoother<T> {
    rate: T,
    current: T,
    target: T,
}

macro_rules! impl_exp {
    ($float:ty) => {
        impl BuildSmoother<$float> for Exp {
            type Smoother = ExpSmoother<$float>;

            #[inline]
            fn build(self, value: $float, sample_rate: f64) -> Self::Smoother {
                let dt = 1000.0 / sample_rate;
                let rate = 1.0 - (-dt / self.ms).exp();

                ExpSmoother {
                    rate: rate as $float,
                    current: value,
                    target: value,
                }
            }
        }

        impl Smoother<$float> for ExpSmoother<$float> {
            type Value = $float;

            #[inline]
            fn reset(&mut self) {
                self.current = self.target;
            }

            #[inline]
            fn set(&mut self, value: $float) {
                self.target = value;
            }

            #[inline]
            fn get(&self) -> Self::Value {
                self.current
            }

            #[inline]
            fn next(&mut self) -> Self::Value {
                if (self.target - self.current).abs() > EPSILON as $float {
                    self.current = (1.0 - self.rate) * self.current + self.rate * self.target;
                } else {
                    self.current = self.target;
                }

                self.current
            }

            #[inline]
            fn is_active(&self) -> bool {
                self.current != self.target
            }
        }
    };
}

impl_exp!(f32);
impl_exp!(f64);

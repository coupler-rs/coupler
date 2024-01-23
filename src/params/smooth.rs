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

pub trait Smoother<T> {
    type Args;
    type Value;

    fn build(value: T, args: Self::Args, sample_rate: f64) -> Self;
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

pub struct ExpArgs<T> {
    pub time_constant: T,
}

impl<T> From<T> for ExpArgs<T> {
    #[inline]
    fn from(value: T) -> ExpArgs<T> {
        ExpArgs {
            time_constant: value,
        }
    }
}

pub struct Exp<T> {
    rate: T,
    current: T,
    target: T,
}

macro_rules! impl_exp {
    ($float:ty) => {
        impl Smoother<$float> for Exp<$float> {
            type Args = ExpArgs<$float>;
            type Value = $float;

            #[inline]
            fn build(value: $float, args: Self::Args, sample_rate: f64) -> Self {
                let dt = 1000.0 / sample_rate as $float;
                let rate = 1.0 - (-dt / args.time_constant).exp();

                Exp {
                    rate,
                    current: value,
                    target: value,
                }
            }

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

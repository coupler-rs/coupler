#![cfg(target_feature = "sse2")]

use std::arch::x86_64;
use std::mem;
use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub struct f32x4(x86_64::__m128);

impl f32x4 {
    #[inline]
    pub fn new(a: f32, b: f32, c: f32, d: f32) -> f32x4 {
        #[repr(C, align(16))]
        struct Align([f32; 4]);

        let values = Align([a, b, c, d]);
        unsafe { f32x4(x86_64::_mm_load_ps(values.0.as_ptr())) }
    }

    #[inline]
    pub fn splat(value: f32) -> f32x4 {
        unsafe { f32x4(x86_64::_mm_set1_ps(value)) }
    }

    #[inline]
    pub fn from_slice(slice: &[f32]) -> f32x4 {
        assert_eq!(slice.len(), 4);
        unsafe { f32x4(x86_64::_mm_loadu_ps(slice.as_ptr())) }
    }

    #[inline]
    pub fn write_to_slice(&self, slice: &mut [f32]) {
        assert_eq!(slice.len(), 4);
        unsafe {
            x86_64::_mm_storeu_ps(slice.as_mut_ptr(), self.0);
        }
    }

    #[inline]
    pub fn shuffle<const MASK: i32>(&self) -> f32x4 {
        unsafe { f32x4(x86_64::_mm_shuffle_ps::<MASK>(self.0, self.0)) }
    }

    #[inline]
    pub fn min(&self, other: f32x4) -> f32x4 {
        unsafe { f32x4(x86_64::_mm_min_ps(self.0, other.0)) }
    }

    #[inline]
    pub fn max(&self, other: f32x4) -> f32x4 {
        unsafe { f32x4(x86_64::_mm_max_ps(self.0, other.0)) }
    }
}

impl Index<usize> for f32x4 {
    type Output = f32;

    #[inline]
    fn index(&self, index: usize) -> &f32 {
        unsafe { &mem::transmute::<&x86_64::__m128, &[f32; 4]>(&self.0)[index] }
    }
}

impl IndexMut<usize> for f32x4 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut f32 {
        unsafe { &mut mem::transmute::<&mut x86_64::__m128, &mut [f32; 4]>(&mut self.0)[index] }
    }
}

impl Add<f32x4> for f32x4 {
    type Output = f32x4;

    #[inline]
    fn add(self, other: f32x4) -> f32x4 {
        unsafe { f32x4(x86_64::_mm_add_ps(self.0, other.0)) }
    }
}

impl AddAssign<f32x4> for f32x4 {
    #[inline]
    fn add_assign(&mut self, other: f32x4) {
        *self = *self + other;
    }
}

impl Sub<f32x4> for f32x4 {
    type Output = f32x4;

    #[inline]
    fn sub(self, other: f32x4) -> f32x4 {
        unsafe { f32x4(x86_64::_mm_sub_ps(self.0, other.0)) }
    }
}

impl SubAssign<f32x4> for f32x4 {
    #[inline]
    fn sub_assign(&mut self, other: f32x4) {
        *self = *self - other;
    }
}

impl Mul<f32x4> for f32x4 {
    type Output = f32x4;

    #[inline]
    fn mul(self, other: f32x4) -> f32x4 {
        unsafe { f32x4(x86_64::_mm_mul_ps(self.0, other.0)) }
    }
}

impl MulAssign<f32x4> for f32x4 {
    #[inline]
    fn mul_assign(&mut self, other: f32x4) {
        *self = *self * other;
    }
}

impl Div<f32x4> for f32x4 {
    type Output = f32x4;

    #[inline]
    fn div(self, other: f32x4) -> f32x4 {
        unsafe { f32x4(x86_64::_mm_div_ps(self.0, other.0)) }
    }
}

impl DivAssign<f32x4> for f32x4 {
    #[inline]
    fn div_assign(&mut self, other: f32x4) {
        *self = *self / other;
    }
}

impl Neg for f32x4 {
    type Output = f32x4;

    #[inline]
    fn neg(self) -> f32x4 {
        f32x4::splat(0.0) - self
    }
}

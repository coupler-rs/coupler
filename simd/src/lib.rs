#![cfg(target_feature = "sse2")]

use std::arch::x86_64;
use std::mem;
use std::ops::*;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
#[repr(transparent)]
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
    pub fn select(mask: m32x4, a: f32x4, b: f32x4) -> f32x4 {
        unsafe {
            let mask = x86_64::_mm_castsi128_ps(mask.0);
            f32x4(x86_64::_mm_or_ps(
                x86_64::_mm_and_ps(mask, a.0),
                x86_64::_mm_andnot_ps(mask, b.0),
            ))
        }
    }

    #[inline]
    pub fn eq(&self, other: f32x4) -> m32x4 {
        unsafe { m32x4(x86_64::_mm_castps_si128(x86_64::_mm_cmpeq_ps(self.0, other.0))) }
    }

    #[inline]
    pub fn ne(&self, other: f32x4) -> m32x4 {
        unsafe { m32x4(x86_64::_mm_castps_si128(x86_64::_mm_cmpneq_ps(self.0, other.0))) }
    }

    #[inline]
    pub fn lt(&self, other: f32x4) -> m32x4 {
        unsafe { m32x4(x86_64::_mm_castps_si128(x86_64::_mm_cmplt_ps(self.0, other.0))) }
    }

    #[inline]
    pub fn gt(&self, other: f32x4) -> m32x4 {
        unsafe { m32x4(x86_64::_mm_castps_si128(x86_64::_mm_cmpgt_ps(self.0, other.0))) }
    }

    #[inline]
    pub fn le(&self, other: f32x4) -> m32x4 {
        unsafe { m32x4(x86_64::_mm_castps_si128(x86_64::_mm_cmple_ps(self.0, other.0))) }
    }

    #[inline]
    pub fn ge(&self, other: f32x4) -> m32x4 {
        unsafe { m32x4(x86_64::_mm_castps_si128(x86_64::_mm_cmpge_ps(self.0, other.0))) }
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

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct m32(i32);

impl m32 {
    pub const FALSE: m32 = m32(0);
    pub const TRUE: m32 = m32(!0);
}

impl From<bool> for m32 {
    fn from(value: bool) -> m32 {
        if value {
            Self::TRUE
        } else {
            Self::FALSE
        }
    }
}

impl From<m32> for bool {
    fn from(value: m32) -> bool {
        if value == m32::TRUE {
            true
        } else {
            false
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct m32x4(x86_64::__m128i);

impl m32x4 {
    #[inline]
    pub fn new(a: m32, b: m32, c: m32, d: m32) -> m32x4 {
        #[repr(C, align(16))]
        struct Align([m32; 4]);

        let values = Align([a, b, c, d]);
        unsafe { m32x4(x86_64::_mm_load_si128(values.0.as_ptr() as *const x86_64::__m128i)) }
    }

    #[inline]
    pub fn splat(value: m32) -> m32x4 {
        unsafe { m32x4(x86_64::_mm_set1_epi32(value.0)) }
    }

    #[inline]
    pub fn shuffle<const MASK: i32>(&self) -> m32x4 {
        unsafe { m32x4(x86_64::_mm_shuffle_epi32::<MASK>(self.0)) }
    }

    #[inline]
    pub fn select(mask: m32x4, a: m32x4, b: m32x4) -> m32x4 {
        unsafe {
            m32x4(x86_64::_mm_or_si128(
                x86_64::_mm_and_si128(mask.0, a.0),
                x86_64::_mm_andnot_si128(mask.0, b.0),
            ))
        }
    }

    #[inline]
    pub fn eq(&self, other: m32x4) -> m32x4 {
        unsafe { m32x4(x86_64::_mm_cmpeq_epi32(self.0, other.0)) }
    }

    #[inline]
    pub fn ne(&self, other: m32x4) -> m32x4 {
        !self.eq(other)
    }
}

impl Index<usize> for m32x4 {
    type Output = m32;

    #[inline]
    fn index(&self, index: usize) -> &m32 {
        unsafe { &mem::transmute::<&x86_64::__m128i, &[m32; 4]>(&self.0)[index] }
    }
}

impl IndexMut<usize> for m32x4 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut m32 {
        unsafe { &mut mem::transmute::<&mut x86_64::__m128i, &mut [m32; 4]>(&mut self.0)[index] }
    }
}

impl BitAnd<m32x4> for m32x4 {
    type Output = m32x4;

    #[inline]
    fn bitand(self, other: m32x4) -> m32x4 {
        unsafe { m32x4(x86_64::_mm_and_si128(self.0, other.0)) }
    }
}

impl BitAndAssign<m32x4> for m32x4 {
    #[inline]
    fn bitand_assign(&mut self, other: m32x4) {
        *self = *self & other;
    }
}

impl BitOr<m32x4> for m32x4 {
    type Output = m32x4;

    #[inline]
    fn bitor(self, other: m32x4) -> m32x4 {
        unsafe { m32x4(x86_64::_mm_or_si128(self.0, other.0)) }
    }
}

impl BitOrAssign<m32x4> for m32x4 {
    #[inline]
    fn bitor_assign(&mut self, other: m32x4) {
        *self = *self | other;
    }
}

impl BitXor<m32x4> for m32x4 {
    type Output = m32x4;

    #[inline]
    fn bitxor(self, other: m32x4) -> m32x4 {
        unsafe { m32x4(x86_64::_mm_xor_si128(self.0, other.0)) }
    }
}

impl BitXorAssign<m32x4> for m32x4 {
    #[inline]
    fn bitxor_assign(&mut self, other: m32x4) {
        *self = *self ^ other;
    }
}

impl Not for m32x4 {
    type Output = m32x4;

    #[inline]
    fn not(self) -> m32x4 {
        unsafe { m32x4(x86_64::_mm_andnot_si128(self.0, x86_64::_mm_set1_epi32(!0))) }
    }
}

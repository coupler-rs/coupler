#![cfg(target_feature = "sse2")]

use std::arch::x86_64::*;
use std::ops::*;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct f32x4(__m128);

impl f32x4 {
    #[inline]
    pub fn new(a: f32, b: f32, c: f32, d: f32) -> f32x4 {
        unsafe { f32x4(_mm_setr_ps(a, b, c, d)) }
    }

    #[inline]
    pub fn splat(value: f32) -> f32x4 {
        unsafe { f32x4(_mm_set1_ps(value)) }
    }

    #[inline]
    pub fn from_slice(slice: &[f32]) -> f32x4 {
        assert_eq!(slice.len(), 4);
        unsafe { f32x4(_mm_loadu_ps(slice.as_ptr())) }
    }

    #[inline]
    pub fn write_to_slice(&self, slice: &mut [f32]) {
        assert_eq!(slice.len(), 4);
        unsafe {
            _mm_storeu_ps(slice.as_mut_ptr(), self.0);
        }
    }

    #[inline]
    pub fn get<const INDEX: i32>(&self) -> f32 {
        assert!(INDEX >= 0 && INDEX < 4);
        unsafe { _mm_cvtss_f32(_mm_shuffle_ps::<INDEX>(self.0, self.0)) }
    }

    #[inline]
    pub fn set<const INDEX: i32>(&mut self, value: f32) {
        *self = self.replace::<INDEX>(value);
    }

    #[inline]
    pub fn replace<const INDEX: i32>(&self, value: f32) -> f32x4 {
        assert!(INDEX >= 0 && INDEX < 4);
        f32x4::select(i32x4::new(0, 1, 2, 3).eq(i32x4::splat(INDEX)), *self, f32x4::splat(value))
    }

    #[inline]
    pub fn shuffle<const MASK: i32>(&self) -> f32x4 {
        assert_eq!(MASK as u32 & 0xFFFFFF00, 0);
        unsafe { f32x4(_mm_shuffle_ps::<MASK>(self.0, self.0)) }
    }

    #[inline]
    pub fn select(mask: m32x4, a: f32x4, b: f32x4) -> f32x4 {
        unsafe {
            let mask = _mm_castsi128_ps(mask.0);
            f32x4(_mm_or_ps(_mm_and_ps(mask, a.0), _mm_andnot_ps(mask, b.0)))
        }
    }

    #[inline]
    pub fn eq(&self, other: f32x4) -> m32x4 {
        unsafe { m32x4(_mm_castps_si128(_mm_cmpeq_ps(self.0, other.0))) }
    }

    #[inline]
    pub fn ne(&self, other: f32x4) -> m32x4 {
        unsafe { m32x4(_mm_castps_si128(_mm_cmpneq_ps(self.0, other.0))) }
    }

    #[inline]
    pub fn lt(&self, other: f32x4) -> m32x4 {
        unsafe { m32x4(_mm_castps_si128(_mm_cmplt_ps(self.0, other.0))) }
    }

    #[inline]
    pub fn gt(&self, other: f32x4) -> m32x4 {
        unsafe { m32x4(_mm_castps_si128(_mm_cmpgt_ps(self.0, other.0))) }
    }

    #[inline]
    pub fn le(&self, other: f32x4) -> m32x4 {
        unsafe { m32x4(_mm_castps_si128(_mm_cmple_ps(self.0, other.0))) }
    }

    #[inline]
    pub fn ge(&self, other: f32x4) -> m32x4 {
        unsafe { m32x4(_mm_castps_si128(_mm_cmpge_ps(self.0, other.0))) }
    }

    #[inline]
    pub fn min(&self, other: f32x4) -> f32x4 {
        unsafe { f32x4(_mm_min_ps(self.0, other.0)) }
    }

    #[inline]
    pub fn max(&self, other: f32x4) -> f32x4 {
        unsafe { f32x4(_mm_max_ps(self.0, other.0)) }
    }

    #[inline]
    pub fn scan(&self) -> f32x4 {
        unsafe {
            let shifted = _mm_castsi128_ps(_mm_slli_si128(_mm_castps_si128(self.0), 4));
            let sum1 = _mm_add_ps(self.0, shifted);
            let shifted = _mm_castsi128_ps(_mm_slli_si128(_mm_castps_si128(sum1), 8));
            f32x4(_mm_add_ps(sum1, shifted))
        }
    }
}

impl From<u32x4> for f32x4 {
    #[inline]
    fn from(value: u32x4) -> f32x4 {
        unsafe { f32x4(_mm_cvtepi32_ps(value.0)) }
    }
}

impl From<i32x4> for f32x4 {
    #[inline]
    fn from(value: i32x4) -> f32x4 {
        unsafe { f32x4(_mm_cvtepi32_ps(value.0)) }
    }
}

impl Add<f32x4> for f32x4 {
    type Output = f32x4;

    #[inline]
    fn add(self, other: f32x4) -> f32x4 {
        unsafe { f32x4(_mm_add_ps(self.0, other.0)) }
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
        unsafe { f32x4(_mm_sub_ps(self.0, other.0)) }
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
        unsafe { f32x4(_mm_mul_ps(self.0, other.0)) }
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
        unsafe { f32x4(_mm_div_ps(self.0, other.0)) }
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
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct u32x4(__m128i);

impl u32x4 {
    #[inline]
    pub fn new(a: u32, b: u32, c: u32, d: u32) -> u32x4 {
        unsafe { u32x4(_mm_setr_epi32(a as i32, b as i32, c as i32, d as i32)) }
    }

    #[inline]
    pub fn splat(value: u32) -> u32x4 {
        unsafe { u32x4(_mm_set1_epi32(value as i32)) }
    }

    #[inline]
    pub fn from_slice(slice: &[u32]) -> u32x4 {
        assert_eq!(slice.len(), 4);
        unsafe { u32x4(_mm_loadu_si128(slice.as_ptr() as *const __m128i)) }
    }

    #[inline]
    pub fn write_to_slice(&self, slice: &mut [u32]) {
        assert_eq!(slice.len(), 4);
        unsafe {
            _mm_storeu_si128(slice.as_mut_ptr() as *mut __m128i, self.0);
        }
    }

    #[inline]
    pub fn get<const INDEX: i32>(&self) -> u32 {
        assert!(INDEX >= 0 && INDEX < 4);
        unsafe { _mm_cvtsi128_si32(_mm_shuffle_epi32::<INDEX>(self.0)) as u32 }
    }

    #[inline]
    pub fn set<const INDEX: i32>(&mut self, value: u32) {
        *self = self.replace::<INDEX>(value);
    }

    #[inline]
    pub fn replace<const INDEX: i32>(&self, value: u32) -> u32x4 {
        assert!(INDEX >= 0 && INDEX < 4);
        u32x4::select(i32x4::new(0, 1, 2, 3).eq(i32x4::splat(INDEX)), *self, u32x4::splat(value))
    }

    #[inline]
    pub fn shuffle<const MASK: i32>(&self) -> u32x4 {
        assert_eq!(MASK as u32 & 0xFFFFFF00, 0);
        unsafe { u32x4(_mm_shuffle_epi32::<MASK>(self.0)) }
    }

    #[inline]
    pub fn select(mask: m32x4, a: u32x4, b: u32x4) -> u32x4 {
        unsafe { u32x4(_mm_or_si128(_mm_and_si128(mask.0, a.0), _mm_andnot_si128(mask.0, b.0))) }
    }

    #[inline]
    pub fn eq(&self, other: u32x4) -> m32x4 {
        unsafe { m32x4(_mm_cmpeq_epi32(self.0, other.0)) }
    }

    #[inline]
    pub fn ne(&self, other: u32x4) -> m32x4 {
        !self.eq(other)
    }

    #[inline]
    pub fn lt(&self, other: u32x4) -> m32x4 {
        unsafe {
            let bias = _mm_set1_epi32(i32::MIN);
            m32x4(_mm_cmplt_epi32(_mm_add_epi32(self.0, bias), _mm_add_epi32(other.0, bias)))
        }
    }

    #[inline]
    pub fn gt(&self, other: u32x4) -> m32x4 {
        unsafe {
            let bias = _mm_set1_epi32(i32::MIN);
            m32x4(_mm_cmpgt_epi32(_mm_add_epi32(self.0, bias), _mm_add_epi32(other.0, bias)))
        }
    }

    #[inline]
    pub fn le(&self, other: u32x4) -> m32x4 {
        !self.gt(other)
    }

    #[inline]
    pub fn ge(&self, other: u32x4) -> m32x4 {
        !self.lt(other)
    }

    #[inline]
    pub fn min(&self, other: u32x4) -> u32x4 {
        u32x4::select(self.lt(other), *self, other)
    }

    #[inline]
    pub fn max(&self, other: u32x4) -> u32x4 {
        u32x4::select(self.gt(other), *self, other)
    }
}

impl From<f32x4> for u32x4 {
    #[inline]
    fn from(value: f32x4) -> u32x4 {
        unsafe { u32x4(_mm_cvtps_epi32(value.0)) }
    }
}

impl From<i32x4> for u32x4 {
    #[inline]
    fn from(value: i32x4) -> u32x4 {
        u32x4(value.0)
    }
}

impl Add<u32x4> for u32x4 {
    type Output = u32x4;

    #[inline]
    fn add(self, other: u32x4) -> u32x4 {
        unsafe { u32x4(_mm_add_epi32(self.0, other.0)) }
    }
}

impl AddAssign<u32x4> for u32x4 {
    #[inline]
    fn add_assign(&mut self, other: u32x4) {
        *self = *self + other;
    }
}

impl Sub<u32x4> for u32x4 {
    type Output = u32x4;

    #[inline]
    fn sub(self, other: u32x4) -> u32x4 {
        unsafe { u32x4(_mm_sub_epi32(self.0, other.0)) }
    }
}

impl SubAssign<u32x4> for u32x4 {
    #[inline]
    fn sub_assign(&mut self, other: u32x4) {
        *self = *self - other;
    }
}

impl BitAnd<u32x4> for u32x4 {
    type Output = u32x4;

    #[inline]
    fn bitand(self, other: u32x4) -> u32x4 {
        unsafe { u32x4(_mm_and_si128(self.0, other.0)) }
    }
}

impl BitAndAssign<u32x4> for u32x4 {
    #[inline]
    fn bitand_assign(&mut self, other: u32x4) {
        *self = *self & other;
    }
}

impl BitOr<u32x4> for u32x4 {
    type Output = u32x4;

    #[inline]
    fn bitor(self, other: u32x4) -> u32x4 {
        unsafe { u32x4(_mm_or_si128(self.0, other.0)) }
    }
}

impl BitOrAssign<u32x4> for u32x4 {
    #[inline]
    fn bitor_assign(&mut self, other: u32x4) {
        *self = *self | other;
    }
}

impl BitXor<u32x4> for u32x4 {
    type Output = u32x4;

    #[inline]
    fn bitxor(self, other: u32x4) -> u32x4 {
        unsafe { u32x4(_mm_xor_si128(self.0, other.0)) }
    }
}

impl BitXorAssign<u32x4> for u32x4 {
    #[inline]
    fn bitxor_assign(&mut self, other: u32x4) {
        *self = *self ^ other;
    }
}

impl Not for u32x4 {
    type Output = u32x4;

    #[inline]
    fn not(self) -> u32x4 {
        unsafe { u32x4(_mm_andnot_si128(self.0, _mm_set1_epi32(!0))) }
    }
}

impl Shl<usize> for u32x4 {
    type Output = u32x4;

    #[inline]
    fn shl(self, bits: usize) -> u32x4 {
        unsafe { u32x4(_mm_sll_epi32(self.0, _mm_setr_epi32(bits as i32, 0, 0, 0))) }
    }
}

impl ShlAssign<usize> for u32x4 {
    #[inline]
    fn shl_assign(&mut self, bits: usize) {
        *self = *self << bits;
    }
}

impl Shr<usize> for u32x4 {
    type Output = u32x4;

    #[inline]
    fn shr(self, bits: usize) -> u32x4 {
        unsafe { u32x4(_mm_srl_epi32(self.0, _mm_setr_epi32(bits as i32, 0, 0, 0))) }
    }
}

impl ShrAssign<usize> for u32x4 {
    #[inline]
    fn shr_assign(&mut self, bits: usize) {
        *self = *self >> bits;
    }
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct i32x4(__m128i);

impl i32x4 {
    #[inline]
    pub fn new(a: i32, b: i32, c: i32, d: i32) -> i32x4 {
        unsafe { i32x4(_mm_setr_epi32(a as i32, b as i32, c as i32, d as i32)) }
    }

    #[inline]
    pub fn splat(value: i32) -> i32x4 {
        unsafe { i32x4(_mm_set1_epi32(value as i32)) }
    }

    #[inline]
    pub fn from_slice(slice: &[i32]) -> i32x4 {
        assert_eq!(slice.len(), 4);
        unsafe { i32x4(_mm_loadu_si128(slice.as_ptr() as *const __m128i)) }
    }

    #[inline]
    pub fn write_to_slice(&self, slice: &mut [i32]) {
        assert_eq!(slice.len(), 4);
        unsafe {
            _mm_storeu_si128(slice.as_mut_ptr() as *mut __m128i, self.0);
        }
    }

    #[inline]
    pub fn get<const INDEX: i32>(&self) -> i32 {
        assert!(INDEX >= 0 && INDEX < 4);
        unsafe { _mm_cvtsi128_si32(_mm_shuffle_epi32::<INDEX>(self.0)) }
    }

    #[inline]
    pub fn set<const INDEX: i32>(&mut self, value: i32) {
        *self = self.replace::<INDEX>(value);
    }

    #[inline]
    pub fn replace<const INDEX: i32>(&self, value: i32) -> i32x4 {
        assert!(INDEX >= 0 && INDEX < 4);
        i32x4::select(i32x4::new(0, 1, 2, 3).eq(i32x4::splat(INDEX)), *self, i32x4::splat(value))
    }

    #[inline]
    pub fn shuffle<const MASK: i32>(&self) -> i32x4 {
        assert_eq!(MASK as u32 & 0xFFFFFF00, 0);
        unsafe { i32x4(_mm_shuffle_epi32::<MASK>(self.0)) }
    }

    #[inline]
    pub fn select(mask: m32x4, a: i32x4, b: i32x4) -> i32x4 {
        unsafe { i32x4(_mm_or_si128(_mm_and_si128(mask.0, a.0), _mm_andnot_si128(mask.0, b.0))) }
    }

    #[inline]
    pub fn eq(&self, other: i32x4) -> m32x4 {
        unsafe { m32x4(_mm_cmpeq_epi32(self.0, other.0)) }
    }

    #[inline]
    pub fn ne(&self, other: i32x4) -> m32x4 {
        !self.eq(other)
    }

    #[inline]
    pub fn lt(&self, other: i32x4) -> m32x4 {
        unsafe { m32x4(_mm_cmplt_epi32(self.0, other.0)) }
    }

    #[inline]
    pub fn gt(&self, other: i32x4) -> m32x4 {
        unsafe { m32x4(_mm_cmpgt_epi32(self.0, other.0)) }
    }

    #[inline]
    pub fn le(&self, other: i32x4) -> m32x4 {
        !self.gt(other)
    }

    #[inline]
    pub fn ge(&self, other: i32x4) -> m32x4 {
        !self.lt(other)
    }

    #[inline]
    pub fn min(&self, other: i32x4) -> i32x4 {
        i32x4::select(self.lt(other), *self, other)
    }

    #[inline]
    pub fn max(&self, other: i32x4) -> i32x4 {
        i32x4::select(self.gt(other), *self, other)
    }
}

impl From<f32x4> for i32x4 {
    #[inline]
    fn from(value: f32x4) -> i32x4 {
        unsafe { i32x4(_mm_cvtps_epi32(value.0)) }
    }
}

impl From<u32x4> for i32x4 {
    #[inline]
    fn from(value: u32x4) -> i32x4 {
        i32x4(value.0)
    }
}

impl Add<i32x4> for i32x4 {
    type Output = i32x4;

    #[inline]
    fn add(self, other: i32x4) -> i32x4 {
        unsafe { i32x4(_mm_add_epi32(self.0, other.0)) }
    }
}

impl AddAssign<i32x4> for i32x4 {
    #[inline]
    fn add_assign(&mut self, other: i32x4) {
        *self = *self + other;
    }
}

impl Sub<i32x4> for i32x4 {
    type Output = i32x4;

    #[inline]
    fn sub(self, other: i32x4) -> i32x4 {
        unsafe { i32x4(_mm_sub_epi32(self.0, other.0)) }
    }
}

impl SubAssign<i32x4> for i32x4 {
    #[inline]
    fn sub_assign(&mut self, other: i32x4) {
        *self = *self - other;
    }
}

impl BitAnd<i32x4> for i32x4 {
    type Output = i32x4;

    #[inline]
    fn bitand(self, other: i32x4) -> i32x4 {
        unsafe { i32x4(_mm_and_si128(self.0, other.0)) }
    }
}

impl BitAndAssign<i32x4> for i32x4 {
    #[inline]
    fn bitand_assign(&mut self, other: i32x4) {
        *self = *self & other;
    }
}

impl BitOr<i32x4> for i32x4 {
    type Output = i32x4;

    #[inline]
    fn bitor(self, other: i32x4) -> i32x4 {
        unsafe { i32x4(_mm_or_si128(self.0, other.0)) }
    }
}

impl BitOrAssign<i32x4> for i32x4 {
    #[inline]
    fn bitor_assign(&mut self, other: i32x4) {
        *self = *self | other;
    }
}

impl BitXor<i32x4> for i32x4 {
    type Output = i32x4;

    #[inline]
    fn bitxor(self, other: i32x4) -> i32x4 {
        unsafe { i32x4(_mm_xor_si128(self.0, other.0)) }
    }
}

impl BitXorAssign<i32x4> for i32x4 {
    #[inline]
    fn bitxor_assign(&mut self, other: i32x4) {
        *self = *self ^ other;
    }
}

impl Not for i32x4 {
    type Output = i32x4;

    #[inline]
    fn not(self) -> i32x4 {
        unsafe { i32x4(_mm_andnot_si128(self.0, _mm_set1_epi32(!0))) }
    }
}

impl Shl<usize> for i32x4 {
    type Output = i32x4;

    #[inline]
    fn shl(self, bits: usize) -> i32x4 {
        unsafe { i32x4(_mm_sll_epi32(self.0, _mm_setr_epi32(bits as i32, 0, 0, 0))) }
    }
}

impl ShlAssign<usize> for i32x4 {
    #[inline]
    fn shl_assign(&mut self, bits: usize) {
        *self = *self << bits;
    }
}

impl Shr<usize> for i32x4 {
    type Output = i32x4;

    #[inline]
    fn shr(self, bits: usize) -> i32x4 {
        unsafe { i32x4(_mm_sra_epi32(self.0, _mm_setr_epi32(bits as i32, 0, 0, 0))) }
    }
}

impl ShrAssign<usize> for i32x4 {
    #[inline]
    fn shr_assign(&mut self, bits: usize) {
        *self = *self >> bits;
    }
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct m32x4(__m128i);

impl m32x4 {
    #[inline]
    fn bool_to_mask(value: bool) -> i32 {
        if value {
            !0
        } else {
            0
        }
    }

    #[inline]
    fn mask_to_bool(value: i32) -> bool {
        value == !0
    }

    #[inline]
    pub fn new(a: bool, b: bool, c: bool, d: bool) -> m32x4 {
        unsafe {
            m32x4(_mm_setr_epi32(
                Self::bool_to_mask(a),
                Self::bool_to_mask(b),
                Self::bool_to_mask(c),
                Self::bool_to_mask(d),
            ))
        }
    }

    #[inline]
    pub fn splat(value: bool) -> m32x4 {
        unsafe { m32x4(_mm_set1_epi32(Self::bool_to_mask(value))) }
    }

    #[inline]
    pub fn get<const INDEX: i32>(&self) -> bool {
        assert!(INDEX >= 0 && INDEX < 4);
        unsafe { Self::mask_to_bool(_mm_cvtsi128_si32(_mm_shuffle_epi32::<INDEX>(self.0))) }
    }

    #[inline]
    pub fn set<const INDEX: i32>(&mut self, value: bool) {
        *self = self.replace::<INDEX>(value);
    }

    #[inline]
    pub fn replace<const INDEX: i32>(&self, value: bool) -> m32x4 {
        assert!(INDEX >= 0 && INDEX < 4);
        m32x4::select(i32x4::new(0, 1, 2, 3).eq(i32x4::splat(INDEX)), *self, m32x4::splat(value))
    }

    #[inline]
    pub fn shuffle<const MASK: i32>(&self) -> m32x4 {
        assert_eq!(MASK as u32 & 0xFFFFFF00, 0);
        unsafe { m32x4(_mm_shuffle_epi32::<MASK>(self.0)) }
    }

    #[inline]
    pub fn select(mask: m32x4, a: m32x4, b: m32x4) -> m32x4 {
        unsafe { m32x4(_mm_or_si128(_mm_and_si128(mask.0, a.0), _mm_andnot_si128(mask.0, b.0))) }
    }

    #[inline]
    pub fn eq(&self, other: m32x4) -> m32x4 {
        unsafe { m32x4(_mm_cmpeq_epi32(self.0, other.0)) }
    }

    #[inline]
    pub fn ne(&self, other: m32x4) -> m32x4 {
        !self.eq(other)
    }
}

impl BitAnd<m32x4> for m32x4 {
    type Output = m32x4;

    #[inline]
    fn bitand(self, other: m32x4) -> m32x4 {
        unsafe { m32x4(_mm_and_si128(self.0, other.0)) }
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
        unsafe { m32x4(_mm_or_si128(self.0, other.0)) }
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
        unsafe { m32x4(_mm_xor_si128(self.0, other.0)) }
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
        unsafe { m32x4(_mm_andnot_si128(self.0, _mm_set1_epi32(!0))) }
    }
}

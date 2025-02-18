use std::iter::{Product, Sum};
use std::{
    arch::x86_64::*,
    mem::transmute,
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use crate::{field_common, ExtensionField, Field, FieldSerde, FieldSerdeResult, GF2};

#[derive(Debug, Clone, Copy)]
pub struct AVX512GF2_127 {
    pub v: __m128i,
}

field_common!(AVX512GF2_127);

impl FieldSerde for AVX512GF2_127 {
    const SERIALIZED_SIZE: usize = 16;

    #[inline(always)]
    fn serialize_into<W: std::io::Write>(&self, mut writer: W) -> FieldSerdeResult<()> {
        unsafe { writer.write_all(transmute::<__m128i, [u8; 16]>(self.v).as_ref())? };
        Ok(())
    }

    #[inline(always)]
    fn deserialize_from<R: std::io::Read>(mut reader: R) -> FieldSerdeResult<Self> {
        let mut u = [0u8; Self::SERIALIZED_SIZE];
        reader.read_exact(&mut u)?;
        u[Self::SERIALIZED_SIZE - 1] &= 0x7F; // Should we do a modular operation here?

        unsafe {
            Ok(AVX512GF2_127 {
                v: transmute::<[u8; Self::SERIALIZED_SIZE], __m128i>(u),
            })
        }
    }

    #[inline(always)]
    fn try_deserialize_from_ecc_format<R: std::io::Read>(mut reader: R) -> FieldSerdeResult<Self> {
        let mut u = [0u8; 32];
        reader.read_exact(&mut u)?;
        assert!(u[15] <= 0x7F); // and ignoring 16 - 31
        Ok(unsafe {
            AVX512GF2_127 {
                v: transmute::<[u8; 16], __m128i>(u[..16].try_into().unwrap()),
            }
        })
    }
}

// mod x^127 + x + 1
impl Field for AVX512GF2_127 {
    const NAME: &'static str = "Galios Field 2^127";

    const SIZE: usize = 128 / 8;

    const FIELD_SIZE: usize = 127; // in bits

    const ZERO: Self = AVX512GF2_127 {
        v: unsafe { std::mem::zeroed() },
    };

    const ONE: Self = AVX512GF2_127 {
        v: unsafe { std::mem::transmute::<[i32; 4], __m128i>([1, 0, 0, 0]) },
    };

    const INV_2: Self = AVX512GF2_127 {
        v: unsafe { std::mem::zeroed() },
    }; // should not be used

    #[inline(always)]
    fn zero() -> Self {
        AVX512GF2_127 {
            v: unsafe { std::mem::zeroed() },
        }
    }

    #[inline(always)]
    fn one() -> Self {
        AVX512GF2_127 {
            v: unsafe { std::mem::transmute::<[i32; 4], __m128i>([1, 0, 0, 0]) }, 
        }
    }

    #[inline(always)]
    fn random_unsafe(mut rng: impl rand::RngCore) -> Self {
        let mut u = [0u8; 16];
        rng.fill_bytes(&mut u);
        u[15] &= 0x7F;
        unsafe {
            AVX512GF2_127 {
                v: *(u.as_ptr() as *const __m128i),
            }
        }
    }

    #[inline(always)]
    fn random_bool(mut rng: impl rand::RngCore) -> Self {
        AVX512GF2_127 {
            v: unsafe { std::mem::transmute::<[u32; 4], __m128i>([rng.next_u32() % 2, 0, 0, 0]) },
        }
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        unsafe { std::mem::transmute::<__m128i, [u8; 16]>(self.v) == [0; 16] }
    }

    #[inline(always)]
    fn exp(&self, exponent: u128) -> Self {
        let mut e = exponent;
        let mut res = Self::one();
        let mut t = *self;
        while e > 0 {
            if e & 1 == 1 {
                res *= t;
            }
            t = t * t;
            e >>= 1;
        }
        res
    }

    #[inline(always)]
    fn inv(&self) -> Option<Self> {
        if self.is_zero() {
            return None;
        }
        let p_m2 = (1u128 << 127) - 2;
        Some(Self::exp(self, p_m2))
    }

    #[inline(always)]
    fn square(&self) -> Self {
        self * self
    }

    #[inline(always)]
    fn as_u32_unchecked(&self) -> u32 {
        unimplemented!("u32 for GF127 doesn't make sense")
    }

    #[inline(always)]
    fn from_uniform_bytes(bytes: &[u8; 32]) -> Self {
        let mut bytes = bytes.clone();
        bytes[15] &= 0x7F;

        unsafe {
            AVX512GF2_127 {
                v: transmute::<[u8; 16], __m128i>(bytes[..16].try_into().unwrap()),
            }
        }
    }
}

impl ExtensionField for AVX512GF2_127 {
    const DEGREE: usize = 127;

    const W: u32 = 0x87;

    const X: Self = AVX512GF2_127 {
        v: unsafe { std::mem::transmute::<[i32; 4], __m128i>([2, 0, 0, 0]) },
    };

    type BaseField = GF2;

    #[inline(always)]
    fn mul_by_base_field(&self, base: &Self::BaseField) -> Self {
        if base.v == 0 {
            Self::zero()
        } else {
            *self
        }
    }

    #[inline(always)]
    fn add_by_base_field(&self, base: &Self::BaseField) -> Self {
        let mut res = *self;
        res.v = unsafe { _mm_xor_si128(res.v, _mm_set_epi64x(0, base.v as i64)) };
        res
    }

    /// 
    #[inline]
    fn mul_by_x(&self) -> Self {
        unsafe {
            // Shift left by 1 bit
            let shifted = _mm_slli_epi64(self.v, 1);

            // Get the most significant bit and move it
            let msb = _mm_srli_epi64(self.v, 63); 
            let msb_moved = _mm_slli_si128(msb, 8); 

            // Combine the shifted value with the moved msb
            let shifted_consolidated = _mm_or_si128(shifted, msb_moved);

            // Create the reduction value (0b11) and the comparison value (1)
            let reduction = {
                let multiplier = _mm_set_epi64x(0, 0b11);
                let one = _mm_set_epi64x(0, 1);

                // Check if the MSB was 1 and create a mask
                let mask = _mm_cmpeq_epi64(
                    _mm_srli_si128(_mm_srli_epi64(shifted, 63), 8), 
                    one);

                _mm_and_si128(mask, multiplier)
            };

            // Apply the reduction conditionally
            let res = _mm_xor_si128(shifted_consolidated, reduction);

            Self { v: res }
        }
    }
}

impl From<GF2> for AVX512GF2_127 {
    #[inline(always)]
    fn from(v: GF2) -> Self {
        AVX512GF2_127 {
            v: unsafe { _mm_set_epi64x(0, v.v as i64) },
        }
    }
}

const X0TO126_MASK: __m128i = unsafe { transmute::<[u8; 16], __m128i>(
    [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F])};
const X127_MASK: __m128i = unsafe { transmute::<[u8; 16], __m128i>(
    [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80])};
const X127_REMINDER: __m128i = unsafe { transmute::<[u8; 16], __m128i>(
    [0b11, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80])};


#[inline(always)]
unsafe fn mm_bitshift_left<const count: usize>(x: __m128i) -> __m128i
{
    let mut carry = _mm_bslli_si128(x, 8); 
    carry = _mm_srli_epi64(carry, 64 - count);  
    let x = _mm_slli_epi64(x, count);
    _mm_or_si128(x, carry)
}


#[inline]
unsafe fn gfmul(a: __m128i, b: __m128i) -> __m128i {
    let xmm_mask = _mm_setr_epi32((0xFFffffff_u32) as i32, 0x0, 0x0, 0x0);

    // a = a0|a1, b = b0|b1

    let mut tmp3 = _mm_clmulepi64_si128(a, b, 0x00); // tmp3 = a0 * b0
    let mut tmp6 = _mm_clmulepi64_si128(a, b, 0x11); // tmp6 = a1 * b1

    // 78 = 0b0100_1110
    let mut tmp4 = _mm_shuffle_epi32(a, 78); // tmp4 = a1|a0
    let mut tmp5 = _mm_shuffle_epi32(b, 78); // tmp5 = b1|b0
    tmp4 = _mm_xor_si128(tmp4, a); // tmp4 = (a0 + a1) | (a0 + a1)
    tmp5 = _mm_xor_si128(tmp5, b); // tmp5 = (b0 + b1) | (b0 + b1)

    tmp4 = _mm_clmulepi64_si128(tmp4, tmp5, 0x00); // tmp4 = (a0 + a1) * (b0 + b1)
    tmp4 = _mm_xor_si128(tmp4, tmp3); // tmp4 = (a0 + a1) * (b0 + b1) - a0 * b0
    tmp4 = _mm_xor_si128(tmp4, tmp6); // tmp4 = (a0 + a1) * (b0 + b1) - a0 * b0 - a1 * b1 = a0 * b1 + a1 * b0

    // tmp4 = e1 | e0
    tmp5 = _mm_slli_si128(tmp4, 8); // tmp5 = e0 | 00
    tmp4 = _mm_srli_si128(tmp4, 8); // tmp4 = 00 | e1
    tmp3 = _mm_xor_si128(tmp3, tmp5); // the lower 128 bits, deg 0 - 127
    tmp6 = _mm_xor_si128(tmp6, tmp4); // the higher 128 bits, deg 128 - 252, the 124 least signicicant bits are non-zero

    // x^0 - x^126
    let x0to126 = _mm_and_si128(tmp3, X0TO126_MASK);

    // x^127 
   tmp4 = _mm_and_si128(tmp3, X127_MASK);
   tmp4 = _mm_cmpeq_epi64(tmp4, X127_MASK);
   tmp4 = _mm_srli_si128(tmp4, 15);
   let x127 = _mm_and_si128(tmp4, X127_REMINDER);

   // x^128 - x^252
   let x128to252 = 
    _mm_and_si128(
        mm_bitshift_left::<2>(tmp6),
        mm_bitshift_left::<1>(tmp6),
    );

    _mm_and_si128(_mm_and_si128(x0to126, x127), x128to252)

    // let mut tmp7 = _mm_srli_epi32(tmp6, 31);
    // let mut tmp8 = _mm_srli_epi32(tmp6, 30);
    // let tmp9 = _mm_srli_epi32(tmp6, 25);

    // tmp7 = _mm_xor_si128(tmp7, tmp8);
    // tmp7 = _mm_xor_si128(tmp7, tmp9);

    // tmp8 = _mm_shuffle_epi32(tmp7, 147);
    // tmp7 = _mm_and_si128(xmm_mask, tmp8);
    // tmp8 = _mm_andnot_si128(xmm_mask, tmp8);

    // tmp3 = _mm_xor_si128(tmp3, tmp8);
    // tmp6 = _mm_xor_si128(tmp6, tmp7);

    // let tmp10 = _mm_slli_epi32(tmp6, 1);
    // tmp3 = _mm_xor_si128(tmp3, tmp10);

    // let tmp11 = _mm_slli_epi32(tmp6, 2);
    // tmp3 = _mm_xor_si128(tmp3, tmp11);

    // let tmp12 = _mm_slli_epi32(tmp6, 7);
    // tmp3 = _mm_xor_si128(tmp3, tmp12);

    // _mm_xor_si128(tmp3, tmp6)

}

impl Default for AVX512GF2_127 {
    #[inline(always)]
    fn default() -> Self {
        Self::zero()
    }
}

impl PartialEq for AVX512GF2_127 {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        unsafe { _mm_test_all_ones(_mm_cmpeq_epi8(self.v, other.v)) == 1 }
    }
}

impl Neg for AVX512GF2_127 {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self {
        self
    }
}

impl From<u32> for AVX512GF2_127 {
    #[inline(always)]
    fn from(v: u32) -> Self {
        AVX512GF2_127 {
            v: unsafe { std::mem::transmute::<[u32; 4], __m128i>([v, 0, 0, 0]) },
        }
    }
}

#[inline(always)]
fn add_internal(a: &AVX512GF2_127, b: &AVX512GF2_127) -> AVX512GF2_127 {
    AVX512GF2_127 {
        v: unsafe { _mm_xor_si128(a.v, b.v) },
    }
}

#[inline(always)]
fn sub_internal(a: &AVX512GF2_127, b: &AVX512GF2_127) -> AVX512GF2_127 {
    AVX512GF2_127 {
        v: unsafe { _mm_xor_si128(a.v, b.v) },
    }
}

#[inline(always)]
fn mul_internal(a: &AVX512GF2_127, b: &AVX512GF2_127) -> AVX512GF2_127 {
    AVX512GF2_127 {
        v: unsafe { gfmul(a.v, b.v) },
    }
}

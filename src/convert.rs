use super::Ratio;

use core::ops::ShlAssign;

#[cfg(feature = "num-bigint")]
use super::BigRational;
#[cfg(feature = "num-bigint")]
use num_bigint::{BigInt, BigUint, Sign, ToBigInt};
#[cfg(feature = "num-bigint")]
use num_traits::One;

use num_integer::Integer;
use num_traits::float::FloatCore;
use num_traits::{Bounded, FromPrimitive, NumCast, Signed, ToPrimitive, Unsigned, Zero};

// From integer
impl<T> From<T> for Ratio<T>
where
    T: Clone + Integer,
{
    fn from(x: T) -> Ratio<T> {
        Ratio::from_integer(x)
    }
}

// From pair (through the `new` constructor)
impl<T> From<(T, T)> for Ratio<T>
where
    T: Clone + Integer,
{
    fn from(pair: (T, T)) -> Ratio<T> {
        Ratio::new(pair.0, pair.1)
    }
}

impl<T> From<Ratio<T>> for (T, T) {
    fn from(val: Ratio<T>) -> Self {
        (val.numer, val.denom)
    }
}

#[cfg(feature = "num-bigint")]
impl Ratio<BigInt> {
    /// Converts a float into a rational number.
    pub fn from_float<T: FloatCore>(f: T) -> Option<BigRational> {
        if !f.is_finite() {
            return None;
        }
        let (mantissa, exponent, sign) = f.integer_decode();
        let bigint_sign = if sign == 1 { Sign::Plus } else { Sign::Minus };
        if exponent < 0 {
            let one: BigInt = One::one();
            let denom: BigInt = one << ((-exponent) as usize);
            let numer: BigUint = FromPrimitive::from_u64(mantissa).unwrap();
            Some(Ratio::new(BigInt::from_biguint(bigint_sign, numer), denom))
        } else {
            let mut numer: BigUint = FromPrimitive::from_u64(mantissa).unwrap();
            numer <<= exponent as usize;
            Some(Ratio::from_integer(BigInt::from_biguint(
                bigint_sign,
                numer,
            )))
        }
    }
}

#[cfg(feature = "num-bigint")]
impl FromPrimitive for Ratio<BigInt> {
    fn from_i64(n: i64) -> Option<Self> {
        Some(Ratio::from_integer(n.into()))
    }

    fn from_i128(n: i128) -> Option<Self> {
        Some(Ratio::from_integer(n.into()))
    }

    fn from_u64(n: u64) -> Option<Self> {
        Some(Ratio::from_integer(n.into()))
    }

    fn from_u128(n: u128) -> Option<Self> {
        Some(Ratio::from_integer(n.into()))
    }

    fn from_f32(n: f32) -> Option<Self> {
        Ratio::from_float(n)
    }

    fn from_f64(n: f64) -> Option<Self> {
        Ratio::from_float(n)
    }
}

macro_rules! from_primitive_integer {
    ($typ:ty, $approx:ident) => {
        impl FromPrimitive for Ratio<$typ> {
            fn from_i64(n: i64) -> Option<Self> {
                <$typ as FromPrimitive>::from_i64(n).map(Ratio::from_integer)
            }

            fn from_i128(n: i128) -> Option<Self> {
                <$typ as FromPrimitive>::from_i128(n).map(Ratio::from_integer)
            }

            fn from_u64(n: u64) -> Option<Self> {
                <$typ as FromPrimitive>::from_u64(n).map(Ratio::from_integer)
            }

            fn from_u128(n: u128) -> Option<Self> {
                <$typ as FromPrimitive>::from_u128(n).map(Ratio::from_integer)
            }

            fn from_f32(n: f32) -> Option<Self> {
                $approx(n, 10e-20, 30)
            }

            fn from_f64(n: f64) -> Option<Self> {
                $approx(n, 10e-20, 30)
            }
        }
    };
}

from_primitive_integer!(i8, approximate_float);
from_primitive_integer!(i16, approximate_float);
from_primitive_integer!(i32, approximate_float);
from_primitive_integer!(i64, approximate_float);
from_primitive_integer!(i128, approximate_float);
from_primitive_integer!(isize, approximate_float);

from_primitive_integer!(u8, approximate_float_unsigned);
from_primitive_integer!(u16, approximate_float_unsigned);
from_primitive_integer!(u32, approximate_float_unsigned);
from_primitive_integer!(u64, approximate_float_unsigned);
from_primitive_integer!(u128, approximate_float_unsigned);
from_primitive_integer!(usize, approximate_float_unsigned);

impl<T: Integer + Signed + Bounded + NumCast + Clone> Ratio<T> {
    pub fn approximate_float<F: FloatCore + NumCast>(f: F) -> Option<Ratio<T>> {
        // 1/10e-20 < 1/2**32 which seems like a good default, and 30 seems
        // to work well. Might want to choose something based on the types in the future, e.g.
        // T::max().recip() and T::bits() or something similar.
        let epsilon = <F as NumCast>::from(10e-20).expect("Can't convert 10e-20");
        approximate_float(f, epsilon, 30)
    }
}

impl<T: Integer + Unsigned + Bounded + NumCast + Clone> Ratio<T> {
    pub fn approximate_float_unsigned<F: FloatCore + NumCast>(f: F) -> Option<Ratio<T>> {
        // 1/10e-20 < 1/2**32 which seems like a good default, and 30 seems
        // to work well. Might want to choose something based on the types in the future, e.g.
        // T::max().recip() and T::bits() or something similar.
        let epsilon = <F as NumCast>::from(10e-20).expect("Can't convert 10e-20");
        approximate_float_unsigned(f, epsilon, 30)
    }
}

fn approximate_float<T, F>(val: F, max_error: F, max_iterations: usize) -> Option<Ratio<T>>
where
    T: Integer + Signed + Bounded + NumCast + Clone,
    F: FloatCore + NumCast,
{
    let negative = val.is_sign_negative();
    let abs_val = val.abs();

    let r = approximate_float_unsigned(abs_val, max_error, max_iterations)?;

    // Make negative again if needed
    Some(if negative { -r } else { r })
}

// No Unsigned constraint because this also works on positive integers and is called
// like that, see above
fn approximate_float_unsigned<T, F>(val: F, max_error: F, max_iterations: usize) -> Option<Ratio<T>>
where
    T: Integer + Bounded + NumCast + Clone,
    F: FloatCore + NumCast,
{
    // Continued fractions algorithm
    // https://web.archive.org/web/20200629111319/http://mathforum.org:80/dr.math/faq/faq.fractions.html#decfrac

    if val < F::zero() || val.is_nan() {
        return None;
    }

    let mut q = val;
    let mut n0 = T::zero();
    let mut d0 = T::one();
    let mut n1 = T::one();
    let mut d1 = T::zero();

    let t_max = T::max_value();
    let t_max_f = <F as NumCast>::from(t_max.clone())?;

    // 1/epsilon > T::MAX
    let epsilon = t_max_f.recip();

    // Overflow
    if q > t_max_f {
        return None;
    }

    for _ in 0..max_iterations {
        let a = match <T as NumCast>::from(q) {
            None => break,
            Some(a) => a,
        };

        let a_f = match <F as NumCast>::from(a.clone()) {
            None => break,
            Some(a_f) => a_f,
        };
        let f = q - a_f;

        // Prevent overflow
        if !a.is_zero()
            && (n1 > t_max.clone() / a.clone()
                || d1 > t_max.clone() / a.clone()
                || a.clone() * n1.clone() > t_max.clone() - n0.clone()
                || a.clone() * d1.clone() > t_max.clone() - d0.clone())
        {
            break;
        }

        let n = a.clone() * n1.clone() + n0.clone();
        let d = a.clone() * d1.clone() + d0.clone();

        n0 = n1;
        d0 = d1;
        n1 = n.clone();
        d1 = d.clone();

        // Simplify fraction. Doing so here instead of at the end
        // allows us to get closer to the target value without overflows
        let g = Integer::gcd(&n1, &d1);
        if !g.is_zero() {
            n1 = n1 / g.clone();
            d1 = d1 / g.clone();
        }

        // Close enough?
        let (n_f, d_f) = match (<F as NumCast>::from(n), <F as NumCast>::from(d)) {
            (Some(n_f), Some(d_f)) => (n_f, d_f),
            _ => break,
        };
        if (n_f / d_f - val).abs() < max_error {
            break;
        }

        // Prevent division by ~0
        if f < epsilon {
            break;
        }
        q = f.recip();
    }

    // Overflow
    if d1.is_zero() {
        return None;
    }

    Some(Ratio::new(n1, d1))
}

#[cfg(not(feature = "num-bigint"))]
macro_rules! to_primitive_small {
    ($($type_name:ty)*) => ($(
        impl ToPrimitive for Ratio<$type_name> {
            fn to_i64(&self) -> Option<i64> {
                self.to_integer().to_i64()
            }

            fn to_i128(&self) -> Option<i128> {
                self.to_integer().to_i128()
            }

            fn to_u64(&self) -> Option<u64> {
                self.to_integer().to_u64()
            }

            fn to_u128(&self) -> Option<u128> {
                self.to_integer().to_u128()
            }

            fn to_f64(&self) -> Option<f64> {
                let float = self.numer.to_f64().unwrap() / self.denom.to_f64().unwrap();
                if float.is_nan() {
                    None
                } else {
                    Some(float)
                }
            }
        }
    )*)
}

#[cfg(not(feature = "num-bigint"))]
to_primitive_small!(u8 i8 u16 i16 u32 i32);

#[cfg(all(target_pointer_width = "32", not(feature = "num-bigint")))]
to_primitive_small!(usize isize);

#[cfg(not(feature = "num-bigint"))]
macro_rules! to_primitive_64 {
    ($($type_name:ty)*) => ($(
        impl ToPrimitive for Ratio<$type_name> {
            fn to_i64(&self) -> Option<i64> {
                self.to_integer().to_i64()
            }

            fn to_i128(&self) -> Option<i128> {
                self.to_integer().to_i128()
            }

            fn to_u64(&self) -> Option<u64> {
                self.to_integer().to_u64()
            }

            fn to_u128(&self) -> Option<u128> {
                self.to_integer().to_u128()
            }

            fn to_f64(&self) -> Option<f64> {
                let float = ratio_to_f64(
                    self.numer as i128,
                    self.denom as i128
                );
                if float.is_nan() {
                    None
                } else {
                    Some(float)
                }
            }
        }
    )*)
}

#[cfg(not(feature = "num-bigint"))]
to_primitive_64!(u64 i64);

#[cfg(all(target_pointer_width = "64", not(feature = "num-bigint")))]
to_primitive_64!(usize isize);

#[cfg(feature = "num-bigint")]
impl<T: Clone + Integer + ToPrimitive + ToBigInt> ToPrimitive for Ratio<T> {
    fn to_i64(&self) -> Option<i64> {
        self.to_integer().to_i64()
    }

    fn to_i128(&self) -> Option<i128> {
        self.to_integer().to_i128()
    }

    fn to_u64(&self) -> Option<u64> {
        self.to_integer().to_u64()
    }

    fn to_u128(&self) -> Option<u128> {
        self.to_integer().to_u128()
    }

    fn to_f64(&self) -> Option<f64> {
        let float = match (self.numer.to_i64(), self.denom.to_i64()) {
            (Some(numer), Some(denom)) => ratio_to_f64(
                <i128 as From<_>>::from(numer),
                <i128 as From<_>>::from(denom),
            ),
            _ => {
                let numer: BigInt = self.numer.to_bigint()?;
                let denom: BigInt = self.denom.to_bigint()?;
                ratio_to_f64(numer, denom)
            }
        };
        if float.is_nan() {
            None
        } else {
            Some(float)
        }
    }
}

trait Bits {
    fn bits(&self) -> u64;
}

#[cfg(feature = "num-bigint")]
impl Bits for BigInt {
    fn bits(&self) -> u64 {
        self.bits()
    }
}

impl Bits for i128 {
    fn bits(&self) -> u64 {
        (128 - self.wrapping_abs().leading_zeros()).into()
    }
}

/// Converts a ratio of `T` to an f64.
///
/// In addition to stated trait bounds, `T` must be able to hold numbers 56 bits larger than
/// the largest of `numer` and `denom`. This is automatically true if `T` is `BigInt`.
fn ratio_to_f64<T: Bits + Clone + Integer + Signed + ShlAssign<usize> + ToPrimitive>(
    numer: T,
    denom: T,
) -> f64 {
    use core::f64::{INFINITY, MANTISSA_DIGITS, MAX_EXP, MIN_EXP, RADIX};

    assert_eq!(
        RADIX, 2,
        "only floating point implementations with radix 2 are supported"
    );

    // Inclusive upper and lower bounds to the range of exactly-representable ints in an f64.
    const MAX_EXACT_INT: i64 = 1i64 << MANTISSA_DIGITS;
    const MIN_EXACT_INT: i64 = -MAX_EXACT_INT;

    let flo_sign = numer.signum().to_f64().unwrap() / denom.signum().to_f64().unwrap();
    if !flo_sign.is_normal() {
        return flo_sign;
    }

    // Fast track: both sides can losslessly be converted to f64s. In this case, letting the
    // FPU do the job is faster and easier. In any other case, converting to f64s may lead
    // to an inexact result: https://stackoverflow.com/questions/56641441/.
    if let (Some(n), Some(d)) = (numer.to_i64(), denom.to_i64()) {
        let exact = MIN_EXACT_INT..=MAX_EXACT_INT;
        if exact.contains(&n) && exact.contains(&d) {
            return n.to_f64().unwrap() / d.to_f64().unwrap();
        }
    }

    // Otherwise, the goal is to obtain a quotient with at least 55 bits. 53 of these bits will
    // be used as the mantissa of the resulting float, and the remaining two are for rounding.
    // There's an error of up to 1 on the number of resulting bits, so we may get either 55 or
    // 56 bits.
    let mut numer = numer.abs();
    let mut denom = denom.abs();
    let (is_diff_positive, absolute_diff) = match numer.bits().checked_sub(denom.bits()) {
        Some(diff) => (true, diff),
        None => (false, denom.bits() - numer.bits()),
    };

    // Filter out overflows and underflows. After this step, the signed difference fits in an
    // isize.
    if is_diff_positive && absolute_diff > MAX_EXP as u64 {
        return INFINITY * flo_sign;
    }
    if !is_diff_positive && absolute_diff > -MIN_EXP as u64 + MANTISSA_DIGITS as u64 + 1 {
        return 0.0 * flo_sign;
    }
    let diff = if is_diff_positive {
        absolute_diff.to_isize().unwrap()
    } else {
        -absolute_diff.to_isize().unwrap()
    };

    // Shift is chosen so that the quotient will have 55 or 56 bits. The exception is if the
    // quotient is going to be subnormal, in which case it may have fewer bits.
    let shift: isize = diff.max(MIN_EXP as isize) - MANTISSA_DIGITS as isize - 2;
    if shift >= 0 {
        denom <<= shift as usize
    } else {
        numer <<= -shift as usize
    };

    let (quotient, remainder) = numer.div_rem(&denom);

    // This is guaranteed to fit since we've set up quotient to be at most 56 bits.
    let mut quotient = quotient.to_u64().unwrap();
    let n_rounding_bits = {
        let quotient_bits = 64 - quotient.leading_zeros() as isize;
        let subnormal_bits = MIN_EXP as isize - shift;
        quotient_bits.max(subnormal_bits) - MANTISSA_DIGITS as isize
    } as usize;
    debug_assert!(n_rounding_bits == 2 || n_rounding_bits == 3);
    let rounding_bit_mask = (1u64 << n_rounding_bits) - 1;

    // Round to 53 bits with round-to-even. For rounding, we need to take into account both
    // our rounding bits and the division's remainder.
    let ls_bit = quotient & (1u64 << n_rounding_bits) != 0;
    let ms_rounding_bit = quotient & (1u64 << (n_rounding_bits - 1)) != 0;
    let ls_rounding_bits = quotient & (rounding_bit_mask >> 1) != 0;
    if ms_rounding_bit && (ls_bit || ls_rounding_bits || !remainder.is_zero()) {
        quotient += 1u64 << n_rounding_bits;
    }
    quotient &= !rounding_bit_mask;

    // The quotient is guaranteed to be exactly representable as it's now 53 bits + 2 or 3
    // trailing zeros, so there is no risk of a rounding error here.
    let q_float = quotient as f64 * flo_sign;
    ldexp(q_float, shift as i32)
}

/// Multiply `x` by 2 to the power of `exp`. Returns an accurate result even if `2^exp` is not
/// representable.
pub(crate) fn ldexp(x: f64, exp: i32) -> f64 {
    use core::f64::{INFINITY, MANTISSA_DIGITS, MAX_EXP, RADIX};

    assert_eq!(
        RADIX, 2,
        "only floating point implementations with radix 2 are supported"
    );

    const EXPONENT_MASK: u64 = 0x7ff << 52;
    const MAX_UNSIGNED_EXPONENT: i32 = 0x7fe;
    const MIN_SUBNORMAL_POWER: i32 = MANTISSA_DIGITS as i32;

    if x.is_zero() || x.is_infinite() || x.is_nan() {
        return x;
    }

    // Filter out obvious over / underflows to make sure the resulting exponent fits in an isize.
    if exp > 3 * MAX_EXP {
        return INFINITY * x.signum();
    } else if exp < -3 * MAX_EXP {
        return 0.0 * x.signum();
    }

    // curr_exp is the x's *biased* exponent, and is in the [-54, MAX_UNSIGNED_EXPONENT] range.
    let (bits, curr_exp) = if !x.is_normal() {
        // If x is subnormal, we make it normal by multiplying by 2^53. This causes no loss of
        // precision or rounding.
        let normal_x = x * 2f64.powi(MIN_SUBNORMAL_POWER);
        let bits = normal_x.to_bits();
        // This cast is safe because the exponent is at most 0x7fe, which fits in an i32.
        (
            bits,
            ((bits & EXPONENT_MASK) >> 52) as i32 - MIN_SUBNORMAL_POWER,
        )
    } else {
        let bits = x.to_bits();
        let curr_exp = (bits & EXPONENT_MASK) >> 52;
        // This cast is safe because the exponent is at most 0x7fe, which fits in an i32.
        (bits, curr_exp as i32)
    };

    // The addition can't overflow because exponent is between 0 and 0x7fe, and exp is between
    // -2*MAX_EXP and 2*MAX_EXP.
    let new_exp = curr_exp + exp;

    if new_exp > MAX_UNSIGNED_EXPONENT {
        INFINITY * x.signum()
    } else if new_exp > 0 {
        // Normal case: exponent is not too large nor subnormal.
        let new_bits = (bits & !EXPONENT_MASK) | ((new_exp as u64) << 52);
        f64::from_bits(new_bits)
    } else if new_exp >= -(MANTISSA_DIGITS as i32) {
        // Result is subnormal but may not be zero.
        // In this case, we increase the exponent by 54 to make it normal, then multiply the end
        // result by 2^-53. This results in a single multiplication with no prior rounding error,
        // so there is no risk of double rounding.
        let new_exp = new_exp + MIN_SUBNORMAL_POWER;
        debug_assert!(new_exp >= 0);
        let new_bits = (bits & !EXPONENT_MASK) | ((new_exp as u64) << 52);
        f64::from_bits(new_bits) * 2f64.powi(-MIN_SUBNORMAL_POWER)
    } else {
        // Result is zero.
        return 0.0 * x.signum();
    }
}

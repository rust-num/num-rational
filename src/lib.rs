// Copyright 2013-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Rational numbers
//!
//! ## Compatibility
//!
//! The `num-rational` crate is tested for rustc 1.15 and greater.

#![doc(html_root_url = "https://docs.rs/num-rational/0.2")]
#![no_std]

#[cfg(feature = "bigint")]
extern crate num_bigint as bigint;
#[cfg(feature = "serde")]
extern crate serde;

extern crate num_integer as integer;
extern crate num_traits as traits;

#[cfg(feature = "std")]
#[cfg_attr(test, macro_use)]
extern crate std;

use core::cmp;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::{Add, Div, Mul, Neg, Rem, Sub};
use core::str::FromStr;
#[cfg(feature = "std")]
use std::error::Error;

#[cfg(feature = "bigint")]
use bigint::{BigInt, BigUint, Sign};

use integer::Integer;
use traits::float::FloatCore;
use traits::{
    Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, FromPrimitive, Inv, Num, NumCast, One,
    Pow, Signed, Zero,
};

/// Represents the ratio between two numbers.
#[derive(Copy, Clone, Debug)]
#[allow(missing_docs)]
pub struct Ratio<T> {
    /// Numerator.
    numer: T,
    /// Denominator.
    denom: T,
}

/// Alias for a `Ratio` of machine-sized integers.
pub type Rational = Ratio<isize>;
/// Alias for a `Ratio` of 32-bit-sized integers.
pub type Rational32 = Ratio<i32>;
/// Alias for a `Ratio` of 64-bit-sized integers.
pub type Rational64 = Ratio<i64>;

#[cfg(feature = "bigint")]
/// Alias for arbitrary precision rationals.
pub type BigRational = Ratio<BigInt>;

impl<T: Clone + Integer> Ratio<T> {
    /// Creates a new `Ratio`. Fails if `denom` is zero.
    #[inline]
    pub fn new(numer: T, denom: T) -> Ratio<T> {
        if denom.is_zero() {
            panic!("denominator == 0");
        }
        let mut ret = Ratio::new_raw(numer, denom);
        ret.reduce();
        ret
    }

    /// Creates a `Ratio` representing the integer `t`.
    #[inline]
    pub fn from_integer(t: T) -> Ratio<T> {
        Ratio::new_raw(t, One::one())
    }

    /// Creates a `Ratio` without checking for `denom == 0` or reducing.
    #[inline]
    pub fn new_raw(numer: T, denom: T) -> Ratio<T> {
        Ratio {
            numer: numer,
            denom: denom,
        }
    }

    /// Converts to an integer, rounding towards zero.
    #[inline]
    pub fn to_integer(&self) -> T {
        self.trunc().numer
    }

    /// Gets an immutable reference to the numerator.
    #[inline]
    pub fn numer<'a>(&'a self) -> &'a T {
        &self.numer
    }

    /// Gets an immutable reference to the denominator.
    #[inline]
    pub fn denom<'a>(&'a self) -> &'a T {
        &self.denom
    }

    /// Returns true if the rational number is an integer (denominator is 1).
    #[inline]
    pub fn is_integer(&self) -> bool {
        self.denom.is_one()
    }

    /// Puts self into lowest terms, with denom > 0.
    fn reduce(&mut self) {
        let g: T = self.numer.gcd(&self.denom);

        // FIXME(#5992): assignment operator overloads
        // self.numer /= g;
        // T: Clone + Integer != T: Clone + NumAssign
        self.numer = self.numer.clone() / g.clone();
        // FIXME(#5992): assignment operator overloads
        // self.denom /= g;
        // T: Clone + Integer != T: Clone + NumAssign
        self.denom = self.denom.clone() / g;

        // keep denom positive!
        if self.denom < T::zero() {
            self.numer = T::zero() - self.numer.clone();
            self.denom = T::zero() - self.denom.clone();
        }
    }

    /// Returns a reduced copy of self.
    ///
    /// In general, it is not necessary to use this method, as the only
    /// method of procuring a non-reduced fraction is through `new_raw`.
    pub fn reduced(&self) -> Ratio<T> {
        let mut ret = self.clone();
        ret.reduce();
        ret
    }

    /// Returns the reciprocal.
    ///
    /// Fails if the `Ratio` is zero.
    #[inline]
    pub fn recip(&self) -> Ratio<T> {
        match self.numer.cmp(&T::zero()) {
            cmp::Ordering::Equal => panic!("numerator == 0"),
            cmp::Ordering::Greater => Ratio::new_raw(self.denom.clone(), self.numer.clone()),
            cmp::Ordering::Less => Ratio::new_raw(
                T::zero() - self.denom.clone(),
                T::zero() - self.numer.clone(),
            ),
        }
    }

    /// Rounds towards minus infinity.
    #[inline]
    pub fn floor(&self) -> Ratio<T> {
        if *self < Zero::zero() {
            let one: T = One::one();
            Ratio::from_integer(
                (self.numer.clone() - self.denom.clone() + one) / self.denom.clone(),
            )
        } else {
            Ratio::from_integer(self.numer.clone() / self.denom.clone())
        }
    }

    /// Rounds towards plus infinity.
    #[inline]
    pub fn ceil(&self) -> Ratio<T> {
        if *self < Zero::zero() {
            Ratio::from_integer(self.numer.clone() / self.denom.clone())
        } else {
            let one: T = One::one();
            Ratio::from_integer(
                (self.numer.clone() + self.denom.clone() - one) / self.denom.clone(),
            )
        }
    }

    /// Rounds to the nearest integer. Rounds half-way cases away from zero.
    #[inline]
    pub fn round(&self) -> Ratio<T> {
        let zero: Ratio<T> = Zero::zero();
        let one: T = One::one();
        let two: T = one.clone() + one.clone();

        // Find unsigned fractional part of rational number
        let mut fractional = self.fract();
        if fractional < zero {
            fractional = zero - fractional
        };

        // The algorithm compares the unsigned fractional part with 1/2, that
        // is, a/b >= 1/2, or a >= b/2. For odd denominators, we use
        // a >= (b/2)+1. This avoids overflow issues.
        let half_or_larger = if fractional.denom().is_even() {
            *fractional.numer() >= fractional.denom().clone() / two.clone()
        } else {
            *fractional.numer() >= (fractional.denom().clone() / two.clone()) + one.clone()
        };

        if half_or_larger {
            let one: Ratio<T> = One::one();
            if *self >= Zero::zero() {
                self.trunc() + one
            } else {
                self.trunc() - one
            }
        } else {
            self.trunc()
        }
    }

    /// Rounds towards zero.
    #[inline]
    pub fn trunc(&self) -> Ratio<T> {
        Ratio::from_integer(self.numer.clone() / self.denom.clone())
    }

    /// Returns the fractional part of a number, with division rounded towards zero.
    ///
    /// Satisfies `self == self.trunc() + self.fract()`.
    #[inline]
    pub fn fract(&self) -> Ratio<T> {
        Ratio::new_raw(self.numer.clone() % self.denom.clone(), self.denom.clone())
    }
}

impl<T: Clone + Integer + Pow<u32, Output = T>> Ratio<T> {
    /// Raises the `Ratio` to the power of an exponent.
    #[inline]
    pub fn pow(&self, expon: i32) -> Ratio<T> {
        Pow::pow(self, expon)
    }
}

macro_rules! pow_impl {
    ($exp:ty) => {
        pow_impl!($exp, $exp);
    };
    ($exp:ty, $unsigned:ty) => {
        impl<T: Clone + Integer + Pow<$unsigned, Output = T>> Pow<$exp> for Ratio<T> {
            type Output = Ratio<T>;
            #[inline]
            fn pow(self, expon: $exp) -> Ratio<T> {
                match expon.cmp(&0) {
                    cmp::Ordering::Equal => One::one(),
                    cmp::Ordering::Less => {
                        let expon = expon.wrapping_abs() as $unsigned;
                        Ratio::new_raw(Pow::pow(self.denom, expon), Pow::pow(self.numer, expon))
                    }
                    cmp::Ordering::Greater => Ratio::new_raw(
                        Pow::pow(self.numer, expon as $unsigned),
                        Pow::pow(self.denom, expon as $unsigned),
                    ),
                }
            }
        }
        impl<'a, T: Clone + Integer + Pow<$unsigned, Output = T>> Pow<$exp> for &'a Ratio<T> {
            type Output = Ratio<T>;
            #[inline]
            fn pow(self, expon: $exp) -> Ratio<T> {
                Pow::pow(self.clone(), expon)
            }
        }
        impl<'a, T: Clone + Integer + Pow<$unsigned, Output = T>> Pow<&'a $exp> for Ratio<T> {
            type Output = Ratio<T>;
            #[inline]
            fn pow(self, expon: &'a $exp) -> Ratio<T> {
                Pow::pow(self, *expon)
            }
        }
        impl<'a, 'b, T: Clone + Integer + Pow<$unsigned, Output = T>> Pow<&'a $exp>
            for &'b Ratio<T>
        {
            type Output = Ratio<T>;
            #[inline]
            fn pow(self, expon: &'a $exp) -> Ratio<T> {
                Pow::pow(self.clone(), *expon)
            }
        }
    };
}

// this is solely to make `pow_impl!` work
trait WrappingAbs: Sized {
    fn wrapping_abs(self) -> Self {
        self
    }
}
impl WrappingAbs for u8 {}
impl WrappingAbs for u16 {}
impl WrappingAbs for u32 {}
impl WrappingAbs for u64 {}
impl WrappingAbs for usize {}

pow_impl!(i8, u8);
pow_impl!(i16, u16);
pow_impl!(i32, u32);
pow_impl!(i64, u64);
pow_impl!(isize, usize);
pow_impl!(u8);
pow_impl!(u16);
pow_impl!(u32);
pow_impl!(u64);
pow_impl!(usize);

// TODO: pow_impl!(BigUint) and pow_impl!(BigInt, BigUint)

#[cfg(feature = "bigint")]
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
            numer = numer << (exponent as usize);
            Some(Ratio::from_integer(BigInt::from_biguint(
                bigint_sign,
                numer,
            )))
        }
    }
}

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

// Comparisons

// Mathematically, comparing a/b and c/d is the same as comparing a*d and b*c, but it's very easy
// for those multiplications to overflow fixed-size integers, so we need to take care.

impl<T: Clone + Integer> Ord for Ratio<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        // With equal denominators, the numerators can be directly compared
        if self.denom == other.denom {
            let ord = self.numer.cmp(&other.numer);
            return if self.denom < T::zero() {
                ord.reverse()
            } else {
                ord
            };
        }

        // With equal numerators, the denominators can be inversely compared
        if self.numer == other.numer {
            let ord = self.denom.cmp(&other.denom);
            return if self.numer < T::zero() {
                ord
            } else {
                ord.reverse()
            };
        }

        // Unfortunately, we don't have CheckedMul to try.  That could sometimes avoid all the
        // division below, or even always avoid it for BigInt and BigUint.
        // FIXME- future breaking change to add Checked* to Integer?

        // Compare as floored integers and remainders
        let (self_int, self_rem) = self.numer.div_mod_floor(&self.denom);
        let (other_int, other_rem) = other.numer.div_mod_floor(&other.denom);
        match self_int.cmp(&other_int) {
            cmp::Ordering::Greater => cmp::Ordering::Greater,
            cmp::Ordering::Less => cmp::Ordering::Less,
            cmp::Ordering::Equal => {
                match (self_rem.is_zero(), other_rem.is_zero()) {
                    (true, true) => cmp::Ordering::Equal,
                    (true, false) => cmp::Ordering::Less,
                    (false, true) => cmp::Ordering::Greater,
                    (false, false) => {
                        // Compare the reciprocals of the remaining fractions in reverse
                        let self_recip = Ratio::new_raw(self.denom.clone(), self_rem);
                        let other_recip = Ratio::new_raw(other.denom.clone(), other_rem);
                        self_recip.cmp(&other_recip).reverse()
                    }
                }
            }
        }
    }
}

impl<T: Clone + Integer> PartialOrd for Ratio<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Clone + Integer> PartialEq for Ratio<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == cmp::Ordering::Equal
    }
}

impl<T: Clone + Integer> Eq for Ratio<T> {}

// NB: We can't just `#[derive(Hash)]`, because it needs to agree
// with `Eq` even for non-reduced ratios.
impl<T: Clone + Integer + Hash> Hash for Ratio<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        recurse(&self.numer, &self.denom, state);

        fn recurse<T: Integer + Hash, H: Hasher>(numer: &T, denom: &T, state: &mut H) {
            if !denom.is_zero() {
                let (int, rem) = numer.div_mod_floor(denom);
                int.hash(state);
                recurse(denom, &rem, state);
            } else {
                denom.hash(state);
            }
        }
    }
}

mod iter_sum_product {
    use core::iter::{Product, Sum};
    use integer::Integer;
    use traits::{One, Zero};
    use Ratio;

    impl<T: Integer + Clone> Sum for Ratio<T> {
        fn sum<I>(iter: I) -> Self
        where
            I: Iterator<Item = Ratio<T>>,
        {
            iter.fold(Self::zero(), |sum, num| sum + num)
        }
    }

    impl<'a, T: Integer + Clone> Sum<&'a Ratio<T>> for Ratio<T> {
        fn sum<I>(iter: I) -> Self
        where
            I: Iterator<Item = &'a Ratio<T>>,
        {
            iter.fold(Self::zero(), |sum, num| sum + num)
        }
    }

    impl<T: Integer + Clone> Product for Ratio<T> {
        fn product<I>(iter: I) -> Self
        where
            I: Iterator<Item = Ratio<T>>,
        {
            iter.fold(Self::one(), |prod, num| prod * num)
        }
    }

    impl<'a, T: Integer + Clone> Product<&'a Ratio<T>> for Ratio<T> {
        fn product<I>(iter: I) -> Self
        where
            I: Iterator<Item = &'a Ratio<T>>,
        {
            iter.fold(Self::one(), |prod, num| prod * num)
        }
    }
}

mod opassign {
    use core::ops::{AddAssign, DivAssign, MulAssign, RemAssign, SubAssign};

    use integer::Integer;
    use traits::NumAssign;
    use Ratio;

    impl<T: Clone + Integer + NumAssign> AddAssign for Ratio<T> {
        fn add_assign(&mut self, other: Ratio<T>) {
            self.numer *= other.denom.clone();
            self.numer += self.denom.clone() * other.numer;
            self.denom *= other.denom;
            self.reduce();
        }
    }

    impl<T: Clone + Integer + NumAssign> DivAssign for Ratio<T> {
        fn div_assign(&mut self, other: Ratio<T>) {
            self.numer *= other.denom;
            self.denom *= other.numer;
            self.reduce();
        }
    }

    impl<T: Clone + Integer + NumAssign> MulAssign for Ratio<T> {
        fn mul_assign(&mut self, other: Ratio<T>) {
            self.numer *= other.numer;
            self.denom *= other.denom;
            self.reduce();
        }
    }

    impl<T: Clone + Integer + NumAssign> RemAssign for Ratio<T> {
        fn rem_assign(&mut self, other: Ratio<T>) {
            self.numer *= other.denom.clone();
            self.numer %= self.denom.clone() * other.numer;
            self.denom *= other.denom;
            self.reduce();
        }
    }

    impl<T: Clone + Integer + NumAssign> SubAssign for Ratio<T> {
        fn sub_assign(&mut self, other: Ratio<T>) {
            self.numer *= other.denom.clone();
            self.numer -= self.denom.clone() * other.numer;
            self.denom *= other.denom;
            self.reduce();
        }
    }

    // a/b + c/1 = (a*1 + b*c) / (b*1) = (a + b*c) / b
    impl<T: Clone + Integer + NumAssign> AddAssign<T> for Ratio<T> {
        fn add_assign(&mut self, other: T) {
            self.numer += self.denom.clone() * other;
            self.reduce();
        }
    }

    impl<T: Clone + Integer + NumAssign> DivAssign<T> for Ratio<T> {
        fn div_assign(&mut self, other: T) {
            self.denom *= other;
            self.reduce();
        }
    }

    impl<T: Clone + Integer + NumAssign> MulAssign<T> for Ratio<T> {
        fn mul_assign(&mut self, other: T) {
            self.numer *= other;
            self.reduce();
        }
    }

    // a/b % c/1 = (a*1 % b*c) / (b*1) = (a % b*c) / b
    impl<T: Clone + Integer + NumAssign> RemAssign<T> for Ratio<T> {
        fn rem_assign(&mut self, other: T) {
            self.numer %= self.denom.clone() * other;
            self.reduce();
        }
    }

    // a/b - c/1 = (a*1 - b*c) / (b*1) = (a - b*c) / b
    impl<T: Clone + Integer + NumAssign> SubAssign<T> for Ratio<T> {
        fn sub_assign(&mut self, other: T) {
            self.numer -= self.denom.clone() * other;
            self.reduce();
        }
    }

    macro_rules! forward_op_assign {
        (impl $imp:ident, $method:ident) => {
            impl<'a, T: Clone + Integer + NumAssign> $imp<&'a Ratio<T>> for Ratio<T> {
                #[inline]
                fn $method(&mut self, other: &Ratio<T>) {
                    self.$method(other.clone())
                }
            }
            impl<'a, T: Clone + Integer + NumAssign> $imp<&'a T> for Ratio<T> {
                #[inline]
                fn $method(&mut self, other: &T) {
                    self.$method(other.clone())
                }
            }
        };
    }

    forward_op_assign!(impl AddAssign, add_assign);
    forward_op_assign!(impl DivAssign, div_assign);
    forward_op_assign!(impl MulAssign, mul_assign);
    forward_op_assign!(impl RemAssign, rem_assign);
    forward_op_assign!(impl SubAssign, sub_assign);
}

macro_rules! forward_ref_ref_binop {
    (impl $imp:ident, $method:ident) => {
        impl<'a, 'b, T: Clone + Integer> $imp<&'b Ratio<T>> for &'a Ratio<T> {
            type Output = Ratio<T>;

            #[inline]
            fn $method(self, other: &'b Ratio<T>) -> Ratio<T> {
                self.clone().$method(other.clone())
            }
        }
        impl<'a, 'b, T: Clone + Integer> $imp<&'b T> for &'a Ratio<T> {
            type Output = Ratio<T>;

            #[inline]
            fn $method(self, other: &'b T) -> Ratio<T> {
                self.clone().$method(other.clone())
            }
        }
    };
}

macro_rules! forward_ref_val_binop {
    (impl $imp:ident, $method:ident) => {
        impl<'a, T> $imp<Ratio<T>> for &'a Ratio<T>
        where
            T: Clone + Integer,
        {
            type Output = Ratio<T>;

            #[inline]
            fn $method(self, other: Ratio<T>) -> Ratio<T> {
                self.clone().$method(other)
            }
        }
        impl<'a, T> $imp<T> for &'a Ratio<T>
        where
            T: Clone + Integer,
        {
            type Output = Ratio<T>;

            #[inline]
            fn $method(self, other: T) -> Ratio<T> {
                self.clone().$method(other)
            }
        }
    };
}

macro_rules! forward_val_ref_binop {
    (impl $imp:ident, $method:ident) => {
        impl<'a, T> $imp<&'a Ratio<T>> for Ratio<T>
        where
            T: Clone + Integer,
        {
            type Output = Ratio<T>;

            #[inline]
            fn $method(self, other: &Ratio<T>) -> Ratio<T> {
                self.$method(other.clone())
            }
        }
        impl<'a, T> $imp<&'a T> for Ratio<T>
        where
            T: Clone + Integer,
        {
            type Output = Ratio<T>;

            #[inline]
            fn $method(self, other: &T) -> Ratio<T> {
                self.$method(other.clone())
            }
        }
    };
}

macro_rules! forward_all_binop {
    (impl $imp:ident, $method:ident) => {
        forward_ref_ref_binop!(impl $imp, $method);
        forward_ref_val_binop!(impl $imp, $method);
        forward_val_ref_binop!(impl $imp, $method);
    };
}

// Arithmetic
forward_all_binop!(impl Mul, mul);
// a/b * c/d = (a*c)/(b*d)
impl<T> Mul<Ratio<T>> for Ratio<T>
where
    T: Clone + Integer,
{
    type Output = Ratio<T>;
    #[inline]
    fn mul(self, rhs: Ratio<T>) -> Ratio<T> {
        Ratio::new(self.numer * rhs.numer, self.denom * rhs.denom)
    }
}
// a/b * c/1 = (a*c) / (b*1) = (a*c) / b
impl<T> Mul<T> for Ratio<T>
where
    T: Clone + Integer,
{
    type Output = Ratio<T>;
    #[inline]
    fn mul(self, rhs: T) -> Ratio<T> {
        Ratio::new(self.numer * rhs, self.denom)
    }
}

macro_rules! impl_primitive_ops_ratio {
    (Ratio<T> for $primitive:ident) => {
        impl<T> Add<Ratio<T>> for $primitive
        where
            T: Clone + Integer + ::core::convert::From<$primitive>
        {
            type Output = Ratio<T>;

            #[inline]
            fn add(self, rhs: Ratio<T>) -> Ratio<T> {
                Ratio::new(rhs.numer + T::from(self), rhs.denom)
            }
        }

        impl<T> Sub<Ratio<T>> for $primitive
        where
            T: Clone + Integer + ::core::convert::From<$primitive>
        {
            type Output = Ratio<T>;

            #[inline]
            fn sub(self, rhs: Ratio<T>) -> Ratio<T> {
                Ratio::new(T::from(self) - rhs.numer, rhs.denom)
            }
        }

        impl<T> Mul<Ratio<T>> for $primitive
        where
            T: Clone + Integer + ::core::convert::From<$primitive>
        {
            type Output = Ratio<T>;

            #[inline]
            fn mul(self, rhs: Ratio<T>) -> Ratio<T> {
                Ratio::new(rhs.numer * T::from(self), rhs.denom)
            }
        }

        impl<T> Div<Ratio<T>> for $primitive
        where
            T: Clone + Integer + ::core::convert::From<$primitive>
        {
            type Output = Ratio<T>;

            #[inline]
            fn div(self, rhs: Ratio<T>) -> Ratio<T> {
                Ratio::new(rhs.numer, rhs.denom * T::from(self))
            }
        }
    };
}

// Implements for isize and BigInt
impl_primitive_ops_ratio!(Ratio<T> for isize);
// Implements for BigInt
impl_primitive_ops_ratio!(Ratio<T> for usize);
// Implements for BigInt, i64, i32, i8
impl_primitive_ops_ratio!(Ratio<T> for i8);
// Implements for isize, i32, i64, BigInt
impl_primitive_ops_ratio!(Ratio<T> for u8);
// Implements for isize, i32, i64, BigInt
impl_primitive_ops_ratio!(Ratio<T> for i16);
// Implements for i32, i64, BigInt
impl_primitive_ops_ratio!(Ratio<T> for u16);
// Implements for i32, i64, BigInt
impl_primitive_ops_ratio!(Ratio<T> for i32);
// Implements for i64, BigInt
impl_primitive_ops_ratio!(Ratio<T> for u32);
// Implements for i64, BigInt
impl_primitive_ops_ratio!(Ratio<T> for i64);
// Implements for BigInt
impl_primitive_ops_ratio!(Ratio<T> for u64);
// Implements for BigInt
#[cfg(feature = "i128")]
impl_primitive_ops_ratio!(Ratio<T> for i128);
// Implements for BigInt
#[cfg(feature = "i128")]
impl_primitive_ops_ratio!(Ratio<T> for u128);

// Implementing these directly because the implementation of Add<T>, Sub<T>,
// Mul<T> and Div<T> for Ratio<T> don't cover these.
macro_rules! impl_bigint_ops_primitive {
    ($primitive:ident) => {
        impl Add<$primitive> for BigRational {
            type Output = BigRational;

            #[inline]
            fn add(self, rhs: $primitive) -> BigRational {
                Ratio::new(self.numer + BigInt::from(rhs), self.denom)
            }
        }

        impl Sub<$primitive> for BigRational {
            type Output = BigRational;

            #[inline]
            fn sub(self, rhs: $primitive) -> BigRational {
                Ratio::new(self.numer - BigInt::from(rhs), self.denom)
            }
        }

        impl Mul<$primitive> for BigRational {
            type Output = BigRational;

            #[inline]
            fn mul(self, rhs: $primitive) -> BigRational {
                Ratio::new(self.numer * BigInt::from(rhs), self.denom)
            }
        }

        impl Div<$primitive> for BigRational {
            type Output = BigRational;

            #[inline]
            fn div(self, rhs: $primitive) -> BigRational {
                Ratio::new(self.numer, self.denom * BigInt::from(rhs))
            }
        }
    };
}

#[cfg(feature = "bigint")]
impl_bigint_ops_primitive!(isize);
#[cfg(feature = "bigint")]
impl_bigint_ops_primitive!(usize);
#[cfg(feature = "bigint")]
impl_bigint_ops_primitive!(i8);
#[cfg(feature = "bigint")]
impl_bigint_ops_primitive!(u8);
#[cfg(feature = "bigint")]
impl_bigint_ops_primitive!(i16);
#[cfg(feature = "bigint")]
impl_bigint_ops_primitive!(u16);
#[cfg(feature = "bigint")]
impl_bigint_ops_primitive!(i32);
#[cfg(feature = "bigint")]
impl_bigint_ops_primitive!(u32);
#[cfg(feature = "bigint")]
impl_bigint_ops_primitive!(i64);
#[cfg(feature = "bigint")]
impl_bigint_ops_primitive!(u64);
#[cfg(all(feature = "i128", feature = "bigint"))]
impl_bigint_ops_primitive!(i128);
#[cfg(all(feature = "i128", feature = "bigint"))]
impl_bigint_ops_primitive!(u128);

forward_all_binop!(impl Div, div);
// (a/b) / (c/d) = (a*d) / (b*c)
impl<T> Div<Ratio<T>> for Ratio<T>
where
    T: Clone + Integer,
{
    type Output = Ratio<T>;

    #[inline]
    fn div(self, rhs: Ratio<T>) -> Ratio<T> {
        Ratio::new(self.numer * rhs.denom, self.denom * rhs.numer)
    }
}
// (a/b) / (c/1) = (a*1) / (b*c) = a / (b*c)
impl<T> Div<T> for Ratio<T>
where
    T: Clone + Integer,
{
    type Output = Ratio<T>;

    #[inline]
    fn div(self, rhs: T) -> Ratio<T> {
        Ratio::new(self.numer, self.denom * rhs)
    }
}

macro_rules! arith_impl {
    (impl $imp:ident, $method:ident) => {
        forward_all_binop!(impl $imp, $method);
        // Abstracts the a/b `op` c/d = (a*d `op` b*c) / (b*d) pattern
        impl<T: Clone + Integer> $imp<Ratio<T>> for Ratio<T> {
            type Output = Ratio<T>;
            #[inline]
            fn $method(self, rhs: Ratio<T>) -> Ratio<T> {
                Ratio::new(
                    (self.numer * rhs.denom.clone()).$method(self.denom.clone() * rhs.numer),
                    self.denom * rhs.denom,
                )
            }
        }
        // Abstracts the a/b `op` c/1 = (a*1 `op` b*c) / (b*1) = (a `op` b*c) / b pattern
        impl<T: Clone + Integer> $imp<T> for Ratio<T> {
            type Output = Ratio<T>;
            #[inline]
            fn $method(self, rhs: T) -> Ratio<T> {
                Ratio::new(self.numer.$method(self.denom.clone() * rhs), self.denom)
            }
        }
    };
}

arith_impl!(impl Add, add);
arith_impl!(impl Sub, sub);
arith_impl!(impl Rem, rem);

// Like `std::try!` for Option<T>, unwrap the value or early-return None.
// Since Rust 1.22 this can be replaced by the `?` operator.
macro_rules! otry {
    ($expr:expr) => {
        match $expr {
            Some(val) => val,
            None => return None,
        }
    };
}

// a/b * c/d = (a*c)/(b*d)
impl<T> CheckedMul for Ratio<T>
where
    T: Clone + Integer + CheckedMul,
{
    #[inline]
    fn checked_mul(&self, rhs: &Ratio<T>) -> Option<Ratio<T>> {
        Some(Ratio::new(
            otry!(self.numer.checked_mul(&rhs.numer)),
            otry!(self.denom.checked_mul(&rhs.denom)),
        ))
    }
}

// (a/b) / (c/d) = (a*d)/(b*c)
impl<T> CheckedDiv for Ratio<T>
where
    T: Clone + Integer + CheckedMul,
{
    #[inline]
    fn checked_div(&self, rhs: &Ratio<T>) -> Option<Ratio<T>> {
        let bc = otry!(self.denom.checked_mul(&rhs.numer));
        if bc.is_zero() {
            None
        } else {
            Some(Ratio::new(otry!(self.numer.checked_mul(&rhs.denom)), bc))
        }
    }
}

// As arith_impl! but for Checked{Add,Sub} traits
macro_rules! checked_arith_impl {
    (impl $imp:ident, $method:ident) => {
        impl<T: Clone + Integer + CheckedMul + $imp> $imp for Ratio<T> {
            #[inline]
            fn $method(&self, rhs: &Ratio<T>) -> Option<Ratio<T>> {
                let ad = otry!(self.numer.checked_mul(&rhs.denom));
                let bc = otry!(self.denom.checked_mul(&rhs.numer));
                let bd = otry!(self.denom.checked_mul(&rhs.denom));
                Some(Ratio::new(otry!(ad.$method(&bc)), bd))
            }
        }
    };
}

// a/b + c/d = (a*d + b*c)/(b*d)
checked_arith_impl!(impl CheckedAdd, checked_add);

// a/b - c/d = (a*d - b*c)/(b*d)
checked_arith_impl!(impl CheckedSub, checked_sub);

impl<T> Neg for Ratio<T>
where
    T: Clone + Integer + Neg<Output = T>,
{
    type Output = Ratio<T>;

    #[inline]
    fn neg(self) -> Ratio<T> {
        Ratio::new_raw(-self.numer, self.denom)
    }
}

impl<'a, T> Neg for &'a Ratio<T>
where
    T: Clone + Integer + Neg<Output = T>,
{
    type Output = Ratio<T>;

    #[inline]
    fn neg(self) -> Ratio<T> {
        -self.clone()
    }
}

impl<T> Inv for Ratio<T>
where
    T: Clone + Integer,
{
    type Output = Ratio<T>;

    #[inline]
    fn inv(self) -> Ratio<T> {
        self.recip()
    }
}

impl<'a, T> Inv for &'a Ratio<T>
where
    T: Clone + Integer,
{
    type Output = Ratio<T>;

    #[inline]
    fn inv(self) -> Ratio<T> {
        self.recip()
    }
}

// Constants
impl<T: Clone + Integer> Zero for Ratio<T> {
    #[inline]
    fn zero() -> Ratio<T> {
        Ratio::new_raw(Zero::zero(), One::one())
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.numer.is_zero()
    }
}

impl<T: Clone + Integer> One for Ratio<T> {
    #[inline]
    fn one() -> Ratio<T> {
        Ratio::new_raw(One::one(), One::one())
    }

    #[inline]
    fn is_one(&self) -> bool {
        self.numer == self.denom
    }
}

impl<T: Clone + Integer> Num for Ratio<T> {
    type FromStrRadixErr = ParseRatioError;

    /// Parses `numer/denom` where the numbers are in base `radix`.
    fn from_str_radix(s: &str, radix: u32) -> Result<Ratio<T>, ParseRatioError> {
        if s.splitn(2, '/').count() == 2 {
            let mut parts = s.splitn(2, '/').map(|ss| {
                T::from_str_radix(ss, radix).map_err(|_| ParseRatioError {
                    kind: RatioErrorKind::ParseError,
                })
            });
            let numer: T = parts.next().unwrap()?;
            let denom: T = parts.next().unwrap()?;
            if denom.is_zero() {
                Err(ParseRatioError {
                    kind: RatioErrorKind::ZeroDenominator,
                })
            } else {
                Ok(Ratio::new(numer, denom))
            }
        } else {
            Err(ParseRatioError {
                kind: RatioErrorKind::ParseError,
            })
        }
    }
}

impl<T: Clone + Integer + Signed> Signed for Ratio<T> {
    #[inline]
    fn abs(&self) -> Ratio<T> {
        if self.is_negative() {
            -self.clone()
        } else {
            self.clone()
        }
    }

    #[inline]
    fn abs_sub(&self, other: &Ratio<T>) -> Ratio<T> {
        if *self <= *other {
            Zero::zero()
        } else {
            self - other
        }
    }

    #[inline]
    fn signum(&self) -> Ratio<T> {
        if self.is_positive() {
            Self::one()
        } else if self.is_zero() {
            Self::zero()
        } else {
            -Self::one()
        }
    }

    #[inline]
    fn is_positive(&self) -> bool {
        (self.numer.is_positive() && self.denom.is_positive())
            || (self.numer.is_negative() && self.denom.is_negative())
    }

    #[inline]
    fn is_negative(&self) -> bool {
        (self.numer.is_negative() && self.denom.is_positive())
            || (self.numer.is_positive() && self.denom.is_negative())
    }
}

// String conversions
impl<T> fmt::Display for Ratio<T>
where
    T: fmt::Display + Eq + One,
{
    /// Renders as `numer/denom`. If denom=1, renders as numer.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.denom.is_one() {
            write!(f, "{}", self.numer)
        } else {
            write!(f, "{}/{}", self.numer, self.denom)
        }
    }
}

impl<T: FromStr + Clone + Integer> FromStr for Ratio<T> {
    type Err = ParseRatioError;

    /// Parses `numer/denom` or just `numer`.
    fn from_str(s: &str) -> Result<Ratio<T>, ParseRatioError> {
        let mut split = s.splitn(2, '/');

        let n = try!(split.next().ok_or(ParseRatioError {
            kind: RatioErrorKind::ParseError
        }));
        let num = try!(FromStr::from_str(n).map_err(|_| ParseRatioError {
            kind: RatioErrorKind::ParseError
        }));

        let d = split.next().unwrap_or("1");
        let den = try!(FromStr::from_str(d).map_err(|_| ParseRatioError {
            kind: RatioErrorKind::ParseError
        }));

        if Zero::is_zero(&den) {
            Err(ParseRatioError {
                kind: RatioErrorKind::ZeroDenominator,
            })
        } else {
            Ok(Ratio::new(num, den))
        }
    }
}

impl<T> Into<(T, T)> for Ratio<T> {
    fn into(self) -> (T, T) {
        (self.numer, self.denom)
    }
}

#[cfg(feature = "serde")]
impl<T> serde::Serialize for Ratio<T>
where
    T: serde::Serialize + Clone + Integer + PartialOrd,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (self.numer(), self.denom()).serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T> serde::Deserialize<'de> for Ratio<T>
where
    T: serde::Deserialize<'de> + Clone + Integer + PartialOrd,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        use serde::de::Unexpected;
        let (numer, denom): (T, T) = try!(serde::Deserialize::deserialize(deserializer));
        if denom.is_zero() {
            Err(Error::invalid_value(
                Unexpected::Signed(0),
                &"a ratio with non-zero denominator",
            ))
        } else {
            Ok(Ratio::new_raw(numer, denom))
        }
    }
}

// FIXME: Bubble up specific errors
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ParseRatioError {
    kind: RatioErrorKind,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum RatioErrorKind {
    ParseError,
    ZeroDenominator,
}

impl fmt::Display for ParseRatioError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.kind.description().fmt(f)
    }
}

#[cfg(feature = "std")]
impl Error for ParseRatioError {
    fn description(&self) -> &str {
        self.kind.description()
    }
}

impl RatioErrorKind {
    fn description(self) -> &'static str {
        match self {
            RatioErrorKind::ParseError => "failed to parse integer",
            RatioErrorKind::ZeroDenominator => "zero value denominator",
        }
    }
}

#[cfg(feature = "bigint")]
impl FromPrimitive for Ratio<BigInt> {
    fn from_i64(n: i64) -> Option<Self> {
        Some(Ratio::from_integer(n.into()))
    }

    #[cfg(has_i128)]
    fn from_i128(n: i128) -> Option<Self> {
        Some(Ratio::from_integer(n.into()))
    }

    fn from_u64(n: u64) -> Option<Self> {
        Some(Ratio::from_integer(n.into()))
    }

    #[cfg(has_i128)]
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

            #[cfg(has_i128)]
            fn from_i128(n: i128) -> Option<Self> {
                <$typ as FromPrimitive>::from_i128(n).map(Ratio::from_integer)
            }

            fn from_u64(n: u64) -> Option<Self> {
                <$typ as FromPrimitive>::from_u64(n).map(Ratio::from_integer)
            }

            #[cfg(has_i128)]
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
#[cfg(has_i128)]
from_primitive_integer!(i128, approximate_float);
from_primitive_integer!(isize, approximate_float);

from_primitive_integer!(u8, approximate_float_unsigned);
from_primitive_integer!(u16, approximate_float_unsigned);
from_primitive_integer!(u32, approximate_float_unsigned);
from_primitive_integer!(u64, approximate_float_unsigned);
#[cfg(has_i128)]
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

fn approximate_float<T, F>(val: F, max_error: F, max_iterations: usize) -> Option<Ratio<T>>
where
    T: Integer + Signed + Bounded + NumCast + Clone,
    F: FloatCore + NumCast,
{
    let negative = val.is_sign_negative();
    let abs_val = val.abs();

    let r = approximate_float_unsigned(abs_val, max_error, max_iterations);

    // Make negative again if needed
    if negative {
        r.map(|r| r.neg())
    } else {
        r
    }
}

// No Unsigned constraint because this also works on positive integers and is called
// like that, see above
fn approximate_float_unsigned<T, F>(val: F, max_error: F, max_iterations: usize) -> Option<Ratio<T>>
where
    T: Integer + Bounded + NumCast + Clone,
    F: FloatCore + NumCast,
{
    // Continued fractions algorithm
    // http://mathforum.org/dr.math/faq/faq.fractions.html#decfrac

    if val < F::zero() || val.is_nan() {
        return None;
    }

    let mut q = val;
    let mut n0 = T::zero();
    let mut d0 = T::one();
    let mut n1 = T::one();
    let mut d1 = T::zero();

    let t_max = T::max_value();
    let t_max_f = match <F as NumCast>::from(t_max.clone()) {
        None => return None,
        Some(t_max_f) => t_max_f,
    };

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

#[cfg(test)]
#[cfg(feature = "std")]
fn hash<T: Hash>(x: &T) -> u64 {
    use std::collections::hash_map::RandomState;
    use std::hash::BuildHasher;
    let mut hasher = <RandomState as BuildHasher>::Hasher::new();
    x.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod test {
    #[cfg(feature = "bigint")]
    use super::BigRational;
    use super::{Ratio, Rational};

    use core::f64;
    use core::i32;
    use core::str::FromStr;
    use integer::Integer;
    use traits::{FromPrimitive, One, Pow, Signed, Zero};

    pub const _0: Rational = Ratio { numer: 0, denom: 1 };
    pub const _1: Rational = Ratio { numer: 1, denom: 1 };
    pub const _2: Rational = Ratio { numer: 2, denom: 1 };
    pub const _NEG2: Rational = Ratio {
        numer: -2,
        denom: 1,
    };
    pub const _1_2: Rational = Ratio { numer: 1, denom: 2 };
    pub const _3_2: Rational = Ratio { numer: 3, denom: 2 };
    pub const _NEG1_2: Rational = Ratio {
        numer: -1,
        denom: 2,
    };
    pub const _1_NEG2: Rational = Ratio {
        numer: 1,
        denom: -2,
    };
    pub const _NEG1_NEG2: Rational = Ratio {
        numer: -1,
        denom: -2,
    };
    pub const _1_3: Rational = Ratio { numer: 1, denom: 3 };
    pub const _NEG1_3: Rational = Ratio {
        numer: -1,
        denom: 3,
    };
    pub const _2_3: Rational = Ratio { numer: 2, denom: 3 };
    pub const _NEG2_3: Rational = Ratio {
        numer: -2,
        denom: 3,
    };

    #[cfg(feature = "bigint")]
    pub fn to_big(n: Rational) -> BigRational {
        Ratio::new(
            FromPrimitive::from_isize(n.numer).unwrap(),
            FromPrimitive::from_isize(n.denom).unwrap(),
        )
    }
    #[cfg(not(feature = "bigint"))]
    pub fn to_big(n: Rational) -> Rational {
        Ratio::new(
            FromPrimitive::from_isize(n.numer).unwrap(),
            FromPrimitive::from_isize(n.denom).unwrap(),
        )
    }

    #[test]
    fn test_test_constants() {
        // check our constants are what Ratio::new etc. would make.
        assert_eq!(_0, Zero::zero());
        assert_eq!(_1, One::one());
        assert_eq!(_2, Ratio::from_integer(2));
        assert_eq!(_1_2, Ratio::new(1, 2));
        assert_eq!(_3_2, Ratio::new(3, 2));
        assert_eq!(_NEG1_2, Ratio::new(-1, 2));
        assert_eq!(_2, From::from(2));
    }

    #[test]
    fn test_new_reduce() {
        let one22 = Ratio::new(2, 2);

        assert_eq!(one22, One::one());
    }
    #[test]
    #[should_panic]
    fn test_new_zero() {
        let _a = Ratio::new(1, 0);
    }

    #[test]
    fn test_approximate_float() {
        assert_eq!(Ratio::from_f32(0.5f32), Some(Ratio::new(1i64, 2)));
        assert_eq!(Ratio::from_f64(0.5f64), Some(Ratio::new(1i32, 2)));
        assert_eq!(Ratio::from_f32(5f32), Some(Ratio::new(5i64, 1)));
        assert_eq!(Ratio::from_f64(5f64), Some(Ratio::new(5i32, 1)));
        assert_eq!(Ratio::from_f32(29.97f32), Some(Ratio::new(2997i64, 100)));
        assert_eq!(Ratio::from_f32(-29.97f32), Some(Ratio::new(-2997i64, 100)));

        assert_eq!(Ratio::<i8>::from_f32(63.5f32), Some(Ratio::new(127i8, 2)));
        assert_eq!(Ratio::<i8>::from_f32(126.5f32), Some(Ratio::new(126i8, 1)));
        assert_eq!(Ratio::<i8>::from_f32(127.0f32), Some(Ratio::new(127i8, 1)));
        assert_eq!(Ratio::<i8>::from_f32(127.5f32), None);
        assert_eq!(Ratio::<i8>::from_f32(-63.5f32), Some(Ratio::new(-127i8, 2)));
        assert_eq!(
            Ratio::<i8>::from_f32(-126.5f32),
            Some(Ratio::new(-126i8, 1))
        );
        assert_eq!(
            Ratio::<i8>::from_f32(-127.0f32),
            Some(Ratio::new(-127i8, 1))
        );
        assert_eq!(Ratio::<i8>::from_f32(-127.5f32), None);

        assert_eq!(Ratio::<u8>::from_f32(-127f32), None);
        assert_eq!(Ratio::<u8>::from_f32(127f32), Some(Ratio::new(127u8, 1)));
        assert_eq!(Ratio::<u8>::from_f32(127.5f32), Some(Ratio::new(255u8, 2)));
        assert_eq!(Ratio::<u8>::from_f32(256f32), None);

        assert_eq!(Ratio::<i64>::from_f64(-10e200), None);
        assert_eq!(Ratio::<i64>::from_f64(10e200), None);
        assert_eq!(Ratio::<i64>::from_f64(f64::INFINITY), None);
        assert_eq!(Ratio::<i64>::from_f64(f64::NEG_INFINITY), None);
        assert_eq!(Ratio::<i64>::from_f64(f64::NAN), None);
        assert_eq!(
            Ratio::<i64>::from_f64(f64::EPSILON),
            Some(Ratio::new(1, 4503599627370496))
        );
        assert_eq!(Ratio::<i64>::from_f64(0.0), Some(Ratio::new(0, 1)));
        assert_eq!(Ratio::<i64>::from_f64(-0.0), Some(Ratio::new(0, 1)));
    }

    #[test]
    fn test_cmp() {
        assert!(_0 == _0 && _1 == _1);
        assert!(_0 != _1 && _1 != _0);
        assert!(_0 < _1 && !(_1 < _0));
        assert!(_1 > _0 && !(_0 > _1));

        assert!(_0 <= _0 && _1 <= _1);
        assert!(_0 <= _1 && !(_1 <= _0));

        assert!(_0 >= _0 && _1 >= _1);
        assert!(_1 >= _0 && !(_0 >= _1));
    }

    #[test]
    fn test_cmp_overflow() {
        use core::cmp::Ordering;

        // issue #7 example:
        let big = Ratio::new(128u8, 1);
        let small = big.recip();
        assert!(big > small);

        // try a few that are closer together
        // (some matching numer, some matching denom, some neither)
        let ratios = [
            Ratio::new(125_i8, 127_i8),
            Ratio::new(63_i8, 64_i8),
            Ratio::new(124_i8, 125_i8),
            Ratio::new(125_i8, 126_i8),
            Ratio::new(126_i8, 127_i8),
            Ratio::new(127_i8, 126_i8),
        ];

        fn check_cmp(a: Ratio<i8>, b: Ratio<i8>, ord: Ordering) {
            #[cfg(feature = "std")]
            println!("comparing {} and {}", a, b);
            assert_eq!(a.cmp(&b), ord);
            assert_eq!(b.cmp(&a), ord.reverse());
        }

        for (i, &a) in ratios.iter().enumerate() {
            check_cmp(a, a, Ordering::Equal);
            check_cmp(-a, a, Ordering::Less);
            for &b in &ratios[i + 1..] {
                check_cmp(a, b, Ordering::Less);
                check_cmp(-a, -b, Ordering::Greater);
                check_cmp(a.recip(), b.recip(), Ordering::Greater);
                check_cmp(-a.recip(), -b.recip(), Ordering::Less);
            }
        }
    }

    #[test]
    fn test_to_integer() {
        assert_eq!(_0.to_integer(), 0);
        assert_eq!(_1.to_integer(), 1);
        assert_eq!(_2.to_integer(), 2);
        assert_eq!(_1_2.to_integer(), 0);
        assert_eq!(_3_2.to_integer(), 1);
        assert_eq!(_NEG1_2.to_integer(), 0);
    }

    #[test]
    fn test_numer() {
        assert_eq!(_0.numer(), &0);
        assert_eq!(_1.numer(), &1);
        assert_eq!(_2.numer(), &2);
        assert_eq!(_1_2.numer(), &1);
        assert_eq!(_3_2.numer(), &3);
        assert_eq!(_NEG1_2.numer(), &(-1));
    }
    #[test]
    fn test_denom() {
        assert_eq!(_0.denom(), &1);
        assert_eq!(_1.denom(), &1);
        assert_eq!(_2.denom(), &1);
        assert_eq!(_1_2.denom(), &2);
        assert_eq!(_3_2.denom(), &2);
        assert_eq!(_NEG1_2.denom(), &2);
    }

    #[test]
    fn test_is_integer() {
        assert!(_0.is_integer());
        assert!(_1.is_integer());
        assert!(_2.is_integer());
        assert!(!_1_2.is_integer());
        assert!(!_3_2.is_integer());
        assert!(!_NEG1_2.is_integer());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_show() {
        use std::string::ToString;
        assert_eq!(format!("{}", _2), "2".to_string());
        assert_eq!(format!("{}", _1_2), "1/2".to_string());
        assert_eq!(format!("{}", _0), "0".to_string());
        assert_eq!(format!("{}", Ratio::from_integer(-2)), "-2".to_string());
    }

    mod arith {
        use super::super::{BigRational, Ratio, Rational, Rational32, Rational64};
        use super::{_0, _1, _1_2, _2, _3_2, _NEG1_2, to_big};
        use traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};

        const _1I32: Rational32 = Ratio { numer: 1, denom: 1 };
        const _1I64: Rational64 = Ratio { numer: 1, denom: 1 };
        const _NEG1: Rational = Ratio { numer: -1, denom: 1 };
        const _NEG1I32: Rational32 = Ratio { numer: -1, denom: 1 };
        const _NEG1I64: Rational64 = Ratio { numer: -1, denom: 1 };

        const _2I32: Rational32 = Ratio { numer: 2, denom: 1 };
        const _2I64: Rational64 = Ratio { numer: 2, denom: 1 };
        const _NEG2: Rational = Ratio { numer: -2, denom: 1 };
        const _NEG2I32: Rational32 = Ratio { numer: -2, denom: 1 };
        const _NEG2I64: Rational64 = Ratio { numer: -2, denom: 1 };

        const _1I32_2I32: Rational32 = Ratio { numer: 1, denom: 2 };
        const _1I64_2I64: Rational64 = Ratio { numer: 1, denom: 2 };

        #[test]
        fn test_add() {
            fn test(a: Rational, b: Rational, c: Rational) {
                assert_eq!(a + b, c);
                assert_eq!(
                    {
                        let mut x = a;
                        x += b;
                        x
                    },
                    c
                );
                assert_eq!(to_big(a) + to_big(b), to_big(c));
                assert_eq!(a.checked_add(&b), Some(c));
                assert_eq!(to_big(a).checked_add(&to_big(b)), Some(to_big(c)));
            }
            fn test_assign(a: Rational, b: isize, c: Rational) {
                assert_eq!(a + b, c);
                assert_eq!(
                    {
                        let mut x = a;
                        x += b;
                        x
                    },
                    c
                );
            }

            test(_1, _1_2, _3_2);
            test(_1, _1, _2);
            test(_1_2, _3_2, _2);
            test(_1_2, _NEG1_2, _0);
            test_assign(_1_2, 1, _3_2);
        }

        #[test]
        fn test_primitive_add_rational() {
            fn test_isize_add_rational(a: isize, b: Rational, c: Rational) {
                assert_eq!(a + b, c);
            }
            fn test_i8_add_rational(a: i8, b: Rational, c: Rational) {
                assert_eq!(a + b, c);
            }
            fn test_u8_add_rational(a: u8, b: Rational, c: Rational) {
                assert_eq!(a + b, c);
            }
            fn test_i16_add_rational(a: i16, b: Rational, c: Rational) {
                assert_eq!(a + b, c);
            }
            test_isize_add_rational(-2isize, _1, _NEG1);
            test_i8_add_rational(-2i8, _1, _NEG1);
            test_u8_add_rational(1u8, _1, _2);
            test_i16_add_rational(-2i16, _1, _NEG1);
        }

        #[test]
        fn test_primitive_add_rational32() {
            fn test_i8_add_rational32(a: i8, b: Rational32, c: Rational32) {
                assert_eq!(a + b, c);
            }
            fn test_u8_add_rational32(a: u8, b: Rational32, c: Rational32) {
                assert_eq!(a + b, c);
            }
            fn test_i16_add_rational32(a: i16, b: Rational32, c: Rational32) {
                assert_eq!(a + b, c);
            }
            fn test_u16_add_rational32(a: u16, b: Rational32, c: Rational32) {
                assert_eq!(a + b, c);
            }
            fn test_i32_add_rational32(a: i32, b: Rational32, c: Rational32) {
                assert_eq!(a + b, c);
            }
            test_i8_add_rational32(-2i8, _1I32, _NEG1I32);
            test_u8_add_rational32(1u8, _1I32, _2I32);
            test_i16_add_rational32(-2i16, _1I32, _NEG1I32);
            test_u16_add_rational32(1u16, _1I32, _2I32);
            test_i32_add_rational32(-2i32, _1I32, _NEG1I32);
        }

        #[test]
        fn test_primitive_add_rational64() {
            fn test_i8_add_rational64(a: i8, b: Rational64, c: Rational64) {
                assert_eq!(a + b, c);
            }
            fn test_u8_add_rational64(a: u8, b: Rational64, c: Rational64) {
                assert_eq!(a + b, c);
            }
            fn test_i16_add_rational64(a: i16, b: Rational64, c: Rational64) {
                assert_eq!(a + b, c);
            }
            fn test_u16_add_rational64(a: u16, b: Rational64, c: Rational64) {
                assert_eq!(a + b, c);
            }
            fn test_i32_add_rational64(a: i32, b: Rational64, c: Rational64) {
                assert_eq!(a + b, c);
            }
            fn test_u32_add_rational64(a: i32, b: Rational64, c: Rational64) {
                assert_eq!(a + b, c);
            }
            fn test_i64_add_rational64(a: i64, b: Rational64, c: Rational64) {
                assert_eq!(a + b, c);
            }

            test_i8_add_rational64(-2i8, _1I64, _NEG1I64);
            test_u8_add_rational64(1u8, _1I64, _2I64);
            test_i16_add_rational64(-2i16, _1I64, _NEG1I64);
            test_u16_add_rational64(1u16, _1I64, _2I64);
            test_i32_add_rational64(-2i32, _1I64, _NEG1I64);
            test_u32_add_rational64(1i32, _1I64, _2I64);
            test_i64_add_rational64(-2i64, _1I64, _NEG1I64);
        }

        #[cfg(feature = "bigint")]
        #[test]
        fn test_primitive_add_bigint() {
            fn test_isize_add_bigrational(a: isize, b: BigRational, c: BigRational) {
                assert_eq!(a + b, c);
            }
            fn test_usize_add_bigrational(a: usize, b: BigRational, c: BigRational) {
                assert_eq!(a + b, c);
            }
            fn test_i8_add_bigrational(a: i8, b: BigRational, c: BigRational) {
                assert_eq!(a + b, c);
            }
            fn test_u8_add_bigrational(a: u8, b: BigRational, c: BigRational) {
                assert_eq!(a + b, c);
            }
            fn test_i16_add_bigrational(a: i16, b: BigRational, c: BigRational) {
                assert_eq!(a + b, c);
            }
            fn test_u16_add_bigrational(a: u16, b: BigRational, c: BigRational) {
                assert_eq!(a + b, c);
            }
            fn test_i32_add_bigrational(a: i32, b: BigRational, c: BigRational) {
                assert_eq!(a + b, c);
            }
            fn test_u32_add_bigrational(a: u32, b: BigRational, c: BigRational) {
                assert_eq!(a + b, c);
            }
            fn test_i64_add_bigrational(a: i64, b: BigRational, c: BigRational) {
                assert_eq!(a + b, c);
            }
            fn test_u64_add_bigrational(a: u64, b: BigRational, c: BigRational) {
                assert_eq!(a + b, c);
            }
            #[cfg(feature = "i128")]
            fn test_i128_add_bigrational(a: i128, b: BigRational, c: BigRational) {
                assert_eq!(a + b, c);
            }
            #[cfg(feature = "i128")]
            fn test_u128_add_bigrational(a: u128, b: BigRational, c: BigRational) {
                assert_eq!(a + b, c);
            }
            test_isize_add_bigrational(-2isize, to_big(_1), to_big(_NEG1));
            test_usize_add_bigrational(1usize, to_big(_1), to_big(_2));
            test_i8_add_bigrational(-2i8, to_big(_1), to_big(_NEG1));
            test_u8_add_bigrational(1u8, to_big(_1), to_big(_2));
            test_i16_add_bigrational(-2i16, to_big(_1), to_big(_NEG1));
            test_u16_add_bigrational(1u16, to_big(_1), to_big(_2));
            test_i32_add_bigrational(-2i32, to_big(_1), to_big(_NEG1));
            test_u32_add_bigrational(1u32, to_big(_1), to_big(_2));
            test_i64_add_bigrational(-2i64, to_big(_1), to_big(_NEG1));
            test_u64_add_bigrational(1u64, to_big(_1), to_big(_2));
            #[cfg(feature = "i128")]
            test_i128_add_bigrational(-2i128, to_big(_1), to_big(_NEG1));
            #[cfg(feature = "i128")]
            test_u128_add_bigrational(1u128, to_big(_1), to_big(_2));
        }

        #[cfg(feature = "bigint")]
        #[test]
        fn test_bigint_add_primitive() {
            fn test_bigrational_add_isize(a: BigRational, b: isize, c: BigRational) {
                assert_eq!(a + b, c);
            }
            fn test_bigrational_add_usize(a: BigRational, b: usize, c: BigRational) {
                assert_eq!(a + b, c);
            }
            fn test_bigrational_add_i8(a: BigRational, b: i8, c: BigRational) {
                assert_eq!(a + b, c);
            }
            fn test_bigrational_add_u8(a: BigRational, b: u8, c: BigRational) {
                assert_eq!(a + b, c);
            }
            fn test_bigrational_add_i16(a: BigRational, b: i16, c: BigRational) {
                assert_eq!(a + b, c);
            }
            fn test_bigrational_add_u16(a: BigRational, b: u16, c: BigRational) {
                assert_eq!(a + b, c);
            }
            fn test_bigrational_add_i32(a: BigRational, b: i32, c: BigRational) {
                assert_eq!(a + b, c);
            }
            fn test_bigrational_add_u32(a: BigRational, b: u32, c: BigRational) {
                assert_eq!(a + b, c);
            }
            fn test_bigrational_add_i64(a: BigRational, b: i64, c: BigRational) {
                assert_eq!(a + b, c);
            }
            fn test_bigrational_add_u64(a: BigRational, b: u64, c: BigRational) {
                assert_eq!(a + b, c);
            }
            #[cfg(feature = "i128")]
            fn test_bigrational_add_i128(a: BigRational, b: i128, c: BigRational) {
                assert_eq!(a + b, c);
            }
            #[cfg(feature = "i128")]
            fn test_bigrational_add_u128(a: BigRational, b: u128, c: BigRational) {
                assert_eq!(a + b, c);
            }

            test_bigrational_add_isize(to_big(_1), -2isize, to_big(_NEG1));
            test_bigrational_add_usize(to_big(_1), 1usize, to_big(_2));
            test_bigrational_add_i8(to_big(_1), -2i8, to_big(_NEG1));
            test_bigrational_add_u8(to_big(_1), 1u8, to_big(_2));
            test_bigrational_add_i16(to_big(_1), -2i16, to_big(_NEG1));
            test_bigrational_add_u16(to_big(_1), 1u16, to_big(_2));
            test_bigrational_add_i32(to_big(_1), -2i32, to_big(_NEG1));
            test_bigrational_add_u32(to_big(_1), 1u32, to_big(_2));
            test_bigrational_add_i64(to_big(_1), -2i64, to_big(_NEG1));
            test_bigrational_add_u64(to_big(_1), 1u64, to_big(_2));
            #[cfg(feature = "i128")]
            test_bigrational_add_i128(to_big(_1), -2i128, to_big(_NEG1));
            #[cfg(feature = "i128")]
            test_bigrational_add_u128(to_big(_1), 1u128, to_big(_2));
        }

        #[test]
        fn test_sub() {
            fn test(a: Rational, b: Rational, c: Rational) {
                assert_eq!(a - b, c);
                assert_eq!(
                    {
                        let mut x = a;
                        x -= b;
                        x
                    },
                    c
                );
                assert_eq!(to_big(a) - to_big(b), to_big(c));
                assert_eq!(a.checked_sub(&b), Some(c));
                assert_eq!(to_big(a).checked_sub(&to_big(b)), Some(to_big(c)));
            }
            fn test_assign(a: Rational, b: isize, c: Rational) {
                assert_eq!(a - b, c);
                assert_eq!(
                    {
                        let mut x = a;
                        x -= b;
                        x
                    },
                    c
                );
            }

            test(_1, _1_2, _1_2);
            test(_3_2, _1_2, _1);
            test(_1, _NEG1_2, _3_2);
            test_assign(_1_2, 1, _NEG1_2);
        }

        #[test]
        fn test_primitive_sub_rational() {
            fn test_isize_sub_rational(a: isize, b: Rational, c: Rational) {
                assert_eq!(a - b, c);
            }
            fn test_i8_sub_rational(a: i8, b: Rational, c: Rational) {
                assert_eq!(a - b, c);
            }
            fn test_u8_sub_rational(a: u8, b: Rational, c: Rational) {
                assert_eq!(a - b, c);
            }
            fn test_i16_sub_rational(a: i16, b: Rational, c: Rational) {
                assert_eq!(a - b, c);
            }
            test_isize_sub_rational(-1isize, _1, _NEG2);
            test_i8_sub_rational(-1i8, _1, _NEG2);
            test_u8_sub_rational(2u8, _1, _1);
            test_i16_sub_rational(-1i16, _1, _NEG2);
        }

        #[test]
        fn test_primitive_sub_rational32() {
            fn test_i8_sub_rational32(a: i8, b: Rational32, c: Rational32) {
                assert_eq!(a - b, c);
            }
            fn test_u8_sub_rational32(a: u8, b: Rational32, c: Rational32) {
                assert_eq!(a - b, c);
            }
            fn test_i16_sub_rational32(a: i16, b: Rational32, c: Rational32) {
                assert_eq!(a - b, c);
            }
            fn test_u16_sub_rational32(a: u16, b: Rational32, c: Rational32) {
                assert_eq!(a - b, c);
            }
            fn test_i32_sub_rational32(a: i32, b: Rational32, c: Rational32) {
                assert_eq!(a - b, c);
            }
            test_i8_sub_rational32(-1i8, _1I32, _NEG2I32);
            test_u8_sub_rational32(2u8, _1I32, _1I32);
            test_i16_sub_rational32(-1i16, _1I32, _NEG2I32);
            test_u16_sub_rational32(2u16, _1I32, _1I32);
            test_i32_sub_rational32(-1i32, _1I32, _NEG2I32);
        }

        #[test]
        fn test_primitive_sub_rational64() {
            fn test_i8_sub_rational64(a: i8, b: Rational64, c: Rational64) {
                assert_eq!(a - b, c);
            }
            fn test_u8_sub_rational64(a: u8, b: Rational64, c: Rational64) {
                assert_eq!(a - b, c);
            }
            fn test_i16_sub_rational64(a: i16, b: Rational64, c: Rational64) {
                assert_eq!(a - b, c);
            }
            fn test_u16_sub_rational64(a: u16, b: Rational64, c: Rational64) {
                assert_eq!(a - b, c);
            }
            fn test_i32_sub_rational64(a: i32, b: Rational64, c: Rational64) {
                assert_eq!(a - b, c);
            }
            fn test_u32_sub_rational64(a: i32, b: Rational64, c: Rational64) {
                assert_eq!(a - b, c);
            }
            fn test_i64_sub_rational64(a: i64, b: Rational64, c: Rational64) {
                assert_eq!(a - b, c);
            }

            test_i8_sub_rational64(-1i8, _1I64, _NEG2I64);
            test_u8_sub_rational64(2u8, _1I64, _1I64);
            test_i16_sub_rational64(-1i16, _1I64, _NEG2I64);
            test_u16_sub_rational64(2u16, _1I64, _1I64);
            test_i32_sub_rational64(-1i32, _1I64, _NEG2I64);
            test_u32_sub_rational64(2i32, _1I64, _1I64);
            test_i64_sub_rational64(-1i64, _1I64, _NEG2I64);
        }

        #[cfg(feature = "bigint")]
        #[test]
        fn test_primitive_sub_bigint() {
            fn test_isize_sub_bigrational(a: isize, b: BigRational, c: BigRational) {
                assert_eq!(a - b, c);
            }
            fn test_usize_sub_bigrational(a: usize, b: BigRational, c: BigRational) {
                assert_eq!(a - b, c);
            }
            fn test_i8_sub_bigrational(a: i8, b: BigRational, c: BigRational) {
                assert_eq!(a - b, c);
            }
            fn test_u8_sub_bigrational(a: u8, b: BigRational, c: BigRational) {
                assert_eq!(a - b, c);
            }
            fn test_i16_sub_bigrational(a: i16, b: BigRational, c: BigRational) {
                assert_eq!(a - b, c);
            }
            fn test_u16_sub_bigrational(a: u16, b: BigRational, c: BigRational) {
                assert_eq!(a - b, c);
            }
            fn test_i32_sub_bigrational(a: i32, b: BigRational, c: BigRational) {
                assert_eq!(a - b, c);
            }
            fn test_u32_sub_bigrational(a: u32, b: BigRational, c: BigRational) {
                assert_eq!(a - b, c);
            }
            fn test_i64_sub_bigrational(a: i64, b: BigRational, c: BigRational) {
                assert_eq!(a - b, c);
            }
            fn test_u64_sub_bigrational(a: u64, b: BigRational, c: BigRational) {
                assert_eq!(a - b, c);
            }
            #[cfg(feature = "i128")]
            fn test_i128_sub_bigrational(a: i128, b: BigRational, c: BigRational) {
                assert_eq!(a - b, c);
            }
            #[cfg(feature = "i128")]
            fn test_u128_sub_bigrational(a: u128, b: BigRational, c: BigRational) {
                assert_eq!(a - b, c);
            }

            test_isize_sub_bigrational(-1isize, to_big(_1), to_big(_NEG2));
            test_usize_sub_bigrational(2usize, to_big(_1), to_big(_1));
            test_i8_sub_bigrational(-1i8, to_big(_1), to_big(_NEG2));
            test_u8_sub_bigrational(2u8, to_big(_1), to_big(_1));
            test_i16_sub_bigrational(-1i16, to_big(_1), to_big(_NEG2));
            test_u16_sub_bigrational(2u16, to_big(_1), to_big(_1));
            test_i32_sub_bigrational(-1i32, to_big(_1), to_big(_NEG2));
            test_u32_sub_bigrational(2u32, to_big(_1), to_big(_1));
            test_i64_sub_bigrational(-1i64, to_big(_1), to_big(_NEG2));
            test_u64_sub_bigrational(2u64, to_big(_1), to_big(_1));
            #[cfg(feature = "i128")]
            test_i128_sub_bigrational(-1i128, to_big(_1), to_big(_NEG2));
            #[cfg(feature = "i128")]
            test_u128_sub_bigrational(2u128, to_big(_1), to_big(_1));
        }

        #[cfg(feature = "bigint")]
        #[test]
        fn test_bigint_sub_primitive() {
            fn test_bigrational_sub_isize(a: BigRational, b: isize, c: BigRational) {
                assert_eq!(a - b, c);
            }
            fn test_bigrational_sub_usize(a: BigRational, b: usize, c: BigRational) {
                assert_eq!(a - b, c);
            }
            fn test_bigrational_sub_i8(a: BigRational, b: i8, c: BigRational) {
                assert_eq!(a - b, c);
            }
            fn test_bigrational_sub_u8(a: BigRational, b: u8, c: BigRational) {
                assert_eq!(a - b, c);
            }
            fn test_bigrational_sub_i16(a: BigRational, b: i16, c: BigRational) {
                assert_eq!(a - b, c);
            }
            fn test_bigrational_sub_u16(a: BigRational, b: u16, c: BigRational) {
                assert_eq!(a - b, c);
            }
            fn test_bigrational_sub_i32(a: BigRational, b: i32, c: BigRational) {
                assert_eq!(a - b, c);
            }
            fn test_bigrational_sub_u32(a: BigRational, b: u32, c: BigRational) {
                assert_eq!(a - b, c);
            }
            fn test_bigrational_sub_i64(a: BigRational, b: i64, c: BigRational) {
                assert_eq!(a - b, c);
            }
            fn test_bigrational_sub_u64(a: BigRational, b: u64, c: BigRational) {
                assert_eq!(a - b, c);
            }
            #[cfg(feature = "i128")]
            fn test_bigrational_sub_i128(a: BigRational, b: i128, c: BigRational) {
                assert_eq!(a - b, c);
            }
            #[cfg(feature = "i128")]
            fn test_bigrational_sub_u128(a: BigRational, b: u128, c: BigRational) {
                assert_eq!(a - b, c);
            }

            test_bigrational_sub_isize(to_big(_2), 1isize, to_big(_1));
            test_bigrational_sub_usize(to_big(_2), 1usize, to_big(_1));
            test_bigrational_sub_i8(to_big(_2), 1i8, to_big(_1));
            test_bigrational_sub_u8(to_big(_2), 1u8, to_big(_1));
            test_bigrational_sub_i16(to_big(_2), 1i16, to_big(_1));
            test_bigrational_sub_u16(to_big(_2), 1u16, to_big(_1));
            test_bigrational_sub_i32(to_big(_2), 1i32, to_big(_1));
            test_bigrational_sub_u32(to_big(_2), 1u32, to_big(_1));
            test_bigrational_sub_i64(to_big(_2), 1i64, to_big(_1));
            test_bigrational_sub_u64(to_big(_2), 1u64, to_big(_1));
            #[cfg(feature = "i128")]
            test_bigrational_sub_i128(to_big(_2), 1i128, to_big(_1));
            #[cfg(feature = "i128")]
            test_bigrational_sub_u128(to_big(_2), 1u128, to_big(_1));
        }
        #[test]
        fn test_mul() {
            fn test(a: Rational, b: Rational, c: Rational) {
                assert_eq!(a * b, c);
                assert_eq!(
                    {
                        let mut x = a;
                        x *= b;
                        x
                    },
                    c
                );
                assert_eq!(to_big(a) * to_big(b), to_big(c));
                assert_eq!(a.checked_mul(&b), Some(c));
                assert_eq!(to_big(a).checked_mul(&to_big(b)), Some(to_big(c)));
            }
            fn test_assign(a: Rational, b: isize, c: Rational) {
                assert_eq!(a * b, c);
                assert_eq!(
                    {
                        let mut x = a;
                        x *= b;
                        x
                    },
                    c
                );
            }

            test(_1, _1_2, _1_2);
            test(_1_2, _3_2, Ratio::new(3, 4));
            test(_1_2, _NEG1_2, Ratio::new(-1, 4));
            test_assign(_1_2, 2, _1);
        }

        #[test]
        fn test_primitive_mul_rational() {
            fn test_isize_mul_rational(a: isize, b: Rational, c: Rational) {
                assert_eq!(a * b, c);
            }
            fn test_i8_mul_rational(a: i8, b: Rational, c: Rational) {
                assert_eq!(a * b, c);
            }
            fn test_u8_mul_rational(a: u8, b: Rational, c: Rational) {
                assert_eq!(a * b, c);
            }
            fn test_i16_mul_rational(a: i16, b: Rational, c: Rational) {
                assert_eq!(a * b, c);
            }
            test_isize_mul_rational(-2isize, _1_2, _NEG1);
            test_i8_mul_rational(-2i8, _1_2, _NEG1);
            test_u8_mul_rational(2u8, _1_2, _1);
            test_i16_mul_rational(-2i16, _1_2, _NEG1);
        }

        #[test]
        fn test_primitive_mul_rational32() {
            fn test_i8_mul_rational32(a: i8, b: Rational32, c: Rational32) {
                assert_eq!(a * b, c);
            }
            fn test_u8_mul_rational32(a: u8, b: Rational32, c: Rational32) {
                assert_eq!(a * b, c);
            }
            fn test_i16_mul_rational32(a: i16, b: Rational32, c: Rational32) {
                assert_eq!(a * b, c);
            }
            fn test_u16_mul_rational32(a: u16, b: Rational32, c: Rational32) {
                assert_eq!(a * b, c);
            }
            fn test_i32_mul_rational32(a: i32, b: Rational32, c: Rational32) {
                assert_eq!(a * b, c);
            }
            test_i8_mul_rational32(-2i8, _1I32_2I32, _NEG1I32);
            test_u8_mul_rational32(2u8, _1I32_2I32, _1I32);
            test_i16_mul_rational32(-2i16, _1I32_2I32, _NEG1I32);
            test_u16_mul_rational32(2u16, _1I32_2I32, _1I32);
            test_i32_mul_rational32(-2i32, _1I32_2I32, _NEG1I32);
        }

        #[test]
        fn test_primitive_mul_rational64() {
            fn test_i8_mul_rational64(a: i8, b: Rational64, c: Rational64) {
                assert_eq!(a * b, c);
            }
            fn test_u8_mul_rational64(a: u8, b: Rational64, c: Rational64) {
                assert_eq!(a * b, c);
            }
            fn test_i16_mul_rational64(a: i16, b: Rational64, c: Rational64) {
                assert_eq!(a * b, c);
            }
            fn test_u16_mul_rational64(a: u16, b: Rational64, c: Rational64) {
                assert_eq!(a * b, c);
            }
            fn test_i32_mul_rational64(a: i32, b: Rational64, c: Rational64) {
                assert_eq!(a * b, c);
            }
            fn test_u32_mul_rational64(a: i32, b: Rational64, c: Rational64) {
                assert_eq!(a * b, c);
            }
            fn test_i64_mul_rational64(a: i64, b: Rational64, c: Rational64) {
                assert_eq!(a * b, c);
            }

            test_i8_mul_rational64(-2i8, _1I64_2I64, _NEG1I64);
            test_u8_mul_rational64(2u8, _1I64_2I64, _1I64);
            test_i16_mul_rational64(-2i16, _1I64_2I64, _NEG1I64);
            test_u16_mul_rational64(2u16, _1I64_2I64, _1I64);
            test_i32_mul_rational64(-2i32, _1I64_2I64, _NEG1I64);
            test_u32_mul_rational64(2i32, _1I64_2I64, _1I64);
            test_i64_mul_rational64(-2i64, _1I64_2I64, _NEG1I64);
        }

        #[cfg(feature = "bigint")]
        #[test]
        fn test_primitive_mul_bigint() {
            fn test_isize_mul_bigrational(a: isize, b: BigRational, c: BigRational) {
                assert_eq!(a * b, c);
            }
            fn test_usize_mul_bigrational(a: usize, b: BigRational, c: BigRational) {
                assert_eq!(a * b, c);
            }
            fn test_i8_mul_bigrational(a: i8, b: BigRational, c: BigRational) {
                assert_eq!(a * b, c);
            }
            fn test_u8_mul_bigrational(a: u8, b: BigRational, c: BigRational) {
                assert_eq!(a * b, c);
            }
            fn test_i16_mul_bigrational(a: i16, b: BigRational, c: BigRational) {
                assert_eq!(a * b, c);
            }
            fn test_u16_mul_bigrational(a: u16, b: BigRational, c: BigRational) {
                assert_eq!(a * b, c);
            }
            fn test_i32_mul_bigrational(a: i32, b: BigRational, c: BigRational) {
                assert_eq!(a * b, c);
            }
            fn test_u32_mul_bigrational(a: u32, b: BigRational, c: BigRational) {
                assert_eq!(a * b, c);
            }
            fn test_i64_mul_bigrational(a: i64, b: BigRational, c: BigRational) {
                assert_eq!(a * b, c);
            }
            fn test_u64_mul_bigrational(a: u64, b: BigRational, c: BigRational) {
                assert_eq!(a * b, c);
            }
            #[cfg(feature = "i128")]
            fn test_i128_mul_bigrational(a: i128, b: BigRational, c: BigRational) {
                assert_eq!(a * b, c);
            }
            #[cfg(feature = "i128")]
            fn test_u128_mul_bigrational(a: u128, b: BigRational, c: BigRational) {
                assert_eq!(a * b, c);
            }
            test_isize_mul_bigrational(-2isize, to_big(_1_2), to_big(_NEG1));
            test_usize_mul_bigrational(2usize, to_big(_1_2), to_big(_1));
            test_i8_mul_bigrational(-2i8, to_big(_1_2), to_big(_NEG1));
            test_u8_mul_bigrational(2u8, to_big(_1_2), to_big(_1));
            test_i16_mul_bigrational(-2i16, to_big(_1_2), to_big(_NEG1));
            test_u16_mul_bigrational(2u16, to_big(_1_2), to_big(_1));
            test_i32_mul_bigrational(-2i32, to_big(_1_2), to_big(_NEG1));
            test_u32_mul_bigrational(2u32, to_big(_1_2), to_big(_1));
            test_i64_mul_bigrational(-2i64, to_big(_1_2), to_big(_NEG1));
            test_u64_mul_bigrational(2u64, to_big(_1_2), to_big(_1));
            #[cfg(feature = "i128")]
            test_i128_mul_bigrational(-2i128, to_big(_1_2), to_big(_NEG1));
            #[cfg(feature = "i128")]
            test_u128_mul_bigrational(2u128, to_big(_1_2), to_big(_1));
        }

        #[cfg(feature = "bigint")]
        #[test]
        fn test_bigint_mul_primitive() {
            fn test_bigrational_mul_isize(a: BigRational, b: isize, c: BigRational) {
                assert_eq!(a * b, c);
            }
            fn test_bigrational_mul_usize(a: BigRational, b: usize, c: BigRational) {
                assert_eq!(a * b, c);
            }
            fn test_bigrational_mul_i8(a: BigRational, b: i8, c: BigRational) {
                assert_eq!(a * b, c);
            }
            fn test_bigrational_mul_u8(a: BigRational, b: u8, c: BigRational) {
                assert_eq!(a * b, c);
            }
            fn test_bigrational_mul_i16(a: BigRational, b: i16, c: BigRational) {
                assert_eq!(a * b, c);
            }
            fn test_bigrational_mul_u16(a: BigRational, b: u16, c: BigRational) {
                assert_eq!(a * b, c);
            }
            fn test_bigrational_mul_i32(a: BigRational, b: i32, c: BigRational) {
                assert_eq!(a * b, c);
            }
            fn test_bigrational_mul_u32(a: BigRational, b: u32, c: BigRational) {
                assert_eq!(a * b, c);
            }
            fn test_bigrational_mul_i64(a: BigRational, b: i64, c: BigRational) {
                assert_eq!(a * b, c);
            }
            fn test_bigrational_mul_u64(a: BigRational, b: u64, c: BigRational) {
                assert_eq!(a * b, c);
            }
            #[cfg(feature = "i128")]
            fn test_bigrational_mul_i128(a: BigRational, b: i128, c: BigRational) {
                assert_eq!(a * b, c);
            }
            #[cfg(feature = "i128")]
            fn test_bigrational_mul_u128(a: BigRational, b: u128, c: BigRational) {
                assert_eq!(a * b, c);
            }

            test_bigrational_mul_isize(to_big(_1_2), -2isize, to_big(_NEG1));
            test_bigrational_mul_usize(to_big(_1_2), 2usize, to_big(_1));
            test_bigrational_mul_i8(to_big(_1_2), -2i8, to_big(_NEG1));
            test_bigrational_mul_u8(to_big(_1_2), 2u8, to_big(_1));
            test_bigrational_mul_i16(to_big(_1_2), -2i16, to_big(_NEG1));
            test_bigrational_mul_u16(to_big(_1_2), 2u16, to_big(_1));
            test_bigrational_mul_i32(to_big(_1_2), -2i32, to_big(_NEG1));
            test_bigrational_mul_u32(to_big(_1_2), 2u32, to_big(_1));
            test_bigrational_mul_i64(to_big(_1_2), -2i64, to_big(_NEG1));
            test_bigrational_mul_u64(to_big(_1_2), 2u64, to_big(_1));
            #[cfg(feature = "i128")]
            test_bigrational_mul_i128(to_big(_1_2), -2i128, to_big(_NEG1));
            #[cfg(feature = "i128")]
            test_bigrational_mul_u128(to_big(_1_2), 2u128, to_big(_1));
        }

        #[test]
        fn test_div() {
            fn test(a: Rational, b: Rational, c: Rational) {
                assert_eq!(a / b, c);
                assert_eq!(
                    {
                        let mut x = a;
                        x /= b;
                        x
                    },
                    c
                );
                assert_eq!(to_big(a) / to_big(b), to_big(c));
                assert_eq!(a.checked_div(&b), Some(c));
                assert_eq!(to_big(a).checked_div(&to_big(b)), Some(to_big(c)));
            }
            fn test_assign(a: Rational, b: isize, c: Rational) {
                assert_eq!(a / b, c);
                assert_eq!(
                    {
                        let mut x = a;
                        x /= b;
                        x
                    },
                    c
                );
            }

            test(_1, _1_2, _2);
            test(_3_2, _1_2, _1 + _2);
            test(_1, _NEG1_2, _NEG1_2 + _NEG1_2 + _NEG1_2 + _NEG1_2);
            test_assign(_1, 2, _1_2);
        }

        #[test]
        fn test_primitive_div_rational() {
            fn test_isize_div_rational(a: isize, b: Rational, c: Rational) {
                assert_eq!(a / b, c);
            }
            fn test_i8_div_rational(a: i8, b: Rational, c: Rational) {
                assert_eq!(a / b, c);
            }
            fn test_u8_div_rational(a: u8, b: Rational, c: Rational) {
                assert_eq!(a / b, c);
            }
            fn test_i16_div_rational(a: i16, b: Rational, c: Rational) {
                assert_eq!(a / b, c);
            }
            test_isize_div_rational(-2isize, _2, _NEG1);
            test_i8_div_rational(-2i8, _2, _NEG1);
            test_u8_div_rational(2u8, _2, _1);
            test_i16_div_rational(-2i16, _2, _NEG1);
        }

        #[test]
        fn test_primitive_div_rational32() {
            fn test_i8_div_rational32(a: i8, b: Rational32, c: Rational32) {
                assert_eq!(a / b, c);
            }
            fn test_u8_div_rational32(a: u8, b: Rational32, c: Rational32) {
                assert_eq!(a / b, c);
            }
            fn test_i16_div_rational32(a: i16, b: Rational32, c: Rational32) {
                assert_eq!(a / b, c);
            }
            fn test_u16_div_rational32(a: u16, b: Rational32, c: Rational32) {
                assert_eq!(a / b, c);
            }
            fn test_i32_div_rational32(a: i32, b: Rational32, c: Rational32) {
                assert_eq!(a / b, c);
            }
            test_i8_div_rational32(-2i8, _2I32, _NEG1I32);
            test_u8_div_rational32(2u8, _2I32, _1I32);
            test_i16_div_rational32(-2i16, _2I32, _NEG1I32);
            test_u16_div_rational32(2u16, _2I32, _1I32);
            test_i32_div_rational32(-2i32, _2I32, _NEG1I32);
        }

        #[test]
        fn test_primitive_div_rational64() {
            fn test_i8_div_rational64(a: i8, b: Rational64, c: Rational64) {
                assert_eq!(a / b, c);
            }
            fn test_u8_div_rational64(a: u8, b: Rational64, c: Rational64) {
                assert_eq!(a / b, c);
            }
            fn test_i16_div_rational64(a: i16, b: Rational64, c: Rational64) {
                assert_eq!(a / b, c);
            }
            fn test_u16_div_rational64(a: u16, b: Rational64, c: Rational64) {
                assert_eq!(a / b, c);
            }
            fn test_i32_div_rational64(a: i32, b: Rational64, c: Rational64) {
                assert_eq!(a / b, c);
            }
            fn test_u32_div_rational64(a: u32, b: Rational64, c: Rational64) {
                assert_eq!(a / b, c);
            }
            fn test_i64_div_rational64(a: i64, b: Rational64, c: Rational64) {
                assert_eq!(a / b, c);
            }
            test_i8_div_rational64(-2i8, _2I64, _NEG1I64);
            test_u8_div_rational64(2u8, _2I64, _1I64);
            test_i16_div_rational64(-2i16, _2I64, _NEG1I64);
            test_u16_div_rational64(2u16, _2I64, _1I64);
            test_i32_div_rational64(-2i32, _2I64, _NEG1I64);
            test_u32_div_rational64(2u32, _2I64, _1I64);
            test_i64_div_rational64(-2i64, _2I64, _NEG1I64);
        }

        #[cfg(feature = "bigint")]
        #[test]
        fn test_primitive_div_bigint() {
            fn test_isize_div_bigrational(a: isize, b: BigRational, c: BigRational) {
                assert_eq!(a / b, c);
            }
            fn test_usize_div_bigrational(a: usize, b: BigRational, c: BigRational) {
                assert_eq!(a / b, c);
            }
            fn test_i8_div_bigrational(a: i8, b: BigRational, c: BigRational) {
                assert_eq!(a / b, c);
            }
            fn test_u8_div_bigrational(a: u8, b: BigRational, c: BigRational) {
                assert_eq!(a / b, c);
            }
            fn test_i16_div_bigrational(a: i16, b: BigRational, c: BigRational) {
                assert_eq!(a / b, c);
            }
            fn test_u16_div_bigrational(a: u16, b: BigRational, c: BigRational) {
                assert_eq!(a / b, c);
            }
            fn test_i32_div_bigrational(a: i32, b: BigRational, c: BigRational) {
                assert_eq!(a / b, c);
            }
            fn test_u32_div_bigrational(a: u32, b: BigRational, c: BigRational) {
                assert_eq!(a / b, c);
            }
            fn test_i64_div_bigrational(a: i64, b: BigRational, c: BigRational) {
                assert_eq!(a / b, c);
            }
            fn test_u64_div_bigrational(a: u64, b: BigRational, c: BigRational) {
                assert_eq!(a / b, c);
            }
            #[cfg(feature = "i128")]
            fn test_i128_div_bigrational(a: i128, b: BigRational, c: BigRational) {
                assert_eq!(a / b, c);
            }
            #[cfg(feature = "i128")]
            fn test_u128_div_bigrational(a: u128, b: BigRational, c: BigRational) {
                assert_eq!(a / b, c);
            }
            test_isize_div_bigrational(-2isize, to_big(_2), to_big(_NEG1));
            test_usize_div_bigrational(2usize, to_big(_2), to_big(_1));
            test_i8_div_bigrational(-2i8, to_big(_2), to_big(_NEG1));
            test_u8_div_bigrational(2u8, to_big(_2), to_big(_1));
            test_i16_div_bigrational(-2i16, to_big(_2), to_big(_NEG1));
            test_u16_div_bigrational(2u16, to_big(_2), to_big(_1));
            test_i32_div_bigrational(-2i32, to_big(_2), to_big(_NEG1));
            test_u32_div_bigrational(2u32, to_big(_2), to_big(_1));
            test_i64_div_bigrational(-2i64, to_big(_2), to_big(_NEG1));
            test_u64_div_bigrational(2u64, to_big(_2), to_big(_1));
            #[cfg(feature = "i128")]
            test_i128_div_bigrational(-2i128, to_big(_2), to_big(_NEG1));
            #[cfg(feature = "i128")]
            test_u128_div_bigrational(2u128, to_big(_2), to_big(_1));
        }

        #[cfg(feature = "bigint")]
        #[test]
        fn test_bigint_div_primitive() {
            fn test_bigrational_div_isize(a: BigRational, b: isize, c: BigRational) {
                assert_eq!(a / b, c);
            }
            fn test_bigrational_div_usize(a: BigRational, b: usize, c: BigRational) {
                assert_eq!(a / b, c);
            }
            fn test_bigrational_div_i8(a: BigRational, b: i8, c: BigRational) {
                assert_eq!(a / b, c);
            }
            fn test_bigrational_div_u8(a: BigRational, b: u8, c: BigRational) {
                assert_eq!(a / b, c);
            }
            fn test_bigrational_div_i16(a: BigRational, b: i16, c: BigRational) {
                assert_eq!(a / b, c);
            }
            fn test_bigrational_div_u16(a: BigRational, b: u16, c: BigRational) {
                assert_eq!(a / b, c);
            }
            fn test_bigrational_div_i32(a: BigRational, b: i32, c: BigRational) {
                assert_eq!(a / b, c);
            }
            fn test_bigrational_div_u32(a: BigRational, b: u32, c: BigRational) {
                assert_eq!(a / b, c);
            }
            fn test_bigrational_div_i64(a: BigRational, b: i64, c: BigRational) {
                assert_eq!(a / b, c);
            }
            fn test_bigrational_div_u64(a: BigRational, b: u64, c: BigRational) {
                assert_eq!(a / b, c);
            }
            #[cfg(feature = "i128")]
            fn test_bigrational_div_i128(a: BigRational, b: i128, c: BigRational) {
                assert_eq!(a / b, c);
            }
            #[cfg(feature = "i128")]
            fn test_bigrational_div_u128(a: BigRational, b: u128, c: BigRational) {
                assert_eq!(a / b, c);
            }

            test_bigrational_div_isize(to_big(_2), -2isize, to_big(_NEG1));
            test_bigrational_div_usize(to_big(_2), 2usize, to_big(_1));
            test_bigrational_div_i8(to_big(_2), -2i8, to_big(_NEG1));
            test_bigrational_div_u8(to_big(_2), 2u8, to_big(_1));
            test_bigrational_div_i16(to_big(_2), -2i16, to_big(_NEG1));
            test_bigrational_div_u16(to_big(_2), 2u16, to_big(_1));
            test_bigrational_div_i32(to_big(_2), -2i32, to_big(_NEG1));
            test_bigrational_div_u32(to_big(_2), 2u32, to_big(_1));
            test_bigrational_div_i64(to_big(_2), -2i64, to_big(_NEG1));
            test_bigrational_div_u64(to_big(_2), 2u64, to_big(_1));
            #[cfg(feature = "i128")]
            test_bigrational_div_i128(to_big(_2), -2i128, to_big(_NEG1));
            #[cfg(feature = "i128")]
            test_bigrational_div_u128(to_big(_2), 2u128, to_big(_1));
        }

        #[test]
        fn test_rem() {
            fn test(a: Rational, b: Rational, c: Rational) {
                assert_eq!(a % b, c);
                assert_eq!(
                    {
                        let mut x = a;
                        x %= b;
                        x
                    },
                    c
                );
                assert_eq!(to_big(a) % to_big(b), to_big(c))
            }
            fn test_assign(a: Rational, b: isize, c: Rational) {
                assert_eq!(a % b, c);
                assert_eq!(
                    {
                        let mut x = a;
                        x %= b;
                        x
                    },
                    c
                );
            }

            test(_3_2, _1, _1_2);
            test(_2, _NEG1_2, _0);
            test(_1_2, _2, _1_2);
            test_assign(_3_2, 1, _1_2);
        }

        #[test]
        fn test_neg() {
            fn test(a: Rational, b: Rational) {
                assert_eq!(-a, b);
                assert_eq!(-to_big(a), to_big(b))
            }

            test(_0, _0);
            test(_1_2, _NEG1_2);
            test(-_1, _1);
        }
        #[test]
        fn test_zero() {
            assert_eq!(_0 + _0, _0);
            assert_eq!(_0 * _0, _0);
            assert_eq!(_0 * _1, _0);
            assert_eq!(_0 / _NEG1_2, _0);
            assert_eq!(_0 - _0, _0);
        }
        #[test]
        #[should_panic]
        fn test_div_0() {
            let _a = _1 / _0;
        }

        #[test]
        fn test_checked_failures() {
            let big = Ratio::new(128u8, 1);
            let small = Ratio::new(1, 128u8);
            assert_eq!(big.checked_add(&big), None);
            assert_eq!(small.checked_sub(&big), None);
            assert_eq!(big.checked_mul(&big), None);
            assert_eq!(small.checked_div(&big), None);
            assert_eq!(_1.checked_div(&_0), None);
        }
    }

    #[test]
    fn test_round() {
        assert_eq!(_1_3.ceil(), _1);
        assert_eq!(_1_3.floor(), _0);
        assert_eq!(_1_3.round(), _0);
        assert_eq!(_1_3.trunc(), _0);

        assert_eq!(_NEG1_3.ceil(), _0);
        assert_eq!(_NEG1_3.floor(), -_1);
        assert_eq!(_NEG1_3.round(), _0);
        assert_eq!(_NEG1_3.trunc(), _0);

        assert_eq!(_2_3.ceil(), _1);
        assert_eq!(_2_3.floor(), _0);
        assert_eq!(_2_3.round(), _1);
        assert_eq!(_2_3.trunc(), _0);

        assert_eq!(_NEG2_3.ceil(), _0);
        assert_eq!(_NEG2_3.floor(), -_1);
        assert_eq!(_NEG2_3.round(), -_1);
        assert_eq!(_NEG2_3.trunc(), _0);

        assert_eq!(_1_2.ceil(), _1);
        assert_eq!(_1_2.floor(), _0);
        assert_eq!(_1_2.round(), _1);
        assert_eq!(_1_2.trunc(), _0);

        assert_eq!(_NEG1_2.ceil(), _0);
        assert_eq!(_NEG1_2.floor(), -_1);
        assert_eq!(_NEG1_2.round(), -_1);
        assert_eq!(_NEG1_2.trunc(), _0);

        assert_eq!(_1.ceil(), _1);
        assert_eq!(_1.floor(), _1);
        assert_eq!(_1.round(), _1);
        assert_eq!(_1.trunc(), _1);

        // Overflow checks

        let _neg1 = Ratio::from_integer(-1);
        let _large_rat1 = Ratio::new(i32::MAX, i32::MAX - 1);
        let _large_rat2 = Ratio::new(i32::MAX - 1, i32::MAX);
        let _large_rat3 = Ratio::new(i32::MIN + 2, i32::MIN + 1);
        let _large_rat4 = Ratio::new(i32::MIN + 1, i32::MIN + 2);
        let _large_rat5 = Ratio::new(i32::MIN + 2, i32::MAX);
        let _large_rat6 = Ratio::new(i32::MAX, i32::MIN + 2);
        let _large_rat7 = Ratio::new(1, i32::MIN + 1);
        let _large_rat8 = Ratio::new(1, i32::MAX);

        assert_eq!(_large_rat1.round(), One::one());
        assert_eq!(_large_rat2.round(), One::one());
        assert_eq!(_large_rat3.round(), One::one());
        assert_eq!(_large_rat4.round(), One::one());
        assert_eq!(_large_rat5.round(), _neg1);
        assert_eq!(_large_rat6.round(), _neg1);
        assert_eq!(_large_rat7.round(), Zero::zero());
        assert_eq!(_large_rat8.round(), Zero::zero());
    }

    #[test]
    fn test_fract() {
        assert_eq!(_1.fract(), _0);
        assert_eq!(_NEG1_2.fract(), _NEG1_2);
        assert_eq!(_1_2.fract(), _1_2);
        assert_eq!(_3_2.fract(), _1_2);
    }

    #[test]
    fn test_recip() {
        assert_eq!(_1 * _1.recip(), _1);
        assert_eq!(_2 * _2.recip(), _1);
        assert_eq!(_1_2 * _1_2.recip(), _1);
        assert_eq!(_3_2 * _3_2.recip(), _1);
        assert_eq!(_NEG1_2 * _NEG1_2.recip(), _1);

        assert_eq!(_3_2.recip(), _2_3);
        assert_eq!(_NEG1_2.recip(), _NEG2);
        assert_eq!(_NEG1_2.recip().denom(), &1);
    }

    #[test]
    #[should_panic(expected = "== 0")]
    fn test_recip_fail() {
        let _a = Ratio::new(0, 1).recip();
    }

    #[test]
    fn test_pow() {
        fn test(r: Rational, e: i32, expected: Rational) {
            assert_eq!(r.pow(e), expected);
            assert_eq!(Pow::pow(r, e), expected);
            assert_eq!(Pow::pow(r, &e), expected);
            assert_eq!(Pow::pow(&r, e), expected);
            assert_eq!(Pow::pow(&r, &e), expected);
        }

        test(_1_2, 2, Ratio::new(1, 4));
        test(_1_2, -2, Ratio::new(4, 1));
        test(_1, 1, _1);
        test(_1, i32::MAX, _1);
        test(_1, i32::MIN, _1);
        test(_NEG1_2, 2, _1_2.pow(2i32));
        test(_NEG1_2, 3, -_1_2.pow(3i32));
        test(_3_2, 0, _1);
        test(_3_2, -1, _3_2.recip());
        test(_3_2, 3, Ratio::new(27, 8));
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_to_from_str() {
        use std::string::{String, ToString};
        fn test(r: Rational, s: String) {
            assert_eq!(FromStr::from_str(&s), Ok(r));
            assert_eq!(r.to_string(), s);
        }
        test(_1, "1".to_string());
        test(_0, "0".to_string());
        test(_1_2, "1/2".to_string());
        test(_3_2, "3/2".to_string());
        test(_2, "2".to_string());
        test(_NEG1_2, "-1/2".to_string());
    }
    #[test]
    fn test_from_str_fail() {
        fn test(s: &str) {
            let rational: Result<Rational, _> = FromStr::from_str(s);
            assert!(rational.is_err());
        }

        let xs = ["0 /1", "abc", "", "1/", "--1/2", "3/2/1", "1/0"];
        for &s in xs.iter() {
            test(s);
        }
    }

    #[cfg(feature = "bigint")]
    #[test]
    fn test_from_float() {
        use traits::float::FloatCore;
        fn test<T: FloatCore>(given: T, (numer, denom): (&str, &str)) {
            let ratio: BigRational = Ratio::from_float(given).unwrap();
            assert_eq!(
                ratio,
                Ratio::new(
                    FromStr::from_str(numer).unwrap(),
                    FromStr::from_str(denom).unwrap()
                )
            );
        }

        // f32
        test(3.14159265359f32, ("13176795", "4194304"));
        test(2f32.powf(100.), ("1267650600228229401496703205376", "1"));
        test(-2f32.powf(100.), ("-1267650600228229401496703205376", "1"));
        test(
            1.0 / 2f32.powf(100.),
            ("1", "1267650600228229401496703205376"),
        );
        test(684729.48391f32, ("1369459", "2"));
        test(-8573.5918555f32, ("-4389679", "512"));

        // f64
        test(3.14159265359f64, ("3537118876014453", "1125899906842624"));
        test(2f64.powf(100.), ("1267650600228229401496703205376", "1"));
        test(-2f64.powf(100.), ("-1267650600228229401496703205376", "1"));
        test(684729.48391f64, ("367611342500051", "536870912"));
        test(-8573.5918555f64, ("-4713381968463931", "549755813888"));
        test(
            1.0 / 2f64.powf(100.),
            ("1", "1267650600228229401496703205376"),
        );
    }

    #[cfg(feature = "bigint")]
    #[test]
    fn test_from_float_fail() {
        use core::{f32, f64};

        assert_eq!(Ratio::from_float(f32::NAN), None);
        assert_eq!(Ratio::from_float(f32::INFINITY), None);
        assert_eq!(Ratio::from_float(f32::NEG_INFINITY), None);
        assert_eq!(Ratio::from_float(f64::NAN), None);
        assert_eq!(Ratio::from_float(f64::INFINITY), None);
        assert_eq!(Ratio::from_float(f64::NEG_INFINITY), None);
    }

    #[test]
    fn test_signed() {
        assert_eq!(_NEG1_2.abs(), _1_2);
        assert_eq!(_3_2.abs_sub(&_1_2), _1);
        assert_eq!(_1_2.abs_sub(&_3_2), Zero::zero());
        assert_eq!(_1_2.signum(), One::one());
        assert_eq!(_NEG1_2.signum(), -<Ratio<isize>>::one());
        assert_eq!(_0.signum(), Zero::zero());
        assert!(_NEG1_2.is_negative());
        assert!(_1_NEG2.is_negative());
        assert!(!_NEG1_2.is_positive());
        assert!(!_1_NEG2.is_positive());
        assert!(_1_2.is_positive());
        assert!(_NEG1_NEG2.is_positive());
        assert!(!_1_2.is_negative());
        assert!(!_NEG1_NEG2.is_negative());
        assert!(!_0.is_positive());
        assert!(!_0.is_negative());
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_hash() {
        assert!(::hash(&_0) != ::hash(&_1));
        assert!(::hash(&_0) != ::hash(&_3_2));

        // a == b -> hash(a) == hash(b)
        let a = Rational::new_raw(4, 2);
        let b = Rational::new_raw(6, 3);
        assert_eq!(a, b);
        assert_eq!(::hash(&a), ::hash(&b));

        let a = Rational::new_raw(123456789, 1000);
        let b = Rational::new_raw(123456789 * 5, 5000);
        assert_eq!(a, b);
        assert_eq!(::hash(&a), ::hash(&b));
    }

    #[test]
    fn test_into_pair() {
        assert_eq!((0, 1), _0.into());
        assert_eq!((-2, 1), _NEG2.into());
        assert_eq!((1, -2), _1_NEG2.into());
    }

    #[test]
    fn test_from_pair() {
        assert_eq!(_0, Ratio::from((0, 1)));
        assert_eq!(_1, Ratio::from((1, 1)));
        assert_eq!(_NEG2, Ratio::from((-2, 1)));
        assert_eq!(_1_NEG2, Ratio::from((1, -2)));
    }

    #[test]
    fn ratio_iter_sum() {
        // generic function to assure the iter method can be called
        // for any Iterator with Item = Ratio<impl Integer> or Ratio<&impl Integer>
        fn iter_sums<T: Integer + Clone>(slice: &[Ratio<T>]) -> [Ratio<T>; 3] {
            let mut manual_sum = Ratio::new(T::zero(), T::one());
            for ratio in slice {
                manual_sum = manual_sum + ratio;
            }
            [manual_sum, slice.iter().sum(), slice.iter().cloned().sum()]
        }
        // collect into array so test works on no_std
        let mut nums = [Ratio::new(0, 1); 1000];
        for (i, r) in (0..1000).map(|n| Ratio::new(n, 500)).enumerate() {
            nums[i] = r;
        }
        let sums = iter_sums(&nums[..]);
        assert_eq!(sums[0], sums[1]);
        assert_eq!(sums[0], sums[2]);
    }

    #[test]
    fn ratio_iter_product() {
        // generic function to assure the iter method can be called
        // for any Iterator with Item = Ratio<impl Integer> or Ratio<&impl Integer>
        fn iter_products<T: Integer + Clone>(slice: &[Ratio<T>]) -> [Ratio<T>; 3] {
            let mut manual_prod = Ratio::new(T::one(), T::one());
            for ratio in slice {
                manual_prod = manual_prod * ratio;
            }
            [
                manual_prod,
                slice.iter().product(),
                slice.iter().cloned().product(),
            ]
        }

        // collect into array so test works on no_std
        let mut nums = [Ratio::new(0, 1); 1000];
        for (i, r) in (0..1000).map(|n| Ratio::new(n, 500)).enumerate() {
            nums[i] = r;
        }
        let products = iter_products(&nums[..]);
        assert_eq!(products[0], products[1]);
        assert_eq!(products[0], products[2]);
    }
}

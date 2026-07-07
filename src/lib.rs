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
//! The `num-rational` crate is tested for rustc 1.60 and greater.

#![no_std]

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

#[cfg(feature = "num-bigint")]
use num_bigint::BigInt;

use num_integer::Integer;
use num_traits::{ConstOne, ConstZero, One, Pow, Signed, Zero};

pub use crate::str::ParseRatioError;

mod cmp;
mod convert;
mod iter;
mod opassign;
mod ops;
mod pow;
mod str;

#[cfg(test)]
mod tests;

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
#[deprecated(
    since = "0.4.0",
    note = "it's better to use a specific size, like `Rational32` or `Rational64`"
)]
pub type Rational = Ratio<isize>;
/// Alias for a `Ratio` of 32-bit-sized integers.
pub type Rational32 = Ratio<i32>;
/// Alias for a `Ratio` of 64-bit-sized integers.
pub type Rational64 = Ratio<i64>;

#[cfg(feature = "num-bigint")]
/// Alias for arbitrary precision rationals.
pub type BigRational = Ratio<BigInt>;

/// These method are `const`.
impl<T> Ratio<T> {
    /// Creates a `Ratio` without checking for `denom == 0` or reducing.
    ///
    /// **There are several methods that will panic if used on a `Ratio` with
    /// `denom == 0`.**
    #[inline]
    pub const fn new_raw(numer: T, denom: T) -> Ratio<T> {
        Ratio { numer, denom }
    }

    /// Deconstructs a `Ratio` into its numerator and denominator.
    #[inline]
    pub fn into_raw(self) -> (T, T) {
        (self.numer, self.denom)
    }

    /// Gets an immutable reference to the numerator.
    #[inline]
    pub const fn numer(&self) -> &T {
        &self.numer
    }

    /// Gets an immutable reference to the denominator.
    #[inline]
    pub const fn denom(&self) -> &T {
        &self.denom
    }
}

impl<T: Clone + Integer> Ratio<T> {
    /// Creates a new `Ratio`.
    ///
    /// **Panics if `denom` is zero.**
    #[inline]
    pub fn new(numer: T, denom: T) -> Ratio<T> {
        let mut ret = Ratio::new_raw(numer, denom);
        ret.reduce();
        ret
    }

    /// Creates a `Ratio` representing the integer `t`.
    #[inline]
    pub fn from_integer(t: T) -> Ratio<T> {
        Ratio::new_raw(t, One::one())
    }

    /// Converts to an integer, rounding towards zero.
    #[inline]
    pub fn to_integer(&self) -> T {
        self.trunc().numer
    }

    /// Returns true if the rational number is an integer (denominator is 1).
    #[inline]
    pub fn is_integer(&self) -> bool {
        self.denom.is_one()
    }

    /// Puts self into lowest terms, with `denom` > 0.
    ///
    /// **Panics if `denom` is zero.**
    fn reduce(&mut self) {
        if self.denom.is_zero() {
            panic!("denominator == 0");
        }
        if self.numer.is_zero() {
            self.denom.set_one();
            return;
        }
        if self.numer == self.denom {
            self.set_one();
            return;
        }
        let g: T = self.numer.gcd(&self.denom);

        // FIXME(#5992): assignment operator overloads
        // T: Clone + Integer != T: Clone + NumAssign

        #[inline]
        fn replace_with<T: Zero>(x: &mut T, f: impl FnOnce(T) -> T) {
            let y = core::mem::replace(x, T::zero());
            *x = f(y);
        }

        // self.numer /= g;
        replace_with(&mut self.numer, |x| x / g.clone());

        // self.denom /= g;
        replace_with(&mut self.denom, |x| x / g);

        // keep denom positive!
        if self.denom < T::zero() {
            replace_with(&mut self.numer, |x| T::zero() - x);
            replace_with(&mut self.denom, |x| T::zero() - x);
        }
    }

    /// Returns a reduced copy of self.
    ///
    /// In general, it is not necessary to use this method, as the only
    /// method of procuring a non-reduced fraction is through `new_raw`.
    ///
    /// **Panics if `denom` is zero.**
    pub fn reduced(&self) -> Ratio<T> {
        let mut ret = self.clone();
        ret.reduce();
        ret
    }

    /// Returns the reciprocal.
    ///
    /// **Panics if the `Ratio` is zero.**
    #[inline]
    pub fn recip(&self) -> Ratio<T> {
        self.clone().into_recip()
    }

    #[inline]
    fn into_recip(self) -> Ratio<T> {
        use core::cmp::Ordering::*;

        match self.numer.cmp(&T::zero()) {
            Equal => panic!("division by zero"),
            Greater => Ratio::new_raw(self.denom, self.numer),
            Less => Ratio::new_raw(T::zero() - self.denom, T::zero() - self.numer),
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
        let half_or_larger = if fractional.denom.is_even() {
            fractional.numer >= fractional.denom / two
        } else {
            fractional.numer >= (fractional.denom / two) + one
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

    /// Raises the `Ratio` to the power of an exponent.
    #[inline]
    pub fn pow(&self, expon: i32) -> Ratio<T>
    where
        for<'a> &'a T: Pow<u32, Output = T>,
    {
        Pow::pow(self, expon)
    }
}

impl<T: Clone + Integer> Default for Ratio<T> {
    /// Returns zero
    fn default() -> Self {
        Ratio::zero()
    }
}

// Constants
impl<T: ConstZero + ConstOne> Ratio<T> {
    /// A constant `Ratio` 0/1.
    pub const ZERO: Self = Self::new_raw(T::ZERO, T::ONE);
}

impl<T: Clone + Integer + ConstZero + ConstOne> ConstZero for Ratio<T> {
    const ZERO: Self = Self::ZERO;
}

impl<T: Clone + Integer> Zero for Ratio<T> {
    #[inline]
    fn zero() -> Ratio<T> {
        Ratio::new_raw(Zero::zero(), One::one())
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.numer.is_zero()
    }

    #[inline]
    fn set_zero(&mut self) {
        self.numer.set_zero();
        self.denom.set_one();
    }
}

impl<T: ConstOne> Ratio<T> {
    /// A constant `Ratio` 1/1.
    pub const ONE: Self = Self::new_raw(T::ONE, T::ONE);
}

impl<T: Clone + Integer + ConstOne> ConstOne for Ratio<T> {
    const ONE: Self = Self::ONE;
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

    #[inline]
    fn set_one(&mut self) {
        self.numer.set_one();
        self.denom.set_one();
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
        let (numer, denom): (T, T) = serde::Deserialize::deserialize(deserializer)?;
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

// Ratio ops often use other "suspicious" ops
#![allow(clippy::suspicious_arithmetic_impl)]
#![allow(clippy::suspicious_op_assign_impl)]

use super::Ratio;

use core::ops::{Add, Div, Mul, Neg, Rem, Sub};

use num_integer::Integer;
use num_traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, Inv, One, Zero};

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
// a/b * c/d = (a/gcd_ad)*(c/gcd_bc) / ((d/gcd_ad)*(b/gcd_bc))
impl<T> Mul<Ratio<T>> for Ratio<T>
where
    T: Clone + Integer,
{
    type Output = Ratio<T>;
    #[inline]
    fn mul(self, rhs: Ratio<T>) -> Ratio<T> {
        let gcd_ad = self.numer.gcd(&rhs.denom);
        let gcd_bc = self.denom.gcd(&rhs.numer);
        Ratio::new(
            self.numer / gcd_ad.clone() * (rhs.numer / gcd_bc.clone()),
            self.denom / gcd_bc * (rhs.denom / gcd_ad),
        )
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
        let gcd = self.denom.gcd(&rhs);
        Ratio::new(self.numer * (rhs / gcd.clone()), self.denom / gcd)
    }
}

forward_all_binop!(impl Div, div);
// (a/b) / (c/d) = (a/gcd_ac)*(d/gcd_bd) / ((c/gcd_ac)*(b/gcd_bd))
impl<T> Div<Ratio<T>> for Ratio<T>
where
    T: Clone + Integer,
{
    type Output = Ratio<T>;

    #[inline]
    fn div(self, rhs: Ratio<T>) -> Ratio<T> {
        let gcd_ac = self.numer.gcd(&rhs.numer);
        let gcd_bd = self.denom.gcd(&rhs.denom);
        Ratio::new(
            self.numer / gcd_ac.clone() * (rhs.denom / gcd_bd.clone()),
            self.denom / gcd_bd * (rhs.numer / gcd_ac),
        )
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
        let gcd = self.numer.gcd(&rhs);
        Ratio::new(self.numer / gcd.clone(), self.denom * (rhs / gcd))
    }
}

macro_rules! arith_impl {
    (impl $imp:ident, $method:ident) => {
        forward_all_binop!(impl $imp, $method);
        // Abstracts a/b `op` c/d = (a*lcm/b `op` c*lcm/d)/lcm where lcm = lcm(b,d)
        impl<T: Clone + Integer> $imp<Ratio<T>> for Ratio<T> {
            type Output = Ratio<T>;
            #[inline]
            fn $method(self, rhs: Ratio<T>) -> Ratio<T> {
                if self.denom == rhs.denom {
                    return Ratio::new(self.numer.$method(rhs.numer), rhs.denom);
                }
                let lcm = self.denom.lcm(&rhs.denom);
                let lhs_numer = self.numer * (lcm.clone() / self.denom);
                let rhs_numer = rhs.numer * (lcm.clone() / rhs.denom);
                Ratio::new(lhs_numer.$method(rhs_numer), lcm)
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

// a/b * c/d = (a*c)/(b*d)
impl<T> CheckedMul for Ratio<T>
where
    T: Clone + Integer + CheckedMul,
{
    #[inline]
    fn checked_mul(&self, rhs: &Ratio<T>) -> Option<Ratio<T>> {
        let gcd_ad = self.numer.gcd(&rhs.denom);
        let gcd_bc = self.denom.gcd(&rhs.numer);
        Some(Ratio::new(
            (self.numer.clone() / gcd_ad.clone())
                .checked_mul(&(rhs.numer.clone() / gcd_bc.clone()))?,
            (self.denom.clone() / gcd_bc).checked_mul(&(rhs.denom.clone() / gcd_ad))?,
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
        if rhs.is_zero() {
            return None;
        }
        let (numer, denom) = if self.denom == rhs.denom {
            (self.numer.clone(), rhs.numer.clone())
        } else if self.numer == rhs.numer {
            (rhs.denom.clone(), self.denom.clone())
        } else {
            let gcd_ac = self.numer.gcd(&rhs.numer);
            let gcd_bd = self.denom.gcd(&rhs.denom);
            (
                (self.numer.clone() / gcd_ac.clone())
                    .checked_mul(&(rhs.denom.clone() / gcd_bd.clone()))?,
                (self.denom.clone() / gcd_bd).checked_mul(&(rhs.numer.clone() / gcd_ac))?,
            )
        };
        // Manual `reduce()`, avoiding sharp edges
        if denom.is_zero() {
            None
        } else if numer.is_zero() {
            Some(Self::zero())
        } else if numer == denom {
            Some(Self::one())
        } else {
            let g = numer.gcd(&denom);
            let numer = numer / g.clone();
            let denom = denom / g;
            let raw = if denom < T::zero() {
                // We need to keep denom positive, but 2's-complement MIN may
                // overflow negation -- instead we can check multiplying -1.
                let n1 = T::zero() - T::one();
                Ratio::new_raw(numer.checked_mul(&n1)?, denom.checked_mul(&n1)?)
            } else {
                Ratio::new_raw(numer, denom)
            };
            Some(raw)
        }
    }
}

// As arith_impl! but for Checked{Add,Sub} traits
macro_rules! checked_arith_impl {
    (impl $imp:ident, $method:ident) => {
        impl<T: Clone + Integer + CheckedMul + $imp> $imp for Ratio<T> {
            #[inline]
            fn $method(&self, rhs: &Ratio<T>) -> Option<Ratio<T>> {
                let gcd = self.denom.clone().gcd(&rhs.denom);
                let lcm = (self.denom.clone() / gcd.clone()).checked_mul(&rhs.denom)?;
                let lhs_numer = (lcm.clone() / self.denom.clone()).checked_mul(&self.numer)?;
                let rhs_numer = (lcm.clone() / rhs.denom.clone()).checked_mul(&rhs.numer)?;
                Some(Ratio::new(lhs_numer.$method(&rhs_numer)?, lcm))
            }
        }
    };
}

// a/b + c/d = (lcm/b*a + lcm/d*c)/lcm, where lcm = lcm(b,d)
checked_arith_impl!(impl CheckedAdd, checked_add);

// a/b - c/d = (lcm/b*a - lcm/d*c)/lcm, where lcm = lcm(b,d)
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

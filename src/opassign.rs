// Ratio ops often use other "suspicious" ops
#![allow(clippy::suspicious_arithmetic_impl)]
#![allow(clippy::suspicious_op_assign_impl)]

use core::ops::{AddAssign, DivAssign, MulAssign, RemAssign, SubAssign};

use crate::Ratio;
use num_integer::Integer;
use num_traits::NumAssign;

impl<T: Clone + Integer + NumAssign> AddAssign for Ratio<T> {
    fn add_assign(&mut self, other: Ratio<T>) {
        if self.denom == other.denom {
            self.numer += other.numer
        } else {
            let lcm = self.denom.lcm(&other.denom);
            let lhs_numer = self.numer.clone() * (lcm.clone() / self.denom.clone());
            let rhs_numer = other.numer * (lcm.clone() / other.denom);
            self.numer = lhs_numer + rhs_numer;
            self.denom = lcm;
        }
        self.reduce();
    }
}

// (a/b) / (c/d) = (a/gcd_ac)*(d/gcd_bd) / ((c/gcd_ac)*(b/gcd_bd))
impl<T: Clone + Integer + NumAssign> DivAssign for Ratio<T> {
    fn div_assign(&mut self, other: Ratio<T>) {
        let gcd_ac = self.numer.gcd(&other.numer);
        let gcd_bd = self.denom.gcd(&other.denom);
        self.numer /= gcd_ac.clone();
        self.numer *= other.denom / gcd_bd.clone();
        self.denom /= gcd_bd;
        self.denom *= other.numer / gcd_ac;
        self.reduce(); // TODO: remove this line. see #8.
    }
}

// a/b * c/d = (a/gcd_ad)*(c/gcd_bc) / ((d/gcd_ad)*(b/gcd_bc))
impl<T: Clone + Integer + NumAssign> MulAssign for Ratio<T> {
    fn mul_assign(&mut self, other: Ratio<T>) {
        let gcd_ad = self.numer.gcd(&other.denom);
        let gcd_bc = self.denom.gcd(&other.numer);
        self.numer /= gcd_ad.clone();
        self.numer *= other.numer / gcd_bc.clone();
        self.denom /= gcd_bc;
        self.denom *= other.denom / gcd_ad;
        self.reduce(); // TODO: remove this line. see #8.
    }
}

impl<T: Clone + Integer + NumAssign> RemAssign for Ratio<T> {
    fn rem_assign(&mut self, other: Ratio<T>) {
        if self.denom == other.denom {
            self.numer %= other.numer
        } else {
            let lcm = self.denom.lcm(&other.denom);
            let lhs_numer = self.numer.clone() * (lcm.clone() / self.denom.clone());
            let rhs_numer = other.numer * (lcm.clone() / other.denom);
            self.numer = lhs_numer % rhs_numer;
            self.denom = lcm;
        }
        self.reduce();
    }
}

impl<T: Clone + Integer + NumAssign> SubAssign for Ratio<T> {
    fn sub_assign(&mut self, other: Ratio<T>) {
        if self.denom == other.denom {
            self.numer -= other.numer
        } else {
            let lcm = self.denom.lcm(&other.denom);
            let lhs_numer = self.numer.clone() * (lcm.clone() / self.denom.clone());
            let rhs_numer = other.numer * (lcm.clone() / other.denom);
            self.numer = lhs_numer - rhs_numer;
            self.denom = lcm;
        }
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
        let gcd = self.numer.gcd(&other);
        self.numer /= gcd.clone();
        self.denom *= other / gcd;
        self.reduce(); // TODO: remove this line. see #8.
    }
}

impl<T: Clone + Integer + NumAssign> MulAssign<T> for Ratio<T> {
    fn mul_assign(&mut self, other: T) {
        let gcd = self.denom.gcd(&other);
        self.denom /= gcd.clone();
        self.numer *= other / gcd;
        self.reduce(); // TODO: remove this line. see #8.
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

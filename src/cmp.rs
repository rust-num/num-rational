use super::Ratio;

use core::cmp::Ordering;
use core::hash::{Hash, Hasher};

use num_integer::Integer;

// Comparisons

// Mathematically, comparing a/b and c/d is the same as comparing a*d and b*c, but it's very easy
// for those multiplications to overflow fixed-size integers, so we need to take care.

impl<T: Clone + Integer> Ord for Ratio<T> {

    fn cmp(&self, other: &Self) -> cmp::Ordering {

        use cmp::Ordering;

        let zero = T::zero();

        // makes self.denom > 0 and other.denom > 0
        if self.denom <= zero || other.denom <= zero {
            return self.reduced().cmp(&other.reduced());
        }

        // return an `Ordering` for the following case
        // let a/b = lhs, c/d = rhs
        // 0 < c < a and 0 < b and 0 < d 
        #[inline]
        fn reduce_cmp<T: Integer>(
            lhs: &Ratio<T>, 
            rhs: &Ratio<T>
        ) -> Ordering {
            if lhs.denom <= rhs.denom {
                return Ordering::Greater;
            }

            let zero = T::zero();

            let (q_n, mut a) = lhs.numer.div_rem(&rhs.numer); // 0 < c < a ∴ q_n ≥ 1
            let (q_d, mut b) = lhs.denom.div_rem(&rhs.denom); // 0 < d < b ∴ q_d ≥ 1
            let mut ord = q_n.cmp(&q_d);
            if ord.is_ne() {
                return ord;
            }
            match (a == zero, b == zero) {
                // a = 0, b = 0 ∴ a*d = 0 = b*c
                (true, true) => return Ordering::Equal,
                // 0 = a, 0 < b, 0 < c ∴ a*d = 0 < b*c
                (true, false) => return Ordering::Less,
                // 0 < a, b = 0, 0 < d ∴ a*d > 0 = b*c
                (false, true) => return Ordering::Greater,
                (false, false) => (),
            }
            let (mut q_n, mut c) = rhs.numer.div_rem(&a); // 0 < a < c ∴ q_n ≥ 1
            let (mut q_d, mut d) = rhs.denom.div_rem(&b); // 0 < b < d ∴ q_d ≥ 1
            ord = q_d.cmp(&q_n); // d/b ?= c/a
            while ord.is_eq() {
                match (c == zero, d == zero) {
                    // c = 0, d = 0 ∴ a*d = 0 = b*c
                    (true, true) => return Ordering::Equal,
                    // 0 < a, c = 0, 0 < d ∴ a*d > 0 = b*c
                    (true, false) => return Ordering::Greater,
                    // 0 < b, 0 < c, d = 0 ∴ a*d = 0 < b*c
                    (false, true) => return Ordering::Less,
                    (false, false) => (),
                }
                (a, b, c, d) = (d, c, b, a);
                (q_n, c) = c.div_rem(&a); // 0 < a < c ∴ q_n ≥ 1
                (q_d, d) = d.div_rem(&b); // 0 < b < d ∴ q_d ≥ 1
                ord = q_d.cmp(&q_n); // d/b ?= c/a
            }
            ord
        }

        match self.numer.cmp(&zero) {
            Ordering::Greater => {
                if other.numer <= zero {
                    return Ordering::Greater;
                }
                match self.numer.cmp(&other.numer) {
                    Ordering::Greater => reduce_cmp(self, other),
                    Ordering::Less => reduce_cmp(other, self).reverse(),
                    Ordering::Equal => other.denom.cmp(&self.denom),
                }
            },
            Ordering::Less => {
                if  zero <= other.numer {
                    return Ordering::Less;
                }
                match self.numer.cmp(&other.numer) {
                    Ordering::Greater => reduce_cmp(other, self),
                    Ordering::Less => reduce_cmp(self, other).reverse(),
                    Ordering::Equal => self.denom.cmp(&other.denom)
                }
            },
            Ordering::Equal => zero.cmp(&other.numer)
        }
    }
}

impl<T: Clone + Integer> PartialOrd for Ratio<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Clone + Integer> PartialEq for Ratio<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
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

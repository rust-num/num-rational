use super::Ratio;

use core::cmp::Ordering;
use core::hash::{Hash, Hasher};

use num_integer::Integer;

// Comparisons

// Mathematically, comparing a/b and c/d is the same as comparing a*d and b*c, but it's very easy
// for those multiplications to overflow fixed-size integers, so we need to take care.

impl<T: Clone + Integer> Ord for Ratio<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
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
            if self.numer.is_zero() {
                return Ordering::Equal;
            }
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
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
            Ordering::Equal => {
                match (self_rem.is_zero(), other_rem.is_zero()) {
                    (true, true) => Ordering::Equal,
                    (true, false) => Ordering::Less,
                    (false, true) => Ordering::Greater,
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

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

        let (mut lhs_nbuf, mut lhs_dbuf, mut rhs_nbuf, mut rhs_dbuf);
        let (mut lhs_numer, mut lhs_denom) = (&self.numer, &self.denom);
        let (mut rhs_numer, mut rhs_denom) = (&other.numer, &other.denom);
        loop {
            match cmp_int(lhs_numer, lhs_denom, rhs_numer, rhs_denom) {
                Ok(ord) => return ord,
                Err((lhs_rem, rhs_rem)) => {
                    lhs_nbuf = lhs_rem;
                    lhs_numer = &lhs_nbuf;
                    rhs_nbuf = rhs_rem;
                    rhs_numer = &rhs_nbuf;
                }
            }

            match cmp_int(lhs_denom, lhs_numer, rhs_denom, rhs_numer) {
                Ok(ord) => return ord.reverse(),
                Err((lhs_rem, rhs_rem)) => {
                    lhs_dbuf = lhs_rem;
                    lhs_denom = &lhs_dbuf;
                    rhs_dbuf = rhs_rem;
                    rhs_denom = &rhs_dbuf;
                }
            }
        }
    }
}

/// Compare as floored integers; if equal then return remainders
#[inline(always)]
fn cmp_int<T: Integer>(
    lhs_numer: &T,
    lhs_denom: &T,
    rhs_numer: &T,
    rhs_denom: &T,
) -> Result<Ordering, (T, T)> {
    // Compare as floored integers and remainders
    let (lhs_int, lhs_rem) = lhs_numer.div_mod_floor(lhs_denom);
    let (rhs_int, rhs_rem) = rhs_numer.div_mod_floor(rhs_denom);
    match lhs_int.cmp(&rhs_int) {
        Ordering::Greater => Ok(Ordering::Greater),
        Ordering::Less => Ok(Ordering::Less),
        Ordering::Equal => match (lhs_rem.is_zero(), rhs_rem.is_zero()) {
            (true, true) => Ok(Ordering::Equal),
            (true, false) => Ok(Ordering::Less),
            (false, true) => Ok(Ordering::Greater),
            (false, false) => Err((lhs_rem, rhs_rem)),
        },
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
        // This looping structure is similar to `Ord::cmp`
        let (mut nbuf, mut dbuf);
        let (mut numer, mut denom) = (&self.numer, &self.denom);

        loop {
            {
                let (int, rem) = numer.div_mod_floor(denom);
                int.hash(state);
                if rem.is_zero() {
                    rem.hash(state);
                    return;
                }
                nbuf = rem;
                numer = &nbuf;
            }

            {
                let (int, rem) = denom.div_mod_floor(numer);
                int.hash(state);
                if rem.is_zero() {
                    rem.hash(state);
                    return;
                }
                dbuf = rem;
                denom = &dbuf;
            }
        }
    }
}

use crate::Ratio;
use core::iter::{Product, Sum};
use num_integer::Integer;
use num_traits::{One, Zero};

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

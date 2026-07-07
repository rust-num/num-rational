use super::super::{Ratio, Rational64};
use super::{to_big, _0, _1, _1_2, _2, _3_2, _5_2, _MAX, _MAX_M1, _MIN, _MIN_P1, _NEG1_2};
use core::fmt::Debug;
use num_integer::Integer;
use num_traits::{Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, NumAssign};

#[test]
fn test_add() {
    fn test(a: Rational64, b: Rational64, c: Rational64) {
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
    fn test_assign(a: Rational64, b: i64, c: Rational64) {
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
fn test_add_overflow() {
    // compares Ratio(1, T::max_value()) + Ratio(1, T::max_value())
    // to Ratio(1+1, T::max_value()) for each integer type.
    // Previously, this calculation would overflow.
    fn test_add_typed_overflow<T>()
    where
        T: Integer + Bounded + Clone + Debug + NumAssign,
    {
        let _1_max = Ratio::new(T::one(), T::max_value());
        let _2_max = Ratio::new(T::one() + T::one(), T::max_value());
        assert_eq!(_1_max.clone() + _1_max.clone(), _2_max);
        assert_eq!(
            {
                let mut tmp = _1_max.clone();
                tmp += _1_max;
                tmp
            },
            _2_max
        );
    }
    test_add_typed_overflow::<u8>();
    test_add_typed_overflow::<u16>();
    test_add_typed_overflow::<u32>();
    test_add_typed_overflow::<u64>();
    test_add_typed_overflow::<usize>();
    test_add_typed_overflow::<u128>();

    test_add_typed_overflow::<i8>();
    test_add_typed_overflow::<i16>();
    test_add_typed_overflow::<i32>();
    test_add_typed_overflow::<i64>();
    test_add_typed_overflow::<isize>();
    test_add_typed_overflow::<i128>();
}

#[test]
fn test_sub() {
    fn test(a: Rational64, b: Rational64, c: Rational64) {
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
    fn test_assign(a: Rational64, b: i64, c: Rational64) {
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
fn test_sub_overflow() {
    // compares Ratio(1, T::max_value()) - Ratio(1, T::max_value()) to T::zero()
    // for each integer type. Previously, this calculation would overflow.
    fn test_sub_typed_overflow<T>()
    where
        T: Integer + Bounded + Clone + Debug + NumAssign,
    {
        let _1_max: Ratio<T> = Ratio::new(T::one(), T::max_value());
        assert!(T::is_zero(&(_1_max.clone() - _1_max.clone()).numer));
        {
            let mut tmp: Ratio<T> = _1_max.clone();
            tmp -= _1_max;
            assert!(T::is_zero(&tmp.numer));
        }
    }
    test_sub_typed_overflow::<u8>();
    test_sub_typed_overflow::<u16>();
    test_sub_typed_overflow::<u32>();
    test_sub_typed_overflow::<u64>();
    test_sub_typed_overflow::<usize>();
    test_sub_typed_overflow::<u128>();

    test_sub_typed_overflow::<i8>();
    test_sub_typed_overflow::<i16>();
    test_sub_typed_overflow::<i32>();
    test_sub_typed_overflow::<i64>();
    test_sub_typed_overflow::<isize>();
    test_sub_typed_overflow::<i128>();
}

#[test]
fn test_mul() {
    fn test(a: Rational64, b: Rational64, c: Rational64) {
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
    fn test_assign(a: Rational64, b: i64, c: Rational64) {
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
fn test_mul_overflow() {
    fn test_mul_typed_overflow<T>()
    where
        T: Integer + Bounded + Clone + Debug + NumAssign + CheckedMul,
    {
        let two = T::one() + T::one();
        let _3 = T::one() + T::one() + T::one();

        // 1/big * 2/3 = 1/(max/4*3), where big is max/2
        // make big = max/2, but also divisible by 2
        let big = T::max_value() / two.clone() / two.clone() * two.clone();
        let _1_big: Ratio<T> = Ratio::new(T::one(), big.clone());
        let _2_3: Ratio<T> = Ratio::new(two.clone(), _3.clone());
        assert_eq!(None, big.clone().checked_mul(&_3.clone()));
        let expected = Ratio::new(T::one(), big / two.clone() * _3.clone());
        assert_eq!(expected.clone(), _1_big.clone() * _2_3.clone());
        assert_eq!(
            Some(expected.clone()),
            _1_big.clone().checked_mul(&_2_3.clone())
        );
        assert_eq!(expected, {
            let mut tmp = _1_big;
            tmp *= _2_3;
            tmp
        });

        // big/3 * 3 = big/1
        // make big = max/2, but make it indivisible by 3
        let big = T::max_value() / two / _3.clone() * _3.clone() + T::one();
        assert_eq!(None, big.clone().checked_mul(&_3.clone()));
        let big_3 = Ratio::new(big.clone(), _3.clone());
        let expected = Ratio::new(big, T::one());
        assert_eq!(expected, big_3.clone() * _3.clone());
        assert_eq!(expected, {
            let mut tmp = big_3;
            tmp *= _3;
            tmp
        });
    }
    test_mul_typed_overflow::<u16>();
    test_mul_typed_overflow::<u8>();
    test_mul_typed_overflow::<u32>();
    test_mul_typed_overflow::<u64>();
    test_mul_typed_overflow::<usize>();
    test_mul_typed_overflow::<u128>();

    test_mul_typed_overflow::<i8>();
    test_mul_typed_overflow::<i16>();
    test_mul_typed_overflow::<i32>();
    test_mul_typed_overflow::<i64>();
    test_mul_typed_overflow::<isize>();
    test_mul_typed_overflow::<i128>();
}

#[test]
fn test_div() {
    fn test(a: Rational64, b: Rational64, c: Rational64) {
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
    fn test_assign(a: Rational64, b: i64, c: Rational64) {
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
fn test_div_overflow() {
    fn test_div_typed_overflow<T>()
    where
        T: Integer + Bounded + Clone + Debug + NumAssign + CheckedMul,
    {
        let two = T::one() + T::one();
        let _3 = T::one() + T::one() + T::one();

        // 1/big / 3/2 = 1/(max/4*3), where big is max/2
        // big ~ max/2, and big is divisible by 2
        let big = T::max_value() / two.clone() / two.clone() * two.clone();
        assert_eq!(None, big.clone().checked_mul(&_3.clone()));
        let _1_big: Ratio<T> = Ratio::new(T::one(), big.clone());
        let _3_two: Ratio<T> = Ratio::new(_3.clone(), two.clone());
        let expected = Ratio::new(T::one(), big / two.clone() * _3.clone());
        assert_eq!(expected.clone(), _1_big.clone() / _3_two.clone());
        assert_eq!(
            Some(expected.clone()),
            _1_big.clone().checked_div(&_3_two.clone())
        );
        assert_eq!(expected, {
            let mut tmp = _1_big;
            tmp /= _3_two;
            tmp
        });

        // 3/big / 3 = 1/big where big is max/2
        // big ~ max/2, and big is not divisible by 3
        let big = T::max_value() / two / _3.clone() * _3.clone() + T::one();
        assert_eq!(None, big.clone().checked_mul(&_3.clone()));
        let _3_big = Ratio::new(_3.clone(), big.clone());
        let expected = Ratio::new(T::one(), big);
        assert_eq!(expected, _3_big.clone() / _3.clone());
        assert_eq!(expected, {
            let mut tmp = _3_big;
            tmp /= _3;
            tmp
        });
    }
    test_div_typed_overflow::<u8>();
    test_div_typed_overflow::<u16>();
    test_div_typed_overflow::<u32>();
    test_div_typed_overflow::<u64>();
    test_div_typed_overflow::<usize>();
    test_div_typed_overflow::<u128>();

    test_div_typed_overflow::<i8>();
    test_div_typed_overflow::<i16>();
    test_div_typed_overflow::<i32>();
    test_div_typed_overflow::<i64>();
    test_div_typed_overflow::<isize>();
    test_div_typed_overflow::<i128>();
}

#[test]
fn test_rem() {
    fn test(a: Rational64, b: Rational64, c: Rational64) {
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
    fn test_assign(a: Rational64, b: i64, c: Rational64) {
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
    test(_3_2, _1_2, _0);
    test(_5_2, _3_2, _1);
    test(_2, _NEG1_2, _0);
    test(_1_2, _2, _1_2);
    test_assign(_3_2, 1, _1_2);
}

#[test]
fn test_rem_overflow() {
    // tests that Ratio(1,2) % Ratio(1, T::max_value()) equals 0
    // for each integer type. Previously, this calculation would overflow.
    fn test_rem_typed_overflow<T>()
    where
        T: Integer + Bounded + Clone + Debug + NumAssign,
    {
        let two = T::one() + T::one();
        // value near to maximum, but divisible by two
        let max_div2 = T::max_value() / two.clone() * two.clone();
        let _1_max: Ratio<T> = Ratio::new(T::one(), max_div2);
        let _1_two: Ratio<T> = Ratio::new(T::one(), two);
        assert!(T::is_zero(&(_1_two.clone() % _1_max.clone()).numer));
        {
            let mut tmp: Ratio<T> = _1_two;
            tmp %= _1_max;
            assert!(T::is_zero(&tmp.numer));
        }
    }
    test_rem_typed_overflow::<u8>();
    test_rem_typed_overflow::<u16>();
    test_rem_typed_overflow::<u32>();
    test_rem_typed_overflow::<u64>();
    test_rem_typed_overflow::<usize>();
    test_rem_typed_overflow::<u128>();

    test_rem_typed_overflow::<i8>();
    test_rem_typed_overflow::<i16>();
    test_rem_typed_overflow::<i32>();
    test_rem_typed_overflow::<i64>();
    test_rem_typed_overflow::<isize>();
    test_rem_typed_overflow::<i128>();
}

#[test]
fn test_neg() {
    fn test(a: Rational64, b: Rational64) {
        assert_eq!(-a, b);
        assert_eq!(-to_big(a), to_big(b))
    }

    test(_0, _0);
    test(_1_2, _NEG1_2);
    test(-_1, _1);
}
#[test]
#[allow(clippy::eq_op)]
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

#[test]
fn test_checked_zeros() {
    assert_eq!(_0.checked_add(&_0), Some(_0));
    assert_eq!(_0.checked_sub(&_0), Some(_0));
    assert_eq!(_0.checked_mul(&_0), Some(_0));
    assert_eq!(_0.checked_div(&_0), None);
}

#[test]
fn test_checked_min() {
    assert_eq!(_MIN.checked_add(&_MIN), None);
    assert_eq!(_MIN.checked_sub(&_MIN), Some(_0));
    assert_eq!(_MIN.checked_mul(&_MIN), None);
    assert_eq!(_MIN.checked_div(&_MIN), Some(_1));
    assert_eq!(_0.checked_add(&_MIN), Some(_MIN));
    assert_eq!(_0.checked_sub(&_MIN), None);
    assert_eq!(_0.checked_mul(&_MIN), Some(_0));
    assert_eq!(_0.checked_div(&_MIN), Some(_0));
    assert_eq!(_1.checked_add(&_MIN), Some(_MIN_P1));
    assert_eq!(_1.checked_sub(&_MIN), None);
    assert_eq!(_1.checked_mul(&_MIN), Some(_MIN));
    assert_eq!(_1.checked_div(&_MIN), None);
    assert_eq!(_MIN.checked_add(&_0), Some(_MIN));
    assert_eq!(_MIN.checked_sub(&_0), Some(_MIN));
    assert_eq!(_MIN.checked_mul(&_0), Some(_0));
    assert_eq!(_MIN.checked_div(&_0), None);
    assert_eq!(_MIN.checked_add(&_1), Some(_MIN_P1));
    assert_eq!(_MIN.checked_sub(&_1), None);
    assert_eq!(_MIN.checked_mul(&_1), Some(_MIN));
    assert_eq!(_MIN.checked_div(&_1), Some(_MIN));
}

#[test]
fn test_checked_max() {
    assert_eq!(_MAX.checked_add(&_MAX), None);
    assert_eq!(_MAX.checked_sub(&_MAX), Some(_0));
    assert_eq!(_MAX.checked_mul(&_MAX), None);
    assert_eq!(_MAX.checked_div(&_MAX), Some(_1));
    assert_eq!(_0.checked_add(&_MAX), Some(_MAX));
    assert_eq!(_0.checked_sub(&_MAX), Some(_MIN_P1));
    assert_eq!(_0.checked_mul(&_MAX), Some(_0));
    assert_eq!(_0.checked_div(&_MAX), Some(_0));
    assert_eq!(_1.checked_add(&_MAX), None);
    assert_eq!(_1.checked_sub(&_MAX), Some(-_MAX_M1));
    assert_eq!(_1.checked_mul(&_MAX), Some(_MAX));
    assert_eq!(_1.checked_div(&_MAX), Some(_MAX.recip()));
    assert_eq!(_MAX.checked_add(&_0), Some(_MAX));
    assert_eq!(_MAX.checked_sub(&_0), Some(_MAX));
    assert_eq!(_MAX.checked_mul(&_0), Some(_0));
    assert_eq!(_MAX.checked_div(&_0), None);
    assert_eq!(_MAX.checked_add(&_1), None);
    assert_eq!(_MAX.checked_sub(&_1), Some(_MAX_M1));
    assert_eq!(_MAX.checked_mul(&_1), Some(_MAX));
    assert_eq!(_MAX.checked_div(&_1), Some(_MAX));
}

#[test]
fn test_checked_min_max() {
    assert_eq!(_MIN.checked_add(&_MAX), Some(-_1));
    assert_eq!(_MIN.checked_sub(&_MAX), None);
    assert_eq!(_MIN.checked_mul(&_MAX), None);
    assert_eq!(
        _MIN.checked_div(&_MAX),
        Some(Ratio::new(_MIN.numer, _MAX.numer))
    );
    assert_eq!(_MAX.checked_add(&_MIN), Some(-_1));
    assert_eq!(_MAX.checked_sub(&_MIN), None);
    assert_eq!(_MAX.checked_mul(&_MIN), None);
    assert_eq!(_MAX.checked_div(&_MIN), None);
}

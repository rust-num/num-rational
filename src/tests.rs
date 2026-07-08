use super::convert::ldexp;
#[cfg(feature = "num-bigint")]
use super::{BigInt, BigRational};
use super::{Ratio, Rational64};

use core::f64;
use core::i32;
use core::i64;
use core::str::FromStr;
use num_integer::Integer;
use num_traits::ToPrimitive;
use num_traits::{FromPrimitive, One, Pow, Signed, Zero};

mod arith;

pub const _0: Rational64 = Ratio { numer: 0, denom: 1 };
pub const _1: Rational64 = Ratio { numer: 1, denom: 1 };
pub const _2: Rational64 = Ratio { numer: 2, denom: 1 };
pub const _NEG2: Rational64 = Ratio {
    numer: -2,
    denom: 1,
};
pub const _8: Rational64 = Ratio { numer: 8, denom: 1 };
pub const _15: Rational64 = Ratio {
    numer: 15,
    denom: 1,
};
pub const _16: Rational64 = Ratio {
    numer: 16,
    denom: 1,
};

pub const _1_2: Rational64 = Ratio { numer: 1, denom: 2 };
pub const _1_8: Rational64 = Ratio { numer: 1, denom: 8 };
pub const _1_15: Rational64 = Ratio {
    numer: 1,
    denom: 15,
};
pub const _1_16: Rational64 = Ratio {
    numer: 1,
    denom: 16,
};
pub const _3_2: Rational64 = Ratio { numer: 3, denom: 2 };
pub const _5_2: Rational64 = Ratio { numer: 5, denom: 2 };
pub const _NEG1_2: Rational64 = Ratio {
    numer: -1,
    denom: 2,
};
pub const _1_NEG2: Rational64 = Ratio {
    numer: 1,
    denom: -2,
};
pub const _NEG1_NEG2: Rational64 = Ratio {
    numer: -1,
    denom: -2,
};
pub const _1_3: Rational64 = Ratio { numer: 1, denom: 3 };
pub const _NEG1_3: Rational64 = Ratio {
    numer: -1,
    denom: 3,
};
pub const _2_3: Rational64 = Ratio { numer: 2, denom: 3 };
pub const _NEG2_3: Rational64 = Ratio {
    numer: -2,
    denom: 3,
};
pub const _MIN: Rational64 = Ratio {
    numer: i64::MIN,
    denom: 1,
};
pub const _MIN_P1: Rational64 = Ratio {
    numer: i64::MIN + 1,
    denom: 1,
};
pub const _MAX: Rational64 = Ratio {
    numer: i64::MAX,
    denom: 1,
};
pub const _MAX_M1: Rational64 = Ratio {
    numer: i64::MAX - 1,
    denom: 1,
};
pub const _BILLION: Rational64 = Ratio {
    numer: 1_000_000_000,
    denom: 1,
};

#[cfg(feature = "num-bigint")]
pub fn to_big(n: Rational64) -> BigRational {
    Ratio::new(
        FromPrimitive::from_i64(n.numer).unwrap(),
        FromPrimitive::from_i64(n.denom).unwrap(),
    )
}
#[cfg(not(feature = "num-bigint"))]
pub fn to_big(n: Rational64) -> Rational64 {
    Ratio::new(
        FromPrimitive::from_i64(n.numer).unwrap(),
        FromPrimitive::from_i64(n.denom).unwrap(),
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
    assert_eq!(Ratio::new(2, 2), One::one());
    assert_eq!(Ratio::new(0, i32::MIN), Zero::zero());
    assert_eq!(Ratio::new(i32::MIN, i32::MIN), One::one());
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
#[allow(clippy::eq_op)]
fn test_cmp() {
    assert!(_0 == _0 && _1 == _1);
    assert!(_0 != _1 && _1 != _0);
    assert!(_0 < _1 && !(_1 < _0));
    assert!(_1 > _0 && !(_0 > _1));

    assert!(_0 <= _0 && _1 <= _1);
    assert!(_0 <= _1 && !(_1 <= _0));

    assert!(_0 >= _0 && _1 >= _1);
    assert!(_1 >= _0 && !(_0 >= _1));

    let _0_2: Rational64 = Ratio::new_raw(0, 2);
    assert_eq!(_0, _0_2);
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
#[ignore] // 2^30 combinations are slow in debug, but manageable with `--release`
fn test_cmp_i8_full() {
    // All (non-reduced) `Ratio<i8>` with normal denom > 0
    let ratios = (i8::MIN..=i8::MAX)
        .flat_map(|numer| (1..=i8::MAX).map(move |denom| Ratio::new_raw(numer, denom)));
    for lhs in ratios.clone() {
        let lhs_float = f64::from(lhs.numer) / f64::from(lhs.denom);
        for rhs in ratios.clone() {
            let rhs_float = f64::from(rhs.numer) / f64::from(rhs.denom);
            let ord_ratio = lhs.cmp(&rhs);
            let ord_float = lhs_float.partial_cmp(&rhs_float).unwrap();
            assert_eq!(
                ord_ratio, ord_float,
                "{lhs} ({lhs_float}) <=> {rhs} ({rhs_float})"
            );

            #[cfg(feature = "std")]
            if ord_ratio.is_eq() {
                assert_eq!(hash_one(&lhs), hash_one(&rhs));
            }
        }
    }
}

#[cfg(feature = "num-bigint")]
fn big_ratios() -> (BigRational, BigRational) {
    let mut numer = BigInt::from(29u32);
    let mut denom = BigInt::from(28u32);
    for _ in 0..13 {
        numer = &numer * &numer;
        denom = &denom * &denom;
    }
    let one = BigInt::one();
    let a = Ratio::new_raw(numer, denom);
    let b = Ratio::new_raw(a.numer() + &one, a.denom() + &one);
    (a, b)
}

#[test]
#[cfg(feature = "num-bigint")]
fn test_cmp_stack_overflow() {
    // When `cmp` was recursive, this test caused a stack overflow. (num-rational#140)
    let (a, b) = big_ratios();
    assert!(a > b);
    assert!(a != b);
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

#[cfg(not(feature = "std"))]
use core::fmt::{self, Write};
#[cfg(not(feature = "std"))]
#[derive(Debug)]
struct NoStdTester {
    cursor: usize,
    buf: [u8; NoStdTester::BUF_SIZE],
}

#[cfg(not(feature = "std"))]
impl NoStdTester {
    fn new() -> NoStdTester {
        NoStdTester {
            buf: [0; Self::BUF_SIZE],
            cursor: 0,
        }
    }

    fn clear(&mut self) {
        self.buf = [0; Self::BUF_SIZE];
        self.cursor = 0;
    }

    const WRITE_ERR: &'static str = "Formatted output too long";
    const BUF_SIZE: usize = 32;
}

#[cfg(not(feature = "std"))]
impl Write for NoStdTester {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.buf[self.cursor] = byte;
            self.cursor += 1;
            if self.cursor >= self.buf.len() {
                return Err(fmt::Error {});
            }
        }
        Ok(())
    }
}

#[cfg(not(feature = "std"))]
impl PartialEq<str> for NoStdTester {
    fn eq(&self, other: &str) -> bool {
        let other = other.as_bytes();
        for index in 0..self.cursor {
            if self.buf.get(index) != other.get(index) {
                return false;
            }
        }
        true
    }
}

macro_rules! assert_fmt_eq {
    ($fmt_args:expr, $string:expr) => {
        #[cfg(not(feature = "std"))]
        {
            let mut tester = NoStdTester::new();
            write!(tester, "{}", $fmt_args).expect(NoStdTester::WRITE_ERR);
            assert_eq!(tester, *$string);
            tester.clear();
        }
        #[cfg(feature = "std")]
        {
            assert_eq!(std::fmt::format($fmt_args), $string);
        }
    };
}

#[test]
fn test_show() {
    // Test:
    // :b :o :x, :X, :?
    // alternate or not (#)
    // positive and negative
    // padding
    // does not test precision (i.e. truncation)
    assert_fmt_eq!(format_args!("{}", _2), "2");
    assert_fmt_eq!(format_args!("{:+}", _2), "+2");
    assert_fmt_eq!(format_args!("{:-}", _2), "2");
    assert_fmt_eq!(format_args!("{}", _1_2), "1/2");
    assert_fmt_eq!(format_args!("{}", -_1_2), "-1/2"); // test negatives
    assert_fmt_eq!(format_args!("{}", _0), "0");
    assert_fmt_eq!(format_args!("{}", -_2), "-2");
    assert_fmt_eq!(format_args!("{:+}", -_2), "-2");
    assert_fmt_eq!(format_args!("{:b}", _2), "10");
    assert_fmt_eq!(format_args!("{:#b}", _2), "0b10");
    assert_fmt_eq!(format_args!("{:b}", _1_2), "1/10");
    assert_fmt_eq!(format_args!("{:+b}", _1_2), "+1/10");
    assert_fmt_eq!(format_args!("{:-b}", _1_2), "1/10");
    assert_fmt_eq!(format_args!("{:b}", _0), "0");
    assert_fmt_eq!(format_args!("{:#b}", _1_2), "0b1/0b10");
    // no std does not support padding
    #[cfg(feature = "std")]
    assert_eq!(&format!("{:010b}", _1_2), "0000001/10");
    #[cfg(feature = "std")]
    assert_eq!(&format!("{:#010b}", _1_2), "0b001/0b10");
    let half_i8: Ratio<i8> = Ratio::new(1_i8, 2_i8);
    assert_fmt_eq!(format_args!("{:b}", -half_i8), "11111111/10");
    assert_fmt_eq!(format_args!("{:#b}", -half_i8), "0b11111111/0b10");
    #[cfg(feature = "std")]
    assert_eq!(&format!("{:05}", Ratio::new(-1_i8, 1_i8)), "-0001");

    assert_fmt_eq!(format_args!("{:o}", _8), "10");
    assert_fmt_eq!(format_args!("{:o}", _1_8), "1/10");
    assert_fmt_eq!(format_args!("{:o}", _0), "0");
    assert_fmt_eq!(format_args!("{:#o}", _1_8), "0o1/0o10");
    #[cfg(feature = "std")]
    assert_eq!(&format!("{:010o}", _1_8), "0000001/10");
    #[cfg(feature = "std")]
    assert_eq!(&format!("{:#010o}", _1_8), "0o001/0o10");
    assert_fmt_eq!(format_args!("{:o}", -half_i8), "377/2");
    assert_fmt_eq!(format_args!("{:#o}", -half_i8), "0o377/0o2");

    assert_fmt_eq!(format_args!("{:x}", _16), "10");
    assert_fmt_eq!(format_args!("{:x}", _15), "f");
    assert_fmt_eq!(format_args!("{:x}", _1_16), "1/10");
    assert_fmt_eq!(format_args!("{:x}", _1_15), "1/f");
    assert_fmt_eq!(format_args!("{:x}", _0), "0");
    assert_fmt_eq!(format_args!("{:#x}", _1_16), "0x1/0x10");
    #[cfg(feature = "std")]
    assert_eq!(&format!("{:010x}", _1_16), "0000001/10");
    #[cfg(feature = "std")]
    assert_eq!(&format!("{:#010x}", _1_16), "0x001/0x10");
    assert_fmt_eq!(format_args!("{:x}", -half_i8), "ff/2");
    assert_fmt_eq!(format_args!("{:#x}", -half_i8), "0xff/0x2");

    assert_fmt_eq!(format_args!("{:X}", _16), "10");
    assert_fmt_eq!(format_args!("{:X}", _15), "F");
    assert_fmt_eq!(format_args!("{:X}", _1_16), "1/10");
    assert_fmt_eq!(format_args!("{:X}", _1_15), "1/F");
    assert_fmt_eq!(format_args!("{:X}", _0), "0");
    assert_fmt_eq!(format_args!("{:#X}", _1_16), "0x1/0x10");
    #[cfg(feature = "std")]
    assert_eq!(format!("{:010X}", _1_16), "0000001/10");
    #[cfg(feature = "std")]
    assert_eq!(format!("{:#010X}", _1_16), "0x001/0x10");
    assert_fmt_eq!(format_args!("{:X}", -half_i8), "FF/2");
    assert_fmt_eq!(format_args!("{:#X}", -half_i8), "0xFF/0x2");

    assert_fmt_eq!(format_args!("{:e}", -_2), "-2e0");
    assert_fmt_eq!(format_args!("{:#e}", -_2), "-2e0");
    assert_fmt_eq!(format_args!("{:+e}", -_2), "-2e0");
    assert_fmt_eq!(format_args!("{:e}", _BILLION), "1e9");
    assert_fmt_eq!(format_args!("{:+e}", _BILLION), "+1e9");
    assert_fmt_eq!(format_args!("{:e}", _BILLION.recip()), "1e0/1e9");
    assert_fmt_eq!(format_args!("{:+e}", _BILLION.recip()), "+1e0/1e9");

    assert_fmt_eq!(format_args!("{:E}", -_2), "-2E0");
    assert_fmt_eq!(format_args!("{:#E}", -_2), "-2E0");
    assert_fmt_eq!(format_args!("{:+E}", -_2), "-2E0");
    assert_fmt_eq!(format_args!("{:E}", _BILLION), "1E9");
    assert_fmt_eq!(format_args!("{:+E}", _BILLION), "+1E9");
    assert_fmt_eq!(format_args!("{:E}", _BILLION.recip()), "1E0/1E9");
    assert_fmt_eq!(format_args!("{:+E}", _BILLION.recip()), "+1E0/1E9");
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
#[should_panic(expected = "division by zero")]
fn test_recip_fail() {
    let _a = Ratio::new(0, 1).recip();
}

#[test]
fn test_pow() {
    fn test(r: Rational64, e: i32, expected: Rational64) {
        assert_eq!(r.pow(e), expected);
        assert_eq!(Pow::pow(r, e), expected);
        assert_eq!(Pow::pow(r, &e), expected);
        assert_eq!(Pow::pow(&r, e), expected);
        assert_eq!(Pow::pow(&r, &e), expected);
        #[cfg(feature = "num-bigint")]
        test_big(r, e, expected);
    }

    #[cfg(feature = "num-bigint")]
    fn test_big(r: Rational64, e: i32, expected: Rational64) {
        let r = BigRational::new_raw(r.numer.into(), r.denom.into());
        let expected = BigRational::new_raw(expected.numer.into(), expected.denom.into());
        assert_eq!((&r).pow(e), expected);
        assert_eq!(Pow::pow(r.clone(), e), expected);
        assert_eq!(Pow::pow(r.clone(), &e), expected);
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
    fn test(r: Rational64, s: String) {
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
        let rational: Result<Rational64, _> = FromStr::from_str(s);
        assert!(rational.is_err());
    }

    let xs = ["0 /1", "abc", "", "1/", "--1/2", "3/2/1", "1/0"];
    for &s in xs.iter() {
        test(s);
    }
}

#[cfg(feature = "num-bigint")]
#[test]
fn test_from_float() {
    use num_traits::float::FloatCore;
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
    test(core::f32::consts::PI, ("13176795", "4194304"));
    test(2f32.powf(100.), ("1267650600228229401496703205376", "1"));
    test(
        -(2f32.powf(100.)),
        ("-1267650600228229401496703205376", "1"),
    );
    test(
        1.0 / 2f32.powf(100.),
        ("1", "1267650600228229401496703205376"),
    );
    test(684729.48391f32, ("1369459", "2"));
    test(-8573.5918555f32, ("-4389679", "512"));

    // f64
    test(
        core::f64::consts::PI,
        ("884279719003555", "281474976710656"),
    );
    test(2f64.powf(100.), ("1267650600228229401496703205376", "1"));
    test(
        -(2f64.powf(100.)),
        ("-1267650600228229401496703205376", "1"),
    );
    test(684729.48391f64, ("367611342500051", "536870912"));
    test(-8573.5918555f64, ("-4713381968463931", "549755813888"));
    test(
        1.0 / 2f64.powf(100.),
        ("1", "1267650600228229401496703205376"),
    );
}

#[cfg(feature = "num-bigint")]
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
    assert_eq!(_NEG1_2.signum(), -<Ratio<i64>>::one());
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

// TODO(MSRV 1.71): use `BuildHasher::hash_one`
#[cfg(feature = "std")]
fn hash_one<T: std::hash::Hash>(x: &T) -> u64 {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};

    let mut hasher = <RandomState as BuildHasher>::Hasher::new();
    x.hash(&mut hasher);
    hasher.finish()
}

#[test]
#[cfg(feature = "std")]
fn test_hash() {
    assert!(hash_one(&_0) != hash_one(&_1));
    assert!(hash_one(&_0) != hash_one(&_3_2));

    // a == b -> hash(a) == hash(b)
    let a = Rational64::new_raw(4, 2);
    let b = Rational64::new_raw(6, 3);
    assert_eq!(a, b);
    assert_eq!(hash_one(&a), hash_one(&b));

    let a = Rational64::new_raw(123456789, 1000);
    let b = Rational64::new_raw(123456789 * 5, 5000);
    assert_eq!(a, b);
    assert_eq!(hash_one(&a), hash_one(&b));
}

#[test]
#[cfg(all(feature = "std", feature = "num-bigint"))]
fn test_hash_stack_overflow() {
    // When `hash` was recursive, this test caused a stack overflow.
    // (related to num-rational#140)
    let (a, b) = big_ratios();
    let _ = hash_one(&a);
    let _ = hash_one(&b);
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

#[test]
fn test_num_zero() {
    let zero = Rational64::zero();
    assert!(zero.is_zero());

    let mut r = Rational64::new(123, 456);
    assert!(!r.is_zero());
    assert_eq!(r + zero, r);

    r.set_zero();
    assert!(r.is_zero());
}

#[test]
fn test_num_one() {
    let one = Rational64::one();
    assert!(one.is_one());

    let mut r = Rational64::new(123, 456);
    assert!(!r.is_one());
    assert_eq!(r * one, r);

    r.set_one();
    assert!(r.is_one());
}

#[test]
fn test_const() {
    const N: Ratio<i32> = Ratio::new_raw(123, 456);
    const N_NUMER: &i32 = N.numer();
    const N_DENOM: &i32 = N.denom();

    assert_eq!(N_NUMER, &123);
    assert_eq!(N_DENOM, &456);

    let r = N.reduced();
    assert_eq!(r.numer(), &(123 / 3));
    assert_eq!(r.denom(), &(456 / 3));
}

#[test]
fn test_ratio_to_i64() {
    assert_eq!(5, Rational64::new(70, 14).to_u64().unwrap());
    assert_eq!(-3, Rational64::new(-31, 8).to_i64().unwrap());
    assert_eq!(None, Rational64::new(-31, 8).to_u64());
}

#[test]
#[cfg(feature = "num-bigint")]
fn test_ratio_to_i128() {
    assert_eq!(
        1i128 << 70,
        Ratio::<i128>::new(1i128 << 77, 1i128 << 7)
            .to_i128()
            .unwrap()
    );
}

#[test]
#[cfg(feature = "num-bigint")]
fn test_big_ratio_to_f64() {
    assert_eq!(
        BigRational::new(
            "1234567890987654321234567890987654321234567890"
                .parse()
                .unwrap(),
            "3".parse().unwrap()
        )
        .to_f64(),
        Some(411522630329218100000000000000000000000000000f64)
    );
    assert_eq!(Ratio::from_float(5e-324).unwrap().to_f64(), Some(5e-324));
    assert_eq!(
        // subnormal
        BigRational::new(BigInt::one(), BigInt::one() << 1050).to_f64(),
        Some(2.0f64.powi(-50).powi(21))
    );
    assert_eq!(
        // definite underflow
        BigRational::new(BigInt::one(), BigInt::one() << 1100).to_f64(),
        Some(0.0)
    );
    assert_eq!(
        BigRational::from(BigInt::one() << 1050).to_f64(),
        Some(core::f64::INFINITY)
    );
    assert_eq!(
        BigRational::from((-BigInt::one()) << 1050).to_f64(),
        Some(core::f64::NEG_INFINITY)
    );
    assert_eq!(
        BigRational::new(
            "1234567890987654321234567890".parse().unwrap(),
            "987654321234567890987654321".parse().unwrap()
        )
        .to_f64(),
        Some(1.2499999893125f64)
    );
    assert_eq!(
        BigRational::new_raw(BigInt::one(), BigInt::zero()).to_f64(),
        Some(core::f64::INFINITY)
    );
    assert_eq!(
        BigRational::new_raw(-BigInt::one(), BigInt::zero()).to_f64(),
        Some(core::f64::NEG_INFINITY)
    );
    assert_eq!(
        BigRational::new_raw(BigInt::zero(), BigInt::zero()).to_f64(),
        None
    );
}

#[test]
fn test_ratio_to_f64() {
    assert_eq!(Ratio::<u8>::new(1, 2).to_f64(), Some(0.5f64));
    assert_eq!(Rational64::new(1, 2).to_f64(), Some(0.5f64));
    assert_eq!(Rational64::new(1, -2).to_f64(), Some(-0.5f64));
    assert_eq!(Rational64::new(0, 2).to_f64(), Some(0.0f64));
    assert_eq!(Rational64::new(0, -2).to_f64(), Some(-0.0f64));
    assert_eq!(Rational64::new((1 << 57) + 1, 1 << 54).to_f64(), Some(8f64));
    assert_eq!(
        Rational64::new((1 << 52) + 1, 1 << 52).to_f64(),
        Some(1.0000000000000002f64),
    );
    assert_eq!(
        Rational64::new((1 << 60) + (1 << 8), 1 << 60).to_f64(),
        Some(1.0000000000000002f64),
    );
    assert_eq!(
        Ratio::<i32>::new_raw(1, 0).to_f64(),
        Some(core::f64::INFINITY)
    );
    assert_eq!(
        Ratio::<i32>::new_raw(-1, 0).to_f64(),
        Some(core::f64::NEG_INFINITY)
    );
    assert_eq!(Ratio::<i32>::new_raw(0, 0).to_f64(), None);
}

#[test]
fn test_ldexp() {
    use core::f64::{INFINITY, MAX_EXP, MIN_EXP, NAN, NEG_INFINITY};
    assert_eq!(ldexp(1.0, 0), 1.0);
    assert_eq!(ldexp(1.0, 1), 2.0);
    assert_eq!(ldexp(0.0, 1), 0.0);
    assert_eq!(ldexp(-0.0, 1), -0.0);

    // Cases where ldexp is equivalent to multiplying by 2^exp because there's no over- or
    // underflow.
    assert_eq!(ldexp(3.5, 5), 3.5 * 2f64.powi(5));
    assert_eq!(ldexp(1.0, MAX_EXP - 1), 2f64.powi(MAX_EXP - 1));
    assert_eq!(ldexp(2.77, MIN_EXP + 3), 2.77 * 2f64.powi(MIN_EXP + 3));

    // Case where initial value is subnormal
    assert_eq!(ldexp(5e-324, 4), 5e-324 * 2f64.powi(4));
    assert_eq!(ldexp(5e-324, 200), 5e-324 * 2f64.powi(200));

    // Near underflow (2^exp is too small to represent, but not x*2^exp)
    assert_eq!(ldexp(4.0, MIN_EXP - 3), 2f64.powi(MIN_EXP - 1));

    // Near overflow
    assert_eq!(ldexp(0.125, MAX_EXP + 3), 2f64.powi(MAX_EXP));

    // Overflow and underflow cases
    assert_eq!(ldexp(1.0, MIN_EXP - 54), 0.0);
    assert_eq!(ldexp(-1.0, MIN_EXP - 54), -0.0);
    assert_eq!(ldexp(1.0, MAX_EXP), INFINITY);
    assert_eq!(ldexp(-1.0, MAX_EXP), NEG_INFINITY);

    // Special values
    assert_eq!(ldexp(INFINITY, 1), INFINITY);
    assert_eq!(ldexp(NEG_INFINITY, 1), NEG_INFINITY);
    assert!(ldexp(NAN, 1).is_nan());
}

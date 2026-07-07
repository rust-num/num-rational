use super::Ratio;

use core::fmt;
use core::fmt::{Binary, Display, Formatter, LowerExp, LowerHex, Octal, UpperExp, UpperHex};
use core::str::FromStr;
#[cfg(feature = "std")]
use std::error::Error;

use num_integer::Integer;
use num_traits::{Num, Zero};

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

// String conversions
macro_rules! impl_formatting {
    ($fmt_trait:ident, $prefix:expr, $fmt_str:expr, $fmt_alt:expr) => {
        impl<T: $fmt_trait + Clone + Integer> $fmt_trait for Ratio<T> {
            #[cfg(feature = "std")]
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                let pre_pad = if self.denom.is_one() {
                    format!($fmt_str, self.numer)
                } else {
                    if f.alternate() {
                        format!(concat!($fmt_str, "/", $fmt_alt), self.numer, self.denom)
                    } else {
                        format!(concat!($fmt_str, "/", $fmt_str), self.numer, self.denom)
                    }
                };
                if let Some(pre_pad) = pre_pad.strip_prefix("-") {
                    f.pad_integral(false, $prefix, pre_pad)
                } else {
                    f.pad_integral(true, $prefix, &pre_pad)
                }
            }
            #[cfg(not(feature = "std"))]
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                let plus = if f.sign_plus() && self.numer >= T::zero() {
                    "+"
                } else {
                    ""
                };
                if self.denom.is_one() {
                    if f.alternate() {
                        write!(f, concat!("{}", $fmt_alt), plus, self.numer)
                    } else {
                        write!(f, concat!("{}", $fmt_str), plus, self.numer)
                    }
                } else {
                    if f.alternate() {
                        write!(
                            f,
                            concat!("{}", $fmt_alt, "/", $fmt_alt),
                            plus, self.numer, self.denom
                        )
                    } else {
                        write!(
                            f,
                            concat!("{}", $fmt_str, "/", $fmt_str),
                            plus, self.numer, self.denom
                        )
                    }
                }
            }
        }
    };
}

impl_formatting!(Display, "", "{}", "{:#}");
impl_formatting!(Octal, "0o", "{:o}", "{:#o}");
impl_formatting!(Binary, "0b", "{:b}", "{:#b}");
impl_formatting!(LowerHex, "0x", "{:x}", "{:#x}");
impl_formatting!(UpperHex, "0x", "{:X}", "{:#X}");
impl_formatting!(LowerExp, "", "{:e}", "{:#e}");
impl_formatting!(UpperExp, "", "{:E}", "{:#E}");

impl<T: FromStr + Clone + Integer> FromStr for Ratio<T> {
    type Err = ParseRatioError;

    /// Parses `numer/denom` or just `numer`.
    fn from_str(s: &str) -> Result<Ratio<T>, ParseRatioError> {
        let mut split = s.splitn(2, '/');

        let n = split.next().ok_or(ParseRatioError {
            kind: RatioErrorKind::ParseError,
        })?;
        let num = FromStr::from_str(n).map_err(|_| ParseRatioError {
            kind: RatioErrorKind::ParseError,
        })?;

        let d = split.next().unwrap_or("1");
        let den = FromStr::from_str(d).map_err(|_| ParseRatioError {
            kind: RatioErrorKind::ParseError,
        })?;

        if Zero::is_zero(&den) {
            Err(ParseRatioError {
                kind: RatioErrorKind::ZeroDenominator,
            })
        } else {
            Ok(Ratio::new(num, den))
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.description().fmt(f)
    }
}

#[cfg(feature = "std")]
impl Error for ParseRatioError {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        self.kind.description()
    }
}

impl RatioErrorKind {
    fn description(&self) -> &'static str {
        match *self {
            RatioErrorKind::ParseError => "failed to parse integer",
            RatioErrorKind::ZeroDenominator => "zero value denominator",
        }
    }
}

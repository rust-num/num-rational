use crate::{FromPrimitive, Ratio};

#[cfg(has_try_from)]
use crate::{approximate_float, approximate_float_unsigned};
#[cfg(has_try_from)]
use core::convert::TryFrom;

macro_rules! impl_try_from {
    ( $($name:ty),* => $into:ty ; $approx:ident) => {
        $(
        #[cfg(has_try_from)]
        impl TryFrom<$name> for Ratio<$into> {
                type Error = ();
                paste::paste! {
                    fn try_from(n: $name) -> Result<Self, ()> {
                        <$into as FromPrimitive>::[<from_ $name>](n)
                            .map(Ratio::from_integer)
                            .ok_or(())
                    }
                }
            }
        )*

        #[cfg(has_try_from)]
        impl TryFrom<f32> for Ratio<$into> {
            type Error = ();
            fn try_from(n: f32) -> Result<Self, ()> {
                $approx(n, 10e-20, 30).ok_or(())
            }
        }

        #[cfg(has_try_from)]
        impl TryFrom<f64> for Ratio<$into> {
            type Error = ();
            fn try_from(n: f64) -> Result<Self, ()> {
                $approx(n, 10e-20, 30).ok_or(())
            }
        }

    };
}

impl_try_from!(i8, u16, i16, u32, i32, u64, i64, u128, i128 => u8 ; approximate_float_unsigned);
impl_try_from!(u8, u16, i16, u32, i32, u64, i64, u128, i128 => i8 ; approximate_float);

impl_try_from!(i16, u32, i32, u64, i64, u128, i128 => u16 ; approximate_float_unsigned);
impl_try_from!(u16, u32, i32, u64, i64, u128, i128 => i16 ; approximate_float);

impl_try_from!(i32, u64, i64, u128, i128 => u32 ; approximate_float_unsigned);
impl_try_from!(u32, u64, i64, u128, i128 => i32 ; approximate_float);

impl_try_from!(i64, u128, i128 => u64 ; approximate_float_unsigned);
impl_try_from!(u64, u128, i128 => i64 ; approximate_float);

impl_try_from!(i128 => u128 ; approximate_float_unsigned);
impl_try_from!(u128 => i128 ; approximate_float);

macro_rules! impl_from {
    ( $($name:ty),* => $into:ty) => {
        $(
        impl From<$name> for Ratio<$into> {
                paste::paste! {
                    fn from(n: $name) -> Self {
                        <$into as FromPrimitive>::[< from_ $name>](n)
                            .map(Ratio::from_integer)
                            .unwrap()
                    }
                }
            }
        )*
    };
}

impl_from!(u8, u16, u32, u64 => u128);
impl_from!(u8, i8, u16, i16, u32, i32, u64, i64 => i128);

impl_from!(u8, u16, u32 => u64);
impl_from!(u8, i8, u16, i16, u32, i32 => i64);

impl_from!(u8, u16 => u32);
impl_from!(u8, i8, u16, i16 => i32);

impl_from!(u8 => u16);
impl_from!(u8, i8 => i16);

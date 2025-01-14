use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::str::FromStr;
use std::ops::*;
use thiserror::Error;
pub mod value;
pub mod ystring;
pub use value::*;
pub use ystring::*;

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Number(pub i64);

impl Number {
    const SCALE: i64 = 1000;
    const SCALE_F32: f32 = 1000.0;
    const SCALE_F64: f64 = 1000.0;
    pub const MAX: Number = Number(i64::MAX);
    pub const MIN: Number = Number(i64::MIN);
    pub const ONE: Number = Number(1 * Self::SCALE);
    pub const ZERO: Number = Number(0);
    const MAX_VAL_F64: f64 = Self::MAX.0 as f64 / Self::SCALE_F64;
    const MIN_VAL_F64: f64 = Self::MIN.0 as f64 / Self::SCALE_F64;
    const MAX_VAL_F32: f32 = Self::MAX_VAL_F64 as f32;
    const MIN_VAL_F32: f32 = Self::MIN_VAL_F64 as f32;

    pub fn new(mut n: f64) -> Self {
        if n.is_finite() && {
            n *= Self::SCALE_F64;
            n <= Self::MAX_VAL_F64 && n >= Self::MIN_VAL_F64
        } {
            Number(n as i64)
        } else {
            Number::MIN
        }
    }

    pub fn new_f32(mut n: f32) -> Self {
        if n.is_finite() && {
            n *= Self::SCALE_F32;
            n <= Self::MAX_VAL_F32 && n >= Self::MIN_VAL_F32
        } {
            Number(n as i64)
        } else {
            Number::MIN
        }
    }

    pub fn as_f64(self) -> f64 {
        self.0 as f64 / Self::SCALE_F64
    }

    pub fn as_f32(self) -> f32 {
        self.0 as f32 / Self::SCALE_F32
    }

    pub fn as_bool(self) -> bool {
        self != Self::ZERO
    }

    pub fn stringify_with_buffer(&self, buffer: &mut YString) {
        let data = buffer.data.as_mut();
        let int = self.0 / Self::SCALE;
        let mut dec = (self.0 % Self::SCALE).unsigned_abs() as u32;
        let neg = int.is_negative();

        let mut int = int.unsigned_abs();
        let mut rem;
        loop {
            rem = (int % 10) as u32;
            int /= 10;
            unsafe {
                let c = std::char::from_digit(rem, 10)
                    .unwrap_or_else(|| std::hint::unreachable_unchecked());
                data.push_unchecked(c as u8);
            }
            if int == 0 {
                break;
            }
        }

        if neg {
            unsafe { data.push_unchecked(b'-'); }
        }

        data.reverse();
        if dec == 0 {
            return;
        }

        unsafe { data.push_unchecked(b'.'); }
        let old_len = data.len();

        for _ in 0..3 {
            rem = dec % 10;
            dec /= 10;
            unsafe {
                let c = std::char::from_digit(rem, 10)
                    .unwrap_or_else(|| std::hint::unreachable_unchecked());
                data.push_unchecked(c as u8);
            }
            if dec == 0 {
                break;
            }
        }

        let len = data.len();
        data[old_len..len].reverse();
    }

    pub fn stringify(&self) -> YString {
        let mut s = YString::default();
        self.stringify_with_buffer(&mut s);
        s
    }

    pub fn div_assign(&mut self, other: Self) -> ValueResult<()> {
        *self = (*self / other)?;
        Ok(())
    }

    fn round_to_new(mut v: f64) -> Self {
        v += 5e-5_f64.copysign(v);
        Self::new(v)
    }

    pub fn pow(self, other: Self) -> Self {
        let v = self.as_f64().powf(other.as_f64());
        Self::round_to_new(v)
    }

    pub fn pow_assign(&mut self, other: Self) {
        *self = self.pow(other);
    }

    pub fn rem_assign(&mut self, other: Self) -> ValueResult<()> {
        *self = (*self % other)?;
        Ok(())
    }

    pub fn pre_inc(&mut self) {
        *self += Number::ONE;
    }

    pub fn pre_dec(&mut self) {
        *self -= Number::ONE;
    }

    pub fn abs(self) -> Self {
        Number(self.0.overflowing_abs().0)
    }

    pub fn sqrt(self) -> Self {
        if self.0.is_negative() || self.0 >= 9223372036854775000 {
            Number::MIN
        } else {
            let v = self.as_f64().sqrt();
            Self::round_to_new(v)
        }
    }

    pub fn sin(self) -> Self {
        Self::new((self.as_f32().to_radians() as f64).sin())
    }

    pub fn cos(self) -> Self {
        Self::new((self.as_f32().to_radians() as f64).cos())
    }

    pub fn tan(self) -> Self {
        Self::new((self.as_f32().to_radians() as f64).tan())
    }

    pub fn asin(self) -> Self {
        Number::new_f32(self.as_f32().asin().to_degrees())
    }

    pub fn acos(self) -> Self {
        Number::new_f32(self.as_f32().acos().to_degrees())
    }

    pub fn atan(self) -> Self {
        let mut atan = self.as_f32().atan().to_degrees();
        if atan == -90.0 {
            atan = 90.0;
        }
        Number::new_f32(atan)
    }

    pub fn fact(self) -> Self {
        if self.0.is_negative() {
            Number::MIN
        } else {
            let mut v = self.0 / Number::SCALE;
            let mut i = 0;
            let mut result = 1_i64;
            while v.is_positive() {
                i += 1;
                v -= 1;
                result = result.wrapping_mul(i);
            }
            Number(result.wrapping_mul(Number::SCALE))
        }
    }
}

impl From<bool> for Number {
    fn from(b: bool) -> Self {
        if b {
            Number::ONE
        } else {
            Number::ZERO
        }
    }
}

impl From<i64> for Number {
    fn from(n: i64) -> Self {
        Number(n * Self::SCALE)
    }
}

impl From<f64> for Number {
    fn from(n: f64) -> Self {
        Number::new(n)
    }
}

impl From<Number> for bool {
    fn from(n: Number) -> Self {
        n != Number::ZERO
    }
}

impl From<Number> for f64 {
    fn from(n: Number) -> Self {
        n.as_f64()
    }
}

impl Debug for Number {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        Display::fmt(&self, f)
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.stringify())
    }
}

#[derive(Debug, Error, Clone, Copy)]
pub enum NumberParseErr {
    #[error("Number can't fit in i64")]
    Overflow,
    #[error("Found unknown char '{0:}'")]
    UnknownChar(char),
}

impl FromStr for Number {
    type Err = NumberParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let neg = s.as_bytes()[0] == b'-';
        let (mut big, small) = if let Some((big, small)) = s.split_once('.') {
            (big.chars(), Some(small.chars()))
        } else {
            (s.chars(), None)
        };
        if neg {
            big.next();
        }

        let mut val: i64 = 0;
        let mut exp: i64 = Number::SCALE / 10;

        for c in big.rev() {
            if c.is_ascii_digit() {
                exp = exp
                    .checked_mul(10)
                    .ok_or(NumberParseErr::Overflow)?;
                let d = exp
                    .checked_mul(c as i64 - '0' as i64)
                    .ok_or(NumberParseErr::Overflow)?;
                val = val.checked_add(d).ok_or(NumberParseErr::Overflow)?;
            } else {
                return Err(NumberParseErr::UnknownChar(c));
            }
        }

        if neg {
            val = val.checked_neg().ok_or(NumberParseErr::Overflow)?;
        }

        if let Some(small) = small {
            exp = Number::SCALE;
            if neg {
                exp = -exp;
            }
            for c in small.take(3) {
                if c.is_ascii_digit() {
                    exp = exp
                        .checked_div(10)
                        .ok_or(NumberParseErr::Overflow)?;
                    let d = exp
                        .checked_mul(c as i64 - '0' as i64)
                        .ok_or(NumberParseErr::Overflow)?;
                    val = val.checked_add(d).ok_or(NumberParseErr::Overflow)?;
                } else {
                    return Err(NumberParseErr::UnknownChar(c));
                }
            }
        }

        Ok(Number(val))
    }
}

impl Add for Number {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Number(self.0.wrapping_add(rhs.0))
    }
}

impl AddAssign for Number {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Number {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Number(self.0.wrapping_sub(rhs.0))
    }
}

impl SubAssign for Number {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Neg for Number {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Number(self.0.overflowing_neg().0)
    }
}

impl Mul for Number {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Number(self.0.wrapping_mul(rhs.0) / Self::SCALE)
    }
}

impl MulAssign for Number {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Not for Number {
    type Output = Self;

    fn not(self) -> Self::Output {
        if self == Number::ZERO {
            Number::ONE
        } else {
            Number::ZERO
        }
    }
}

impl Div for Number {
    type Output = ValueResult<Number>;

    fn div(self, rhs: Self) -> Self::Output {
        if rhs.0 == 0 {
            Err(RuntimeErr::DivZero)
        } else {
            Ok(Number(self.0.wrapping_mul(Number::SCALE).wrapping_div(rhs.0)))
        }
    }
}

impl Rem for Number {
    type Output = ValueResult<Number>;

    fn rem(self, rhs: Self) -> Self::Output {
        if rhs.0 == 0 {
            Err(RuntimeErr::ModZero)
        } else {
            Ok(Number(self.0.wrapping_rem(rhs.0)))
        }
    }
}

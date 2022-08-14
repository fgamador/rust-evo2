use std::convert::From;
use std::fmt;
use std::ops::AddAssign;
use std::ops::Mul;
use std::ops::Sub;
use std::ops::SubAssign;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct F32Positive {
    value: f32,
}

impl F32Positive {
    pub fn checked(value: f32) -> Self {
        assert!(value >= 0.0);
        Self { value }
    }

    pub const fn unchecked(value: f32) -> Self {
        Self { value }
    }

    pub fn clipped(value: f32) -> Self {
        Self { value: value.max(0.0) }
    }

    pub const fn value(self) -> f32 {
        self.value
    }

    pub fn min(self, other: F32Positive) -> F32Positive {
        Self::unchecked(self.value.min(other.value))
    }
}

impl fmt::Display for F32Positive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl From<F32Positive> for f32 {
    fn from(num: F32Positive) -> Self {
        num.value()
    }
}

impl From<f32> for F32Positive {
    fn from(num: f32) -> Self {
        Self::checked(num)
    }
}

impl AddAssign for F32Positive {
    fn add_assign(&mut self, other: Self) {
        *self = Self::unchecked(self.value() + other.value());
    }
}

impl Mul for F32Positive {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        Self::unchecked(self.value() * other.value())
    }
}

impl Sub for F32Positive {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self::clipped(self.value() - other.value())
    }
}

impl SubAssign for F32Positive {
    fn sub_assign(&mut self, other: Self) {
        *self = Self::clipped(self.value() - other.value());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct F32ZeroToOne {
    value: f32,
}

impl F32ZeroToOne {
    pub fn checked(value: f32) -> Self {
        assert!((0.0..=1.0).contains(&value));
        Self { value }
    }

    pub const fn unchecked(value: f32) -> Self {
        Self { value }
    }

    pub fn clipped(value: f32) -> Self {
        Self { value: value.max(0.0).min(1.0) }
    }

    pub const fn value(self) -> f32 {
        self.value
    }
}

impl fmt::Display for F32ZeroToOne {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl From<F32ZeroToOne> for f32 {
    fn from(num: F32ZeroToOne) -> Self {
        num.value()
    }
}

impl From<f32> for F32ZeroToOne {
    fn from(num: f32) -> Self {
        Self::checked(num)
    }
}

impl AddAssign for F32ZeroToOne {
    fn add_assign(&mut self, other: Self) {
        *self = Self::clipped(self.value() + other.value());
    }
}

impl SubAssign for F32ZeroToOne {
    fn sub_assign(&mut self, other: Self) {
        *self = Self::clipped(self.value() - other.value());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct F32ZeroToOnePerF32Positive {
    value: f32,
}

impl F32ZeroToOnePerF32Positive {
    pub fn checked(value: f32) -> Self {
        assert!((0.0..=1.0).contains(&value));
        Self { value }
    }

    pub const fn unchecked(value: f32) -> Self {
        Self { value }
    }

    pub fn clipped(value: f32) -> Self {
        Self { value: value.max(0.0).min(1.0) }
    }

    pub const fn value(self) -> f32 {
        self.value
    }
}

impl fmt::Display for F32ZeroToOnePerF32Positive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl From<F32ZeroToOnePerF32Positive> for f32 {
    fn from(num: F32ZeroToOnePerF32Positive) -> Self {
        num.value()
    }
}

impl From<f32> for F32ZeroToOnePerF32Positive {
    fn from(num: f32) -> Self {
        Self::checked(num)
    }
}

impl Mul<F32Positive> for F32ZeroToOnePerF32Positive {
    type Output = F32ZeroToOne;

    fn mul(self, other: F32Positive) -> Self::Output {
        F32ZeroToOne::clipped(self.value() * other.value())
    }
}

impl Mul<F32ZeroToOnePerF32Positive> for F32Positive {
    type Output = F32ZeroToOne;

    fn mul(self, other: F32ZeroToOnePerF32Positive) -> Self::Output {
        other.mul(self)
    }
}

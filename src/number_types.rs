use std::convert::From;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct F32Positive {
    value: f32,
}

impl F32Positive {
    // TODO convert to macro with checking?
    pub const fn unchecked(num: f32) -> Self {
        // assert!(num >= 0.0);
        F32Positive { value: num }
    }

    pub const fn value(&self) -> f32 {
        self.value
    }
}

impl From<F32Positive> for f32 {
    fn from(num: F32Positive) -> Self {
        num.value
    }
}

impl From<f32> for F32Positive {
    fn from(num: f32) -> Self {
        assert!(num >= 0.0);
        F32Positive { value: num }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct F32ZeroToOne {
    value: f32,
}

impl From<F32ZeroToOne> for f32 {
    fn from(num: F32ZeroToOne) -> Self {
        num.value
    }
}

impl From<f32> for F32ZeroToOne {
    fn from(num: f32) -> Self {
        assert!((0.0..=1.0).contains(&num));
        F32ZeroToOne { value: num }
    }
}

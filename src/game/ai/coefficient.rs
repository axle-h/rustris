use std::ops::{Add, Mul, RangeInclusive};
use std::fmt::{Debug, Display, Formatter};
use rand::distr::{Distribution, StandardUniform};
use rand::Rng;

const COEFFICIENT_STEP: f64 = 0.0001;
const RANDOM_COEFFICIENT_MIN: Coefficient = Coefficient::new((-100.0 / COEFFICIENT_STEP) as i64);
const RANDOM_COEFFICIENT_MAX: Coefficient = Coefficient::new((100.0 / COEFFICIENT_STEP) as i64);
const RANDOM_RAW_COEFFICIENT_RANGE: RangeInclusive<i64> = RANDOM_COEFFICIENT_MIN.0 ..= RANDOM_COEFFICIENT_MAX.0;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Coefficient(pub i64);

impl Coefficient {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1);

    pub const fn new(magnitude: i64) -> Self {
        Self(magnitude)
    }

    pub fn from_f64(value: f64) -> Self {
        Self((value / COEFFICIENT_STEP).round() as i64)
    }
}

impl Display for Coefficient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value: f64 = (*self).into();
        write!(f, "{:.4}", value)
    }
}

impl Debug for Coefficient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value: f64 = (*self).into();
        write!(f, "{}", value)
    }
}

impl Mul<f64> for Coefficient {
    type Output = f64;

    fn mul(self, rhs: f64) -> Self::Output {
        let value: f64 = self.into();
        value * rhs
    }
}

impl Mul<Coefficient> for Coefficient {
    type Output = Coefficient;

    fn mul(self, rhs: Coefficient) -> Self::Output {
        let lhs: f64 = self.into();
        let rhs: f64 = rhs.into();
        Self::from_f64(lhs * rhs)
    }
}

impl Mul<Coefficient> for f64 {
    type Output = f64;

    fn mul(self, rhs: Coefficient) -> Self::Output {
        let value: f64 = rhs.into();
        self * value
    }
}

impl Add<Coefficient> for Coefficient {
    type Output = Coefficient;
    fn add(self, rhs: Coefficient) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Into<f64> for Coefficient {
    fn into(self) -> f64 {
        self.0 as f64 * COEFFICIENT_STEP
    }
}

impl From<f64> for Coefficient {
    fn from(value: f64) -> Self {
        Self::from_f64(value)
    }
}

impl From<i64> for Coefficient {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl Distribution<Coefficient> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Coefficient {
        Coefficient(rng.random_range(RANDOM_RAW_COEFFICIENT_RANGE))
    }
}
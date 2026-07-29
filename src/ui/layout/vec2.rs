use std::ops::{Add, AddAssign, Div, Mul, Sub};

/// The unit a layout is measured in, e.g. terminal cells or pixels.
pub trait Scalar:
    Copy
    + PartialOrd
    + std::fmt::Debug
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
{
    const ZERO: Self;
    const ONE: Self;
}

macro_rules! impl_scalar {
    ($($ty:ty => $zero:expr, $one:expr);* $(;)?) => {
        $(
            impl Scalar for $ty {
                const ZERO: Self = $zero;
                const ONE: Self = $one;
            }
        )*
    };
}

impl_scalar! {
    u16 => 0, 1;
    u32 => 0, 1;
    u64 => 0, 1;
    usize => 0, 1;
    f32 => 0.0, 1.0;
    f64 => 0.0, 1.0;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Vec2<U>(pub(crate) U, pub(crate) U);

/// Comparisons go through `PartialOrd`, so a NaN operand yields `rhs`.
fn max<U: Scalar>(lhs: U, rhs: U) -> U {
    if lhs > rhs { lhs } else { rhs }
}

fn min<U: Scalar>(lhs: U, rhs: U) -> U {
    if lhs < rhs { lhs } else { rhs }
}

impl<U: std::fmt::Debug> std::fmt::Debug for Vec2<U> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Vec2({:?}, {:?})", self.0, self.1))
    }
}

impl<U> From<[U; 2]> for Vec2<U> {
    fn from([x, y]: [U; 2]) -> Self {
        Self(x, y)
    }
}

impl<U> From<Vec2<U>> for [U; 2] {
    fn from(Vec2(x, y): Vec2<U>) -> Self {
        [x, y]
    }
}

impl<U: Scalar> Add for Vec2<U> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl<U: Scalar> Sub for Vec2<U> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl<U: Scalar> Mul for Vec2<U> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0, self.1 * rhs.1)
    }
}

impl<U: Scalar> Div for Vec2<U> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0, self.1 / rhs.1)
    }
}

impl<U: Scalar> AddAssign for Vec2<U> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<U: Scalar> Vec2<U> {
    pub(crate) fn zero() -> Self {
        Self(U::ZERO, U::ZERO)
    }

    pub(crate) fn one() -> Self {
        Self(U::ONE, U::ONE)
    }

    pub(crate) fn splat(value: U) -> Self {
        Self(value, value)
    }

    pub(crate) fn max(self, rhs: Self) -> Self {
        Self(max(self.0, rhs.0), max(self.1, rhs.1))
    }

    pub(crate) fn min(self, rhs: Self) -> Self {
        Self(min(self.0, rhs.0), min(self.1, rhs.1))
    }

    pub(crate) fn fits(&self, other: Self) -> bool {
        self.0 <= other.0 && self.1 <= other.1
    }

    pub(crate) fn flip(&self) -> Self {
        Self(self.1, self.0)
    }

    pub(crate) fn saturating_sub(&self, other: Vec2<U>) -> Vec2<U> {
        *self - self.min(other)
    }
}

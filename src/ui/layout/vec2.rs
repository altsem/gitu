use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Vec2(pub(crate) u16, pub(crate) u16);

impl std::fmt::Debug for Vec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Vec2({}, {})", self.0, self.1))
    }
}

impl From<[u16; 2]> for Vec2 {
    fn from([x, y]: [u16; 2]) -> Self {
        Self(x, y)
    }
}

impl From<Vec2> for [u16; 2] {
    fn from(Vec2(x, y): Vec2) -> Self {
        [x, y]
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl Mul for Vec2 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0, self.1 * rhs.1)
    }
}

impl Div for Vec2 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0, self.1 / rhs.1)
    }
}

impl Rem for Vec2 {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0, self.1 % rhs.1)
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
        self.1 -= rhs.1;
    }
}

impl MulAssign for Vec2 {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
        self.1 *= rhs.1;
    }
}

impl DivAssign for Vec2 {
    fn div_assign(&mut self, rhs: Self) {
        self.0 /= rhs.0;
        self.1 /= rhs.1;
    }
}

impl RemAssign for Vec2 {
    fn rem_assign(&mut self, rhs: Self) {
        self.0 %= rhs.0;
        self.1 %= rhs.1;
    }
}

impl Vec2 {
    pub(crate) fn max(self, rhs: Self) -> Self {
        Self(self.0.max(rhs.0), self.1.max(rhs.1))
    }

    pub(crate) fn min(self, rhs: Self) -> Self {
        Self(self.0.min(rhs.0), self.1.min(rhs.1))
    }

    pub(crate) fn fits(&self, other: Self) -> bool {
        self.0 <= other.0 && self.1 <= other.1
    }

    pub(crate) fn flip(&self) -> Self {
        Self(self.1, self.0)
    }

    pub(crate) fn saturating_sub(&self, other: Vec2) -> Vec2 {
        *self - self.min(other)
    }
}

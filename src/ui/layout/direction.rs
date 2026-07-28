use super::vec2::{Scalar, Vec2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Horizontal,
    Vertical,
}

impl Direction {
    /// The unit vector along this direction, used to mask a `Vec2` down to one
    /// axis.
    pub(crate) fn axis<U: Scalar>(&self) -> Vec2<U> {
        match self {
            Direction::Horizontal => Vec2(U::ONE, U::ZERO),
            Direction::Vertical => Vec2(U::ZERO, U::ONE),
        }
    }
}

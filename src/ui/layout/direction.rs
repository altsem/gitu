use super::vec2::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Horizontal,
    Vertical,
    Stacked,
}

impl Direction {
    pub(crate) fn axis(&self) -> Vec2 {
        match self {
            Direction::Horizontal => Vec2(1, 0),
            Direction::Vertical => Vec2(0, 1),
            Direction::Stacked => Vec2(0, 0),
        }
    }
}

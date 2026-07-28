use super::vec2::{Scalar, Vec2};

use super::direction::Direction;

/// Shorthand for [`Opts::new`], which is spelled out at every node.
pub fn opts<U: Scalar>() -> Opts<U> {
    Opts::new()
}

#[derive(Debug, Copy, Clone)]
pub struct Opts<U> {
    /// Layout direction for children of this node.
    pub(crate) dir: Direction,
    pub(crate) fill: Vec2<U>,
    /// The space between each direct child of this node.
    pub(crate) gap: U,
    pub(crate) pad: U,
}

impl<U: Scalar> Default for Opts<U> {
    fn default() -> Self {
        Self::new()
    }
}

impl<U: Scalar> Opts<U> {
    pub fn new() -> Self {
        Self {
            dir: Direction::Horizontal,
            fill: Vec2::zero(),
            gap: U::ZERO,
            pad: U::ZERO,
        }
    }

    pub fn fill_x(self) -> Self {
        Self {
            fill: Vec2(U::ONE, U::ZERO),
            ..self
        }
    }

    #[allow(dead_code)]
    pub fn fill_y(self) -> Self {
        Self {
            fill: Vec2(U::ZERO, U::ONE),
            ..self
        }
    }

    pub fn fill_xy(self) -> Self {
        Self {
            fill: Vec2::one(),
            ..self
        }
    }

    pub fn gap(self, gap: U) -> Self {
        Self { gap, ..self }
    }

    pub fn pad(self, pad: U) -> Self {
        Self { pad, ..self }
    }

    pub(crate) fn is_main_fill(&self, parent: &Self) -> bool {
        self.fill * parent.dir.axis() != Vec2::zero()
    }

    pub(crate) fn is_cross_fill(&self, parent: &Self) -> bool {
        self.fill * parent.dir.axis().flip() != Vec2::zero()
    }
}

#[derive(Debug)]
pub(crate) struct Node<T, U> {
    pub(crate) data: Option<T>,
    /// layout options
    pub(crate) opts: Opts<U>,
    /// space actually occupied by this node, updated as nodes are added
    pub(crate) size: Vec2<U>,
    /// Intrinsic (content) size of this subtree, ignoring fill shrinking.
    /// A fill node may shrink below this, but an ancestor that does *not* fill
    /// a given axis still needs the content extent to size itself on that axis.
    pub(crate) content: Vec2<U>,
    /// Offset from parent's top-left corner, updated as nodes are added.
    /// This will remain `None` if there's no valid position for the element.
    pub(crate) pos: Option<Vec2<U>>,
}

#[cfg(test)]
mod tests {

    #[test]
    fn is_axis_fill() {
        let horizontal = super::Opts {
            dir: super::Direction::Horizontal,
            ..Default::default()
        };
        let vertical = super::Opts {
            dir: super::Direction::Vertical,
            ..Default::default()
        };

        assert!(super::opts::<u16>().fill_x().is_main_fill(&horizontal));
        assert!(super::opts::<u16>().fill_y().is_main_fill(&vertical));
        assert!(super::opts::<u16>().fill_xy().is_main_fill(&horizontal));
        assert!(super::opts::<u16>().fill_xy().is_main_fill(&vertical));

        assert!(!super::opts::<u16>().fill_x().is_main_fill(&vertical));
        assert!(!super::opts::<u16>().fill_y().is_main_fill(&horizontal));
        assert!(!super::opts::<u16>().fill_x().is_cross_fill(&horizontal));
        assert!(!super::opts::<u16>().fill_y().is_cross_fill(&vertical));

        assert!(!super::opts::<u16>().is_main_fill(&horizontal));
        assert!(!super::opts::<u16>().is_main_fill(&vertical));
        assert!(!super::opts::<u16>().is_cross_fill(&horizontal));
        assert!(!super::opts::<u16>().is_cross_fill(&vertical));
    }
}

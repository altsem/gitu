use super::vec2::{Scalar, Vec2};

use super::direction::Direction;

/// Shorthand for [`Opts::new`], which is spelled out at every node.
pub fn opts<U: Scalar>() -> Opts<U> {
    Opts::new()
}

#[derive(Debug, Copy, Clone)]
pub struct Opts<U> {
    pub(crate) fill: [bool; 2],
    pub(crate) gap: U,
    pub(crate) pad: U,
    pub(crate) wrap: Option<bool>,
}

impl<U: Scalar> Default for Opts<U> {
    fn default() -> Self {
        Self::new()
    }
}

impl<U: Scalar> Opts<U> {
    pub fn new() -> Self {
        Self {
            fill: [false, false],
            gap: U::ZERO,
            pad: U::ZERO,
            wrap: None,
        }
    }

    pub fn fill_x(self) -> Self {
        Self {
            fill: [true, self.fill[1]],
            ..self
        }
    }

    pub fn fill_y(self) -> Self {
        Self {
            fill: [self.fill[0], true],
            ..self
        }
    }

    pub fn fill_xy(self) -> Self {
        self.fill_x().fill_y()
    }

    pub fn gap(self, gap: U) -> Self {
        Self { gap, ..self }
    }

    pub fn pad(self, pad: U) -> Self {
        Self { pad, ..self }
    }

    /// Lets a child that doesn't fit start a new line, rather than being cut
    /// off at the edge.
    #[allow(dead_code)]
    pub fn wrap(self) -> Self {
        Self {
            wrap: Some(true),
            ..self
        }
    }

    #[allow(dead_code)]
    pub fn no_wrap(self) -> Self {
        Self {
            wrap: Some(false),
            ..self
        }
    }

    pub(crate) fn is_main_fill(&self, parent_dir: Direction) -> bool {
        self.fills(parent_dir)
    }

    pub(crate) fn is_cross_fill(&self, parent_dir: Direction) -> bool {
        self.fills(parent_dir.flip())
    }

    fn fills(&self, dir: Direction) -> bool {
        match dir {
            Direction::Horizontal => self.fill[0],
            Direction::Vertical => self.fill[1],
        }
    }

    /// `ONE` on each axis this node fills and `ZERO` elsewhere, to mask a
    /// `Vec2` down to just the filling axes.
    pub(crate) fn fill_mask(&self) -> Vec2<U> {
        let bit = |fills| if fills { U::ONE } else { U::ZERO };

        Vec2(bit(self.fill[0]), bit(self.fill[1]))
    }
}

#[derive(Debug)]
pub(crate) enum NodeData<L, C> {
    Leaf(L),
    Container(Option<C>),
}

#[derive(Debug)]
pub(crate) struct Node<L, C, U> {
    pub(crate) data: NodeData<L, C>,
    pub(crate) dir: Direction,
    pub(crate) opts: Opts<U>,
    /// space actually occupied by this node, updated as nodes are added
    pub(crate) size: Vec2<U>,
    /// Extent this node's children came to in the fit pass, with any filling
    /// descendant shrunk away.
    pub(crate) fitted: Vec2<U>,
    /// Offset from parent's top-left corner, updated as nodes are added.
    /// This will remain `None` if there's no valid position for the element.
    pub(crate) pos: Option<Vec2<U>>,
}

impl<L, C, U> Node<L, C, U> {
    pub(crate) fn as_leaf(&self) -> Option<&L> {
        match &self.data {
            NodeData::Leaf(leaf) => Some(leaf),
            NodeData::Container(_) => None,
        }
    }

    pub(crate) fn is_leaf(&self) -> bool {
        matches!(self.data, NodeData::Leaf(_))
    }

    pub(crate) fn is_wrapping(&self) -> bool {
        self.opts
            .wrap
            .unwrap_or(matches!(self.dir, Direction::Horizontal))
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn is_axis_fill() {
        let horizontal = super::Direction::Horizontal;
        let vertical = super::Direction::Vertical;

        assert!(super::opts::<u16>().fill_x().is_main_fill(horizontal));
        assert!(super::opts::<u16>().fill_y().is_main_fill(vertical));
        assert!(super::opts::<u16>().fill_xy().is_main_fill(horizontal));
        assert!(super::opts::<u16>().fill_xy().is_main_fill(vertical));

        assert!(!super::opts::<u16>().fill_x().is_main_fill(vertical));
        assert!(!super::opts::<u16>().fill_y().is_main_fill(horizontal));
        assert!(!super::opts::<u16>().fill_x().is_cross_fill(horizontal));
        assert!(!super::opts::<u16>().fill_y().is_cross_fill(vertical));

        assert!(!super::opts::<u16>().is_main_fill(horizontal));
        assert!(!super::opts::<u16>().is_main_fill(vertical));
        assert!(!super::opts::<u16>().is_cross_fill(horizontal));
        assert!(!super::opts::<u16>().is_cross_fill(vertical));
    }

    #[test]
    fn filling_one_axis_leaves_the_other_alone() {
        assert_eq!([true, true], super::opts::<u16>().fill_x().fill_y().fill);
        assert_eq!([true, true], super::opts::<u16>().fill_y().fill_x().fill);
        assert_eq!(
            super::opts::<u16>().fill_xy().fill,
            super::opts::<u16>().fill_x().fill_y().fill
        );
    }
}

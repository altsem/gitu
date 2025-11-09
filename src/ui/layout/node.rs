use super::vec2::Vec2;

use super::direction::Direction;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Sizing {
    Fit,
    Flex,
}

pub const OPTS: Opts = Opts {
    dir: Direction::Horizontal,
    gap: 0,
    pad: 0,
    sizing: Sizing::Fit,
};

#[derive(Debug, Copy, Clone)]
pub struct Opts {
    /// Layout direction for children of this node.
    pub(crate) dir: Direction,
    /// The space between each direct child of this node.
    pub(crate) gap: u16,
    pub(crate) sizing: Sizing,
    pub(crate) pad: u16,
}

impl Default for Opts {
    fn default() -> Self {
        OPTS
    }
}

impl Opts {
    pub fn gap(self, gap: u16) -> Self {
        Self { gap, ..self }
    }

    pub fn grow(self) -> Opts {
        Self {
            sizing: Sizing::Flex,
            ..self
        }
    }

    pub(crate) fn pad(self, pad: u16) -> Opts {
        Self { pad, ..self }
    }
}

#[derive(Debug)]
pub(crate) struct Node<T> {
    pub(crate) data: Option<T>,
    /// layout options
    pub(crate) opts: Opts,
    /// space actually occupied by this node, updated as nodes are added
    pub(crate) size: Vec2,
    /// Offset from parent's top-left corner, updated as nodes are added.
    /// This will remain `None` if there's no valid position for the element.
    pub(crate) pos: Option<Vec2>,
}

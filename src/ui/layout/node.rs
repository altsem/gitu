use super::vec2::Vec2;

use super::direction::Direction;

pub const OPTS: Opts = Opts {
    dir: Direction::Horizontal,
    fill: Vec2(0, 0),
    gap: 0,
    pad: 0,
};

#[derive(Debug, Copy, Clone)]
pub struct Opts {
    /// Layout direction for children of this node.
    pub(crate) dir: Direction,
    pub(crate) fill: Vec2,
    /// The space between each direct child of this node.
    pub(crate) gap: u16,
    pub(crate) pad: u16,
}

impl Default for Opts {
    fn default() -> Self {
        OPTS
    }
}

impl Opts {
    pub fn fill_x(self) -> Opts {
        Self {
            fill: Vec2(1, 0),
            ..self
        }
    }

    #[allow(dead_code)]
    pub fn fill_y(self) -> Opts {
        Self {
            fill: Vec2(0, 1),
            ..self
        }
    }

    pub fn fill_xy(self) -> Opts {
        Self {
            fill: Vec2(1, 1),
            ..self
        }
    }

    pub fn gap(self, gap: u16) -> Self {
        Self { gap, ..self }
    }

    pub fn pad(self, pad: u16) -> Opts {
        Self { pad, ..self }
    }

    pub(crate) fn is_main_fill(&self, parent: &Opts) -> bool {
        self.fill * parent.dir.axis() != Vec2(0, 0)
    }

    pub(crate) fn is_cross_fill(&self, parent: &Opts) -> bool {
        self.fill * parent.dir.axis().flip() != Vec2(0, 0)
    }
}

#[derive(Debug)]
pub(crate) struct Node<T> {
    pub(crate) data: Option<T>,
    /// layout options
    pub(crate) opts: Opts,
    /// space actually occupied by this node, updated as nodes are added
    pub(crate) size: Vec2,
    /// Intrinsic (content) size of this subtree, ignoring fill shrinking.
    /// A fill node may shrink below this, but an ancestor that does *not* fill
    /// a given axis still needs the content extent to size itself on that axis.
    pub(crate) content: Vec2,
    /// Offset from parent's top-left corner, updated as nodes are added.
    /// This will remain `None` if there's no valid position for the element.
    pub(crate) pos: Option<Vec2>,
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

        assert!(super::OPTS.fill_x().is_main_fill(&horizontal));
        assert!(super::OPTS.fill_y().is_main_fill(&vertical));
        assert!(super::OPTS.fill_xy().is_main_fill(&horizontal));
        assert!(super::OPTS.fill_xy().is_main_fill(&vertical));

        assert!(!super::OPTS.fill_x().is_main_fill(&vertical));
        assert!(!super::OPTS.fill_y().is_main_fill(&horizontal));
        assert!(!super::OPTS.fill_x().is_cross_fill(&horizontal));
        assert!(!super::OPTS.fill_y().is_cross_fill(&vertical));

        assert!(!super::OPTS.is_main_fill(&horizontal));
        assert!(!super::OPTS.is_main_fill(&vertical));
        assert!(!super::OPTS.is_cross_fill(&horizontal));
        assert!(!super::OPTS.is_cross_fill(&vertical));
    }
}

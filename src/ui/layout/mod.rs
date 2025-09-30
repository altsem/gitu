mod direction;
mod node;
mod vec2;

use unicode_segmentation::UnicodeSegmentation;

use direction::Direction;
use node::{Align, *};
use vec2::Vec2;

pub use node::OPTS;

const ROOT_INDEX: usize = usize::MAX;

#[derive(Debug)]
pub struct LayoutTree<T> {
    data: Vec<Node<T>>,
    index: TreeIndex,
}

#[derive(Debug)]
pub(crate) struct TreeIndex {
    parents: Vec<usize>,
    current_parent: usize,
}

impl TreeIndex {
    pub(crate) fn new() -> Self {
        TreeIndex {
            parents: Vec::new(),
            current_parent: ROOT_INDEX,
        }
    }

    pub(crate) fn iter_roots(&self) -> impl Iterator<Item = usize> {
        self.parents
            .first()
            .map(|_node| 0)
            .into_iter()
            .chain(self.iter_siblings_after(0))
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = usize> {
        0..self.parents.len()
    }

    pub(crate) fn iter_siblings_after(&self, index: usize) -> impl Iterator<Item = usize> {
        let start = index + 1;
        let parent_index = self.parents[index];

        self.parents[start..]
            .iter()
            .take_while(move |&&parent| parent >= parent_index)
            .enumerate()
            .filter(move |&(_i, &parent)| parent == parent_index)
            .map(move |(i, _depth)| start + i)
    }

    pub(crate) fn iter_children(&self, index: usize) -> impl Iterator<Item = usize> {
        let start = index + 1;

        self.parents[start..]
            .iter()
            .take_while(move |&&parent| parent >= index)
            .enumerate()
            .filter(move |&(_i, &parent)| parent == index)
            .map(move |(i, _depth)| start + i)
    }

    pub(crate) fn iter_all_children(&self, index: usize) -> impl Iterator<Item = usize> {
        let start = index + 1;

        self.parents[start..]
            .iter()
            .take_while(move |&&parent| parent >= index)
            .enumerate()
            .map(move |(i, _depth)| start + i)
    }
}

impl<T> LayoutTree<T> {
    pub fn new() -> Self {
        LayoutTree {
            data: Vec::new(),
            index: TreeIndex::new(),
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.index.parents.clear();
        self.index.current_parent = ROOT_INDEX;
    }

    pub(crate) fn add(&mut self, data: Node<T>) {
        self.data.push(data);
        self.index.parents.push(self.index.current_parent);
    }

    pub(crate) fn add_with_children<F: FnOnce(&mut Self)>(&mut self, data: Node<T>, insert_fn: F) {
        self.add(data);
        let our_parent = self.index.current_parent;
        self.index.current_parent = self.index.parents.len() - 1;

        insert_fn(self);

        self.index.current_parent = our_parent;
    }
}

impl<T> Default for LayoutTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutTree<&'static str> {
    /// Add a text leaf, calculating size based on string length
    pub fn text(&mut self, text: &'static str) -> &mut Self {
        let width = text.graphemes(true).count();
        self.leaf_with_size(text, [width as u16, 1]);
        self
    }
}

impl<T: std::fmt::Debug + Clone> LayoutTree<T> {
    pub fn horizontal<F: FnOnce(&mut LayoutTree<T>)>(
        &mut self,
        opts: Opts,
        layout_fn: F,
    ) -> &mut Self {
        self.nest(
            Opts {
                dir: Direction::Horizontal,
                ..opts
            },
            layout_fn,
        )
    }

    pub fn vertical<F: FnOnce(&mut LayoutTree<T>)>(
        &mut self,
        opts: Opts,
        layout_fn: F,
    ) -> &mut Self {
        self.nest(
            Opts {
                dir: Direction::Vertical,
                ..opts
            },
            layout_fn,
        )
    }

    pub fn stacked<F: FnOnce(&mut LayoutTree<T>)>(
        &mut self,
        opts: Opts,
        layout_fn: F,
    ) -> &mut Self {
        self.nest(
            Opts {
                dir: Direction::Stacked,
                ..opts
            },
            layout_fn,
        )
    }

    pub fn leaf(&mut self, data: T) -> &mut Self {
        self.leaf_with_size(data, [1, 1])
    }

    pub fn leaf_with_size(&mut self, data: T, size: [u16; 2]) -> &mut Self {
        self.add(node::Node {
            data: Some(data),
            opts: OPTS,
            size: size.into(),
            pos: None,
        });

        self
    }

    fn nest<F>(&mut self, options: Opts, layout_fn: F) -> &mut Self
    where
        F: FnOnce(&mut LayoutTree<T>),
    {
        self.add_with_children(
            node::Node {
                data: None,
                opts: options,
                size: Vec2(0, 0),
                pos: None,
            },
            layout_fn,
        );

        self
    }

    pub fn compute(&mut self, total_size: [u16; 2]) {
        let Some(root) = self.index.iter_roots().next() else {
            panic!("no root");
        };

        self.compute_subtree(root, Vec2(0, 0), total_size.into());
    }

    fn compute_subtree(&mut self, parent: usize, start: Vec2, total_size: Vec2) {
        let Opts {
            dir, gap, align, ..
        } = self.data[parent].opts;

        let mut current_child = self.index.iter_children(parent).next();
        let (mut cursor, mut children_size) = (start, Vec2(0, 0));

        while let Some(child) = current_child {
            self.compute_subtree(child, cursor, total_size);

            let child_data = &mut self.data[child];

            if (cursor + child_data.size).fits(total_size) {
                child_data.pos = Some(cursor);
            } else {
                // Child doesn't fit where cursor currently is
                let next_line = start * dir.axis() + (cursor + child_data.size) * dir.axis().flip();

                if (next_line + child_data.size).fits(total_size) {
                    // Fits next line
                    cursor = next_line;
                    child_data.pos = Some(cursor);
                } else if (cursor + Vec2(1, 1)).fits(total_size) {
                    // We can fit at least one cell where the cursor currently is
                    child_data.pos = Some(cursor);
                    child_data.size = child_data.size.min(total_size - cursor);
                } else if (next_line + Vec2(1, 1)).fits(total_size) {
                    // We can fit at least one cell on the next line
                    child_data.pos = Some(cursor);
                    child_data.size = child_data.size.min(total_size - cursor);
                } else {
                    // There's absolutely no room left
                    child_data.pos = None;
                    child_data.size = Vec2(0, 0);
                }
            }

            children_size = children_size.max(cursor - start + child_data.size);
            cursor += dir.axis() * (Vec2(gap, gap) + child_data.size);

            current_child = self.index.iter_siblings_after(child).next();
        }

        if align == Align::End {
            let remaining_size = total_size - start - children_size;
            let shift = remaining_size * dir.axis();

            for child in self.index.iter_all_children(parent) {
                if let Some(ref mut pos) = self.data[child].pos {
                    *pos += shift;
                }
            }

            children_size += shift;
        }

        let parent_data = &mut self.data[parent];
        parent_data.size = parent_data.size.max(children_size);
    }

    pub fn iter(&self) -> impl Iterator<Item = LayoutItem<&T>> {
        self.index.iter().filter_map(|index| {
            let Node {
                data: Some(data),
                opts: _,
                size,
                pos,
            } = &self.data[index]
            else {
                return None;
            };

            Some(LayoutItem {
                data,
                pos: (*pos)?.into(),
                size: (*size).into(),
            })
        })
    }
}

#[derive(Debug)]
pub struct LayoutItem<T> {
    pub data: T,
    pub pos: [u16; 2],
    pub size: [u16; 2],
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Render the layout to a string for testing-purposes, does not support unicode
    fn render_to_string(layout: LayoutTree<&'static str>) -> String {
        let Some(root) = layout.index.iter_roots().next() else {
            panic!("no root");
        };
        let node = &layout.data[root];
        let mut grid = vec![vec![' '; node.size.0 as usize]; node.size.1 as usize];

        // Fill the grid with positioned content
        for LayoutItem { data, pos, size } in layout.iter() {
            let content_str = data.to_string();
            if pos[1] < node.size.1 {
                for (char_idx, ch) in content_str[..size[0] as usize].chars().enumerate() {
                    let x = pos[0] + char_idx as u16;
                    grid[pos[1] as usize][x as usize] = ch;
                }
            }
        }

        grid.into_iter()
            .map(|row| row.into_iter().collect::<String>().trim_end().to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn single_text() {
        let mut layout = LayoutTree::new();

        layout.vertical(OPTS, |layout| {
            layout.text("Hello");
            layout.text("lol");
        });

        layout.compute([5, 2]);
        insta::assert_snapshot!(render_to_string(layout));
    }

    #[test]
    fn horizontal_layout() {
        let mut layout = LayoutTree::new();

        layout.horizontal(OPTS, |layout| {
            layout.text("A");
            layout.text("BB");
            layout.text("CCC");
        });

        layout.compute([6, 1]);
        insta::assert_snapshot!(render_to_string(layout));
    }

    #[test]
    fn vertical_layout() {
        let mut layout = LayoutTree::new();

        layout.vertical(OPTS, |layout| {
            layout.text("First");
            layout.text("Second");
            layout.text("Third");
        });

        layout.compute([6, 3]);
        insta::assert_snapshot!(render_to_string(layout));
    }

    #[test]
    fn nested_layouts() {
        let mut layout = LayoutTree::new();

        layout.horizontal(OPTS, |layout| {
            // 0
            layout.vertical(OPTS, |layout| {
                // 1
                layout.text("A"); // 2
                layout.text("B"); // 3
            });
            layout.vertical(OPTS, |layout| {
                // 4
                layout.text("C"); // 5
                layout.text("D"); // 6
            });
        });

        layout.compute([2, 2]);
        insta::assert_snapshot!(render_to_string(layout));
    }

    #[test]
    fn clear_layout() {
        let mut layout = LayoutTree::new();

        layout.text("Test");

        layout.clear();
        assert_eq!(layout.iter().count(), 0);
    }

    #[test]
    fn out_of_bounds_horizontal() {
        let mut layout = LayoutTree::new();

        layout.vertical(OPTS, |layout| {
            layout.horizontal(OPTS, |layout| {
                layout.text("12345");
                layout.text("The very start of this will be visible (a T)");
            });
            layout.horizontal(OPTS, |layout| {
                layout.text("123456");
                layout.text("This is completely outside of the layout and ignored");
            });
        });

        layout.compute([6, 4]);
        insta::assert_snapshot!(render_to_string(layout));
    }

    #[test]
    fn out_of_bounds_vertical() {
        let mut layout = LayoutTree::new();

        layout.horizontal(OPTS, |layout| {
            layout.vertical(OPTS, |layout| {
                layout.text("1");
                layout.text("2");
            });
            layout.vertical(OPTS, |layout| {
                layout.text("1");
                layout.text("2");
                layout.text("X");
            });
        });

        layout.compute([2, 2]);
        insta::assert_snapshot!(render_to_string(layout));
    }

    #[test]
    fn out_of_bounds_horizontal_align_end() {
        let mut layout = LayoutTree::new();

        layout.vertical(OPTS, |layout| {
            layout.horizontal(OPTS.align_end(), |layout| {
                layout.text("12345");
                layout.text("The very start of this will be visible (a T)");
            });
            layout.horizontal(OPTS.align_end(), |layout| {
                layout.text("123456");
                layout.text("This is completely outside of the layout and ignored");
            });
        });

        layout.compute([6, 4]);
        insta::assert_snapshot!(render_to_string(layout));
    }

    #[test]
    fn out_of_bounds_vertical_align_end() {
        let mut layout = LayoutTree::new();

        layout.horizontal(OPTS, |layout| {
            layout.vertical(OPTS.align_end(), |layout| {
                layout.text("1");
                layout.text("2");
                layout.text("X");
            });
        });

        layout.compute([1, 2]);
        insta::assert_snapshot!(render_to_string(layout));
    }

    #[test]
    fn unicode_text_width() {
        let mut layout = LayoutTree::new();

        layout.horizontal(OPTS, |layout| {
            layout.text("café").text("naïve");
        });

        layout.compute([10, 1]);
        let items: Vec<_> = layout.iter().collect();
        assert_eq!(items[0].size, [4, 1]); // café has 4 graphemes
    }

    #[test]
    fn horizontal_gap() {
        let mut layout = LayoutTree::new();

        layout.horizontal(OPTS.gap(2), |layout| {
            layout.text("one");
            layout.text("two");
        });

        layout.compute([8, 1]);
        insta::assert_snapshot!(render_to_string(layout));
    }

    #[test]
    fn vertical_gap() {
        let mut layout = LayoutTree::new();

        layout.vertical(OPTS.gap(1), |layout| {
            layout.text("one");
            layout.text("two");
        });

        layout.compute([3, 3]);
        insta::assert_snapshot!(render_to_string(layout));
    }

    #[test]
    fn stacked() {
        let mut layout = LayoutTree::new();

        layout.stacked(OPTS, |layout| {
            layout.text("This is under. (leftovers here)");
            layout.text("This is on top");
        });

        layout.compute([31, 2]);
        insta::assert_snapshot!(render_to_string(layout));
    }

    #[test]
    fn align_bottom() {
        let mut layout = LayoutTree::new();

        layout.stacked(OPTS, |layout| {
            layout.vertical(OPTS, |layout| {
                layout.text("Stack 1");
            });

            layout.vertical(OPTS.align_end(), |layout| {
                layout.text("Stack 2, bottom aligned");
            });
        });

        layout.compute([30, 3]);
        insta::assert_snapshot!(render_to_string(layout));
    }

    #[test]
    fn align_right() {
        let mut layout = LayoutTree::new();

        layout.horizontal(OPTS.align_end().gap(1), |layout| {
            layout.text("Aligned");
            layout.text("to");
            layout.text("the");
            layout.text("right");
        });

        layout.compute([40, 1]);
        insta::assert_snapshot!(render_to_string(layout));
    }

    #[test]
    fn gitu_mockup() {
        let mut layout = LayoutTree::new();

        layout.stacked(OPTS, |layout| {
            layout.vertical(OPTS.gap(1), |layout| {
                layout.vertical(OPTS, |layout| {
                    layout.text("On branch master");
                    layout.vertical(OPTS, |layout| {
                        layout.text("Your branch is up to date with 'origin/master'");
                    });
                });

                layout.vertical(OPTS, |layout| {
                    layout.text("Recent commits");
                    layout.vertical(OPTS, |layout| {
                        layout.text("b3492a8 master origin/master chore: update dependencies");
                        layout.text("013844c refactor: appease linter");
                        layout.text("5536ea3 feat: Show the diff on the stash detail screen");
                    });
                });
            });

            layout.vertical(OPTS.align_end(), |layout| {
                layout.text("───────────────────────────────────────────────────────────────");

                layout.horizontal(OPTS.gap(2), |layout| {
                    layout.vertical(OPTS, |layout| {
                        layout.text("Help");
                        layout.text("Y Show Refs");
                        layout.text("<tab> Toggle section");
                        layout.text("k/<up> Up ");
                        layout.text("j/<down> Down");
                        layout.text("<ctrl+k>/<ctrl+up> Up line");
                        layout.text("<ctrl+j>/<ctrl+down> Down line");
                        layout.text("<alt+k>/<alt+up> Prev section");
                        layout.text("<alt+j>/<alt+down> Next section");
                        layout.text("<alt+h>/<alt+left> Parent section");
                        layout.text("<ctrl+u> Half page up");
                        layout.text("<ctrl+d> Half page down");
                        layout.text("g Refresh");
                        layout.text("q/<esc> Quit/Close");
                    });
                    layout.vertical(OPTS, |layout| {
                        layout.text("Submenu");
                        layout.text("b Branch");
                        layout.text("c Commit");
                        layout.text("f Fetch");
                        layout.text("h/? Help");
                        layout.text("l Log");
                        layout.text("M Remote");
                        layout.text("F Pull");
                        layout.text("P Push");
                        layout.text("r Rebase");
                        layout.text("X Reset");
                        layout.text("V Revert");
                        layout.text("z Stash");
                        layout.text("");
                    });
                    layout.vertical(OPTS, |layout| {
                        layout.text("@@ -271,7 +271,7");
                        layout.text("s Stage");
                        layout.text("u Unstage");
                        layout.text("<enter> Show");
                        layout.text("K Discard");
                        layout.text("");
                        layout.text("");
                        layout.text("");
                        layout.text("");
                        layout.text("");
                        layout.text("");
                        layout.text("");
                        layout.text("");
                        layout.text("");
                    });
                });
            });
        });

        layout.compute([80, 30]);
        insta::assert_snapshot!(render_to_string(layout));
    }
}

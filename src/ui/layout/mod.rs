mod direction;
mod node;
mod vec2;

use std::iter;

use unicode_segmentation::UnicodeSegmentation;

use direction::Direction;
use node::*;
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

    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn text(&mut self, text: &'static str) -> &mut Self {
        let width = text.graphemes(true).count();
        self.leaf_with_size(text, [width as u16, 1]);
        self
    }
}

impl<T: std::fmt::Debug + Clone> LayoutTree<T> {
    pub fn horizontal<F: FnOnce(&mut LayoutTree<T>)>(
        &mut self,
        data: Option<T>,
        opts: Opts,
        layout_fn: F,
    ) -> &mut Self {
        {
            let ode = node::Node {
                data,
                opts: Opts {
                    dir: Direction::Horizontal,
                    ..opts
                },
                size: Vec2(0, 0),
                pos: None,
            };

            self.add_with_children(ode, layout_fn);
            self
        }
    }

    pub fn vertical<F: FnOnce(&mut LayoutTree<T>)>(
        &mut self,
        data: Option<T>,
        opts: Opts,
        layout_fn: F,
    ) -> &mut Self {
        {
            let node = node::Node {
                data,
                opts: Opts {
                    dir: Direction::Vertical,
                    ..opts
                },
                size: Vec2(0, 0),
                pos: None,
            };

            self.add_with_children(node, layout_fn);
            self
        }
    }

    #[allow(dead_code)]
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

    pub fn compute(&mut self, avail_size: [u16; 2]) {
        let Some(root) = self.index.iter_roots().next() else {
            panic!("no root");
        };

        let size = Vec2::from(avail_size);
        self.compute_subtree(root, Vec2(0, 0), size, Vec2(0, 0), Sizing::Fit);

        let grow = Vec2::from(avail_size).saturating_sub(self.data[root].size);
        self.compute_subtree(root, Vec2(0, 0), avail_size.into(), grow, Sizing::Flex);
    }

    fn compute_subtree(
        &mut self,
        parent: usize,
        start: Vec2,
        avail_size: Vec2,
        parent_grow: Vec2,
        pass: Sizing,
    ) {
        let Some(child) = self.index.iter_children(parent).next() else {
            return;
        };

        let Opts {
            dir,
            gap,
            pad,
            sizing,
            ..
        } = self.data[parent].opts;

        let mut current_child = Some(child);
        let mut cursor = Vec2(pad, pad) * dir.axis();
        let mut size = Vec2(0, 0);
        let mut grow_iter = self.iter_distribute_size(parent, parent_grow, dir);

        while let Some(child) = current_child {
            let child_grow = grow_iter.next().unwrap();
            let child_avail_size =
                if pass == Sizing::Flex && self.data[child].opts.sizing == Sizing::Flex {
                    self.data[child].size + child_grow
                } else {
                    avail_size.saturating_sub(cursor)
                };

            self.compute_subtree(child, start + cursor, child_avail_size, child_grow, pass);

            let child_data = &mut self.data[child];

            if (cursor + child_data.size).fits(avail_size) {
                child_data.pos = Some(start + cursor);
            } else {
                // Child doesn't fit where cursor currently is
                let next_line = size * dir.axis().flip();

                if (next_line + child_data.size).fits(avail_size) {
                    // Fits completely on next line
                    cursor = next_line;
                    child_data.pos = Some(start + cursor);
                } else if (cursor + Vec2(1, 1)).fits(avail_size) {
                    // Can't wrap, but we can fit at least one cell where the cursor currently is
                    child_data.pos = Some(start + cursor);
                    child_data.size = child_data.size.min(avail_size.saturating_sub(cursor));
                } else {
                    // There's absolutely no room left anywhere
                    child_data.pos = None;
                }
            }

            size = size.max(cursor + child_data.size);
            cursor += dir.axis() * (Vec2(gap, gap) + child_data.size);

            current_child = self.index.iter_siblings_after(child).next();
        }

        size += Vec2(pad, pad) * dir.axis();

        if pass == Sizing::Flex && sizing == Sizing::Flex {
            self.data[parent].size += parent_grow;
        } else if pass == Sizing::Fit && sizing == Sizing::Flex {
            // Zero-out the axis that is supposed to grow
            self.data[parent].size = size * dir.axis().flip();
        } else {
            self.data[parent].size = size;
        }
    }

    fn iter_distribute_size(
        &self,
        parent: usize,
        size_to_distribute: Vec2,
        dir: Direction,
    ) -> impl Iterator<Item = Vec2> + use<T> {
        let along_axis = size_to_distribute * dir.axis();
        let div = if along_axis != Vec2(0, 0) {
            self.index
                .iter_children(parent)
                .filter(|&child| self.data[child].opts.sizing == Sizing::Flex)
                .count()
                .max(1) as u16
        } else {
            1
        };

        let quot = along_axis / Vec2(div, div);
        let rem = along_axis % Vec2(div, div);
        let off_axis_amount = size_to_distribute * dir.axis().flip();
        iter::once(quot + rem + off_axis_amount).chain(iter::once(quot + off_axis_amount).cycle())
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
    use itertools::Itertools;

    use super::*;

    /// Render the layout to a string for testing purposes.
    /// Note: ASCII only — does not support Unicode beyond single-byte chars.
    fn render_to_string(layout: LayoutTree<&'static str>) -> String {
        let Some(root) = layout.index.iter_roots().next() else {
            panic!("no root");
        };

        let node = &layout.data[root];
        let width = node.size.0 as usize;
        let height = node.size.1 as usize;

        let mut grid = vec![' '; height * width];

        for LayoutItem { data, pos, size } in layout.iter() {
            let x0 = pos[0] as usize;
            let y0 = pos[1] as usize;
            let item_width = size[0] as usize;

            for (i, c) in data.chars().take(item_width).enumerate() {
                grid[y0 * width + (x0 + i)] = c;
            }
        }

        grid.chunks(width)
            .map(|row| row.iter().collect::<String>().trim_end().to_string())
            .join("\n")
    }

    #[test]
    fn single_text() {
        let mut layout = LayoutTree::new();

        layout.vertical(None, OPTS, |layout| {
            layout.text("Hello");
            layout.text("lol");
        });

        layout.compute([5, 2]);
        insta::assert_snapshot!(render_to_string(layout));
    }

    #[test]
    fn horizontal_layout() {
        let mut layout = LayoutTree::new();

        layout.horizontal(None, OPTS, |layout| {
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

        layout.vertical(None, OPTS, |layout| {
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

        layout.horizontal(None, OPTS, |layout| {
            // 0
            layout.vertical(None, OPTS, |layout| {
                // 1
                layout.text("A"); // 2
                layout.text("B"); // 3
            });
            layout.vertical(None, OPTS, |layout| {
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

        layout.vertical(None, OPTS, |layout| {
            layout.horizontal(None, OPTS, |layout| {
                layout.text("12345");
                layout.text("The very start of this will be visible (a T)");
            });
            layout.horizontal(None, OPTS, |layout| {
                layout.text("123456");
                layout.text("This is completely outside of the layout and ignored");
            });
        });

        layout.compute([6, 4]);
        insta::assert_snapshot!(render_to_string(layout));
    }

    // TODO wrapping test (code commented above)
    // #[test]
    // fn test_horizontal_wrap() {
    //     let mut layout = LayoutTree::new();

    //     layout.horizontal(None, OPTS, |layout| {
    //         layout.text("AAA");
    //         layout.text("BBB");
    //         layout.text("CCC");
    //     });

    //     layout.compute([6, 2]);
    //     let result = render_to_string(layout);
    //     println!("Result:\n{}", result);
    //     // Should wrap: "AAABBB" on first line, "CCC" on second line
    //     assert_eq!(result, "AAABBB\nCCC");
    // }

    // TODO wrapping test (code commented above)
    // #[test]
    // fn test_wrap_before_truncate() {
    //     let mut layout = LayoutTree::new();

    //     layout.horizontal(None, OPTS, |layout| {
    //         layout.text("AAAA");
    //         layout.text("BBBB");
    //     });

    //     layout.compute([6, 2]);
    //     let result = render_to_string(layout);
    //     println!("Result:\n{}", result);
    //     // With 6 chars width and 2 rows:
    //     // "AAAA" fits (4 chars), then "BBBB" doesn't fit in remaining 2 chars
    //     // Should wrap "BBBB" to next line rather than truncating to "BB"
    //     assert_eq!(result, "AAAA\nBBBB");
    // }

    #[test]
    fn test_no_trailing_newline() {
        let mut layout = LayoutTree::new();

        layout.vertical(None, OPTS, |layout| {
            layout.text("Line 1");
            layout.text("Line 2");
        });

        layout.compute([10, 2]);
        let result = render_to_string(layout);
        println!("Result bytes: {:?}", result.as_bytes());
        println!("Result repr: {:?}", result);
        // Should not have trailing newline
        assert!(!result.ends_with('\n'), "Should not have trailing newline");
        assert_eq!(result, "Line 1\nLine 2");
    }

    #[test]
    fn out_of_bounds_vertical() {
        let mut layout = LayoutTree::new();

        layout.horizontal(None, OPTS, |layout| {
            layout.vertical(None, OPTS, |layout| {
                layout.text("1");
                layout.text("2");
            });
            layout.vertical(None, OPTS, |layout| {
                layout.text("1");
                layout.text("2");
                layout.text("X");
            });
        });

        layout.compute([2, 2]);
        insta::assert_snapshot!(render_to_string(layout));
    }

    #[test]
    fn unicode_text_width() {
        let mut layout = LayoutTree::new();

        layout.horizontal(None, OPTS, |layout| {
            layout.text("café").text("naïve");
        });

        layout.compute([10, 1]);
        let items: Vec<_> = layout.iter().collect();
        assert_eq!(items[0].size, [4, 1]); // café has 4 graphemes
    }

    #[test]
    fn horizontal_gap() {
        let mut layout = LayoutTree::new();

        layout.horizontal(None, OPTS.gap(2), |layout| {
            layout.text("one");
            layout.text("two");
        });

        layout.compute([8, 1]);
        insta::assert_snapshot!(render_to_string(layout));
    }

    #[test]
    fn vertical_gap() {
        let mut layout = LayoutTree::new();

        layout.vertical(None, OPTS.gap(1), |layout| {
            layout.text("one");
            layout.text("two");
        });

        layout.compute([3, 3]);
        insta::assert_snapshot!(render_to_string(layout));
    }

    #[test]
    fn grow() {
        let mut layout = LayoutTree::new();

        layout.vertical(None, OPTS, |layout| {
            layout.vertical(None, OPTS.grow(), |layout| {
                layout.text("flex");
            });
            layout.text("actual");
        });

        layout.compute([8, 3]);
        insta::assert_snapshot!(render_to_string(layout));
    }

    #[test]
    fn overflow() {
        let mut layout = LayoutTree::new();

        layout.vertical(None, OPTS, |layout| {
            layout.text("one");
            layout.text("twoooo");
        });

        layout.compute([20, 1]);
        insta::assert_snapshot!(render_to_string(layout));
    }

    #[test]
    fn shrink() {
        let mut layout = LayoutTree::new();

        layout.vertical(None, OPTS, |layout| {
            layout.vertical(None, OPTS.grow(), |layout| {
                layout.text("flex 1");
                layout.text("flex 2");
            });
            layout.text("actual");
        });

        layout.compute([20, 2]);
        insta::assert_snapshot!(render_to_string(layout));
    }

    #[test]
    fn gitu_mockup() {
        let mut layout = LayoutTree::new();

        layout.vertical(None, OPTS, |layout| {
            layout.vertical(None, OPTS.grow().gap(1), |layout| {
                layout.vertical(None, OPTS, |layout| {
                    layout.text("On branch master");
                    layout.vertical(None, OPTS, |layout| {
                        layout.text("Your branch is up to date with 'origin/master'");
                    });
                });

                layout.vertical(None, OPTS, |layout| {
                    layout.text("Recent commits");
                    layout.vertical(None, OPTS, |layout| {
                        layout.text(
                            "9eb6a63 refactor/ui origin/refactor/ui fix more rendering issues",
                        );
                        layout.text("b5fffd4 fix styling issues in Screen");
                        layout.text("61e6c1b refactor: extract type of LayoutTree");
                        layout.text("df3bcb5 get rid of frequent clone() in LayoutTree");
                        layout.text("9864859 refactor(ui): less allocs");
                        layout.text(
                            "aa2811e refactor: new LayoutTree module to improve on ui headaches",
                        );
                        layout.text("5374ab3 master origin/master test: add file:// in clone_and_commit fn as well");
                        layout.text("7a66235 test: get rid of setup_init, and try fix test-repo assertion");
                        layout.text("75463c8 test/fix-ci test: forgot to create testfiles/ when running tests");
                    });
                });
            });

            layout.vertical(None, OPTS, |layout| {
                layout.text("───────────────────────────────────────────────────────────────");

                layout.horizontal(None, OPTS.gap(2), |layout| {
                    layout.vertical(None, OPTS, |layout| {
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
                    layout.vertical(None, OPTS, |layout| {
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
                    layout.vertical(None, OPTS, |layout| {
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

        layout.compute([80, 25]);
        let root = layout.index.iter_roots().next().unwrap();
        let root_size = layout.data[root].size;
        eprintln!("Root size: {:?}", root_size);
        let result = render_to_string(layout);
        eprintln!("Result has {} lines", result.lines().count());
        eprintln!("Result ends with newline: {}", result.ends_with('\n'));
        eprintln!("Result len: {}", result.len());
        eprintln!(
            "Last 3 bytes: {:?}",
            &result.as_bytes()[result.len().saturating_sub(3)..]
        );
        insta::assert_snapshot!(result);
    }
}

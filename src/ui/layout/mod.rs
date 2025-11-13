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

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Pass {
    Fit,
    Fill,
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
                content: Vec2(0, 0),
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
                content: Vec2(0, 0),
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
            content: size.into(),
            pos: None,
        });

        self
    }

    pub fn compute(&mut self, avail_size: [u16; 2]) {
        let Some(root) = self.index.iter_roots().next() else {
            panic!("no root");
        };

        let size = Vec2::from(avail_size);
        self.data[root].pos = Some(Vec2(0, 0));

        for pass in [Pass::Fit, Pass::Fill] {
            if let Some(root_size) = self.compute_subtree(root, Vec2(0, 0), size, pass) {
                self.data[root].size = root_size;
            }
        }
    }

    fn compute_subtree(
        &mut self,
        parent: usize,
        outer_start: Vec2,
        outer_avail_size: Vec2,
        pass: Pass,
    ) -> Option<Vec2> {
        let child = self.index.iter_children(parent).next()?;
        let parent_opts = self.data[parent].opts;
        let main_axis = parent_opts.dir.axis();
        let cross_axis = main_axis.flip();
        let padding = Vec2(parent_opts.pad, parent_opts.pad) * main_axis;
        let avail_size = outer_avail_size
            .saturating_sub(padding)
            .saturating_sub(padding);

        let mut current_child = Some(child);
        let mut cursor = Vec2(0, 0);
        let mut size = Vec2(0, 0);
        // Intrinsic extent, accumulated independently of any fill shrinking.
        let mut content_cursor = Vec2(0, 0);
        let mut content = Vec2(0, 0);

        let fill_avail = match pass {
            Pass::Fit => Vec2(0, 0),
            Pass::Fill => avail_size.saturating_sub(self.data[parent].size),
        };

        let mut main_fill_iter = self.dist_main_fill(parent, fill_avail);
        let cross_fill = avail_size * cross_axis;

        while let Some(child) = current_child {
            let child_start = outer_start + padding + cursor;
            let is_main_fill = self.data[child].opts.is_main_fill(&parent_opts);
            let is_cross_fill = self.data[child].opts.is_cross_fill(&parent_opts);

            let mut child_size = match pass {
                Pass::Fit => {
                    let child_avail_size = avail_size.saturating_sub(cursor);

                    // `flow` is the shrinkable extent, `child_content` the intrinsic one.
                    // They only differ where a descendant fills an axis.
                    let (flow, child_content) =
                        match self.compute_subtree(child, child_start, child_avail_size, pass) {
                            Some(flow) => (flow, self.data[child].content),
                            None => (self.data[child].size, self.data[child].size),
                        };

                    self.data[child].content = child_content;
                    content = content.max(content_cursor + child_content);
                    content_cursor +=
                        main_axis * (Vec2(parent_opts.gap, parent_opts.gap) + child_content);

                    // On an axis this child fills it may shrink, so take the flow
                    // extent. On an axis it does not fill, its size is its content.
                    let fill_mask = self.data[child].opts.fill;
                    let content_mask = Vec2(1, 1).saturating_sub(fill_mask);
                    let resolved = flow * fill_mask + child_content * content_mask;

                    if is_main_fill {
                        // Zero-out the main axis that this child will later fill,
                        // this enables it to shrink.
                        resolved * cross_axis
                    } else {
                        resolved
                    }
                }
                Pass::Fill => {
                    let fill = {
                        let mut sum = Vec2(0, 0);
                        if is_main_fill {
                            sum += main_fill_iter.next().unwrap()
                        }
                        if is_cross_fill {
                            sum += cross_fill
                        }
                        sum
                    };

                    let child_avail_size = avail_size
                        .saturating_sub(cursor)
                        .min(self.data[child].size + fill);

                    if self
                        .compute_subtree(child, child_start, child_avail_size, pass)
                        .is_some()
                    {
                        child_avail_size
                    } else {
                        self.data[child].size
                    }
                }
            };

            let mut child_pos = None;

            if (cursor + child_size).fits(avail_size) {
                child_pos = Some(outer_start + padding + cursor);
            } else {
                // Child doesn't fit where cursor currently is
                let next_line = size * cross_axis;

                if (next_line + child_size).fits(avail_size) {
                    // Fits completely on next line
                    cursor = next_line;
                    child_pos = Some(outer_start + padding + cursor);
                } else if (cursor + Vec2(1, 1)).fits(avail_size) {
                    // Can't wrap, but we can fit at least one cell where the cursor currently is
                    child_pos = Some(outer_start + padding + cursor);
                    child_size = child_size.min(avail_size.saturating_sub(cursor));
                }
            }

            if child_pos.is_some() {
                size = size.max(cursor + child_size);
                cursor += main_axis * (Vec2(parent_opts.gap, parent_opts.gap) + child_size);
            }

            self.data[child].pos = child_pos;
            self.data[child].size = child_size;

            current_child = self.index.iter_siblings_after(child).next();
        }

        // Content is accumulated without wrapping, so it can come out smaller on
        // the cross axis than what was actually laid out. Never report less than
        // the placed extent.
        let content = content.max(size);

        size += padding + padding;
        self.data[parent].content = content + padding + padding;

        Some(size)
    }

    fn dist_main_fill(
        &self,
        parent: usize,
        size_to_distribute: Vec2,
    ) -> impl Iterator<Item = Vec2> + use<T> {
        let axis = self.data[parent].opts.dir.axis();
        let along_axis = size_to_distribute * axis;
        let count = self
            .index
            .iter_children(parent)
            .filter(|&child| self.data[child].opts.is_main_fill(&self.data[parent].opts))
            .count() as u16;

        let (quot, rem) = if count > 0 {
            let quot = along_axis / Vec2(count, count);
            let rem = along_axis % Vec2(count, count);
            (quot, rem)
        } else {
            (Vec2(0, 0), Vec2(0, 0))
        };

        iter::once(quot + rem)
            .chain(iter::once(quot).cycle())
            .take(count as usize)
    }

    pub fn iter(&self) -> impl Iterator<Item = LayoutItem<&T>> {
        self.index.iter().filter_map(|index| {
            let Node {
                data: Some(data),
                opts: _,
                size,
                content: _,
                pos,
            } = &self.data[index]
            else {
                return None;
            };

            if size.0 == 0 || size.1 == 0 {
                return None;
            }

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
    fn test_iter_distribute_size_no_flex() {
        let mut layout = LayoutTree::new();
        layout.vertical(None, OPTS, |layout| {
            // Neither should grow
            layout.text("Hello");
            layout.text("Hello");
        });

        let mut iter = layout.dist_main_fill(0, Vec2(10, 3));
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_iter_distribute_size_one_flex() {
        let mut layout = LayoutTree::new();
        layout.vertical(None, OPTS, |layout| {
            layout.horizontal(None, OPTS, |layout| {
                layout.text("One");
            });
            // This should shrink to 0 vertically and then grow to 3
            layout.horizontal(None, OPTS.fill_y(), |layout| {
                layout.text("Two");
            });
        });

        let mut iter = layout.dist_main_fill(0, Vec2(10, 3));
        assert_eq!(Some(Vec2(0, 3)), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_iter_distribute_size_all_flex() {
        let mut layout = LayoutTree::new();
        layout.vertical(None, OPTS, |layout| {
            // Both should grow, favoring the first
            layout.horizontal(None, OPTS.fill_y(), |layout| {
                layout.text("One");
            });
            layout.horizontal(None, OPTS.fill_y(), |layout| {
                layout.text("Two");
            });
        });

        let mut iter = layout.dist_main_fill(0, Vec2(10, 5));
        assert_eq!(Some(Vec2(0, 3)), iter.next());
        assert_eq!(Some(Vec2(0, 2)), iter.next());
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

    #[test]
    fn test_horizontal_wrap() {
        let mut layout = LayoutTree::new();

        layout.horizontal(None, OPTS, |layout| {
            layout.text("AAA");
            layout.text("BBB");
            layout.text("CCC");
        });

        layout.compute([6, 2]);
        let result = render_to_string(layout);
        println!("Result:\n{}", result);
        // Should wrap: "AAABBB" on first line, "CCC" on second line
        assert_eq!(result, "AAABBB\nCCC");
    }

    #[test]
    fn test_wrap_before_truncate() {
        let mut layout = LayoutTree::new();

        layout.horizontal(None, OPTS, |layout| {
            layout.text("AAAA");
            layout.text("BBBB");
        });

        layout.compute([6, 2]);
        let result = render_to_string(layout);
        println!("Result:\n{}", result);
        // With 6 chars width and 2 rows:
        // "AAAA" fits (4 chars), then "BBBB" doesn't fit in remaining 2 chars
        // Should wrap "BBBB" to next line rather than truncating to "BB"
        assert_eq!(result, "AAAA\nBBBB");
    }

    #[test]
    fn nested_grow_wrap() {
        let mut layout = LayoutTree::new();

        layout.vertical(None, OPTS, |layout| {
            layout.vertical(None, OPTS.fill_y(), |layout| {
                layout.horizontal(None, OPTS, |layout| {
                    layout.text("word1");
                    layout.text("word2");
                    layout.text("word3");
                });
            });
        });

        layout.compute([10, 2]);
        let mut iter = layout.iter().map(|e| (*e.data, e.pos, e.size));
        assert_eq!(Some(("word1", [0, 0], [5, 1])), iter.next());
        assert_eq!(Some(("word2", [5, 0], [5, 1])), iter.next());
        assert_eq!(Some(("word3", [0, 1], [5, 1])), iter.next());
        assert_eq!(None, iter.next());
    }

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
            layout.vertical(None, OPTS.fill_y(), |layout| {
                layout.text("flex");
            });
            layout.text("actual");
        });

        layout.compute([8, 3]);
        insta::assert_snapshot!(render_to_string(layout));
    }

    #[test]
    fn nested_grow_preserves_cross_axis() {
        let mut layout = LayoutTree::new();

        layout.vertical(Some("root"), OPTS, |layout| {
            layout.horizontal(Some("grow"), OPTS.fill_xy(), |layout| {
                layout.text("hello");
            });
        });

        layout.compute([20, 10]);

        let mut iter = layout.iter().map(|e| (*e.data, e.pos, e.size));

        assert_eq!(Some(("root", [0, 0], [20, 10])), iter.next());
        assert_eq!(Some(("grow", [0, 0], [20, 10])), iter.next());
        assert_eq!(Some(("hello", [0, 0], [5, 1])), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn overflow() {
        let mut layout = LayoutTree::new();

        layout.vertical(None, OPTS, |layout| {
            layout.text("one");
            layout.text("twoooo");
            layout.text("three");
        });

        layout.compute([6, 1]);
        insta::assert_snapshot!(render_to_string(layout));
    }

    #[test]
    fn shrink() {
        let mut layout = LayoutTree::new();

        layout.vertical(None, OPTS, |layout| {
            layout.vertical(None, OPTS.fill_y(), |layout| {
                layout.text("flex 1");
                layout.text("flex 2");
            });
            layout.text("actual");
        });

        layout.compute([6, 2]);
        insta::assert_snapshot!(render_to_string(layout));
    }

    #[test]
    fn shrinks_nested() {
        let mut layout = LayoutTree::new();

        layout.vertical(None, OPTS, |layout| {
            layout.vertical(None, OPTS.fill_y(), |layout| {
                layout.vertical(None, OPTS, |layout| {
                    layout.text("This should not be visible'");
                });
            });

            layout.vertical(None, OPTS, |layout| {
                layout.text("WEEEEE");
            });
        });

        layout.compute([40, 1]);
        let mut iter = layout.iter().map(|e| (*e.data, e.pos, e.size));
        assert_eq!(Some(("WEEEEE", [0, 0], [6, 1])), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn nested_horizontal_fill_does_not_hide_siblings() {
        let mut layout = LayoutTree::new();

        layout.vertical(None, OPTS, |layout| {
            // 0,0 -> 0,5
            layout.vertical(None, OPTS.fill_y(), |layout| {
                // 0,1 -> 0,1
                layout.horizontal(None, OPTS.fill_x(), |layout| {
                    // 0,1 -> 0,1
                    layout.horizontal(None, OPTS.fill_x(), |layout| {
                        layout.text("visible");
                    });
                });
            });
        });

        layout.compute([20, 5]);
        let mut iter = layout.iter().map(|e| (*e.data, e.pos, e.size));
        assert_eq!(Some(("visible", [0, 0], [7, 1])), iter.next());
        assert_eq!(None, iter.next());

        // insta::assert_snapshot!(render_to_string(layout));
    }

    #[test]
    fn gitu_mockup() {
        let mut layout = LayoutTree::new();

        layout.vertical(None, OPTS, |layout| {
            layout.vertical(None, OPTS.fill_xy(), |layout| {
                // Screen
                layout.vertical(None, OPTS.fill_x(), |layout| {
                    layout.horizontal(Some(""), OPTS.fill_x(), |layout| {
                        layout.text("On branch master");
                    });
                    layout.text("Your branch is up to date with 'origin/master'");
                });

                layout.text("");
                layout.text("Recent commits");
                layout.text("9eb6a63 refactor/ui origin/refactor/ui fix more rendering issues");
                layout.text("b5fffd4 fix styling issues in Screen");
                layout.text("61e6c1b refactor: extract type of LayoutTree");
                layout.text("df3bcb5 get rid of frequent clone() in LayoutTree");
                layout.text("9864859 refactor(ui): less allocs");
                layout.text("aa2811e refactor: new LayoutTree module to improve on ui headaches");
                layout.text(
                    "5374ab3 master origin/master test: add file:// in clone_and_commit fn as well",
                );
                layout.text("7a66235 test: get rid of setup_init, and try fix test-repo assertion");
                layout.text(
                    "75463c8 test/fix-ci test: forgot to create testfiles/ when running tests",
                );
            });

            layout.vertical(None, OPTS, |layout| {
                // Menu
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
                        layout.text("g+r Refresh");
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
        insta::assert_snapshot!(render_to_string(layout));
    }
}

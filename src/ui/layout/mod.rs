mod direction;
mod node;
mod vec2;

use std::iter;

use direction::Direction;
use node::*;
use vec2::Vec2;

pub use node::{Opts, opts};
pub use vec2::Scalar;

const ROOT: usize = 0;
const NO_PARENT: usize = usize::MAX;

/// Sizes an item as it is added to the tree, in whichever unit the layout is
/// measured in.
pub trait Measure {
    type Unit: Scalar;

    fn measure(&self) -> [Self::Unit; 2];
}

/// LayoutTree contains the intermediate data used for computing a layout.
/// `L` is the Leaf data type.
/// `C` is the Container data type.
#[derive(Debug)]
pub struct LayoutTree<L: Measure, C = ()> {
    data: Vec<Node<L, C, L::Unit>>,
    index: TreeIndex,
}

#[derive(Debug)]
struct TreeIndex {
    parents: Vec<usize>,
    current_parent: usize,
}

impl TreeIndex {
    fn new() -> Self {
        TreeIndex {
            parents: Vec::new(),
            current_parent: ROOT,
        }
    }

    fn iter(&self) -> impl Iterator<Item = usize> {
        0..self.parents.len()
    }

    fn iter_siblings_after(&self, index: usize) -> impl Iterator<Item = usize> {
        let start = index + 1;
        let parent_index = self.parents[index];

        self.parents[start..]
            .iter()
            .take_while(move |&&parent| parent >= parent_index)
            .enumerate()
            .filter(move |&(_i, &parent)| parent == parent_index)
            .map(move |(i, _depth)| start + i)
    }

    fn iter_children(&self, index: usize) -> impl Iterator<Item = usize> {
        let start = index + 1;

        self.parents[start..]
            .iter()
            .take_while(move |&&parent| parent >= index)
            .enumerate()
            .filter(move |&(_i, &parent)| parent == index)
            .map(move |(i, _depth)| start + i)
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum Pass {
    Fit,
    Fill,
}

impl<L: Measure, C> Default for LayoutTree<L, C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<L: Measure, C> LayoutTree<L, C> {
    pub fn new() -> Self {
        let mut tree = LayoutTree {
            data: Vec::new(),
            index: TreeIndex::new(),
        };

        tree.push_root();
        tree
    }

    fn push_root(&mut self) {
        self.data.push(Node {
            data: NodeData::Container(None),
            dir: Direction::Vertical,
            opts: Opts::new(),
            size: Vec2::zero(),
            fitted: Vec2::zero(),
            pos: None,
        });

        self.index.parents.push(NO_PARENT);
        self.index.current_parent = ROOT;
    }

    fn add(&mut self, data: Node<L, C, L::Unit>) {
        self.data.push(data);
        self.index.parents.push(self.index.current_parent);
    }

    fn add_with_children<F: FnOnce(&mut Self)>(&mut self, data: Node<L, C, L::Unit>, insert_fn: F) {
        self.add(data);
        let our_parent = self.index.current_parent;
        self.index.current_parent = self.index.parents.len() - 1;

        insert_fn(self);

        self.index.current_parent = our_parent;
    }

    /// A container whose children flow left to right.
    pub fn row<F: FnOnce(&mut Self)>(&mut self, opts: Opts<L::Unit>, layout_fn: F) {
        self.container(Direction::Horizontal, None, opts, layout_fn);
    }

    /// A [`LayoutTree::row`] carrying `data`, for annotating the area its
    /// children end up occupying.
    pub fn row_with<F: FnOnce(&mut Self)>(&mut self, data: C, opts: Opts<L::Unit>, layout_fn: F) {
        self.container(Direction::Horizontal, Some(data), opts, layout_fn);
    }

    /// A container whose children flow top to bottom.
    pub fn col<F: FnOnce(&mut Self)>(&mut self, opts: Opts<L::Unit>, layout_fn: F) {
        self.container(Direction::Vertical, None, opts, layout_fn);
    }

    /// A [`LayoutTree::col`] carrying `data`, for annotating the area its
    /// children end up occupying.
    #[allow(dead_code)]
    pub fn col_with<F: FnOnce(&mut Self)>(&mut self, data: C, opts: Opts<L::Unit>, layout_fn: F) {
        self.container(Direction::Vertical, Some(data), opts, layout_fn);
    }

    fn container<F: FnOnce(&mut Self)>(
        &mut self,
        dir: Direction,
        data: Option<C>,
        opts: Opts<L::Unit>,
        layout_fn: F,
    ) {
        self.add_with_children(
            node::Node {
                data: NodeData::Container(data),
                dir,
                opts,
                size: Vec2::zero(),
                fitted: Vec2::zero(),
                pos: None,
            },
            layout_fn,
        );
    }

    pub fn leaf(&mut self, data: L) {
        let size = Vec2::from(data.measure());

        self.add(node::Node {
            data: NodeData::Leaf(data),
            dir: Direction::Horizontal,
            opts: opts(),
            size,
            fitted: size,
            pos: None,
        });
    }

    /// Places every node within `avail_size`, handing back the result to read
    /// positions off.
    pub fn compute(&mut self, avail_size: [L::Unit; 2]) -> Computed<'_, L, C> {
        let size = Vec2::from(avail_size);
        self.data[ROOT].pos = Some(Vec2::zero());

        for pass in [Pass::Fit, Pass::Fill] {
            if let Some(root_size) = self.compute_subtree(ROOT, Vec2::zero(), size, pass) {
                self.data[ROOT].size = root_size;
            }
        }

        Computed { tree: self }
    }

    fn compute_subtree(
        &mut self,
        parent: usize,
        outer_start: Vec2<L::Unit>,
        outer_avail_size: Vec2<L::Unit>,
        pass: Pass,
    ) -> Option<Vec2<L::Unit>> {
        let child = self.index.iter_children(parent).next()?;
        let parent_dir = self.data[parent].dir;
        let parent_opts = self.data[parent].opts;
        let wrap = self.data[parent].is_wrapping();
        let main_axis = parent_dir.axis();
        let cross_axis = main_axis.flip();
        let padding = Vec2::splat(parent_opts.pad) * main_axis;
        let avail_size = outer_avail_size
            .saturating_sub(padding)
            .saturating_sub(padding);

        let mut current_child = Some(child);
        let mut cursor = Vec2::zero();
        let mut size = Vec2::zero();

        let fill_avail = match pass {
            Pass::Fit => Vec2::zero(),
            Pass::Fill => avail_size.saturating_sub(self.data[parent].fitted),
        };

        let mut main_fill_iter = self.dist_main_fill(parent, fill_avail);
        let cross_fill = avail_size * cross_axis;

        while let Some(child) = current_child {
            let child_start = outer_start + padding + cursor;
            let is_main_fill = self.data[child].opts.is_main_fill(parent_dir);
            let is_cross_fill = self.data[child].opts.is_cross_fill(parent_dir);

            let mut child_size = match pass {
                Pass::Fit => {
                    let child_avail_size = avail_size.saturating_sub(cursor);

                    let fitted =
                        match self.compute_subtree(child, child_start, child_avail_size, pass) {
                            Some(fitted) => fitted,
                            None => self.data[child].size,
                        };

                    if is_main_fill {
                        // Zero-out the main axis that this child will later fill,
                        // this enables it to shrink.
                        fitted * cross_axis
                    } else {
                        fitted
                    }
                }
                Pass::Fill => {
                    let fill = {
                        let mut sum = Vec2::zero();
                        if is_main_fill {
                            sum += main_fill_iter.next().unwrap()
                        }
                        if is_cross_fill {
                            sum += cross_fill
                        }
                        sum
                    };

                    let fill_mask = self.data[child].opts.fill_mask();
                    let fixed_mask = Vec2::one().saturating_sub(fill_mask);

                    let remaining = avail_size.saturating_sub(cursor);
                    let granted = remaining.min(self.data[child].size + fill);
                    let child_avail_size = granted * fill_mask + remaining * fixed_mask;

                    match self.compute_subtree(child, child_start, child_avail_size, pass) {
                        Some(placed) => granted * fill_mask + placed * fixed_mask,
                        None => self.data[child].size,
                    }
                }
            };

            let mut child_pos = None;

            if (cursor + child_size).fits(avail_size) {
                child_pos = Some(outer_start + padding + cursor);
            } else {
                // Child doesn't fit where cursor currently is
                let next_line = size * cross_axis;

                if wrap && (next_line + child_size).fits(avail_size) {
                    // Fits completely on next line
                    cursor = next_line;
                    child_pos = Some(outer_start + padding + cursor);
                } else if (cursor + Vec2::one()).fits(avail_size) {
                    // Can't wrap, but we can fit at least one cell where the cursor currently is
                    child_pos = Some(outer_start + padding + cursor);
                    child_size = child_size.min(avail_size.saturating_sub(cursor));
                }
            }

            if child_pos.is_some() {
                size = size.max(cursor + child_size);
                cursor += main_axis * (Vec2::splat(parent_opts.gap) + child_size);
            }

            self.data[child].pos = child_pos;
            self.data[child].size = child_size;

            current_child = self.index.iter_siblings_after(child).next();
        }

        size += padding + padding;

        if pass == Pass::Fit {
            self.data[parent].fitted = size;
        }

        Some(size)
    }

    fn dist_main_fill(
        &self,
        parent: usize,
        size_to_distribute: Vec2<L::Unit>,
    ) -> impl Iterator<Item = Vec2<L::Unit>> + use<L, C> {
        let axis = self.data[parent].dir.axis();
        let along_axis = size_to_distribute * axis;
        let count = self
            .index
            .iter_children(parent)
            .filter(|&child| self.data[child].opts.is_main_fill(self.data[parent].dir))
            .count();

        let divisor = Vec2::splat((0..count).fold(L::Unit::ZERO, |sum, _| sum + L::Unit::ONE));

        let (quot, rem) = if count > 0 {
            let quot = along_axis / divisor;
            let rem = along_axis - quot * divisor;
            (quot, rem)
        } else {
            (Vec2::zero(), Vec2::zero())
        };

        iter::once(quot + rem)
            .chain(iter::once(quot).cycle())
            .take(count)
    }
}

/// A laid-out [`LayoutTree`]
#[derive(Debug)]
pub struct Computed<'a, L: Measure, C> {
    tree: &'a LayoutTree<L, C>,
}

impl<L: Measure, C> Computed<'_, L, C> {
    pub fn iter(&self) -> impl Iterator<Item = LayoutItem<Payload<'_, L, C>, L::Unit>> {
        self.tree.index.iter().filter_map(|index| {
            let node = &self.tree.data[index];

            let data = match &node.data {
                NodeData::Leaf(leaf) => Payload::Leaf(leaf),
                NodeData::Container(Some(container)) => Payload::Container(container),
                NodeData::Container(None) => return None,
            };

            if node.size.0 == L::Unit::ZERO || node.size.1 == L::Unit::ZERO {
                return None;
            }

            Some(LayoutItem {
                data,
                pos: node.pos?.into(),
                size: node.size.into(),
            })
        })
    }

    /// The extent the tree came to, which is at most what it was given.
    #[allow(dead_code)]
    pub fn size(&self) -> [L::Unit; 2] {
        self.tree.data[ROOT].size.into()
    }
}

/// The inserted data read back from the tree. Can be either a Leaf or a Container.
#[derive(Debug)]
pub enum Payload<'a, L, C> {
    Leaf(&'a L),
    Container(&'a C),
}

#[derive(Debug)]
pub struct LayoutItem<T, U> {
    pub data: T,
    pub pos: [U; 2],
    pub size: [U; 2],
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use unicode_width::UnicodeWidthStr;

    use super::*;

    /// The tree under test: `&str` for both leaves and container annotations.
    type TestTree = LayoutTree<&'static str, &'static str>;

    /// Test trees use one type for both payloads, so assertions can read either.
    fn payload(data: Payload<'_, &'static str, &'static str>) -> &'static str {
        match data {
            Payload::Leaf(text) | Payload::Container(text) => text,
        }
    }

    impl Measure for &str {
        type Unit = u16;

        fn measure(&self) -> [u16; 2] {
            [UnicodeWidthStr::width(*self) as u16, 1]
        }
    }

    #[derive(Debug, Clone)]
    struct Fractional(f32, f32);

    impl Measure for Fractional {
        type Unit = f32;

        fn measure(&self) -> [f32; 2] {
            [self.0, self.1]
        }
    }

    #[test]
    fn float_units_are_placed_at_fractional_offsets() {
        let mut layout: LayoutTree<Fractional> = LayoutTree::new();

        layout.row(opts(), |layout| {
            layout.leaf(Fractional(2.5, 1.0));
            layout.leaf(Fractional(2.5, 1.0));
        });

        let computed = layout.compute([5.0, 1.0]);
        let mut iter = computed.iter().map(|item| (item.pos, item.size));
        assert_eq!(Some(([0.0, 0.0], [2.5, 1.0])), iter.next());
        assert_eq!(Some(([2.5, 0.0], [2.5, 1.0])), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn float_fill_distributes_the_whole_extent() {
        let mut layout: LayoutTree<Fractional> = LayoutTree::new();

        for _ in 0..3 {
            layout.row(opts().fill_y(), |layout| {
                layout.leaf(Fractional(1.0, 1.0));
            });
        }

        // 10 does not divide evenly by 3, which is where a `%`-derived
        // remainder would hand the first filler space already shared out.
        let shares = layout.dist_main_fill(ROOT, Vec2(0.0, 10.0)).collect_vec();
        assert_eq!(3, shares.len());

        let total: f32 = shares.iter().map(|share| share.1).sum();
        assert!((total - 10.0).abs() < 1e-4, "distributed {total} of 10.0");
    }

    /// Render the layout to a string for testing purposes.
    /// Note: ASCII only — does not support Unicode beyond single-byte chars.
    fn render_to_string(computed: Computed<'_, &'static str, &'static str>) -> String {
        let [width, height] = computed.size();
        let (width, height) = (width as usize, height as usize);

        let mut grid = vec![' '; height * width];

        for LayoutItem { data, pos, size } in computed.iter() {
            let x0 = pos[0] as usize;
            let y0 = pos[1] as usize;
            let item_width = size[0] as usize;

            for (i, c) in payload(data).chars().take(item_width).enumerate() {
                grid[y0 * width + (x0 + i)] = c;
            }
        }

        grid.chunks(width)
            .map(|row| row.iter().collect::<String>().trim_end().to_string())
            .join("\n")
    }

    #[test]
    fn test_iter_distribute_size_no_flex() {
        let mut layout = TestTree::new();
        // Neither should grow
        layout.leaf("Hello");
        layout.leaf("Hello");

        let mut iter = layout.dist_main_fill(ROOT, Vec2(10, 3));
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_iter_distribute_size_one_flex() {
        let mut layout = TestTree::new();
        layout.row(opts(), |layout| {
            layout.leaf("One");
        });
        // This should shrink to 0 vertically and then grow to 3
        layout.row(opts().fill_y(), |layout| {
            layout.leaf("Two");
        });

        let mut iter = layout.dist_main_fill(ROOT, Vec2(10, 3));
        assert_eq!(Some(Vec2(0, 3)), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_iter_distribute_size_all_flex() {
        let mut layout = TestTree::new();
        // Both should grow, favoring the first
        layout.row(opts().fill_y(), |layout| {
            layout.leaf("One");
        });
        layout.row(opts().fill_y(), |layout| {
            layout.leaf("Two");
        });

        let mut iter = layout.dist_main_fill(ROOT, Vec2(10, 5));
        assert_eq!(Some(Vec2(0, 3)), iter.next());
        assert_eq!(Some(Vec2(0, 2)), iter.next());
    }

    #[test]
    fn single_text() {
        let mut layout = TestTree::new();

        layout.col(opts(), |layout| {
            layout.leaf("Hello");
            layout.leaf("lol");
        });

        insta::assert_snapshot!(render_to_string(layout.compute([5, 2])));
    }

    #[test]
    fn fill_node_wraps_within_the_extent_it_gets() {
        let mut layout = TestTree::new();

        layout.row(opts(), |layout| {
            // Measures as one row at the full width, but only gets 6 columns.
            layout.row(opts().fill_x(), |layout| {
                layout.leaf("hello");
                layout.leaf(" ");
                layout.leaf("world");
            });
            layout.leaf("author");
        });

        insta::assert_snapshot!(render_to_string(layout.compute([12, 2])));
    }

    #[test]
    fn horizontal_layout() {
        let mut layout = TestTree::new();

        layout.row(opts(), |layout| {
            layout.leaf("A");
            layout.leaf("BB");
            layout.leaf("CCC");
        });

        insta::assert_snapshot!(render_to_string(layout.compute([6, 1])));
    }

    #[test]
    fn vertical_layout() {
        let mut layout = TestTree::new();

        layout.col(opts(), |layout| {
            layout.leaf("First");
            layout.leaf("Second");
            layout.leaf("Third");
        });

        insta::assert_snapshot!(render_to_string(layout.compute([6, 3])));
    }

    #[test]
    fn nested_layouts() {
        let mut layout = TestTree::new();

        layout.row(opts(), |layout| {
            // 0
            layout.col(opts(), |layout| {
                // 1
                layout.leaf("A"); // 2
                layout.leaf("B"); // 3
            });
            layout.col(opts(), |layout| {
                // 4
                layout.leaf("C"); // 5
                layout.leaf("D"); // 6
            });
        });

        insta::assert_snapshot!(render_to_string(layout.compute([2, 2])));
    }

    #[test]
    fn every_top_level_node_is_laid_out() {
        let mut layout = TestTree::new();

        layout.leaf("first");
        layout.leaf("second");

        let computed = layout.compute([20, 2]);
        let mut iter = computed.iter().map(|e| (payload(e.data), e.pos));
        assert_eq!(Some(("first", [0, 0])), iter.next());
        assert_eq!(Some(("second", [0, 1])), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn top_level_containers_stack_downwards() {
        let mut layout = TestTree::new();

        layout.row(opts(), |layout| {
            layout.leaf("aaa");
        });
        layout.row(opts(), |layout| {
            layout.leaf("bbb");
        });

        insta::assert_snapshot!(render_to_string(layout.compute([20, 2])));
    }

    #[test]
    fn empty_layout_computes_without_panicking() {
        let mut layout = TestTree::new();

        assert_eq!(0, layout.compute([20, 2]).iter().count());
    }

    #[test]
    fn out_of_bounds_horizontal() {
        let mut layout = TestTree::new();

        layout.col(opts(), |layout| {
            layout.row(opts(), |layout| {
                layout.leaf("12345");
                layout.leaf("The very start of this will be visible (a T)");
            });
            layout.row(opts(), |layout| {
                layout.leaf("123456");
                layout.leaf("This is completely outside of the layout and ignored");
            });
        });

        insta::assert_snapshot!(render_to_string(layout.compute([6, 4])));
    }

    #[test]
    fn test_horizontal_wrap() {
        let mut layout = TestTree::new();

        layout.row(opts(), |layout| {
            layout.leaf("AAA");
            layout.leaf("BBB");
            layout.leaf("CCC");
        });

        let result = render_to_string(layout.compute([6, 2]));
        println!("Result:\n{}", result);
        // Should wrap: "AAABBB" on first line, "CCC" on second line
        assert_eq!(result, "AAABBB\nCCC");
    }

    #[test]
    fn a_wrapped_row_does_not_reserve_space_it_never_uses() {
        let mut layout = TestTree::new();

        layout.col(opts().fill_y(), |layout| {
            layout.leaf("top");
        });
        layout.col(opts(), |layout| {
            layout.row(opts(), |layout| {
                layout.col(opts(), |layout| {
                    layout.row(opts(), |layout| {
                        layout.leaf("aaaa");
                        layout.leaf("bbbb");
                    });
                });
                layout.leaf("cc");
            });
        });

        let computed = layout.compute([6, 6]);
        let mut iter = computed.iter().map(|e| (payload(e.data), e.pos));

        // The bottom block is two rows, so it sits flush at rows 4 and 5.
        assert_eq!(Some(("top", [0, 0])), iter.next());
        assert_eq!(Some(("aaaa", [0, 4])), iter.next());
        assert_eq!(Some(("bbbb", [0, 5])), iter.next());
        assert_eq!(Some(("cc", [4, 4])), iter.next());
        assert_eq!(None, iter.next());
    }

    /// A filling node has its main axis zeroed so that it can shrink, so its
    /// own size is not what is left over for the fill pass to share out.
    /// Sharing out the whole extent instead leaves nothing for a sibling that
    /// doesn't fill.
    #[test]
    fn a_fill_inside_a_fill_leaves_room_for_its_siblings() {
        let mut layout = TestTree::new();

        layout.col(opts().fill_y(), |layout| {
            layout.col(opts().fill_y(), |layout| {
                layout.leaf("a");
            });
            layout.col(opts(), |layout| {
                layout.leaf("b");
            });
        });

        let computed = layout.compute([3, 5]);
        let mut iter = computed.iter().map(|e| (payload(e.data), e.pos));

        // The inner fill takes rows 0..4, leaving the last row for "b".
        assert_eq!(Some(("a", [0, 0])), iter.next());
        assert_eq!(Some(("b", [0, 4])), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn a_column_clips_rather_than_starting_a_new_column() {
        let mut layout = TestTree::new();

        layout.col(opts(), |layout| {
            layout.leaf("aa");
            layout.leaf("bb");
            layout.leaf("cc");
        });

        let computed = layout.compute([4, 2]);
        let mut iter = computed.iter().map(|e| (payload(e.data), e.pos));
        assert_eq!(Some(("aa", [0, 0])), iter.next());
        assert_eq!(Some(("bb", [0, 1])), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn a_column_wraps_into_columns_when_asked_to() {
        let mut layout = TestTree::new();

        layout.col(opts().wrap(), |layout| {
            layout.leaf("aa");
            layout.leaf("bb");
            layout.leaf("cc");
        });

        let computed = layout.compute([4, 2]);
        let mut iter = computed.iter().map(|e| (payload(e.data), e.pos));
        assert_eq!(Some(("aa", [0, 0])), iter.next());
        assert_eq!(Some(("bb", [0, 1])), iter.next());
        assert_eq!(Some(("cc", [2, 0])), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_wrap_before_truncate() {
        let mut layout = TestTree::new();

        layout.row(opts(), |layout| {
            layout.leaf("AAAA");
            layout.leaf("BBBB");
        });

        let result = render_to_string(layout.compute([6, 2]));
        println!("Result:\n{}", result);
        // With 6 chars width and 2 rows:
        // "AAAA" fits (4 chars), then "BBBB" doesn't fit in remaining 2 chars
        // Should wrap "BBBB" to next line rather than truncating to "BB"
        assert_eq!(result, "AAAA\nBBBB");
    }

    #[test]
    fn nested_grow_wrap() {
        let mut layout = TestTree::new();

        layout.col(opts(), |layout| {
            layout.col(opts().fill_y(), |layout| {
                layout.row(opts(), |layout| {
                    layout.leaf("word1");
                    layout.leaf("word2");
                    layout.leaf("word3");
                });
            });
        });

        let computed = layout.compute([10, 2]);
        let mut iter = computed.iter().map(|e| (payload(e.data), e.pos, e.size));
        assert_eq!(Some(("word1", [0, 0], [5, 1])), iter.next());
        assert_eq!(Some(("word2", [5, 0], [5, 1])), iter.next());
        assert_eq!(Some(("word3", [0, 1], [5, 1])), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_no_trailing_newline() {
        let mut layout = TestTree::new();

        layout.col(opts(), |layout| {
            layout.leaf("Line 1");
            layout.leaf("Line 2");
        });

        let result = render_to_string(layout.compute([10, 2]));
        println!("Result bytes: {:?}", result.as_bytes());
        println!("Result repr: {:?}", result);
        // Should not have trailing newline
        assert!(!result.ends_with('\n'), "Should not have trailing newline");
        assert_eq!(result, "Line 1\nLine 2");
    }

    #[test]
    fn out_of_bounds_vertical() {
        let mut layout = TestTree::new();

        layout.row(opts(), |layout| {
            layout.col(opts(), |layout| {
                layout.leaf("1");
                layout.leaf("2");
            });
            layout.col(opts(), |layout| {
                layout.leaf("1");
                layout.leaf("2");
                layout.leaf("X");
            });
        });

        insta::assert_snapshot!(render_to_string(layout.compute([2, 2])));
    }

    #[test]
    fn unicode_text_width() {
        let mut layout = TestTree::new();

        layout.row(opts(), |layout| {
            layout.leaf("café");
            layout.leaf("naïve");
        });

        let computed = layout.compute([10, 1]);
        let items: Vec<_> = computed.iter().collect();
        assert_eq!(items[0].size, [4, 1]); // café has 4 graphemes
    }

    #[test]
    fn horizontal_gap() {
        let mut layout = TestTree::new();

        layout.row(opts().gap(2), |layout| {
            layout.leaf("one");
            layout.leaf("two");
        });

        insta::assert_snapshot!(render_to_string(layout.compute([8, 1])));
    }

    #[test]
    fn vertical_gap() {
        let mut layout = TestTree::new();

        layout.col(opts().gap(1), |layout| {
            layout.leaf("one");
            layout.leaf("two");
        });

        insta::assert_snapshot!(render_to_string(layout.compute([3, 3])));
    }

    #[test]
    fn grow() {
        let mut layout = TestTree::new();

        layout.col(opts(), |layout| {
            layout.col(opts().fill_y(), |layout| {
                layout.leaf("flex");
            });
            layout.leaf("actual");
        });

        insta::assert_snapshot!(render_to_string(layout.compute([8, 3])));
    }

    #[test]
    fn nested_grow_preserves_cross_axis() {
        let mut layout = TestTree::new();

        layout.col_with("root", opts(), |layout| {
            layout.row_with("grow", opts().fill_xy(), |layout| {
                layout.leaf("hello");
            });
        });

        let computed = layout.compute([20, 10]);

        let mut iter = computed.iter().map(|e| (payload(e.data), e.pos, e.size));

        assert_eq!(Some(("root", [0, 0], [20, 10])), iter.next());
        assert_eq!(Some(("grow", [0, 0], [20, 10])), iter.next());
        assert_eq!(Some(("hello", [0, 0], [5, 1])), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn overflow() {
        let mut layout = TestTree::new();

        layout.col(opts(), |layout| {
            layout.leaf("one");
            layout.leaf("twoooo");
            layout.leaf("three");
        });

        insta::assert_snapshot!(render_to_string(layout.compute([6, 1])));
    }

    #[test]
    fn shrink() {
        let mut layout = TestTree::new();

        layout.col(opts(), |layout| {
            layout.col(opts().fill_y(), |layout| {
                layout.leaf("flex 1");
                layout.leaf("flex 2");
            });
            layout.leaf("actual");
        });

        insta::assert_snapshot!(render_to_string(layout.compute([6, 2])));
    }

    #[test]
    fn shrinks_nested() {
        let mut layout = TestTree::new();

        layout.col(opts(), |layout| {
            layout.col(opts().fill_y(), |layout| {
                layout.col(opts(), |layout| {
                    layout.leaf("This should not be visible'");
                });
            });

            layout.col(opts(), |layout| {
                layout.leaf("WEEEEE");
            });
        });

        let computed = layout.compute([40, 1]);
        let mut iter = computed.iter().map(|e| (payload(e.data), e.pos, e.size));
        assert_eq!(Some(("WEEEEE", [0, 0], [6, 1])), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn nested_horizontal_fill_does_not_hide_siblings() {
        let mut layout = TestTree::new();

        layout.col(opts(), |layout| {
            // 0,0 -> 0,5
            layout.col(opts().fill_y(), |layout| {
                // 0,1 -> 0,1
                layout.row(opts().fill_x(), |layout| {
                    // 0,1 -> 0,1
                    layout.row(opts().fill_x(), |layout| {
                        layout.leaf("visible");
                    });
                });
            });
        });

        let computed = layout.compute([20, 5]);
        let mut iter = computed.iter().map(|e| (payload(e.data), e.pos, e.size));
        assert_eq!(Some(("visible", [0, 0], [7, 1])), iter.next());
        assert_eq!(None, iter.next());

        // insta::assert_snapshot!(render_to_string(layout));
    }

    #[test]
    fn gitu_mockup() {
        let mut layout = TestTree::new();

        layout.col(opts(), |layout| {
            layout.col(opts().fill_xy(), |layout| {
                // Screen
                layout.col(opts().fill_x(), |layout| {
                    layout.row_with("", opts().fill_x(), |layout| {
                        layout.leaf("On branch master");
                    });
                    layout.leaf("Your branch is up to date with 'origin/master'");
                });

                layout.leaf("");
                layout.leaf("Recent commits");
                layout.leaf("9eb6a63 refactor/ui origin/refactor/ui fix more rendering issues");
                layout.leaf("b5fffd4 fix styling issues in Screen");
                layout.leaf("61e6c1b refactor: extract type of LayoutTree");
                layout.leaf("df3bcb5 get rid of frequent clone() in LayoutTree");
                layout.leaf("9864859 refactor(ui): less allocs");
                layout.leaf("aa2811e refactor: new LayoutTree module to improve on ui headaches");
                layout.leaf(
                    "5374ab3 master origin/master test: add file:// in clone_and_commit fn as well",
                );
                layout.leaf("7a66235 test: get rid of setup_init, and try fix test-repo assertion");
                layout.leaf(
                    "75463c8 test/fix-ci test: forgot to create testfiles/ when running tests",
                );
            });

            layout.col(opts(), |layout| {
                // Menu
                layout.leaf("───────────────────────────────────────────────────────────────");

                layout.row(opts().gap(2), |layout| {
                    layout.col(opts(), |layout| {
                        layout.leaf("Help");
                        layout.leaf("Y Show Refs");
                        layout.leaf("<tab> Toggle section");
                        layout.leaf("k/<up> Up ");
                        layout.leaf("j/<down> Down");
                        layout.leaf("<ctrl+k>/<ctrl+up> Up line");
                        layout.leaf("<ctrl+j>/<ctrl+down> Down line");
                        layout.leaf("<alt+k>/<alt+up> Prev section");
                        layout.leaf("<alt+j>/<alt+down> Next section");
                        layout.leaf("<alt+h>/<alt+left> Parent section");
                        layout.leaf("<ctrl+u> Half page up");
                        layout.leaf("<ctrl+d> Half page down");
                        layout.leaf("g+r Refresh");
                        layout.leaf("q/<esc> Quit/Close");
                    });
                    layout.col(opts(), |layout| {
                        layout.leaf("Submenu");
                        layout.leaf("b Branch");
                        layout.leaf("c Commit");
                        layout.leaf("f Fetch");
                        layout.leaf("h/? Help");
                        layout.leaf("l Log");
                        layout.leaf("M Remote");
                        layout.leaf("F Pull");
                        layout.leaf("P Push");
                        layout.leaf("r Rebase");
                        layout.leaf("X Reset");
                        layout.leaf("V Revert");
                        layout.leaf("z Stash");
                        layout.leaf("");
                    });
                    layout.col(opts(), |layout| {
                        layout.leaf("@@ -271,7 +271,7");
                        layout.leaf("s Stage");
                        layout.leaf("u Unstage");
                        layout.leaf("<enter> Show");
                        layout.leaf("K Discard");
                        layout.leaf("");
                        layout.leaf("");
                        layout.leaf("");
                        layout.leaf("");
                        layout.leaf("");
                        layout.leaf("");
                        layout.leaf("");
                        layout.leaf("");
                        layout.leaf("");
                    });
                });
            });
        });

        insta::assert_snapshot!(render_to_string(layout.compute([80, 25])));
    }
}

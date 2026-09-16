//! Logic and types for commit tree views (`git log --graph` layout).
//!
//! [`tree_roots`] walks the commits reachable from one or more root commits
//! and returns one [`TreeRow`] per output row. Each row carries the graph
//! prefix (e.g. `"*   "` or `"|/"`) and, for commit rows, the commit's
//! details. The graph layout itself is computed by [`Graph::rows`], a port
//! of git's own `graph.c` state machine (sans colors, truncation and visual
//! roots).

use crate::{Res, error::Error, item_data::Ref, items::short_age};
use git2::{Oid, Repository};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(crate) struct TreeRow {
    /// Graph prefix for the row, e.g. `"*   "` or `"|\\  "`.
    pub(crate) graph: String,
    /// Does this row correspond to a commit?
    pub(crate) commit: Option<TreeCommit>,
}

/// The fields of [`crate::item_data::ItemData::Commit`], so a tree row can be
/// turned into an `Item` without extra lookups.
#[derive(Debug, Clone)]
pub(crate) struct TreeCommit {
    pub(crate) oid: String,
    pub(crate) short_id: String,
    pub(crate) associated_references: Vec<Ref>,
    pub(crate) summary: String,
    pub(crate) author: String,
    pub(crate) age: String,
}

/// The oids of every local branch tip.
pub(crate) fn local_branch_roots(repo: &Repository) -> Vec<Oid> {
    branch_roots(repo, git2::BranchType::Local).collect()
}

/// The oids of every branch tip (local and remote).
pub(crate) fn all_branch_roots(repo: &Repository) -> Vec<Oid> {
    branch_roots(repo, git2::BranchType::Local)
        .chain(branch_roots(repo, git2::BranchType::Remote))
        .collect()
}

/// The oids of every reference that points at a commit (branches, tags,
/// stashes, ...).
pub(crate) fn all_ref_roots(repo: &Repository) -> Vec<Oid> {
    let mut ids = Vec::new();
    if let Ok(references) = repo.references() {
        for reference in references.flatten() {
            if Ref::from_reference(&reference).is_none() {
                continue;
            }
            if let Ok(commit) = reference.peel_to_commit() {
                ids.push(commit.id());
            }
        }
    }
    ids
}

/// The oids of the tips of every branch of the given kind.
fn branch_roots<'repo>(
    repo: &'repo Repository,
    branch_type: git2::BranchType,
) -> impl Iterator<Item = Oid> + 'repo {
    repo.branches(Some(branch_type))
        .into_iter()
        .flatten()
        .filter_map(|entry| match entry {
            Ok((branch, _branch_type)) => branch.get().peel_to_commit().ok(),
            Err(_) => None,
        })
        .map(|commit| commit.id())
}

/// Collect the commits reachable from any of `roots`, up to `limit` of them,
/// and lay them out as a tree. An oid reachable from more than one root is
/// included only once.
pub(crate) fn tree_roots(repo: &Repository, limit: usize, roots: &[Oid]) -> Res<Vec<TreeRow>> {
    if roots.is_empty() {
        return Ok(vec![]);
    }

    let mut revwalk = repo.revwalk().map_err(Error::ReadLog)?;
    // Walk in the same (date-ordered) sequence `git log` uses, so multiple
    // roots (e.g. the local-branches view) come out in log order rather
    // than in an order that depends on the roots' push order. The
    // topological constraint keeps parents coming after their children
    // even when commits share a committer date; without it the heap order
    // of equal-dated commits can put a parent before its child, and the
    // graph then draws that line as if it were a new branch.
    revwalk
        .set_sorting(git2::Sort::TIME | git2::Sort::TOPOLOGICAL)
        .map_err(Error::ReadLog)?;
    for root in roots {
        revwalk.push(*root).map_err(Error::ReadLog)?;
    }

    let oids: Vec<Oid> = revwalk
        .map(|oid| oid.map_err(Error::ReadLog))
        .take(limit)
        .collect::<Res<Vec<_>>>()?;

    let references = references(repo)?;

    // The commits, in display order, each indexed by oid for parent lookup.
    let mut index_by_oid: HashMap<Oid, usize> = HashMap::with_capacity(oids.len());
    let mut nodes = Vec::with_capacity(oids.len());
    for (i, oid) in oids.iter().enumerate() {
        index_by_oid.insert(*oid, i);
        let commit = repo.find_commit(*oid).map_err(Error::ReadLog)?;
        let short_id = commit.as_object().short_id().map_err(Error::ReadOid)?;
        nodes.push(Node {
            commit: TreeCommit {
                oid: oid.to_string(),
                short_id: String::from_utf8_lossy(&short_id).to_string(),
                associated_references: references.get(oid).cloned().unwrap_or_default(),
                summary: commit.summary().unwrap_or_default().to_string(),
                author: commit.author().name().unwrap_or_default().to_string(),
                age: short_age(commit.author().when()),
            },
            parents: commit.parent_ids().collect(),
        });
    }

    // The interesting parents of each commit: those that made it into the
    // walk (e.g. ones cut off by the limit are not interesting, mirroring
    // git's simplified history).
    let parents: Vec<Vec<usize>> = nodes
        .iter()
        .map(|node| {
            node.parents
                .iter()
                .filter_map(|parent| index_by_oid.get(parent).copied())
                .collect()
        })
        .collect();

    // Lay the rows out, then attach the commit data to the rows that carry
    // a commit.
    let rows = Graph::rows(parents)
        .into_iter()
        .map(|(graph, commit_idx)| TreeRow {
            graph,
            commit: commit_idx.map(|i| nodes[i].commit.clone()),
        })
        .collect();

    Ok(rows)
}

#[derive(Debug)]
struct Node {
    commit: TreeCommit,
    parents: Vec<Oid>,
}

/// Every reference that points at a commit, grouped by that commit's oid.
fn references(repo: &Repository) -> Res<HashMap<Oid, Vec<Ref>>> {
    let mut references: HashMap<Oid, Vec<Ref>> = HashMap::new();
    for reference in repo.references().map_err(Error::ReadLog)?.flatten() {
        if let (Ok(target), Some(ref_kind)) =
            (reference.peel_to_commit(), Ref::from_reference(&reference))
        {
            references.entry(target.id()).or_default().push(ref_kind);
        }
    }
    Ok(references)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Padding,
    PreCommit,
    Commit,
    PostMerge,
    Collapsing,
}

const MERGE_CHARS: [char; 3] = ['/', '|', '\\'];

/// The state machine that lays out graph lines the way `git log --graph`
/// does; drive it in one go with [`Graph::rows`].
///
/// A port of git's `graph.c` (the `columns` / `new_columns` / `mapping`
/// machinery). Column and mapping entries refer to the index of the
/// corresponding commit in the display-order list.
pub(crate) struct Graph {
    /// Commit indices of the lines as they stood before the current commit.
    columns: Vec<usize>,
    /// Commit indices of the lines as they stand after the current commit.
    new_columns: Vec<usize>,
    /// Maps each display slot (2 slots per visual column) to the index of the
    /// `new_columns` entry it eventually collapses onto; -1 for empty slots.
    mapping: Vec<i32>,
    mapping_size: usize,
    old_mapping: Vec<i32>,
    width: usize,
    edges_added: i32,
    prev_edges_added: i32,
    merge_layout: i32,
    /// The (interesting) parents of each commit, as indices into the
    /// display-order list.
    parents: Vec<Vec<usize>>,
    /// Index of the commit currently being displayed.
    commit: usize,
    /// Position of the current commit within `columns`.
    commit_index: usize,
    prev_commit_index: usize,
    state: State,
    prev_state: State,
    expansion_row: usize,
}

impl Graph {
    /// Lays out the graph rows for the given topology. `parents[i]` are the
    /// indices (in display order) of commit `i`'s (interesting) parents. Each
    /// result row is the row's graph prefix paired with the index of the commit
    /// it carries, if any.
    pub(crate) fn rows(parents: Vec<Vec<usize>>) -> Vec<(String, Option<usize>)> {
        let num_commits = parents.len();
        let mut graph = Graph::new(parents);
        let mut rows = Vec::new();
        for i in 0..num_commits {
            graph.update(i);
            let mut shown_commit = false;
            loop {
                let (prefix, is_commit_line) = graph.next_line();
                rows.push((prefix, is_commit_line.then_some(i)));
                shown_commit |= is_commit_line;

                // The commit's output is complete once its commit line has been
                // printed and the state machine settled back to padding.
                if shown_commit && graph.state == State::Padding {
                    break;
                }
            }
        }
        rows
    }

    fn new(parents: Vec<Vec<usize>>) -> Self {
        Self {
            columns: vec![],
            new_columns: vec![],
            mapping: vec![],
            mapping_size: 0,
            old_mapping: vec![],
            width: 0,
            edges_added: 0,
            prev_edges_added: 0,
            merge_layout: 0,
            parents,
            commit: 0,
            commit_index: 0,
            prev_commit_index: 0,
            state: State::Padding,
            prev_state: State::Padding,
            expansion_row: 0,
        }
    }

    /// Number of (interesting) parents of the current commit.
    fn num_parents(&self) -> usize {
        self.parents[self.commit].len()
    }

    /// Starts outputting the lines for the commit at `commit`.
    fn update(&mut self, commit: usize) {
        self.commit = commit;

        // Store the old commit_index in prev_commit_index; update_columns()
        // will update commit_index for this commit.
        self.prev_commit_index = self.commit_index;

        self.update_columns();

        self.expansion_row = 0;

        // With the normal driver the previous commit always runs down to the
        // padding state, so we land straight on the commit line (or on the
        // pre-commit expansion rows of a wide merge).
        if self.needs_pre_commit_line() {
            self.state = State::PreCommit;
        } else {
            self.state = State::Commit;
        }
    }

    /// Outputs the next line. The second result is true only for the line
    /// that carries the commit itself.
    fn next_line(&mut self) -> (String, bool) {
        let mut line = String::new();
        let is_commit_line = match self.state {
            State::Padding => {
                self.output_padding_line(&mut line);
                false
            }
            State::PreCommit => {
                self.output_pre_commit_line(&mut line);
                false
            }
            State::Commit => {
                self.output_commit_line(&mut line);
                true
            }
            State::PostMerge => {
                self.output_post_merge_line(&mut line);
                false
            }
            State::Collapsing => {
                self.output_collapsing_line(&mut line);
                false
            }
        };

        // Pad so that all lines for a commit have the same width and the
        // fields printed to the right of the graph stay aligned.
        if line.len() < self.width {
            line.push_str(&" ".repeat(self.width - line.len()));
        }

        (line, is_commit_line)
    }

    fn update_columns(&mut self) {
        // Swap columns with new_columns: columns now holds the state for the
        // current commit, and new_columns becomes the storage for the state
        // after this commit.
        std::mem::swap(&mut self.columns, &mut self.new_columns);
        let num_columns = self.columns.len();
        self.new_columns.clear();

        // At most num_columns + num_parents columns for the next commit.
        let max_new_columns = num_columns + self.num_parents();
        self.mapping = vec![-1; 2 * max_new_columns];
        self.mapping_size = 2 * max_new_columns;
        self.width = 0;
        self.prev_edges_added = self.edges_added;
        self.edges_added = 0;

        // Populate new_columns and mapping. Some of the parents of this
        // commit may already be in columns; in that case new_columns holds a
        // single entry for each such commit, and mapping records where each
        // line is supposed to end up after the collapsing is performed.
        let mut seen_this = false;
        for i in 0..=num_columns {
            let col_commit = if i == num_columns {
                if seen_this {
                    break;
                }
                self.commit
            } else {
                self.columns[i]
            };

            if col_commit == self.commit {
                seen_this = true;
                self.commit_index = i;
                self.merge_layout = -1;

                for parent in self.parents[self.commit].clone() {
                    self.insert_into_new_columns(parent, i as i32);
                }

                // The current commit always takes up at least 2 spaces.
                if self.num_parents() == 0 {
                    self.width += 2;
                }
            } else {
                self.insert_into_new_columns(col_commit, -1);
            }
        }

        // Shrink mapping to the minimum necessary.
        while self.mapping_size > 1 && self.mapping[self.mapping_size - 1] < 0 {
            self.mapping_size -= 1;
        }
    }

    /// Records a line in new_columns (adding it if absent) and extends
    /// mapping with its target display slot. `idx` is the position of the
    /// line in `columns`, or -1 for a line that is not in `columns`.
    fn insert_into_new_columns(&mut self, commit: usize, idx: i32) {
        let i = match self.new_columns.iter().position(|&c| c == commit) {
            Some(i) => i,
            None => {
                self.new_columns.push(commit);
                self.new_columns.len() - 1
            }
        };

        if self.num_parents() > 1 && idx >= 0 && self.merge_layout == -1 {
            // The first parent of a merge: choose a layout based on whether
            // the parent appears in a column to the left of the merge.
            let dist = idx - i as i32;
            let shift = if dist > 1 { 2 * dist - 3 } else { 1 };

            self.merge_layout = if dist > 0 { 0 } else { 1 };
            self.edges_added = self.num_parents() as i32 + self.merge_layout - 2;

            let mapping_idx = (self.width as i32 + (self.merge_layout - 1) * shift) as usize;
            self.width += 2 * self.merge_layout as usize;
            self.mapping[mapping_idx] = i as i32;
        } else if self.edges_added > 0 && i as i32 == self.mapping[self.width - 2] {
            // Some columns have already been added by a merge, but this
            // commit was found in the last existing column: make the two
            // edges join immediately.
            self.edges_added = -1;
            self.mapping[self.width - 2] = i as i32;
        } else {
            self.mapping[self.width] = i as i32;
            self.width += 2;
        }
    }

    fn needs_pre_commit_line(&self) -> bool {
        self.num_parents() >= 3 && (self.commit_index as i32) < (self.columns.len() as i32 - 1)
    }

    fn num_dashed_parents(&self) -> i32 {
        self.num_parents() as i32 + self.merge_layout - 3
    }

    fn num_expansion_rows(&self) -> i32 {
        self.num_dashed_parents() * 2
    }

    /// The mapping is up to date if each entry is at its target, or is 1
    /// greater than its target (in which case a '/' will be printed, so it
    /// will look correct on the next row).
    fn mapping_correct(&self) -> bool {
        (0..self.mapping_size).all(|i| {
            let target = self.mapping[i];
            target < 0 || target == i as i32 / 2
        })
    }

    fn update_state(&mut self, state: State) {
        self.prev_state = self.state;
        self.state = state;
    }

    fn output_padding_line(&self, line: &mut String) {
        // A padding row, that leaves all branch lines unchanged.
        for _ in &self.new_columns {
            line.push('|');
            line.push(' ');
        }
    }

    fn output_pre_commit_line(&mut self, line: &mut String) {
        // A row that increases the space around a commit with multiple
        // parents, to make room for it. Only called with 3 or more parents.
        let mut seen_this = false;
        for (i, &col_commit) in self.columns.iter().enumerate() {
            if col_commit == self.commit {
                seen_this = true;
                line.push('|');
                line.push_str(&" ".repeat(self.expansion_row));
            } else if seen_this && self.expansion_row == 0 {
                // If the previous commit was a merge and ended in the
                // post-merge state, continue to print its branch lines as
                // '\'; otherwise print them as '|'.
                if self.prev_state == State::PostMerge && self.prev_commit_index < i {
                    line.push('\\');
                } else {
                    line.push('|');
                }
            } else if seen_this && self.expansion_row > 0 {
                line.push('\\');
            } else {
                line.push('|');
            }
            line.push(' ');
        }

        self.expansion_row += 1;
        if self.expansion_row >= self.num_expansion_rows() as usize {
            self.update_state(State::Commit);
        }
    }

    fn output_commit_line(&mut self, line: &mut String) {
        let num_columns = self.columns.len();
        let mut seen_this = false;

        // Iterate up to and including num_columns, since the current commit
        // may not be in any of the existing columns (this happens when the
        // commit has no children that have already been processed).
        for i in 0..=num_columns {
            let col_commit = if i == num_columns {
                if seen_this {
                    break;
                }
                self.commit
            } else {
                self.columns[i]
            };

            if col_commit == self.commit {
                seen_this = true;
                line.push('*');

                if self.num_parents() > 2 {
                    self.draw_octopus_merge(line);
                }
            } else if seen_this && self.edges_added > 1 {
                line.push('\\');
            } else if seen_this && self.edges_added == 1 {
                // A right-skewed 2-way merge, or a left-skewed 3-way merge.
                // If the previous line was a post-merge line, the branch
                // line coming into this commit may have been '\'; keep
                // printing it as '\' so it looks nicer.
                if self.prev_state == State::PostMerge
                    && self.prev_edges_added > 0
                    && self.prev_commit_index < i
                {
                    line.push('\\');
                } else {
                    line.push('|');
                }
            } else if self.prev_state == State::Collapsing
                && self.old_mapping.get(2 * i + 1).copied().unwrap_or(-1) == i as i32
                && self.mapping.get(2 * i).copied().unwrap_or(-1) < i as i32
            {
                line.push('/');
            } else {
                line.push('|');
            }
            line.push(' ');
        }

        if self.num_parents() > 1 {
            self.update_state(State::PostMerge);
        } else if self.mapping_correct() {
            self.update_state(State::Padding);
        } else {
            self.update_state(State::Collapsing);
        }
    }

    /// Draws the horizontal dashes of an octopus merge.
    fn draw_octopus_merge(&self, line: &mut String) {
        let dashed_parents = self.num_dashed_parents();
        for i in 0..dashed_parents {
            line.push('-');
            line.push(if i == dashed_parents - 1 { '.' } else { '-' });
        }
    }

    fn output_post_merge_line(&mut self, line: &mut String) {
        let num_columns = self.columns.len();
        let first_parent = self.parents[self.commit].first().copied();
        let mut seen_this = false;
        let mut parent_col = None;

        for i in 0..=num_columns {
            let col_commit = if i == num_columns {
                if seen_this {
                    break;
                }
                self.commit
            } else {
                self.columns[i]
            };

            if col_commit == self.commit {
                // Find the columns for the parent commits in new_columns and
                // use those to format the edges.
                seen_this = true;
                let mut idx = self.merge_layout;
                for (j, _) in self.parents[self.commit].iter().enumerate() {
                    line.push(MERGE_CHARS[idx as usize]);

                    if idx == 2 {
                        if self.edges_added > 0 || j < self.num_parents() - 1 {
                            line.push(' ');
                        }
                    } else {
                        idx += 1;
                    }
                }
                if self.edges_added == 0 {
                    line.push(' ');
                }
            } else if seen_this {
                if self.edges_added > 0 {
                    line.push('\\');
                } else {
                    line.push('|');
                }
                line.push(' ');
            } else {
                line.push('|');
                if self.merge_layout != 0 || i as i32 != self.commit_index as i32 - 1 {
                    if parent_col.is_some() {
                        line.push('_');
                    } else {
                        line.push(' ');
                    }
                }
            }

            if Some(col_commit) == first_parent {
                parent_col = Some(i);
            }
        }

        if self.mapping_correct() {
            self.update_state(State::Padding);
        } else {
            self.update_state(State::Collapsing);
        }
    }

    fn output_collapsing_line(&mut self, line: &mut String) {
        // Swap the mapping and old_mapping arrays.
        std::mem::swap(&mut self.mapping, &mut self.old_mapping);

        // Clear out the mapping array.
        self.mapping = vec![-1; self.mapping_size];

        let mut used_horizontal = false;
        let mut horizontal_edge = -1;
        let mut horizontal_edge_target = -1;

        for i in 0..self.mapping_size {
            let target = self.old_mapping[i];
            if target < 0 {
                continue;
            }

            // update_columns() always inserts the leftmost column first, so
            // each branch's target is either its current location or to the
            // left of it; we never have to move branches to the right.
            if target * 2 == i as i32 {
                // This column is already in the correct place.
                self.mapping[i] = target;
            } else if self.mapping[i - 1] < 0 {
                // Nothing is to the left: move to the left by one.
                self.mapping[i - 1] = target;

                // If there isn't already an edge moving horizontally, select
                // this one. The screen column of the first horizontal line
                // is target*2+3.
                if horizontal_edge == -1 {
                    horizontal_edge = i as i32;
                    horizontal_edge_target = target;
                    for j in (target * 2 + 3..i as i32 - 2).step_by(2) {
                        self.mapping[j as usize] = target;
                    }
                }
            } else if self.mapping[i - 1] == target {
                // There is a branch line to our left already, and it is our
                // target: combine with this line, since we share the same
                // parent commit.
            } else {
                // There is a branch line to our left, but it isn't our
                // target: cross over it. The space just to the left of this
                // branch should always be empty.
                self.mapping[i - 2] = target;

                if horizontal_edge == -1 {
                    horizontal_edge_target = target;
                    horizontal_edge = i as i32 - 1;
                    for j in (target * 2 + 3..i as i32 - 2).step_by(2) {
                        self.mapping[j as usize] = target;
                    }
                }
            }
        }

        // Copy the current mapping array into old_mapping.
        self.old_mapping = self.mapping.clone();

        // The new mapping may be 1 smaller than the old mapping.
        if self.mapping[self.mapping_size - 1] < 0 {
            self.mapping_size -= 1;
        }

        // Output a line based on the new mapping info.
        for i in 0..self.mapping_size {
            let target = self.mapping[i];

            if target < 0 {
                line.push(' ');
            } else if target * 2 == i as i32 {
                line.push('|');
            } else if target == horizontal_edge_target && i as i32 != horizontal_edge - 1 {
                // Set the mappings for all but the first segment to -1 so
                // that they won't continue into the next line.
                if i as i32 != target * 2 + 3 {
                    self.mapping[i] = -1;
                }
                used_horizontal = true;
                line.push('_');
            } else {
                if used_horizontal && (i as i32) < horizontal_edge {
                    self.mapping[i] = -1;
                }
                line.push('/');
            }
        }

        if self.mapping_correct() {
            self.update_state(State::Padding);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs the graph driver over `parents` (indices into the commit list,
    /// in display order) and returns the emitted lines with the commit line
    /// marked by a 'C' prefix.
    fn render(parents: Vec<Vec<usize>>) -> Vec<String> {
        Graph::rows(parents)
            .into_iter()
            .map(|(line, is_commit)| {
                format!(
                    "{}{}",
                    if is_commit.is_some() { "C" } else { " " },
                    line.trim_end()
                )
            })
            .collect()
    }

    #[test]
    fn linear_history() {
        // 0 <- 1 <- 2
        let lines = render(vec![vec![1], vec![2], vec![]]);
        assert_eq!(lines, ["C*", "C*", "C*"]);
    }

    #[test]
    fn siblings() {
        // 0, 1 are siblings on top of 2 (the shape from the issue example).
        let lines = render(vec![vec![2], vec![2], vec![]]);
        assert_eq!(lines, ["C*", "C| *", " |/", "C*"]);
    }
}

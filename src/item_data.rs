use std::{ops::Range, path::PathBuf, rc::Rc};

use crate::git::diff::Diff;

#[derive(Clone, Debug)]
pub(crate) enum ItemData {
    Raw(String),
    AllUnstaged(usize),
    AllStaged(usize),
    AllUntracked(Vec<PathBuf>),
    Reference {
        prefix: &'static str,
        kind: RefKind,
    },
    Commit {
        oid: String,
        short_id: String,
        associated_references: Vec<RefKind>,
        summary: String,
    },
    Untracked(PathBuf),
    Delta {
        diff: Rc<Diff>,
        file_i: usize,
    },
    Hunk {
        diff: Rc<Diff>,
        file_i: usize,
        hunk_i: usize,
    },
    HunkLine {
        diff: Rc<Diff>,
        file_i: usize,
        hunk_i: usize,
        line_i: usize,
        line_range: Range<usize>,
    },
    Stash {
        message: String,
        stash_ref: String,
        id: usize,
    },
    Header(SectionHeader),
    BranchStatus(String, u32, u32),
    Error(String),
}

impl Default for ItemData {
    fn default() -> Self {
        ItemData::Raw(String::new())
    }
}

#[derive(Clone, Debug)]
pub(crate) enum RefKind {
    Tag(String),
    Branch(String),
    Remote(String),
}

#[derive(Clone, Debug)]
pub(crate) enum SectionHeader {
    Remote(String),
    Tags,
    Branches,
    NoBranch,
    OnBranch(String),
    Rebase(String, String),
    Merge(String),
    Revert(String),
    Stashes,
    RecentCommits,
    Commit(String),
}

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

impl ItemData {
    pub(crate) fn is_section(&self) -> bool {
        matches!(
            self,
            ItemData::AllUnstaged(_)
                | ItemData::AllStaged(_)
                | ItemData::AllUntracked(_)
                | ItemData::Untracked(_)
                | ItemData::Delta { .. }
                | ItemData::Hunk { .. }
                | ItemData::Header(_)
                | ItemData::BranchStatus(_, _, _)
        )
    }

    pub(crate) fn to_ref_kind(&self) -> Option<RefKind> {
        match self {
            Self::Reference { kind, .. } => Some(kind.clone()),
            _ => None,
        }
    }
}

impl Default for ItemData {
    fn default() -> Self {
        ItemData::Raw(String::new())
    }
}

#[derive(PartialEq, Clone, Debug)]
pub(crate) enum RefKind {
    Tag(String),
    Branch(String),
    Remote(String),
}

impl RefKind {
    /// Convert to fully qualified refname (e.g., "refs/heads/main", "refs/tags/v1.0.0")
    pub(crate) fn to_full_refname(&self) -> String {
        match self {
            RefKind::Branch(name) => format!("refs/heads/{}", name),
            RefKind::Tag(name) => format!("refs/tags/{}", name),
            RefKind::Remote(name) => format!("refs/remotes/{}", name),
        }
    }

    /// Get the shorthand name without refs/ prefix
    pub(crate) fn shorthand(&self) -> &str {
        match self {
            RefKind::Branch(name) | RefKind::Tag(name) | RefKind::Remote(name) => name,
        }
    }

    /// Convert a git2::Reference to RefKind, returning None if the reference has no shorthand
    pub(crate) fn from_reference(reference: &git2::Reference<'_>) -> Option<Self> {
        let shorthand = reference.shorthand()?.to_string();

        Some(if reference.is_branch() {
            RefKind::Branch(shorthand)
        } else if reference.is_tag() {
            RefKind::Tag(shorthand)
        } else {
            RefKind::Remote(shorthand)
        })
    }
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

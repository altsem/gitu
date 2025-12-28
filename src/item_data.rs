use std::{ops::Range, path::PathBuf, rc::Rc};

use crate::{Res, error::Error, git::diff::Diff};

#[derive(Clone, Debug)]
pub(crate) enum ItemData {
    Raw(String),
    AllUnstaged(usize),
    AllStaged(usize),
    AllUntracked(Vec<PathBuf>),
    Reference {
        prefix: &'static str,
        kind: Ref,
    },
    Commit {
        oid: String,
        short_id: String,
        associated_references: Vec<Ref>,
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

    pub(crate) fn rev(&self) -> Option<Rev> {
        match &self {
            ItemData::Reference { kind, .. } => Some(Rev::Ref(kind.clone())),
            ItemData::Commit {
                oid,
                associated_references,
                ..
            } => associated_references
                .first()
                .cloned()
                .map(Rev::Ref)
                .or_else(|| Some(Rev::Commit(oid.to_owned()))),
            _ => None,
        }
    }
}

impl Default for ItemData {
    fn default() -> Self {
        ItemData::Raw(String::new())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Rev {
    Ref(Ref),
    Commit(String),
}

impl Rev {
    pub(crate) fn from_reference(reference: &git2::Reference<'_>) -> Res<Self> {
        let shorthand = String::from_utf8_lossy(reference.shorthand_bytes()).to_string();

        if reference.is_branch() {
            Ok(Rev::Ref(Ref::Head(shorthand)))
        } else if reference.is_tag() {
            Ok(Rev::Ref(Ref::Tag(shorthand)))
        } else if reference.is_remote() {
            Ok(Rev::Ref(Ref::Remote(shorthand)))
        } else {
            let commit = reference.peel_to_commit().map_err(Error::ReadOid)?;
            Ok(Rev::Commit(commit.id().to_string()))
        }
    }

    pub(crate) fn shorthand(&self) -> &str {
        match self {
            Rev::Ref(r) => r.shorthand(),
            Rev::Commit(c) => c,
        }
    }
}

/// Represent a reference in git, as found in `.git/refs/heads`, `.git/refs/tags`, etc.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Ref {
    Tag(String),
    Head(String),
    Remote(String),
}

impl Ref {
    /// Convert to fully qualified refname (e.g., "refs/heads/main", "refs/tags/v1.0.0")
    pub(crate) fn to_full_refname(&self) -> String {
        match self {
            Ref::Head(name) => format!("refs/heads/{}", name),
            Ref::Tag(name) => format!("refs/tags/{}", name),
            Ref::Remote(name) => format!("refs/remotes/{}", name),
        }
    }

    /// Get the shorthand name without refs/ prefix
    pub(crate) fn shorthand(&self) -> &str {
        match self {
            Ref::Head(name) | Ref::Tag(name) | Ref::Remote(name) => name,
        }
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
    CherryPick(String),
    Stashes,
    RecentCommits,
    Commit(String),
    StagedChanges(usize),
    UnstagedChanges(usize),
    UntrackedFiles(usize),
}

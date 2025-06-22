use std::{iter, ops::Range, path::PathBuf, rc::Rc};

use ratatui::text::{Line, Span};

use crate::{config::Config, git::diff::Diff, gitu_diff::Status, highlight};

#[derive(Clone, Debug, Default)]
pub(crate) enum ItemData {
    #[default]
    Empty,
    AllUnstaged,
    AllStaged,
    AllUntracked(Vec<PathBuf>),
    Reference(RefKind),
    Commit {
        oid: String,
        short_id: String,
        associated_references: Vec<RefKind>,
        summary: String,
    },
    File(PathBuf),
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
        commit: String,
        id: usize,
    },
    Header(SectionHeader),
    BranchStatus(String, usize, usize),
    Error(String),
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
    UpstreamGone(String),
    Rebase(String, String),
    Merge(String),
    Revert(String),
    Stashes,
    RecentCommits,
}

impl ItemData {
    // FIXME this can go back to returning just one single `Line`
    pub fn to_line<'a>(&'a self, config: Rc<Config>) -> Line<'a> {
        match self {
            ItemData::Empty => Line::raw(""),
            ItemData::AllUnstaged => Line::styled("Unstaged changes", &config.style.section_header),
            ItemData::AllStaged => Line::styled("Staged changes", &config.style.section_header),
            ItemData::AllUntracked(_) => {
                Line::styled("Untracked files", &config.style.section_header)
            }
            ItemData::Reference(ref_kind) => {
                let (reference, style) = match ref_kind {
                    RefKind::Tag(tag) => (tag, &config.style.tag),
                    RefKind::Branch(branch) => (branch, &config.style.branch),
                    RefKind::Remote(remote) => (remote, &config.style.remote),
                };
                // TODO create prefix
                Line::styled(reference, style)
            }
            ItemData::Commit {
                short_id,
                associated_references,
                summary,
                ..
            } => Line::from_iter(itertools::intersperse(
                iter::once(Span::styled(short_id, &config.style.hash))
                    .chain(
                        associated_references
                            .iter()
                            .map(|reference| match reference {
                                RefKind::Tag(tag) => Span::styled(tag, &config.style.tag),
                                RefKind::Branch(branch) => {
                                    Span::styled(branch, &config.style.branch)
                                }
                                RefKind::Remote(remote) => {
                                    Span::styled(remote, &config.style.remote)
                                }
                            }),
                    )
                    .chain([Span::raw(summary)]),
                Span::raw(" "),
            )),
            ItemData::File(path) => Line::styled(path.to_string_lossy(), &config.style.file_header),
            ItemData::Delta { diff, file_i } => {
                let file_diff = &diff.file_diffs[*file_i];

                let content = format!(
                    "{:8}   {}",
                    format!("{:?}", file_diff.header.status).to_lowercase(),
                    match file_diff.header.status {
                        Status::Renamed => format!(
                            "{} -> {}",
                            &Rc::clone(diff).text[file_diff.header.old_file.clone()],
                            &Rc::clone(diff).text[file_diff.header.new_file.clone()]
                        ),
                        _ => Rc::clone(diff).text[file_diff.header.new_file.clone()].to_string(),
                    }
                );

                Line::styled(content, &config.style.file_header)
            }
            ItemData::Hunk {
                diff,
                file_i,
                hunk_i,
            } => {
                let file_diff = &diff.file_diffs[*file_i];
                let hunk = &file_diff.hunks[*hunk_i];

                let content = &diff.text[hunk.header.range.clone()];

                Line::styled(content, &config.style.hunk_header)
            }
            ItemData::HunkLine {
                diff,
                file_i,
                hunk_i,
                line_range,
                ..
            } => {
                let hunk = diff.hunk(*file_i, *hunk_i);

                Line::raw(&hunk[line_range.clone()])
            }
            ItemData::Stash { message, id, .. } => {
                Line::styled(format!("Stash@{id} {message}"), &config.style.hash)
            }
            ItemData::Header(header) => {
                let content = match header {
                    SectionHeader::Remote(remote) => format!("Remote {remote}"),
                    SectionHeader::Tags => "Tags".to_string(),
                    SectionHeader::Branches => "Branches".to_string(),
                    SectionHeader::NoBranch => "No branch".to_string(),
                    SectionHeader::OnBranch(branch) => format!("On branch {branch}"),
                    SectionHeader::UpstreamGone(upstream) => {
                        format!("Your branch is based on '{upstream}', but the upstream is gone.")
                    }
                    SectionHeader::Rebase(head, onto) => format!("Rebasing {head} onto {onto}"),
                    SectionHeader::Merge(head) => format!("Merging {head}"),
                    SectionHeader::Revert(head) => format!("Reverting {head}"),
                    SectionHeader::Stashes => "Stashes".to_string(),
                    SectionHeader::RecentCommits => "Recent commits".to_string(),
                };

                Line::styled(content, &config.style.section_header)
            }
            ItemData::BranchStatus(upstream, ahead, behind) => {
                let content = if *ahead == 0 && *behind == 0 {
                    format!("Your branch is up to date with '{upstream}'.")
                } else if *ahead > 0 && *behind == 0 {
                    format!("Your branch is ahead of '{upstream}' by {ahead} commit(s).",)
                } else if *ahead == 0 && *behind > 0 {
                    format!("Your branch is behind '{upstream}' by {behind} commit(s).",)
                } else {
                    format!("Your branch and '{upstream}' have diverged,\nand have {ahead} and {behind} different commits each, respectively.")
                };

                Line::raw(content)
            }
            ItemData::Error(err) => Line::raw(format!("{err}")),
        }
    }
}

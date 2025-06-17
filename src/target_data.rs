use std::{iter, path::PathBuf, rc::Rc};

use ratatui::text::{Line, Span};

use crate::{config::Config, git::diff::Diff, gitu_diff::Status};

#[derive(Clone, Debug)]
pub(crate) enum TargetData {
    Empty,
    AllUnstaged,
    AllStaged,
    AllUntracked(Vec<PathBuf>),
    Reference(RefKind),
    /// fields:
    /// - oid
    /// - short id
    /// - associated references
    /// - summary
    Commit(String, String, Vec<RefKind>, String),
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
}

impl TargetData {
    pub fn to_line<'a>(&self, config: Rc<Config>) -> Line<'a> {
        match self {
            TargetData::Empty => Line::raw(""),
            TargetData::AllUnstaged => {
                Line::styled("Unstaged changes", &config.style.section_header)
            }
            TargetData::AllStaged => Line::styled("Staged changes", &config.style.section_header),
            TargetData::AllUntracked(_) => {
                Line::styled("Untracked files", &config.style.section_header)
            }
            TargetData::Reference(ref_kind) => {
                // FIXME can we avoid cloning?
                let (reference, style) = match ref_kind {
                    RefKind::Tag(tag) => (tag.clone(), &config.style.tag),
                    RefKind::Branch(branch) => (branch.clone(), &config.style.branch),
                    RefKind::Remote(remote) => (remote.clone(), &config.style.remote),
                };
                // TODO create prefix
                Line::styled(reference, style)
            }
            TargetData::Commit(_, short_id, associated_references, summary) => {
                // FIXME avoid clones
                let spans: Vec<_> = itertools::intersperse(
                    iter::once(Span::styled(short_id.clone(), &config.style.hash))
                        .chain(
                            associated_references
                                .iter()
                                .map(|reference| match reference {
                                    RefKind::Tag(tag) => {
                                        Span::styled(tag.clone(), &config.style.tag)
                                    }
                                    RefKind::Branch(branch) => {
                                        Span::styled(branch.clone(), &config.style.branch)
                                    }
                                    RefKind::Remote(remote) => {
                                        Span::styled(remote.clone(), &config.style.remote)
                                    }
                                }),
                        )
                        .chain([Span::raw(summary.clone())]),
                    Span::raw(" "),
                )
                .collect();

                Line::from(spans)
            }
            TargetData::File(path) => Line::styled(
                path.to_string_lossy().to_string(),
                &config.style.file_header,
            ),
            TargetData::Delta { diff, file_i } => {
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
            TargetData::Hunk {
                diff,
                file_i,
                hunk_i,
            } => {
                let file_diff = &diff.file_diffs[*file_i];
                let hunk = &file_diff.hunks[*hunk_i];

                let content = diff.text[hunk.header.range.clone()].to_string();

                Line::styled(content, &config.style.hunk_header)
            }
            TargetData::HunkLine {
                diff,
                file_i,
                hunk_i,
                ..
            } => {
                let highlighted: Vec<_> =
                    crate::highlight::highlight_hunk_lines(&config, diff, *file_i, *hunk_i)
                        .flat_map(|line_highlights| {
                            let spans: Vec<_> = line_highlights
                                .iter()
                                .map(|(range, style)| {
                                    // FIXME avoid allocation?
                                    Span::styled(
                                        diff.text[range.clone()].replace('\t', "    "),
                                        *style,
                                    )
                                })
                                .collect();
                            spans
                        })
                        .collect();

                Line::from(highlighted)
            }
            TargetData::Stash { message, id, .. } => {
                Line::styled(format!("Stash@{id} {message}"), &config.style.hash)
            }
            TargetData::Header(header) => {
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
                };

                Line::styled(content, &config.style.section_header)
            }
            TargetData::BranchStatus(upstream, ahead, behind) => {
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
            TargetData::Error(err) => Line::raw(format!("{err}")),
        }
    }
}

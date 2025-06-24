use std::{iter, ops::Range, path::PathBuf, rc::Rc};

use ratatui::text::{Line, Span};

use crate::{config::Config, git::diff::Diff, gitu_diff::Status, highlight};

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
    UpstreamGone(String),
    Rebase(String, String),
    Merge(String),
    Revert(String),
    Stashes,
    RecentCommits,
    Commit(String),
}

impl ItemData {
    pub fn to_line<'a>(&'a self, config: Rc<Config>) -> Line<'a> {
        match self {
            ItemData::Raw(content) => Line::raw(content),
            ItemData::AllUnstaged(count) => Line::from(vec![
                Span::styled("Unstaged changes", &config.style.section_header),
                Span::raw(format!(" ({count})")),
            ]),
            ItemData::AllStaged(count) => Line::from(vec![
                Span::styled("Staged changes", &config.style.section_header),
                Span::raw(format!(" ({count})")),
            ]),
            ItemData::AllUntracked(_) => {
                Line::styled("Untracked files", &config.style.section_header)
            }
            ItemData::Reference { kind, prefix } => {
                let (reference, style) = match kind {
                    RefKind::Tag(tag) => (tag, &config.style.tag),
                    RefKind::Branch(branch) => (branch, &config.style.branch),
                    RefKind::Remote(remote) => (remote, &config.style.remote),
                };

                let prefixed_reference = format!("{prefix}{reference}");

                Line::styled(prefixed_reference, style)
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
                line_i,
            } => {
                let hunk_highlights =
                    highlight::highlight_hunk(&config, &Rc::clone(&diff), *file_i, *hunk_i);

                let hunk_content = &diff.hunk_content(*file_i, *hunk_i);
                let hunk_line = &hunk_content[line_range.clone()];

                Line::from_iter(
                    hunk_highlights
                        .get_hunk_line(*line_i)
                        .iter()
                        .map(|(range, style)| Span::styled(&hunk_line[range.clone()], *style)),
                )
            }
            ItemData::Stash { message, id, .. } => Line::from(vec![
                Span::styled(format!("stash@{id}"), &config.style.hash),
                Span::raw(format!(" {message}")),
            ]),
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
                    SectionHeader::Commit(oid) => format!("commit {oid}"),
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

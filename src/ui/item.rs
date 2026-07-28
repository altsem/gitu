use std::borrow::Cow;

use crate::style::Style;

use crate::config::Config;
use crate::gitu_diff::Status;
use crate::highlight;
use crate::item_data::{ItemData, Ref, SectionHeader};
use crate::items::Item;
use crate::ui::layout::opts;
use crate::ui::{UiTree, layout_span};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// What an author name is cut down to, so that a long one doesn't crowd out the
/// summary.
const AUTHOR_WIDTH: usize = 15;

/// Lays out an [`Item`] as spans in the caller's container, which is expected to
/// be a single row.
///
/// Every span is patched onto `base`, letting the caller supply a background
/// style such as the selection highlight.
pub(crate) fn layout_item<'a>(
    layout: &mut UiTree<'a>,
    item: &'a Item,
    config: &Config,
    base: Style,
) {
    let style = &config.style;

    match &item.data {
        ItemData::Raw(content) => {
            layout_span(layout, (content.as_str().into(), base));
        }
        ItemData::AllUnstaged(count) => {
            layout_span(
                layout,
                (
                    "Unstaged changes".into(),
                    base.patch(Style::from(&style.section_header)),
                ),
            );
            layout_span(layout, (format!(" ({count})").into(), base));
        }
        ItemData::AllStaged(count) => {
            layout_span(
                layout,
                (
                    "Staged changes".into(),
                    base.patch(Style::from(&style.section_header)),
                ),
            );
            layout_span(layout, (format!(" ({count})").into(), base));
        }
        ItemData::AllUntracked(_) => {
            layout_span(
                layout,
                (
                    "Untracked files".into(),
                    base.patch(Style::from(&style.section_header)),
                ),
            );
        }
        ItemData::Reference { kind, prefix } => {
            layout_span(layout, ((*prefix).into(), base));
            layout_reference(layout, kind, config, base);
        }
        ItemData::Commit {
            short_id,
            associated_references,
            summary,
            author,
            age,
            ..
        } => {
            layout_span(
                layout,
                (
                    short_id.as_str().into(),
                    base.patch(Style::from(&style.hash)),
                ),
            );

            layout.horizontal(None, opts().fill_x(), |layout| {
                for reference in associated_references {
                    layout_span(layout, (" ".into(), base));
                    layout_reference(layout, reference, config, base);
                }

                layout_span(layout, (" ".into(), base));
                layout_span(layout, (summary.as_str().into(), base));
            });

            layout_span(layout, (" ".into(), base));
            layout_span(
                layout,
                (
                    truncate(author, AUTHOR_WIDTH),
                    base.patch(Style::from(&style.author)),
                ),
            );
            layout_span(layout, (" ".into(), base));
            layout_span(
                layout,
                (age.as_str().into(), base.patch(Style::from(&style.age))),
            );
            layout_span(layout, (" ".into(), base));
        }
        ItemData::Untracked(path) => {
            layout_span(
                layout,
                (
                    path.to_string_lossy(),
                    base.patch(Style::from(&style.file_header)),
                ),
            );
        }
        ItemData::Delta { diff, file_i, .. } => {
            let file_diff = &diff.file_diffs[*file_i];

            let content = format!(
                "{:8}   {}",
                format!("{:?}", file_diff.header.status).to_lowercase(),
                match file_diff.header.status {
                    Status::Renamed | Status::Copied => format!(
                        "{} -> {}",
                        file_diff.header.old_file.fmt(&diff.text),
                        file_diff.header.new_file.fmt(&diff.text)
                    ),
                    Status::Deleted => file_diff.header.old_file.fmt(&diff.text).to_string(),
                    Status::Added => file_diff.header.new_file.fmt(&diff.text).to_string(),
                    Status::Modified => file_diff.header.new_file.fmt(&diff.text).to_string(),
                    Status::Unmerged => file_diff.header.new_file.fmt(&diff.text).to_string(),
                }
            );

            layout_span(
                layout,
                (content.into(), base.patch(Style::from(&style.file_header))),
            );
        }
        ItemData::Hunk {
            diff,
            file_i,
            hunk_i,
        } => {
            let hunk = &diff.file_diffs[*file_i].hunks[*hunk_i];
            let content = &diff.text[hunk.header.range.clone()];

            layout_span(
                layout,
                (content.into(), base.patch(Style::from(&style.hunk_header))),
            );
        }
        ItemData::HunkLine {
            diff,
            file_i,
            hunk_i,
            line_range,
            line_i,
        } => {
            let hunk_highlights =
                highlight::highlight_hunk(item.id, config, diff, *file_i, *hunk_i);
            let hunk_line = &diff.hunk_content(*file_i, *hunk_i)[line_range.clone()];

            for (highlight_range, highlight_style) in hunk_highlights.get_line_highlights(*line_i) {
                layout_span(
                    layout,
                    (
                        hunk_line[highlight_range.clone()]
                            .replace('\t', "    ")
                            .into(),
                        base.patch(*highlight_style),
                    ),
                );
            }
        }
        ItemData::Stash { message, id, .. } => {
            layout_span(
                layout,
                (
                    format!("stash@{id}").into(),
                    base.patch(Style::from(&style.hash)),
                ),
            );
            layout_span(layout, (format!(" {message}").into(), base));
        }
        ItemData::Header(header) => {
            let content: Cow<str> = match header {
                SectionHeader::Remote(remote) => format!("Remote {remote}").into(),
                SectionHeader::Tags => "Tags".into(),
                SectionHeader::Branches => "Branches".into(),
                SectionHeader::NoBranch => "No branch".into(),
                SectionHeader::OnBranch(branch) => format!("On branch {branch}").into(),
                SectionHeader::Rebase(head, onto) => format!("Rebasing {head} onto {onto}").into(),
                SectionHeader::Merge(head) => format!("Merging {head}").into(),
                SectionHeader::Revert(head) => format!("Reverting {head}").into(),
                SectionHeader::CherryPick(head) => format!("Cherry-picking {head}").into(),
                SectionHeader::Stashes => "Stashes".into(),
                SectionHeader::RecentCommits => "Recent commits".into(),
                SectionHeader::Commit(oid) => format!("commit {oid}").into(),
                SectionHeader::StashRef(stash_ref) => stash_ref.as_str().into(),
                SectionHeader::StagedChanges(count) => format!("Staged changes ({count})").into(),
                SectionHeader::UnstagedChanges(count) => {
                    format!("Unstaged changes ({count})").into()
                }
                SectionHeader::UntrackedFiles(count) => format!("Untracked files ({count})").into(),
                SectionHeader::Blame(file, commit) => format!("Blame {file} @ {commit}").into(),
            };

            layout_span(
                layout,
                (content, base.patch(Style::from(&style.section_header))),
            );
        }
        ItemData::BranchStatus(upstream, ahead, behind) => {
            let content = if *ahead == 0 && *behind == 0 {
                format!("Your branch is up to date with '{upstream}'.")
            } else if *ahead > 0 && *behind == 0 {
                format!("Your branch is ahead of '{upstream}' by {ahead} commit(s).")
            } else if *ahead == 0 && *behind > 0 {
                format!("Your branch is behind '{upstream}' by {behind} commit(s).")
            } else {
                format!(
                    "Your branch and '{upstream}' have diverged,\nand have {ahead} and {behind} different commits each, respectively."
                )
            };

            layout_span(layout, (content.into(), base));
        }
        ItemData::Error(err) => {
            layout_span(layout, (err.as_str().into(), base));
        }
        ItemData::BlameHeader {
            short_hash,
            summary,
            ..
        } => {
            layout_span(
                layout,
                (
                    format!("{short_hash:<8}").into(),
                    base.patch(Style::from(&style.hash)),
                ),
            );
            layout_span(layout, (" ".into(), base));
            layout_span(layout, (summary.as_str().into(), base));
        }
        ItemData::BlameCodeLine {
            blame_file,
            line_i,
            line_num,
            content,
            ..
        } => {
            let base = base.patch(Style::from(&style.blame.code_line));

            layout_span(
                layout,
                (
                    format!("{line_num:>4} ").into(),
                    base.patch(Style::from(&style.blame.line_num)),
                ),
            );

            for (range, highlight_style) in blame_file.highlights.get_line_highlights(*line_i) {
                if !range.is_empty() && range.end <= content.len() {
                    layout_span(
                        layout,
                        (
                            content[range.clone()].replace('\t', "    ").into(),
                            base.patch(*highlight_style),
                        ),
                    );
                }
            }
        }
    }
}

/// Cuts `text` down to `width` columns, the last of which is an ellipsis.
fn truncate(text: &str, width: usize) -> Cow<'_, str> {
    if text.width() <= width {
        return Cow::Borrowed(text);
    }

    let mut truncated = String::new();
    let mut taken = 0;

    for grapheme in text.graphemes(true) {
        if taken + grapheme.width() > width - 1 {
            break;
        }

        truncated.push_str(grapheme);
        taken += grapheme.width();
    }

    truncated.push('…');
    Cow::Owned(truncated)
}

fn layout_reference<'a>(layout: &mut UiTree<'a>, reference: &'a Ref, config: &Config, base: Style) {
    let (name, style) = match reference {
        Ref::Tag(tag) => (tag, &config.style.tag),
        Ref::Head(branch) => (branch, &config.style.branch),
        Ref::Remote(remote) => (remote, &config.style.remote),
    };

    layout_span(
        layout,
        (name.as_str().into(), base.patch(Style::from(style))),
    );
}

use core::str;
use gitu_diff::FileDiff;
use std::{io::BufRead, ops::Range};

// TODO Change the old "Diff" struct of Gitu to:
//      - not contain String/Text copies of fields
//      - don't re-assemble this back into lines via Ratatui's Line/Span structs,
//        error prone and messy.
//      - Store the final styling alongside the original diff:
//        ```
//        raw: Vec<u8>
//        styles: Vec<(Range<usize>, *Style*), <- Merged line diff/inline diff/syntax highlights
//        ````
//
// The whole diff could now be stored as the original string output from git.
// We'll then use the parsed structure of `Range<usize>`'s to reference into the original diff
// to:
// - perform diff highlighting
// - interpret parts of the diff
//
// TODO Re-introduce in-line diff highlighting.
// TODO Re-introduce syntax highlighting. Maybe don't highlight entire files like done before.
//      Could try include the hunk function context part in the syntax hihlight too.
//      Should be much easier to merge with diff highlights, now they are both `Range<usize>`.

#[derive(Debug, Clone)]
pub(crate) struct Diff {
    pub text: Vec<u8>,
    pub file_diffs: Vec<FileDiff>,
}

#[derive(Debug)]
pub(crate) enum PatchMode {
    Normal,
    Reverse,
}

pub(crate) fn format_patch(diff: &Diff, file_i: usize, hunk_i: usize) -> String {
    // format!("{}{}{}\n", &self.file_header, self.header, self.content)
    // TODO

    let file_diff = &diff.file_diffs[file_i];
    format!(
        "{}{}",
        str::from_utf8(&diff.text[file_diff.header.range.clone()]).unwrap(),
        str::from_utf8(&diff.text[file_diff.hunks[hunk_i].range.clone()]).unwrap()
    )
}

pub(crate) fn format_line_patch(
    diff: &Diff,
    file_i: usize,
    hunk_i: usize,
    line_range: Range<usize>,
    mode: PatchMode,
) -> String {
    // let modified_content = self
    //     .content
    //     .lines
    //     .iter()
    //     .enumerate()
    //     .filter_map(|(i, line)| {
    //         let add = match mode {
    //             PatchMode::Normal => '+',
    //             PatchMode::Reverse => '-',
    //         };

    //         let remove = match mode {
    //             PatchMode::Normal => '-',
    //             PatchMode::Reverse => '+',
    //         };

    //         let patch_line = format!("{line}");

    //         if line_range.contains(&i) {
    //             Some(patch_line)
    //         } else if patch_line.starts_with(add) {
    //             None
    //         } else if let Some(stripped) = patch_line.strip_prefix(remove) {
    //             Some(format!(" {}", stripped))
    //         } else {
    //             Some(patch_line)
    //         }
    //     })
    //     .join("\n");

    // format!("{}{}{}\n", &self.file_header, self.header, modified_content)
    todo!()
}

pub(crate) fn first_diff_line(diff: &Diff, file_i: usize, hunk_i: usize) -> usize {
    if let Some(change) = diff.file_diffs[file_i].hunks[hunk_i]
        .content
        .changes
        .first()
    {
        change
            .removed
            .as_ref()
            .or(change.added.as_ref())
            .map(|change_range| diff.text[..change_range.start].lines().count())
            .unwrap_or(0)
    } else {
        0
    }
}

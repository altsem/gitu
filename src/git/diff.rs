use crate::gitu_diff::FileDiff;
use std::ops::Range;

#[derive(Debug, Clone)]
pub(crate) struct Diff {
    pub text: String,
    pub file_diffs: Vec<FileDiff>,
}

#[derive(Debug)]
pub(crate) enum PatchMode {
    Normal,
    Reverse,
}

impl Diff {
    pub(crate) fn mask_old_hunk(&self, file_i: usize, hunk_i: usize) -> String {
        let content = &self.text[self.file_diffs[file_i].hunks[hunk_i].content.range.clone()];
        mask_hunk_content(content, '-', '+')
    }

    pub(crate) fn mask_new_hunk(&self, file_i: usize, hunk_i: usize) -> String {
        let content = &self.text[self.file_diffs[file_i].hunks[hunk_i].content.range.clone()];
        mask_hunk_content(content, '+', '-')
    }

    pub(crate) fn format_patch(&self, file_i: usize, hunk_i: usize) -> String {
        let file_diff = &self.file_diffs[file_i];
        format!(
            "{}{}",
            &self.text[file_diff.header.range.clone()],
            &self.text[file_diff.hunks[hunk_i].range.clone()]
        )
    }

    pub(crate) fn format_line_patch(
        &self,
        file_i: usize,
        hunk_i: usize,
        line_range: Range<usize>,
        mode: PatchMode,
    ) -> String {
        let hunk = &self.file_diffs[file_i].hunks[hunk_i];
        let file_header = &self.text[self.file_diffs[file_i].header.range.clone()];
        let hunk_header = &self.text[hunk.header.range.clone()];
        let hunk_content = &self.text[hunk.content.range.clone()];

        let modified_content = hunk_content
            .split_inclusive('\n')
            .enumerate()
            .filter_map(|(i, line)| {
                let add = match mode {
                    PatchMode::Normal => '+',
                    PatchMode::Reverse => '-',
                };

                let remove = match mode {
                    PatchMode::Normal => '-',
                    PatchMode::Reverse => '+',
                };

                if line_range.contains(&i) {
                    Some(line.to_string())
                } else if line.starts_with(add) {
                    None
                } else if let Some(stripped) = line.strip_prefix(remove) {
                    Some(format!(" {}", stripped))
                } else {
                    Some(line.to_string())
                }
            })
            .collect::<String>();

        format!("{}{}{}", file_header, hunk_header, modified_content)
    }

    pub(crate) fn first_diff_line(&self, file_i: usize, hunk_i: usize) -> usize {
        if let Some(change) = self.file_diffs[file_i].hunks[hunk_i]
            .content
            .changes
            .first()
        {
            self.text[..change.old.start].lines().count()
        } else {
            0
        }
    }
}

fn mask_hunk_content(content: &str, keep: char, mask: char) -> String {
    let mut result = String::new();

    content.split_inclusive('\n').for_each(|line| {
        if line.starts_with(mask) {
            if line.ends_with("\r\n") {
                for _ in 0..(line.len() - 2) {
                    result.push(' ');
                }
                result.push_str("\r\n");
            } else if line.ends_with('\n') {
                for _ in 0..(line.len() - 1) {
                    result.push(' ');
                }
                result.push('\n');
            }
        } else if line.starts_with(keep) {
            result.push(' ');
            result.push_str(&line[1..]);
        } else if !line.starts_with('\\') {
            result.push_str(line);
        }
    });

    result
}

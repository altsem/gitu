use std::fmt::Display;

use itertools::Itertools;

#[derive(Debug, Clone)]
pub(crate) struct Diff {
    pub deltas: Vec<Delta>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Delta {
    pub file_header: String,
    pub old_file: String,
    pub new_file: String,
    pub hunks: Vec<Hunk>,
}

// TODO Is this needed?
impl Display for Delta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.file_header)?;
        for hunk in self.hunks.iter() {
            f.write_str(&hunk.to_string())?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Hunk {
    pub file_header: String,
    pub old_file: String,
    pub new_file: String,
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub header_suffix: String,
    pub content: String,
}

impl Hunk {
    pub(crate) fn display_header(&self) -> String {
        format!(
            "@@ -{},{} +{},{} @@",
            self.old_start, self.old_lines, self.new_start, self.new_lines
        )
    }

    pub(crate) fn header(&self) -> String {
        format!(
            "@@ -{},{} +{},{} @@{}",
            self.old_start, self.old_lines, self.new_start, self.new_lines, self.header_suffix
        )
    }

    pub(crate) fn format_patch(&self) -> String {
        format!("{}{}\n{}", &self.file_header, self.header(), &self.content)
    }

    pub(crate) fn old_content(&self) -> String {
        self.content
            .lines()
            .filter(|line| !line.starts_with('+'))
            .map(|line| &line[1..])
            .join("\n")
    }

    pub(crate) fn new_content(&self) -> String {
        self.content
            .lines()
            .filter(|line| !line.starts_with('-'))
            .map(|line| &line[1..])
            .join("\n")
    }

    pub(crate) fn first_diff_line(&self) -> u32 {
        self.content
            .lines()
            .enumerate()
            .filter(|(_, line)| line.starts_with('+') || line.starts_with('-'))
            .map(|(i, _)| i)
            .next()
            .unwrap_or(0) as u32
            + self.new_start
    }
}

impl Display for Hunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display_header())?;
        f.write_str(&self.content)?;
        Ok(())
    }
}

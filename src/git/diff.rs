use itertools::Itertools;
use std::{fmt::Display, path::PathBuf};

#[derive(Debug, Clone)]
pub(crate) struct Diff {
    pub deltas: Vec<Delta>,
}

#[derive(Debug, Clone)]
pub(crate) struct Delta {
    pub file_header: String,
    pub old_file: PathBuf,
    pub new_file: PathBuf,
    pub hunks: Vec<Hunk>,
    pub status: git2::Delta,
}

#[derive(Debug, Clone)]
pub(crate) struct Hunk {
    pub file_header: String,
    pub new_file: PathBuf,
    pub new_start: u32,
    pub header: String,
    pub content: String,
}

impl Hunk {
    pub(crate) fn format_patch(&self) -> String {
        format!("{}{}", &self.file_header, self)
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
        f.write_str(&self.header)?;
        f.write_str(&self.content)?;
        Ok(())
    }
}

use std::{fmt::Display, ops::Range};

use regex::Regex;

#[derive(Debug, Clone)]
pub struct Diff {
    pub deltas: Vec<Delta>,
}

impl Display for Diff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for delta in self.deltas.iter() {
            f.write_str(&delta.to_string())?;
        }

        Ok(())
    }
}

impl From<git2::Diff<'_>> for Diff {
    fn from(value: git2::Diff<'_>) -> Self {
        let mut deltas = vec![];

        value
            .print(git2::DiffFormat::Patch, |git2_delta, maybe_hunk, line| {
                let string_line = String::from_utf8_lossy(line.content()).to_string();

                match line.origin() {
                    'F' => deltas.push(Delta {
                        old_file: diff_file_path(&git2_delta.old_file()),
                        new_file: diff_file_path(&git2_delta.new_file()),
                        file_header: string_line,
                        hunks: vec![],
                    }),
                    'H' => {
                        let hunk = maybe_hunk.unwrap();
                        let delta = deltas.last_mut().unwrap();
                        delta
                            .hunks
                            .push(Hunk::new(delta.file_header.clone(), hunk, String::new()))
                    }
                    ' ' | '+' | '-' => deltas
                        .last_mut()
                        .unwrap()
                        .hunks
                        .last_mut()
                        .unwrap()
                        .content
                        .push_str(&(line.origin().to_string() + &string_line)),
                    _ => panic!("Unexpected line origin: {}", line.origin()),
                }

                true
            })
            .unwrap();

        Diff { deltas }
    }
}

fn diff_file_path(old_file: &git2::DiffFile<'_>) -> String {
    String::from_utf8_lossy(old_file.path_bytes().unwrap()).to_string()
}

#[derive(Debug, Clone)]
pub struct Delta {
    pub file_header: String,
    pub old_file: String,
    pub new_file: String,
    pub hunks: Vec<Hunk>,
}

impl Display for Delta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.file_header)?;
        for hunk in self.hunks.iter() {
            f.write_str(&hunk.to_string())?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Hunk {
    pub file_header: String,
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    header_suffix: String,
    pub content: String,
}

impl Hunk {
    fn new(file_header: String, diff_hunk: git2::DiffHunk, content: String) -> Self {
        // TODO init once
        let hunk_header_prefix_regex = Regex::new(r"@@ -\d+,\d+ \+\d+,\d+ @@").unwrap();
        Self {
            file_header,
            old_start: diff_hunk.old_start(),
            old_lines: diff_hunk.old_lines(),
            new_start: diff_hunk.new_start(),
            new_lines: diff_hunk.new_lines(),
            header_suffix: hunk_header_prefix_regex
                .replace(&String::from_utf8_lossy(diff_hunk.header()), "")
                .to_string(),
            content,
        }
    }

    pub fn header(&self) -> String {
        format!(
            "@@ -{},{} +{},{} @@{}",
            self.old_start, self.old_lines, self.new_start, self.new_lines, self.header_suffix
        )
    }

    pub fn select(&self, range: Range<usize>) -> Option<Self> {
        let modified_lines = self
            .content
            .split("\n")
            .enumerate()
            .filter_map(|(i, line)| {
                if range.contains(&i) || line.starts_with(" ") || line == "" {
                    Some(line.to_string())
                } else if line.starts_with("+") {
                    None
                } else if line.starts_with("-") {
                    Some(line.replacen("-", " ", 1))
                } else {
                    panic!("Unexpected case: {}", line);
                }
            })
            .collect::<Vec<_>>();

        let added = modified_lines
            .iter()
            .filter(|line| line.starts_with("+"))
            .count();

        let removed = modified_lines
            .iter()
            .filter(|line| line.starts_with("-"))
            .count();

        if added == 0 && removed == 0 {
            return None;
        }

        Some(Self {
            new_lines: self.old_lines + added as u32 - removed as u32,
            content: modified_lines.join("\n"),
            ..self.clone()
        })
    }

    pub fn format_patch(&self) -> String {
        format!("{}{}{}", self.file_header, self.header(), self.content)
    }
}

impl Display for Hunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.header())?;
        f.write_str(&self.content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn format_diff_preserved() {
        let buffer = include_str!("example3.patch");
        let diff = git2::Diff::from_buffer(buffer.as_bytes()).unwrap();
        let result = super::Diff::from(diff).to_string();

        let diff_diff = super::Diff::from(
            git2::Diff::from_buffer(
                &git2::Patch::from_buffers(buffer.as_bytes(), None, result.as_bytes(), None, None)
                    .unwrap()
                    .to_buf()
                    .unwrap(),
            )
            .unwrap(),
        );

        println!("{}", diff_diff);
        assert!(diff_diff.deltas.is_empty());
    }

    #[test]
    fn select_lines() {
        let buffer = include_str!("example2.patch");
        let diff = git2::Diff::from_buffer(buffer.as_bytes()).unwrap();
        let patch = super::Diff::from(diff);
        let hunk = patch.deltas[0].hunks.first().unwrap();
        let result = hunk.select(4..7).unwrap();

        println!("Pre-select {}", hunk);
        println!("Post-select {}", result);

        assert!(result.content.lines().nth(3).unwrap().starts_with(" "));
        assert!(result.content.lines().nth(4).unwrap().starts_with("-"));
        assert!(result.content.lines().nth(5).unwrap().starts_with("+"));
        assert!(result.content.lines().nth(6).unwrap().starts_with("+"));
        assert!(result.content.lines().nth(7).unwrap().starts_with(" "));
        assert_eq!(9, result.new_lines);
    }

    #[test]
    fn select_nothing() {
        let buffer = include_str!("example2.patch");
        let diff = git2::Diff::from_buffer(buffer.as_bytes()).unwrap();
        let patch = super::Diff::from(diff);
        let hunk = patch.deltas[0].hunks.first().unwrap();
        let result = hunk.select(0..0);

        assert!(result.is_none());
    }
}

use std::{fmt::Display, ops::Range};

use git2::{Diff, DiffHunk};
use regex::Regex;

#[derive(Debug, Clone)]
pub struct Patch {
    pub header: String,
    pub hunks: Vec<Hunk>,
}

impl Display for Patch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.header)?;
        for hunk in self.hunks.iter() {
            f.write_str(&hunk.to_string())?;
        }

        Ok(())
    }
}

impl From<Diff<'_>> for Patch {
    fn from(value: Diff<'_>) -> Self {
        let mut header = String::new();
        let mut hunks = vec![];

        value
            .print(git2::DiffFormat::Patch, |_delta, maybe_hunk, line| {
                use std::fmt::Write;
                let string_line = String::from_utf8_lossy(line.content());

                match line.origin() {
                    'F' => header.write_str(&string_line).unwrap(),
                    'H' => {
                        let hunk = maybe_hunk.unwrap();
                        hunks.push(Hunk::new(hunk, String::new()))
                    }
                    ' ' | '+' | '-' => hunks
                        .last_mut()
                        .unwrap()
                        .content
                        .push_str(&(line.origin().to_string() + &string_line)),
                    _ => panic!("Unexpected line origin: {}", line.origin()),
                }

                true
            })
            .unwrap();

        Patch { header, hunks }
    }
}

#[derive(Debug, Clone)]
pub struct Hunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    header_suffix: String,
    pub content: String,
}

impl Hunk {
    fn new(diff_hunk: DiffHunk<'_>, content: String) -> Self {
        // TODO init once
        let hunk_header_prefix_regex = Regex::new(r"@@ -\d+,\d+ \+\d+,\d+ @@ ").unwrap();
        Self {
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
            "@@ -{},{} +{},{} @@ {}",
            self.old_start, self.old_lines, self.new_start, self.new_lines, self.header_suffix
        )
    }

    pub fn select(&self, range: Range<usize>) -> Self {
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

        Self {
            new_lines: self.old_lines + added as u32 - removed as u32,
            content: modified_lines.join("\n"),
            ..self.clone()
        }
    }

    pub fn has_diff(&self) -> bool {
        self.content
            .lines()
            .any(|line| line.starts_with("+") || line.starts_with("-"))
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
    use git2::Diff;

    #[test]
    fn format_diff_preserved() {
        let buffer = include_str!("example2.patch");
        let diff = Diff::from_buffer(buffer.as_bytes()).unwrap();
        assert_eq!(buffer, super::Patch::from(diff).to_string());
    }

    #[test]
    fn select_lines() {
        let buffer = include_str!("example2.patch");
        let diff = Diff::from_buffer(buffer.as_bytes()).unwrap();
        let patch = super::Patch::from(diff);
        let hunk = patch.hunks.first().unwrap();
        let result = hunk.select(4..7);

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
        let diff = Diff::from_buffer(buffer.as_bytes()).unwrap();
        let patch = super::Patch::from(diff);
        let hunk = patch.hunks.first().unwrap();
        let result = hunk.select(0..1);

        println!("Pre-select {}", hunk);
        println!("Post-select {}", result);

        assert!(!result.has_diff());
    }
}

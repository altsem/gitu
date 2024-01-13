use regex::Regex;
use std::{fmt::Display, ops::Range};

const DELTAS_REGEX: &str = r"(?<header>diff --git a\/\S+ b\/\S+
([^@].*
)*--- (:?a\/)?(?<old_file>\S+)
\+\+\+ (:?b\/)?(?<new_file>\S+)
)(?<hunk>(:?[ @\-+\u{1b}].*
)*)";

const HUNKS_REGEX: &str = r"@@ \-(?<old_start>\d+),(?<old_lines>\d+) \+(?<new_start>\d+),(?<new_lines>\d+) @@(?<header_suffix>.*
)(?<content>(:?[ \-+\u{1b}].*
)*)";

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

impl Diff {
    pub fn parse(diff_str: &str) -> Self {
        let deltas_regex = &Regex::new(DELTAS_REGEX).unwrap();
        let hunks_regex = Regex::new(HUNKS_REGEX).unwrap();

        Self {
            deltas: deltas_regex
                .captures_iter(diff_str)
                .map(|cap| {
                    let header = group_as_string(&cap, "header");
                    let hunk = group_as_string(&cap, "hunk");

                    Delta {
                        file_header: header.clone(),
                        old_file: group_as_string(&cap, "old_file"),
                        new_file: group_as_string(&cap, "new_file"),
                        hunks: hunks_regex
                            .captures_iter(&hunk)
                            .map(|hunk_cap| Hunk {
                                file_header: header.clone(),
                                old_start: group_as_u32(&hunk_cap, "old_start"),
                                old_lines: group_as_u32(&hunk_cap, "old_lines"),
                                new_start: group_as_u32(&hunk_cap, "new_start"),
                                new_lines: group_as_u32(&hunk_cap, "new_lines"),
                                header_suffix: group_as_string(&hunk_cap, "header_suffix"),
                                content: group_as_string(&hunk_cap, "content"),
                            })
                            .collect::<Vec<_>>(),
                    }
                })
                .collect::<Vec<_>>(),
        }
    }
}

fn group_as_string(cap: &regex::Captures<'_>, group: &str) -> String {
    cap.name(group)
        .unwrap_or_else(|| panic!("{} group not matching", group))
        .as_str()
        .to_string()
}

fn group_as_u32(cap: &regex::Captures<'_>, group: &str) -> u32 {
    cap.name(group)
        .unwrap_or_else(|| panic!("{} group not matching", group))
        .as_str()
        .parse()
        .unwrap_or_else(|_| panic!("Couldn't parse {}", group))
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hunk {
    pub file_header: String,
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    header_suffix: String,
    content: String,
}

impl Hunk {
    pub fn display_header(&self) -> String {
        format!(
            "@@ -{},{} +{},{} @@",
            self.old_start, self.old_lines, self.new_start, self.new_lines
        )
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
            .split('\n')
            .enumerate()
            .filter_map(|(i, line)| {
                if range.contains(&i) || line.starts_with(' ') || line.is_empty() {
                    Some(line.to_string())
                } else if line.starts_with('+') {
                    None
                } else if line.starts_with('-') {
                    Some(line.replacen('-', " ", 1))
                } else {
                    panic!("Unexpected case: {}", line);
                }
            })
            .collect::<Vec<_>>();

        let added = modified_lines
            .iter()
            .filter(|line| line.starts_with('+'))
            .count();

        let removed = modified_lines
            .iter()
            .filter(|line| line.starts_with('-'))
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
        format!(
            "{}{}{}",
            strip_ansi_escapes::strip_str(&self.file_header),
            strip_ansi_escapes::strip_str(self.header()),
            strip_ansi_escapes::strip_str(&self.content)
        )
    }

    pub fn content_lines<'a>(&'a self) -> Box<dyn Iterator<Item = DiffLine> + 'a> {
        Box::new(self.content.lines().map(|line| DiffLine {
            plain: strip_ansi_escapes::strip_str(line),
            colored: line.to_string(),
        }))
    }
}

pub struct DiffLine {
    pub plain: String,
    pub colored: String,
}

impl Display for Hunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display_header())?;
        f.write_str(&self.content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Diff;

    #[test]
    fn parse_example() {
        let diff = Diff::parse(include_str!("example.patch"));
        assert_eq!(diff.deltas.len(), 2);
        assert_eq!(diff.deltas[0].hunks.len(), 2);
        assert_eq!(diff.deltas[1].hunks.len(), 2);
    }
}

use pest::Parser;
use pest_derive::Parser;
use std::fmt::Display;

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

#[derive(Parser)]
#[grammar = "diff.pest"]
struct DiffParser;

impl Diff {
    pub fn parse(input: &str) -> Self {
        let mut deltas = vec![];

        for diff in DiffParser::parse(Rule::diffs, input).expect("Error parsing diff") {
            let mut old_file = None;
            let mut new_file = None;
            let mut file_header = None;
            let mut hunks = vec![];

            match diff.as_rule() {
                Rule::diff => {
                    for diff_field in diff.into_inner() {
                        match diff_field.as_rule() {
                            Rule::diff_header => {
                                file_header = Some(diff_field.as_str().to_string());

                                for diff_header_field in diff_field.into_inner() {
                                    match diff_header_field.as_rule() {
                                        Rule::old_file => {
                                            old_file = Some(diff_header_field.as_str().to_string())
                                        }
                                        Rule::new_file => {
                                            new_file = Some(diff_header_field.as_str().to_string())
                                        }
                                        Rule::header_extra => {}
                                        rule => panic!("No rule {:?}", rule),
                                    }
                                }
                            }
                            Rule::hunk => {
                                let mut old_range = None;
                                let mut new_range = None;
                                let mut context = None;
                                let mut body = None;

                                for hunk_field in diff_field.into_inner() {
                                    match hunk_field.as_rule() {
                                        Rule::old_range => {
                                            old_range = Some(parse_range(hunk_field))
                                        }
                                        Rule::new_range => {
                                            new_range = Some(parse_range(hunk_field))
                                        }
                                        Rule::context => {
                                            context = Some(hunk_field.as_str().to_string())
                                        }
                                        Rule::hunk_body => {
                                            body = Some(hunk_field.as_str().to_string())
                                        }
                                        rule => panic!("No rule {:?}", rule),
                                    }
                                }

                                hunks.push(Hunk {
                                    file_header: file_header.clone().unwrap(),
                                    old_file: old_file.clone().unwrap(),
                                    new_file: new_file.clone().unwrap(),
                                    old_start: old_range.unwrap().0,
                                    old_lines: old_range.unwrap().1,
                                    new_start: new_range.unwrap().0,
                                    new_lines: new_range.unwrap().1,
                                    header_suffix: context.unwrap(),
                                    content: body.unwrap(),
                                });
                            }
                            rule => panic!("No rule {:?}", rule),
                        }
                    }
                }
                rule => panic!("No rule {:?}", rule),
            }

            deltas.push(Delta {
                file_header: file_header.unwrap(),
                old_file: old_file.unwrap(),
                new_file: new_file.unwrap(),
                hunks,
            })
        }

        Self { deltas }
    }
}

fn parse_range(hunk_field: pest::iterators::Pair<'_, Rule>) -> (u32, u32) {
    let mut start = None;
    let mut lines = None;

    for range_field in hunk_field.into_inner() {
        match range_field.as_rule() {
            Rule::start => {
                start = Some(
                    range_field
                        .as_str()
                        .parse()
                        .expect("Error parsing range start"),
                );
            }
            Rule::lines => {
                lines = Some(
                    range_field
                        .as_str()
                        .parse()
                        .expect("Error parsing range lines"),
                );
            }
            rule => panic!("No rule {:?}", rule),
        }
    }
    (
        start.expect("No range start"),
        lines.expect("No range lines"),
    )
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
    pub old_file: String,
    pub new_file: String,
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    header_suffix: String,
    pub content: String,
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

    pub fn format_patch(&self) -> String {
        format!(
            "{}{}{}",
            strip_ansi_escapes::strip_str(&self.file_header),
            strip_ansi_escapes::strip_str(self.header()),
            strip_ansi_escapes::strip_str(&self.content)
        )
    }
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

    #[test]
    fn parse_example_empty_file() {
        let diff = Diff::parse(include_str!("example_empty_file.patch"));
        assert_eq!(diff.deltas.len(), 1);
        assert_eq!(diff.deltas[0].hunks.len(), 0);
    }
}

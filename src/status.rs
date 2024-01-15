use regex::Regex;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Status {
    pub branch_status: BranchStatus,
    pub files: Vec<StatusFile>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BranchStatus {
    pub local: String,
    pub remote: String,
    pub ahead: u32,
    pub behind: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct StatusFile {
    pub status_code: [char; 2],
    pub orig_path: Option<String>,
    pub path: String,
}

impl StatusFile {
    pub fn is_unmerged(&self) -> bool {
        match self.status_code {
            ['D', 'D']
            | ['A', 'U']
            | ['U', 'D']
            | ['U', 'A']
            | ['D', 'U']
            | ['A', 'A']
            | ['U', 'U'] => true,
            _ => false,
        }
    }

    pub fn is_untracked(&self) -> bool {
        self.status_code == ['?', '?']
    }
}

lazy_static::lazy_static! {
    static ref BRANCH_REGEX: Regex = Regex::new(
        r"^## (?<local>\S+)\.\.\.(?<remote>\S+)(?: \[(:?ahead (?<ahead>\d+))?(:?, )?(:?behind (?<behind>\d+))?\])?$",
    ).unwrap();
    static ref FILE_REGEX: Regex = Regex::new(r"^(?<code>..) (?:(?<orig_path>.*) -> )?(?<path>.*)$").unwrap();
}

impl Status {
    pub fn parse(input: &str) -> Self {
        let mut local = "".to_string();
        let mut remote = "".to_string();
        let mut ahead = 0;
        let mut behind = 0;
        let mut files = vec![];

        for line in input.lines() {
            if let Some(cap) = BRANCH_REGEX.captures(line) {
                local = cap.name("local").unwrap().as_str().to_string();
                remote = cap.name("remote").unwrap().as_str().to_string();
                ahead = cap
                    .name("ahead")
                    .map(|str| str.as_str().parse().unwrap())
                    .unwrap_or(0);
                behind = cap
                    .name("behind")
                    .map(|str| str.as_str().parse().unwrap())
                    .unwrap_or(0);
            } else if let Some(cap) = FILE_REGEX.captures(line) {
                let code = cap.name("code").unwrap().as_str();
                let chars = &mut code.chars();
                let status_code = [chars.next().unwrap(), chars.next().unwrap()];

                files.push(StatusFile {
                    status_code,
                    orig_path: cap.name("orig_path").map(|str| str.as_str().to_string()),
                    path: cap.name("path").unwrap().as_str().to_string(),
                });
            } else {
                panic!("Can't parse {}", line);
            }
        }

        Self {
            branch_status: BranchStatus {
                local,
                remote,
                ahead,
                behind,
            },
            files,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple() {
        let input = r#"
## master...origin/master
 M src/git.rs
 R foo -> bar
?? spaghet
"#
        .trim();

        assert_eq!(
            Status::parse(input),
            Status {
                branch_status: BranchStatus {
                    local: "master".to_string(),
                    remote: "origin/master".to_string(),
                    ahead: 0,
                    behind: 0
                },
                files: vec![
                    StatusFile {
                        status_code: [' ', 'M'],
                        orig_path: None,
                        path: "src/git.rs".to_string()
                    },
                    StatusFile {
                        status_code: [' ', 'R'],
                        orig_path: Some("foo".to_string()),
                        path: "bar".to_string()
                    },
                    StatusFile {
                        status_code: ['?', '?'],
                        orig_path: None,
                        path: "spaghet".to_string()
                    },
                ]
            }
        );
    }

    #[test]
    fn parse_ahead() {
        let input = r#"
## master...origin/master [ahead 1]
"#
        .trim();

        assert_eq!(
            Status::parse(input),
            Status {
                branch_status: BranchStatus {
                    local: "master".to_string(),
                    remote: "origin/master".to_string(),
                    ahead: 1,
                    behind: 0
                },
                files: vec![]
            }
        );
    }

    #[test]
    fn parse_behind() {
        let input = r#"
## master...origin/master [behind 1]
"#
        .trim();

        assert_eq!(
            Status::parse(input),
            Status {
                branch_status: BranchStatus {
                    local: "master".to_string(),
                    remote: "origin/master".to_string(),
                    ahead: 0,
                    behind: 1
                },
                files: vec![]
            }
        );
    }

    #[test]
    fn parse_diverge() {
        let input = r#"
## master...origin/master [ahead 1, behind 1]
"#
        .trim();

        assert_eq!(
            Status::parse(input),
            Status {
                branch_status: BranchStatus {
                    local: "master".to_string(),
                    remote: "origin/master".to_string(),
                    ahead: 1,
                    behind: 1
                },
                files: vec![]
            }
        );
    }
}

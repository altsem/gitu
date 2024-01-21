use pest::Parser;
use pest_derive::Parser;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Status {
    pub branch_status: BranchStatus,
    pub files: Vec<StatusFile>,
}

#[derive(Parser)]
#[grammar = "status.pest"] // relative to src
struct StatusParser;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BranchStatus {
    pub local: String,
    pub remote: Option<String>,
    pub ahead: u32,
    pub behind: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct StatusFile {
    pub status_code: [char; 2],
    pub path: String,
    pub new_path: Option<String>,
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

impl Status {
    pub fn parse(input: &str) -> Self {
        let mut local = None;
        let mut remote = None;
        let mut ahead = 0;
        let mut behind = 0;
        let mut files = vec![];

        for line in StatusParser::parse(Rule::lines, input).expect("Error parsing status") {
            match line.as_rule() {
                Rule::branch_status => {
                    for pair in line.into_inner() {
                        match pair.as_rule() {
                            Rule::local => local = Some(pair.as_str().to_string()),
                            Rule::remote => remote = Some(pair.as_str().to_string()),
                            Rule::ahead => ahead = pair.as_str().parse().unwrap(),
                            Rule::behind => behind = pair.as_str().parse().unwrap(),
                            rule => panic!("No rule {:?}", rule),
                        }
                    }
                }
                Rule::file_status => {
                    let mut status_code = None;
                    let mut path = None;
                    let mut new_path = None;

                    for pair in line.into_inner() {
                        match pair.as_rule() {
                            Rule::code => {
                                let mut chars = pair.as_str().chars();
                                status_code = Some([chars.next().unwrap(), chars.next().unwrap()]);
                            }
                            Rule::file => path = Some(pair.as_str().to_string()),
                            Rule::new_file => new_path = Some(pair.as_str().to_string()),
                            rule => panic!("No rule {:?}", rule),
                        }
                    }

                    files.push(StatusFile {
                        status_code: status_code.expect("Error parsing status_code"),
                        path: path.expect("Error parsing path"),
                        new_path,
                    });
                }
                _ => panic!("No rule {:?}", line.as_rule()),
            }
        }

        Status {
            branch_status: BranchStatus {
                local: local.expect("Error parsing local"),
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
        let input = "## master...origin/master\n M src/git.rs\n R foo -> bar\n?? spaghet\n";

        assert_eq!(
            Status::parse(input),
            Status {
                branch_status: BranchStatus {
                    local: "master".to_string(),
                    remote: Some("origin/master".to_string()),
                    ahead: 0,
                    behind: 0
                },
                files: vec![
                    StatusFile {
                        status_code: [' ', 'M'],
                        path: "src/git.rs".to_string(),
                        new_path: None,
                    },
                    StatusFile {
                        status_code: [' ', 'R'],
                        path: "foo".to_string(),
                        new_path: Some("bar".to_string())
                    },
                    StatusFile {
                        status_code: ['?', '?'],
                        path: "spaghet".to_string(),
                        new_path: None,
                    },
                ]
            }
        );
    }

    #[test]
    fn parse_ahead() {
        let input = "## master...origin/master [ahead 1]\n";

        assert_eq!(
            Status::parse(input),
            Status {
                branch_status: BranchStatus {
                    local: "master".to_string(),
                    remote: Some("origin/master".to_string()),
                    ahead: 1,
                    behind: 0
                },
                files: vec![]
            }
        );
    }

    #[test]
    fn parse_behind() {
        let input = "## master...origin/master [behind 1]\n";

        assert_eq!(
            Status::parse(input),
            Status {
                branch_status: BranchStatus {
                    local: "master".to_string(),
                    remote: Some("origin/master".to_string()),
                    ahead: 0,
                    behind: 1
                },
                files: vec![]
            }
        );
    }

    #[test]
    fn parse_diverge() {
        let input = "## master...origin/master [ahead 1, behind 1]\n";

        assert_eq!(
            Status::parse(input),
            Status {
                branch_status: BranchStatus {
                    local: "master".to_string(),
                    remote: Some("origin/master".to_string()),
                    ahead: 1,
                    behind: 1
                },
                files: vec![]
            }
        );
    }

    #[test]
    fn parse_no_remote() {
        let input = "## test.lol\n";

        assert_eq!(
            Status::parse(input),
            Status {
                branch_status: BranchStatus {
                    local: "test.lol".to_string(),
                    remote: None,
                    ahead: 0,
                    behind: 0
                },
                files: vec![]
            }
        );
    }

    #[test]
    fn unmerged() {
        let input = "## mergetest...origin/mergetest\nUU src/main.rs\n";
        assert!(Status::parse(input).files[0].is_unmerged());
    }
}

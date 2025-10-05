use std::{error::Error, str::FromStr};

use pest::Parser;
use pest_derive::Parser;

use crate::git::status::{BranchStatus, Status, StatusFile};

// TODO Get rid of this, use libgit2 instead
#[derive(Parser)]
#[grammar = "git/parse/status/status.pest"] // relative to src
struct StatusParser;

impl FromStr for Status {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut local = None;
        let mut remote = None;
        let mut ahead = 0;
        let mut behind = 0;
        let mut files = vec![];

        for line in StatusParser::parse(Rule::status_lines, s)? {
            match line.as_rule() {
                Rule::no_repo => (),
                Rule::branch_status => {
                    for pair in line.into_inner() {
                        match pair.as_rule() {
                            Rule::no_commits => (),
                            Rule::no_branch => (),
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
                            Rule::file => path = Some(pair.as_str().to_string().into()),
                            Rule::new_file => new_path = Some(pair.as_str().to_string().into()),
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

        Ok(Status {
            branch_status: BranchStatus {
                local,
                remote,
                ahead,
                behind,
            },
            files,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::git::{status::BranchStatus, status::Status, status::StatusFile};

    #[test]
    fn parse_simple() {
        let input = "## master...origin/master\n M src/git.rs\n R foo -> bar\n?? spaghet\n";

        assert_eq!(
            Status::from_str(input).unwrap(),
            Status {
                branch_status: BranchStatus {
                    local: Some("master".to_string()),
                    remote: Some("origin/master".to_string()),
                    ahead: 0,
                    behind: 0
                },
                files: vec![
                    StatusFile {
                        status_code: [' ', 'M'],
                        path: "src/git.rs".to_string().into(),
                        new_path: None,
                    },
                    StatusFile {
                        status_code: [' ', 'R'],
                        path: "foo".to_string().into(),
                        new_path: Some("bar".to_string().into())
                    },
                    StatusFile {
                        status_code: ['?', '?'],
                        path: "spaghet".to_string().into(),
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
            Status::from_str(input).unwrap(),
            Status {
                branch_status: BranchStatus {
                    local: Some("master".to_string()),
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
            Status::from_str(input).unwrap(),
            Status {
                branch_status: BranchStatus {
                    local: Some("master".to_string()),
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
            Status::from_str(input).unwrap(),
            Status {
                branch_status: BranchStatus {
                    local: Some("master".to_string()),
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
            Status::from_str(input).unwrap(),
            Status {
                branch_status: BranchStatus {
                    local: Some("test.lol".to_string()),
                    remote: None,
                    ahead: 0,
                    behind: 0
                },
                files: vec![]
            }
        );
    }

    #[test]
    fn messy_file_name() {
        let input = r#"## master...origin/master
?? "spaghet lol.testing !@#$%^&*()"
?? src/diff.pest
"#;
        assert_eq!(Status::from_str(input).unwrap().files.len(), 2);
    }

    #[test]
    fn no_branch() {
        assert_eq!(
            Status::from_str("## HEAD (no branch)\n").unwrap(),
            Status {
                branch_status: BranchStatus {
                    local: None,
                    remote: None,
                    ahead: 0,
                    behind: 0
                },
                files: vec![]
            }
        );
    }
}

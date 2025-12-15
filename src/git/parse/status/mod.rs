use std::{error::Error, str::FromStr};

use nom::{
    IResult, Parser as _,
    branch::alt,
    bytes::complete::{escaped, is_not, tag, take_until, take_while1},
    character::complete::{anychar, char, digit1, line_ending, space1},
    combinator::{map, map_res, opt, recognize, verify},
    multi::many0,
    sequence::{delimited, pair, preceded, separated_pair, terminated},
};

use crate::git::status::{BranchStatus, Status, StatusFile};

impl FromStr for Status {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse_status(s) {
            Ok((_, status)) => Ok(status),
            Err(e) => Err(format!("Failed to parse status: {:?}", e).into()),
        }
    }
}

fn parse_status(input: &str) -> IResult<&str, Status> {
    map(
        pair(
            terminated(parse_branch_line, line_ending),
            many0(terminated(parse_file_status, line_ending)),
        ),
        |(branch_status, files)| Status {
            branch_status,
            files,
        },
    )
    .parse(input)
}

fn parse_branch_line(input: &str) -> IResult<&str, BranchStatus> {
    preceded(
        tag("## "),
        alt((
            parse_no_commits,
            parse_no_branch,
            parse_branch_status,
            map(tag(""), |_| BranchStatus {
                local: None,
                remote: None,
                ahead: 0,
                behind: 0,
            }),
        )),
    )
    .parse(input)
}

fn parse_no_commits(input: &str) -> IResult<&str, BranchStatus> {
    map(preceded(tag("No commits yet on "), parse_branch), |local| {
        BranchStatus {
            local: Some(local.to_string()),
            remote: None,
            ahead: 0,
            behind: 0,
        }
    })
    .parse(input)
}

fn parse_no_branch(input: &str) -> IResult<&str, BranchStatus> {
    map(tag("HEAD (no branch)"), |_| BranchStatus {
        local: None,
        remote: None,
        ahead: 0,
        behind: 0,
    })
    .parse(input)
}

fn parse_branch_status(input: &str) -> IResult<&str, BranchStatus> {
    map(
        (
            parse_branch,
            opt(preceded(tag("..."), parse_branch)),
            opt(preceded(space1, parse_ahead_behind)),
        ),
        |(local, remote, ahead_behind)| {
            let (ahead, behind) = ahead_behind.unwrap_or((0, 0));
            BranchStatus {
                local: Some(local.to_string()),
                remote: remote.map(|s| s.to_string()),
                ahead,
                behind,
            }
        },
    )
    .parse(input)
}

fn parse_branch(input: &str) -> IResult<&str, &str> {
    // Parse branch name - take until we hit "...", space, or newline
    alt((
        take_until("..."),
        take_while1(|c: char| c != ' ' && c != '\n' && c != '\r'),
    ))
    .parse(input)
}

fn parse_ahead_behind(input: &str) -> IResult<&str, (u32, u32)> {
    alt((
        map(tag("[gone]"), |_| (0, 0)),
        delimited(
            char('['),
            map(
                separated_pair(
                    opt(parse_ahead_count),
                    opt(tag(", ")),
                    opt(parse_behind_count),
                ),
                |(ahead, behind)| (ahead.unwrap_or(0), behind.unwrap_or(0)),
            ),
            char(']'),
        ),
    ))
    .parse(input)
}

fn parse_ahead_count(input: &str) -> IResult<&str, u32> {
    map_res(preceded(tag("ahead "), digit1), |s: &str| s.parse::<u32>()).parse(input)
}

fn parse_behind_count(input: &str) -> IResult<&str, u32> {
    map_res(preceded(tag("behind "), digit1), |s: &str| s.parse::<u32>()).parse(input)
}

fn parse_file_status(input: &str) -> IResult<&str, StatusFile> {
    map(
        (
            parse_status_code,
            space1,
            parse_file_path,
            opt(preceded(tag(" -> "), parse_file_path)),
        ),
        |(status_code, _, path, new_path)| StatusFile {
            status_code,
            path: unescape(path.to_string()),
            new_path: new_path.map(|p| unescape(p.to_string())),
        },
    )
    .parse(input)
}

fn parse_status_code(input: &str) -> IResult<&str, [char; 2]> {
    map(pair(parse_status_char, parse_status_char), |(a, b)| [a, b]).parse(input)
}

fn parse_status_char(input: &str) -> IResult<&str, char> {
    verify(anychar, |c| {
        c.is_ascii_alphabetic() || *c == '?' || *c == '!' || *c == ' '
    })
    .parse(input)
}

fn parse_file_path(input: &str) -> IResult<&str, &str> {
    alt((
        // Quoted path with escapes - return the whole thing including quotes
        recognize(delimited(
            char('"'),
            escaped(is_not("\"\\"), '\\', anychar),
            char('"'),
        )),
        // Unquoted path
        take_while1(|c: char| c != '\n' && c != '\r' && c != ' '),
    ))
    .parse(input)
}

fn unescape(path: String) -> String {
    if path.starts_with('"') && path.ends_with('"') {
        String::from_utf8_lossy(
            &smashquote::unescape_bytes(&path.as_bytes()[1..path.len() - 1]).unwrap(),
        )
        .into_owned()
    } else {
        path
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

    #[test]
    fn branch_with_multiple_dots() {
        let input = "## feature.v1.2.3\n";
        let result = Status::from_str(input).unwrap();
        assert_eq!(
            result.branch_status.local,
            Some("feature.v1.2.3".to_string())
        );
        assert_eq!(result.branch_status.remote, None);
    }

    #[test]
    fn branch_with_dots_and_remote() {
        let input = "## feature.v1.2...origin/feature.v1.2\n";
        let result = Status::from_str(input).unwrap();
        assert_eq!(result.branch_status.local, Some("feature.v1.2".to_string()));
        assert_eq!(
            result.branch_status.remote,
            Some("origin/feature.v1.2".to_string())
        );
    }

    #[test]
    fn branch_with_slashes() {
        let input = "## feature/cool-stuff...origin/feature/cool-stuff\n";
        let result = Status::from_str(input).unwrap();
        assert_eq!(
            result.branch_status.local,
            Some("feature/cool-stuff".to_string())
        );
        assert_eq!(
            result.branch_status.remote,
            Some("origin/feature/cool-stuff".to_string())
        );
    }

    #[test]
    fn branch_with_underscores_and_dashes() {
        let input = "## my_branch-v2...origin/my_branch-v2\n";
        let result = Status::from_str(input).unwrap();
        assert_eq!(result.branch_status.local, Some("my_branch-v2".to_string()));
        assert_eq!(
            result.branch_status.remote,
            Some("origin/my_branch-v2".to_string())
        );
    }

    #[test]
    fn large_ahead_behind_counts() {
        let input = "## master...origin/master [ahead 123, behind 456]\n";
        let result = Status::from_str(input).unwrap();
        assert_eq!(result.branch_status.ahead, 123);
        assert_eq!(result.branch_status.behind, 456);
    }

    #[test]
    fn gone_remote_branch() {
        let input = "## feature...origin/feature [gone]\n";
        let result = Status::from_str(input).unwrap();
        assert_eq!(result.branch_status.local, Some("feature".to_string()));
        assert_eq!(
            result.branch_status.remote,
            Some("origin/feature".to_string())
        );
        assert_eq!(result.branch_status.ahead, 0);
        assert_eq!(result.branch_status.behind, 0);
    }

    #[test]
    fn no_commits_yet() {
        let input = "## No commits yet on main\n";
        let result = Status::from_str(input).unwrap();
        assert_eq!(result.branch_status.local, Some("main".to_string()));
        assert_eq!(result.branch_status.remote, None);
    }

    #[test]
    fn all_file_status_codes() {
        let input = r#"## master
 M modified
 D deleted
 R renamed -> newname
 C copied -> copiedname
?? untracked
!! ignored
MM staged-and-modified
AM added-and-modified
DM deleted-and-modified
RM renamed-and-modified -> renamed-target
A  added
D  deleted-staged
M  modified-staged
R  renamed-staged -> renamed-staged-target
C  copied-staged -> copied-staged-target
"#;
        let result = Status::from_str(input).unwrap();
        assert_eq!(result.files.len(), 15);

        // Check specific status codes
        assert_eq!(result.files[0].status_code, [' ', 'M']);
        assert_eq!(result.files[0].path, "modified");

        assert_eq!(result.files[1].status_code, [' ', 'D']);
        assert_eq!(result.files[2].status_code, [' ', 'R']);
        assert_eq!(result.files[2].new_path, Some("newname".to_string()));

        assert_eq!(result.files[5].status_code, ['!', '!']);
        assert_eq!(result.files[6].status_code, ['M', 'M']);
    }

    #[test]
    fn file_with_spaces() {
        let input = r#"## master
?? "file with spaces.txt"
"#;
        let result = Status::from_str(input).unwrap();
        assert_eq!(result.files[0].path, "file with spaces.txt");
    }

    #[test]
    fn file_with_tabs() {
        let input = "## master\n?? \"file\\twith\\ttabs.txt\"\n";
        let result = Status::from_str(input).unwrap();
        // Git escapes tabs but our unescape should handle it
        assert_eq!(result.files[0].path, "file\twith\ttabs.txt");
    }

    #[test]
    fn file_with_newlines_escaped() {
        let input = "## master\n?? \"file\\nwith\\nnewlines.txt\"\n";
        let result = Status::from_str(input).unwrap();
        // Git escapes newlines but our unescape should handle it
        assert_eq!(result.files[0].path, "file\nwith\nnewlines.txt");
    }

    #[test]
    fn file_with_quotes() {
        let input = r#"## master
?? "file\"with\"quotes.txt"
"#;
        let result = Status::from_str(input).unwrap();
        // Git escapes quotes but our unescape should handle it
        assert_eq!(result.files[0].path, r#"file"with"quotes.txt"#);
    }

    #[test]
    fn file_with_backslashes() {
        let input = r#"## master
?? "file\\with\\backslashes.txt"
"#;
        let result = Status::from_str(input).unwrap();
        // Git escapes backslashes but our unescape should handle it
        assert_eq!(result.files[0].path, r"file\with\backslashes.txt");
    }

    #[test]
    fn file_with_unicode() {
        let input = "## master\n?? \"file-with-émojis-🎉.txt\"\n";
        let result = Status::from_str(input).unwrap();
        assert_eq!(result.files[0].path, "file-with-émojis-🎉.txt");
    }

    #[test]
    fn file_with_unicode_unquoted() {
        let input = "## master\n?? file-émojis-🎉.txt\n";
        let result = Status::from_str(input).unwrap();
        assert_eq!(result.files[0].path, "file-émojis-🎉.txt");
    }

    #[test]
    fn renamed_file_both_quoted() {
        let input = r#"## master
 R "old name.txt" -> "new name.txt"
"#;
        let result = Status::from_str(input).unwrap();
        assert_eq!(result.files[0].path, "old name.txt");
        assert_eq!(result.files[0].new_path, Some("new name.txt".to_string()));
    }

    #[test]
    fn renamed_file_with_dots() {
        let input = "## master\n R old.file.name -> new.file.name\n";
        let result = Status::from_str(input).unwrap();
        assert_eq!(result.files[0].path, "old.file.name");
        assert_eq!(result.files[0].new_path, Some("new.file.name".to_string()));
    }

    #[test]
    fn file_in_subdirectory() {
        let input = "## master\n?? src/module/file.rs\n";
        let result = Status::from_str(input).unwrap();
        assert_eq!(result.files[0].path, "src/module/file.rs");
    }

    #[test]
    fn file_in_deep_subdirectory() {
        let input = "## master\n?? deeply/nested/directory/structure/file.txt\n";
        let result = Status::from_str(input).unwrap();
        assert_eq!(
            result.files[0].path,
            "deeply/nested/directory/structure/file.txt"
        );
    }

    #[test]
    fn multiple_files_various_statuses() {
        let input = r#"## develop...origin/develop [ahead 2, behind 3]
MM src/main.rs
A  src/new_module.rs
 M README.md
 D deprecated.txt
R  old.rs -> new.rs
?? untracked.txt
"#;
        let result = Status::from_str(input).unwrap();
        assert_eq!(result.branch_status.local, Some("develop".to_string()));
        assert_eq!(
            result.branch_status.remote,
            Some("origin/develop".to_string())
        );
        assert_eq!(result.branch_status.ahead, 2);
        assert_eq!(result.branch_status.behind, 3);
        assert_eq!(result.files.len(), 6);
    }

    #[test]
    fn empty_status_with_remote() {
        let input = "## main...origin/main\n";
        let result = Status::from_str(input).unwrap();
        assert_eq!(result.files.len(), 0);
        assert_eq!(result.branch_status.local, Some("main".to_string()));
        assert_eq!(result.branch_status.remote, Some("origin/main".to_string()));
    }

    #[test]
    fn branch_with_numbers() {
        let input = "## release-2024-01...origin/release-2024-01\n";
        let result = Status::from_str(input).unwrap();
        assert_eq!(
            result.branch_status.local,
            Some("release-2024-01".to_string())
        );
    }

    #[test]
    fn remote_with_namespace() {
        let input = "## master...upstream/namespace/master\n";
        let result = Status::from_str(input).unwrap();
        assert_eq!(
            result.branch_status.remote,
            Some("upstream/namespace/master".to_string())
        );
    }

    #[test]
    fn file_with_special_chars_in_name() {
        let input = r#"## master
?? "file!@#$%^&*().txt"
"#;
        let result = Status::from_str(input).unwrap();
        assert_eq!(result.files[0].path, "file!@#$%^&*().txt");
    }

    #[test]
    fn file_starting_with_dash() {
        let input = "## master\n?? \"-file.txt\"\n";
        let result = Status::from_str(input).unwrap();
        assert_eq!(result.files[0].path, "-file.txt");
    }

    #[test]
    fn file_starting_with_dot() {
        let input = "## master\n?? .hidden-file\n";
        let result = Status::from_str(input).unwrap();
        assert_eq!(result.files[0].path, ".hidden-file");
    }

    #[test]
    fn mixed_quoted_and_unquoted_files() {
        let input = r#"## master
?? normal-file.txt
?? "file with spaces.txt"
?? another-normal.txt
?? "special!@#.txt"
"#;
        let result = Status::from_str(input).unwrap();
        assert_eq!(result.files.len(), 4);
        assert_eq!(result.files[0].path, "normal-file.txt");
        assert_eq!(result.files[1].path, "file with spaces.txt");
        assert_eq!(result.files[2].path, "another-normal.txt");
        assert_eq!(result.files[3].path, "special!@#.txt");
    }

    #[test]
    fn only_ahead() {
        let input = "## master...origin/master [ahead 10]\n";
        let result = Status::from_str(input).unwrap();
        assert_eq!(result.branch_status.ahead, 10);
        assert_eq!(result.branch_status.behind, 0);
    }

    #[test]
    fn only_behind() {
        let input = "## master...origin/master [behind 20]\n";
        let result = Status::from_str(input).unwrap();
        assert_eq!(result.branch_status.ahead, 0);
        assert_eq!(result.branch_status.behind, 20);
    }

    #[test]
    fn file_name_with_arrow_in_quotes() {
        let input = r#"## master
?? "file -> with -> arrows.txt"
"#;
        let result = Status::from_str(input).unwrap();
        assert_eq!(result.files[0].path, "file -> with -> arrows.txt");
        assert_eq!(result.files[0].new_path, None);
    }

    #[test]
    fn complex_rename_scenario() {
        let input = r#"## master
 R "old -> file.txt" -> "new -> file.txt"
"#;
        let result = Status::from_str(input).unwrap();
        assert_eq!(result.files[0].path, "old -> file.txt");
        assert_eq!(
            result.files[0].new_path,
            Some("new -> file.txt".to_string())
        );
    }

    #[test]
    fn branch_name_edge_cases() {
        // Test various valid git branch names
        let test_cases = vec![
            "feature/ABC-123",
            "hotfix-2024.01.15",
            "user/john.doe/feature",
            "v1.2.3-rc.1",
            "123-numeric-prefix",
        ];

        for branch in test_cases {
            let input = format!("## {}\n", branch);
            let result = Status::from_str(&input).unwrap();
            assert_eq!(result.branch_status.local, Some(branch.to_string()));
        }
    }

    #[test]
    fn stress_test_many_files() {
        let mut input = String::from("## master\n");
        for i in 0..100 {
            input.push_str(&format!("?? file{}.txt\n", i));
        }
        let result = Status::from_str(&input).unwrap();
        assert_eq!(result.files.len(), 100);
    }

    #[test]
    fn file_with_consecutive_spaces() {
        let input = r#"## master
?? "file  with   multiple    spaces.txt"
"#;
        let result = Status::from_str(input).unwrap();
        assert_eq!(result.files[0].path, "file  with   multiple    spaces.txt");
    }

    #[test]
    fn all_status_combinations() {
        // Test common two-character status codes
        let status_chars = vec![' ', 'M', 'A', 'D', 'R', 'C', '?', '!'];
        let mut input = String::from("## master\n");
        let mut count = 0;

        for c1 in &status_chars {
            for c2 in &status_chars {
                input.push(*c1);
                input.push(*c2);
                input.push_str(&format!(
                    " file_{}_{}.txt\n",
                    if *c1 == ' ' { "space" } else { &c1.to_string() },
                    if *c2 == ' ' { "space" } else { &c2.to_string() }
                ));
                count += 1;
            }
        }

        let result = Status::from_str(&input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().files.len(), count);
    }
}

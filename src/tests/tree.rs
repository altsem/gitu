//! Tests for the commit tree view (`git log --graph` layout).
//!
//! Topologies are described by a spec: `;`-separated lists of the parent
//! indices of each commit, in display order (commit 0 is the newest, e.g.
//! the current HEAD). Parents always have a higher index than their child.
//! e.g. `"1 2;3;4;4;"` is a merge of two diverging lines:
//!
//! ```text
//!      0
//!     / \
//!    1   2
//!   /     \
//!  3 ----- 4
//! ```
//!
//! Verification strategy:
//! - The graph engine is checked against real `git log --graph`: it is fed
//!   the exact commit order git displays and must reproduce git's output
//!   byte for byte (graph prefix per line, and which line is the commit
//!   line).
//! - `tree_roots` is checked separately: it walks in the same order as
//!   `git log <revs>`, and its rows must carry the right graph prefix and
//!   commit data. Note that `git log --graph` itself can
//!   display commits in a different order (its graph lookahead shows the
//!   second-parent line of a merge before the first-parent line continues),
//!   so a byte-for-byte comparison of `tree_roots` against
//!   `git log --graph` is only valid for topologies where both orders agree;
//!   the fuzz test covers those end to end.

use std::{
    collections::HashMap,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use temp_dir::TempDir;

use regex::Regex;

use crate::git::tree;

/// Commits of the spec, as parent-index lists.
fn spec_parents(spec: &str) -> Vec<Vec<u32>> {
    spec.split(';')
        .map(|s| {
            s.split_whitespace()
                .map(|p| p.parse().unwrap())
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Builds a repo from a spec using `git commit-tree` plumbing, so commits can
/// have arbitrary parents (including unrelated histories). Committer dates
/// increase with the spec index (commit 0 is newest), so the plain
/// (date-ordered) revision walk agrees with the spec order. Returns the
/// commits' oids by spec index (commit 0 is newest).
fn build_spec_repo(spec: &str) -> (TempDir, git2::Repository, Vec<String>) {
    let parents = spec_parents(spec);
    let dir = TempDir::new().unwrap();
    let dir_path = dir.path();

    git(
        dir_path,
        None,
        &["git", "init", "-q", "--initial-branch=main"],
    );
    git(dir_path, None, &["git", "config", "user.name", "CI"]);
    git(
        dir_path,
        None,
        &["git", "config", "user.email", "ci@example.com"],
    );

    let tree_oid = git(dir_path, None, &["git", "mktree"]);
    let tree_oid = tree_oid.trim().to_string();

    // Oids by spec index; commits are created bottom-up (parents first).
    let mut oids: Vec<Option<String>> = vec![None; parents.len()];
    for i in (0..parents.len()).rev() {
        let date = format!("2025-01-01T{:02}:00:00+00:00", 23 - i);
        let mut args: Vec<&str> = vec!["git", "commit-tree", &tree_oid];
        for &p in &parents[i] {
            args.push("-p");
            args.push(oids[p as usize].as_ref().unwrap());
        }
        let oid = git_with_input(
            dir_path,
            Some(&date),
            &args,
            format!("commit {i}\n").as_bytes(),
        );
        oids[i] = Some(oid.trim().to_string());
    }
    git(
        dir_path,
        None,
        &[
            "git",
            "update-ref",
            "refs/heads/main",
            oids[0].as_ref().unwrap(),
        ],
    );

    let repo = git2::Repository::open(dir_path).unwrap();
    (dir, repo, oids.into_iter().map(Option::unwrap).collect())
}

fn git(dir: &Path, date: Option<&str>, args: &[&str]) -> String {
    git_with_input(dir, date, args, b"")
}

fn git_with_input(dir: &Path, date: Option<&str>, args: &[&str], input: &[u8]) -> String {
    let mut cmd = Command::new(args[0]);
    cmd.args(&args[1..]).current_dir(dir);
    // Isolate from the host git config and any env vars other tests set.
    cmd.env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_AUTHOR_NAME", "CI")
        .env("GIT_AUTHOR_EMAIL", "ci@example.com")
        .env("GIT_COMMITTER_NAME", "CI")
        .env("GIT_COMMITTER_EMAIL", "ci@example.com")
        .env("LC_ALL", "C")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());
    if let Some(date) = date {
        cmd.env("GIT_AUTHOR_DATE", date)
            .env("GIT_COMMITTER_DATE", date);
    }
    let mut child = cmd
        .spawn()
        .unwrap_or_else(|e| panic!("failed to run {args:?}: {e}"));
    child.stdin.as_mut().unwrap().write_all(input).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "failed to run {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

/// Position of a full 40-char sha1 in a `--graph` line, if any.
fn find_hash(l: &str) -> Option<usize> {
    l.find(|c: char| c.is_ascii_hexdigit()).filter(|&p| {
        p + 40 <= l.len()
            && l[p..p + 40].chars().all(|c| c.is_ascii_hexdigit())
            && (l.as_bytes().get(p + 40).is_none() || l.as_bytes()[p + 40] == b' ')
    })
}

/// Parses `git log --graph --format=%H` into (prefix, Some(oid)) lines.
fn parse_graph(out: &str) -> Vec<(String, Option<String>)> {
    out.lines()
        .map(|l| match find_hash(l) {
            Some(p) if l[p..].trim_end().len() == 40 => (
                l[..p].trim_end().to_string(),
                Some(l[p..p + 40].to_string()),
            ),
            _ => (l.trim_end().to_string(), None),
        })
        .collect()
}

/// The commit order of a plain (date-ordered) `git log` for the given revs,
/// as oids. Empty `revs` means HEAD.
fn git_log_order(dir: &Path, revs: &[&str]) -> Vec<String> {
    let mut args: Vec<&str> = vec!["git", "log", "--format=%H"];
    args.extend(revs);
    git(dir, None, &args)
        .lines()
        .map(|s| s.to_string())
        .collect()
}

/// The commit order git displays for a `--graph` log of the given revs, as
/// (oid, parents). Empty `revs` means HEAD.
fn git_graph_order(dir: &Path, revs: &[&str]) -> Vec<(String, Vec<String>)> {
    let mut args: Vec<&str> = vec!["git", "log", "--graph", "--format=%H %P"];
    args.extend(revs);
    git(dir, None, &args)
        .lines()
        .filter_map(|l| {
            let p = find_hash(l)?;
            let oid = l[p..p + 40].to_string();
            let parents = l[p + 40..]
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
            Some((oid, parents))
        })
        .collect()
}

/// Runs the graph engine over the given (oid, parents) display order and
/// returns its lines as (prefix, Some(oid)).
fn engine_lines_for(commits: &[(String, Vec<String>)]) -> Vec<(String, Option<String>)> {
    let index: HashMap<String, usize> = commits
        .iter()
        .enumerate()
        .map(|(i, (o, _))| (o.clone(), i))
        .collect();

    let parents: Vec<Vec<usize>> = commits
        .iter()
        .map(|(_, parents)| {
            parents
                .iter()
                .filter_map(|p| index.get(p.as_str()).copied())
                .collect()
        })
        .collect();

    tree::Graph::rows(parents)
        .into_iter()
        .map(|(line, i)| (line.trim_end().to_string(), i.map(|i| commits[i].0.clone())))
        .collect()
}

/// Check 1: the engine, fed the commit order git displays, reproduces
/// `git log --graph` byte for byte.
fn assert_engine_matches_git(dir: &Path, spec: &str) {
    let theirs = parse_graph(&git(dir, None, &["git", "log", "--graph", "--format=%H"]));
    let commits = git_graph_order(dir, &[]);
    let mine = engine_lines_for(&commits);
    assert_eq!(
        mine,
        theirs,
        "engine output differs from `git log --graph` for spec {spec:?}\n--- engine ---\n{}\n--- git ---\n{}",
        lines_str(&mine),
        lines_str(&theirs),
    );
}

/// Check 2: `tree_roots` walks in the same order as `git log <revs>` and
/// attaches the right graph prefix and commit data to each row.
fn assert_rows(dir: &Path, revs: &[&str], spec: &str, rows: &[tree::TreeRow]) {
    let plain = git_log_order(dir, revs);
    let row_oids: Vec<String> = rows
        .iter()
        .filter_map(|r| r.commit.as_ref().map(|c| c.oid.clone()))
        .collect();

    // The walk order (rows that carry a commit) must match plain `git log`
    // in every case, including multiple roots.
    assert_eq!(row_oids, plain, "walk order for spec {spec:?}");

    // Expected rows: `git log --graph` itself when its commit order equals
    // the app's walk order, otherwise the engine run over the walk order
    // (the layout is covered by check 1; this checks tree_roots' plumbing).
    let mut graph_args: Vec<&str> = vec!["git", "log", "--graph", "--format=%H"];
    graph_args.extend(revs);
    let graph = parse_graph(&git(dir, None, &graph_args));
    let graph_oids: Vec<String> = graph.iter().filter_map(|(_, o)| o.clone()).collect();
    let expected: Vec<(String, Option<String>)> = if graph_oids == row_oids {
        graph
    } else {
        let mut log_args: Vec<&str> = vec!["git", "log", "--format=%H %P"];
        log_args.extend(revs);
        let log_out = git(dir, None, &log_args);
        let by_oid: HashMap<&str, Vec<String>> = log_out
            .lines()
            .map(|l| {
                let (o, ps) = l.split_once(' ').unwrap();
                (
                    o,
                    ps.split_whitespace().map(String::from).collect::<Vec<_>>(),
                )
            })
            .collect();
        // Rebuild the commit list in the app's walk order.
        let commits: Vec<(String, Vec<String>)> = row_oids
            .iter()
            .map(|o| (o.clone(), by_oid[o.as_str()].clone()))
            .collect();
        engine_lines_for(&commits)
    };
    assert_eq!(rows.len(), expected.len(), "row count for spec {spec:?}");
    for (i, (row, (want_prefix, want_oid))) in rows.iter().zip(expected.iter()).enumerate() {
        assert_eq!(
            row.graph.trim_end(),
            want_prefix,
            "row {i} graph for spec {spec:?}"
        );
        assert_eq!(
            row.commit.as_ref().map(|c| c.oid.clone()),
            want_oid.clone(),
            "row {i} commit for spec {spec:?}"
        );
    }

    // Per-commit data.
    let mut data_args: Vec<&str> = vec!["git", "log", "--format=%H%x00%s%x00%an"];
    data_args.extend(revs);
    let expected_data: HashMap<String, (String, String)> = git(dir, None, &data_args)
        .lines()
        .map(|l| {
            let (h, rest) = l.split_once('\0').unwrap();
            let (s, a) = rest.split_once('\0').unwrap();
            (h.to_string(), (s.to_string(), a.to_string()))
        })
        .collect();
    let refs: HashMap<String, Vec<String>> = {
        let out = git(
            dir,
            None,
            &["git", "for-each-ref", "--format=%(objectname) %(refname)"],
        );
        let mut m: HashMap<String, Vec<String>> = HashMap::new();
        for l in out.lines() {
            let (oid, name) = l.split_once(' ').unwrap();
            m.entry(oid.to_string()).or_default().push(name.to_string());
        }
        m
    };

    for (i, row) in rows.iter().enumerate() {
        let Some(commit) = row.commit.as_ref() else {
            continue;
        };
        let (summary, author) = &expected_data[&commit.oid];
        assert_eq!(&commit.summary, summary, "row {i} summary");
        assert_eq!(&commit.author, author, "row {i} author");
        assert!(!commit.age.is_empty(), "row {i} age");
        assert_eq!(
            commit.short_id,
            git(dir, None, &["git", "rev-parse", "--short", &commit.oid]).trim(),
            "row {i} short_id"
        );
        let want_refs: Vec<String> = refs
            .get(&commit.oid)
            .map(|names| {
                names
                    .iter()
                    .map(|n| {
                        if let Some(b) = n.strip_prefix("refs/heads/") {
                            format!("Head({b})")
                        } else if let Some(b) = n.strip_prefix("refs/tags/") {
                            format!("Tag({b})")
                        } else if let Some(b) = n.strip_prefix("refs/remotes/") {
                            format!("Remote({b})")
                        } else {
                            format!("?({n})")
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();
        let got_refs: Vec<String> = commit
            .associated_references
            .iter()
            .map(|r| match r {
                crate::item_data::Ref::Head(n) => format!("Head({n})"),
                crate::item_data::Ref::Tag(n) => format!("Tag({n})"),
                crate::item_data::Ref::Remote(n) => format!("Remote({n})"),
                crate::item_data::Ref::Other(n) => format!("Other({n})"),
            })
            .collect();
        let mut got_refs = got_refs;
        let mut want_refs = want_refs;
        got_refs.sort();
        want_refs.sort();
        assert_eq!(got_refs, want_refs, "row {i} refs for spec {spec:?}");
    }
}

/// `tree_roots` drops commits whose message does not match `msg_regex`,
/// and the surviving graph simply ends where a filtered commit used to
/// continue it.
#[test]
fn tree_roots_message_filter() {
    //     0
    //    / \
    //  1   2
    //  |   |
    //  3   4
    const SPEC: &str = "1 2;3;4;4;";
    let (_dir, repo, oids) = build_spec_repo(SPEC);

    // Only commits 1 and 4 survive; both used to descend into 3, so the
    // tree comes out as two disconnected lines.
    let re = Regex::new("commit [14]").unwrap();
    let head = git2::Oid::from_str(&oids[0]).unwrap();
    let rows = tree::tree_roots(&repo, usize::MAX, &[head], Some(re)).unwrap();
    let got: Vec<&str> = rows
        .iter()
        .filter_map(|row| row.commit.as_ref().map(|commit| commit.oid.as_str()))
        .collect();
    assert_eq!(got, vec![oids[1].as_str(), oids[4].as_str()]);
}

/// `limit` counts displayed commits, so it is applied after `msg_regex`
/// filters, like `git log -n N --grep`.
#[test]
fn tree_roots_limit_after_filter() {
    const SPEC: &str = "1;2;3;4;";
    let (_dir, repo, oids) = build_spec_repo(SPEC);
    let head = git2::Oid::from_str(&oids[0]).unwrap();

    // Messages are "commit 0" through "commit 4"; keep only the odd ones
    // and limit to two: both matching commits survive the limit.
    let re = Regex::new("commit [13]").unwrap();
    let rows = tree::tree_roots(&repo, 2, &[head], Some(re)).unwrap();
    let got: Vec<&str> = rows
        .iter()
        .filter_map(|row| row.commit.as_ref().map(|commit| commit.oid.as_str()))
        .collect();
    assert_eq!(got, vec![oids[1].as_str(), oids[3].as_str()]);
}

/// The rows of `tree_roots` rooted at HEAD (the single-rev case the log
/// screen starts from).
fn assert_head_rows(dir: &Path, repo: &git2::Repository, spec: &str) {
    let head = repo.head().unwrap().peel_to_commit().unwrap().id();
    let rows = tree::tree_roots(repo, usize::MAX, &[head], None).unwrap();
    assert_rows(dir, &["HEAD"], spec, &rows);
}

fn lines_str(lines: &[(String, Option<String>)]) -> String {
    lines
        .iter()
        .map(|(p, o)| format!("{p}{o:?}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Runs both checks against a spec-built repo.
fn assert_tree_matches_git(spec: &str) {
    let (dir, repo, _oids) = build_spec_repo(spec);
    assert_engine_matches_git(dir.path(), spec);
    assert_head_rows(dir.path(), &repo, spec);
}

/// A tree of all local branch tips (`local_branch_roots` + `tree_roots`),
/// the path behind the log screen's "local branches" op.
#[test]
fn tree_of_local_branches() {
    //      0 (main)
    //     / \
    //   1   2 (right)
    //  /     \
    // 3 (left) 4
    const SPEC: &str = "1 2;3;4;4;";
    let (dir, repo, _oids) = build_spec_repo(SPEC);
    let dir = dir.path();
    let plain = git_log_order(dir, &[]);
    git(dir, None, &["git", "branch", "left", &plain[3]]);
    git(dir, None, &["git", "branch", "right", &plain[2]]);

    // Every local branch tip is a root, no more and no less.
    let roots = tree::local_branch_roots(&repo);
    let mut root_oids: Vec<String> = roots.iter().map(|r| r.to_string()).collect();
    root_oids.sort();
    let mut want_roots = vec![plain[0].clone(), plain[2].clone(), plain[3].clone()];
    want_roots.sort();
    assert_eq!(root_oids, want_roots, "local branch roots");

    // The tree of those roots matches `git log` over the same refs, in the
    // same root order.
    let name_by_oid: HashMap<&str, &str> = [
        (plain[0].as_str(), "main"),
        (plain[2].as_str(), "right"),
        (plain[3].as_str(), "left"),
    ]
    .into_iter()
    .collect();
    let revs: Vec<&str> = roots
        .iter()
        .map(|r| name_by_oid[r.to_string().as_str()])
        .collect();
    let rows = tree::tree_roots(&repo, usize::MAX, &roots, None).unwrap();
    assert_rows(dir, &revs, SPEC, &rows);
}

/// Fuzz: trees of up to 3 local branch tips over random topologies, checked
/// against `git log` over the same refs (and the graph layout against
/// `git log --graph`, as in check 1).
#[test]
fn random_branch_trees() {
    let mut rng: u64 = 0x9e3779b97f4a7c15;
    let mut next = move || {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (rng >> 33) as usize
    };

    for _ in 0..20 {
        let n = 4 + next() % 12;
        // Linear chain (commit i has parent i+1), then random extra
        // parents, which creates merges/octopuses.
        let mut parents: Vec<Vec<u32>> = (0..n).map(|i| vec![(i + 1) as u32]).collect();
        parents[n - 1] = vec![];
        for _ in 0..(3 * n / 2) {
            let i = next() % (n - 1);
            let j = i + 1 + next() % (n - i - 1);
            if !parents[i].contains(&(j as u32)) && parents[i].len() < 5 {
                parents[i].push(j as u32);
                parents[i].sort_unstable();
            }
        }
        let spec = parents
            .iter()
            .map(|ps| {
                ps.iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>()
            .join(";");

        // Pick 1-2 extra branch tips (main always points at commit 0).
        let k = 1 + next() % 2;
        let mut candidates: Vec<u32> = (1..n as u32).collect();
        for i in (1..candidates.len()).rev() {
            let j = next() % (i + 1);
            candidates.swap(i, j);
        }
        let tips: Vec<u32> = candidates.into_iter().take(k).collect();

        let (dir, repo, _oids) = build_spec_repo(&spec);
        let dir = dir.path();
        let plain = git_log_order(dir, &[]);
        let mut name_by_oid: HashMap<String, String> =
            [(plain[0].clone(), "main".to_string())].into();
        for (j, &tip) in tips.iter().enumerate() {
            let name = format!("t{j}");
            git(dir, None, &["git", "branch", &name, &plain[tip as usize]]);
            name_by_oid.insert(plain[tip as usize].clone(), name);
        }

        assert_engine_matches_git(dir, &spec);

        let roots = tree::local_branch_roots(&repo);
        let revs: Vec<String> = roots
            .iter()
            .map(|r| name_by_oid.get(&r.to_string()).unwrap().clone())
            .collect();
        let rev_refs: Vec<&str> = revs.iter().map(String::as_str).collect();
        let rows = tree::tree_roots(&repo, usize::MAX, &roots, None).unwrap();
        assert_rows(dir, &rev_refs, &spec, &rows);
    }
}

/// Byte-for-byte check against git for a collapse row that has nothing to
/// its left: that row is ` /`, not `|/` (in `|/` the `|` is a different,
/// surviving line). The shape needs a root commit shown mid-walk, its line
/// dying, while another line is still live to its right:
///
/// ```text
///       D (main)
///      / \
///     X   Y - E (tip)
/// ```
///
/// Walk order D, X, E, Y: X (a root) is shown at the leftmost slot and its
/// line dies, and Y's line collapses left onto the now-empty slot.
#[test]
fn collapse_row_with_nothing_to_the_left_is_a_space() {
    const SPEC: &str = "1 3;;3;";
    let (dir, repo, oids) = build_spec_repo(SPEC);
    let dir = dir.path();
    // E (spec index 2) is a child of Y and unreachable from main, so it is
    // not in `git log`; branch from the spec oid directly.
    git(dir, None, &["git", "branch", "tip", &oids[2]]);

    let roots = tree::local_branch_roots(&repo);
    let rows = tree::tree_roots(&repo, usize::MAX, &roots, None).unwrap();
    assert_rows(dir, &["main", "tip"], SPEC, &rows);

    assert!(
        rows.iter()
            .any(|r| r.commit.is_none() && r.graph.trim_end() == " /"),
        "expected a ` /` collapse row (no line to its left), got: {}",
        rows.iter()
            .map(|r| format!("{:?}", r.graph.trim_end()))
            .collect::<Vec<_>>()
            .join("\n"),
    );
}

#[test]
fn linear_history() {
    assert_tree_matches_git("1;2;");
}

#[test]
fn merge_of_diverging_lines() {
    //     0
    //    / \
    //   1   2
    //  /     \
    // 3 ----- 4
    assert_tree_matches_git("1 2;3;4;4;");
}

#[test]
fn merge_of_unrelated_roots() {
    // 0 merges two commits with no common history.
    assert_tree_matches_git("1 2;;;");
}

#[test]
fn octopus_merge_of_unrelated_roots() {
    assert_tree_matches_git("1 2 3;;;;");
}

#[test]
fn nested_octopus() {
    // 0 is a 4-way merge; its second parent is itself a 4-way merge.
    assert_tree_matches_git("1 2 3 4;5;5;5;5;6 7 8 9;10;10;10;10;;");
}

#[test]
fn stacked_merges() {
    // 0 merges 1 and 2, and 2 is itself a merge.
    assert_tree_matches_git("1 2;3;3 4;5;5;");
}

#[test]
fn complex_history() {
    // Long line with a side branch, a deep second side branch, and two
    // merges at different depths (crossing `|` lines and a `_|_` collapse).
    assert_tree_matches_git("1 5;3;3;4;5;6;7;9;9;10;11;12;13;");
}

/// Check 3: end-to-end byte-for-byte match of the HEAD-rooted tree against
/// `git log --graph`, over random topologies where the plain and graph
/// orders agree (a chain plus extra parent edges: the second parent of a
/// merge is always an ancestor of the first, so git's lookahead does not
/// reorder anything).
#[test]
fn random_topologies_match_git_graph_exactly() {
    let mut rng: u64 = 0x9e3779b97f4a7c15;
    let mut next = move || {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (rng >> 33) as usize
    };

    for _ in 0..40 {
        let n = 2 + next() % 14;
        // Linear chain (commit i has parent i+1), then random extra
        // parents, which creates merges/octopuses. Every extra parent is
        // an ancestor of the first parent, so the plain and --graph commit
        // orders agree and the comparison below is end to end.
        let mut parents: Vec<Vec<u32>> = (0..n).map(|i| vec![(i + 1) as u32]).collect();
        parents[n - 1] = vec![];
        for _ in 0..(3 * n / 2) {
            let i = next() % (n - 1);
            let j = i + 1 + next() % (n - i - 1);
            if !parents[i].contains(&(j as u32)) && parents[i].len() < 5 {
                parents[i].push(j as u32);
                parents[i].sort_unstable();
            }
        }
        let spec = parents
            .iter()
            .map(|ps| {
                ps.iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>()
            .join(";");

        let (dir, repo, _oids) = build_spec_repo(&spec);

        // For these topologies the plain and --graph orders agree, so the
        // tree must match `git log --graph` byte for byte.
        let theirs = parse_graph(&git(
            dir.path(),
            None,
            &["git", "log", "--graph", "--format=%H"],
        ));
        let head = repo.head().unwrap().peel_to_commit().unwrap().id();
        let rows = tree::tree_roots(&repo, usize::MAX, &[head], None).unwrap();
        let mine: Vec<(String, Option<String>)> = rows
            .iter()
            .map(|r| {
                (
                    r.graph.trim_end().to_string(),
                    r.commit.as_ref().map(|c| c.oid.clone()),
                )
            })
            .collect();
        assert_eq!(
            mine,
            theirs,
            "tree differs from `git log --graph` for spec {spec:?}\n--- tree ---\n{}\n--- git ---\n{}",
            lines_str(&mine),
            lines_str(&theirs),
        );
    }
}

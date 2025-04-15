use core::ops::Range;
use std::fmt::{self, Debug};

#[derive(Debug, Clone)]
pub struct Commit {
    pub header: CommitHeader,
    pub diff: Vec<FileDiff>,
}

#[derive(Debug, Clone)]
pub struct CommitHeader {
    pub range: Range<usize>,
    pub hash: Range<usize>,
}

#[derive(Debug, Clone)]
pub struct FileDiff {
    pub range: Range<usize>,
    pub header: DiffHeader,
    pub hunks: Vec<Hunk>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Status {
    Added,
    Deleted,
    Modified,
    Renamed,
    Copied,
    Unmerged,
}

#[derive(Debug, Clone)]
pub struct DiffHeader {
    pub range: Range<usize>,
    pub old_file: Range<usize>,
    pub new_file: Range<usize>,
    pub status: Status,
}

#[derive(Debug, Clone)]
pub struct Hunk {
    pub range: Range<usize>,
    pub header: HunkHeader,
    pub content: HunkContent,
}

#[derive(Debug, Clone)]
pub struct HunkContent {
    pub range: Range<usize>,
    pub changes: Vec<Change>,
}

#[derive(Debug, Clone)]
pub struct HunkHeader {
    pub range: Range<usize>,
    pub old_line_start: u32,
    pub old_line_count: u32,
    pub new_line_start: u32,
    pub new_line_count: u32,
    pub fn_ctx: Range<usize>,
}

#[derive(Debug, Clone)]
pub struct Change {
    pub old: Range<usize>,
    pub new: Range<usize>,
}

#[derive(Debug)]
pub struct ParseError<'a> {
    input: &'a str,
    pos: usize,
    expected: &'static str,
}

impl<'a> ParseError<'a> {
    fn new(parser: &Parser<'a>, expected: &'static str) -> Self {
        Self {
            input: parser.input,
            pos: parser.pos,
            expected,
        }
    }
}

impl fmt::Display for ParseError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Expected {:?} at {:?}",
            self.expected,
            &self.input[self.pos..]
        )
    }
}

impl std::error::Error for ParseError<'_> {}

pub struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    /// Parses a diff file and returns a vector of Diff structures.
    ///
    /// The returned ranges refer to the original input bytes.
    ///
    /// # Example
    ///
    /// ```
    /// let input = "diff --git a/file1.txt b/file2.txt\n\
    /// index 0000000..1111111 100644\n\
    /// --- a/file1.txt\n\
    /// +++ b/file2.txt\n\
    /// @@ -1,2 +1,2 @@\n\
    /// -foo\n\
    /// +bar\n";
    ///
    /// let diff = gitu_diff::Parser::new(input).parse_diff().unwrap();
    /// assert_eq!(diff[0].header.new_file, 25..34); // "file2.txt"
    /// ```

    pub fn parse_commit(&mut self) -> Result<Commit, ParseError> {
        let header = self.parse_commit_header()?;
        let diff = self.parse_diff()?;

        Ok(Commit { header, diff })
    }

    pub fn parse_diff(&mut self) -> Result<Vec<FileDiff>, ParseError> {
        let mut diffs = vec![];

        if self.input.is_empty() {
            return Ok(vec![]);
        }

        while let Ok(unmerged) = self.parse_unmerged_file() {
            diffs.push(unmerged);
        }

        if !diffs.is_empty() {
            return Ok(diffs);
        }

        while self.is_at_diff_header() {
            diffs.push(self.parse_file_diff()?);
        }

        if self.pos < self.input.len() {
            return Err(ParseError::new(self, "*EOF*"));
        }

        Ok(diffs.into())
    }

    fn parse_commit_header(&mut self) -> Result<CommitHeader, ParseError<'a>> {
        let start = self.pos;

        self.read("commit ")?;
        let hash = self.read_rest_of_line();

        self.skip_until_diff_header();

        Ok(CommitHeader {
            range: start..self.pos,
            hash,
        })
    }

    fn skip_until_diff_header(&mut self) {
        while self.pos < self.input.len() && !self.is_at_diff_header() {
            self.read_rest_of_line();
        }
    }

    fn is_at_diff_header(&mut self) -> bool {
        self.peek("diff")
    }

    fn parse_file_diff(&mut self) -> Result<FileDiff, ParseError<'a>> {
        let diff_start = self.pos;
        let header = self.parse_diff_header()?;

        let mut hunks = vec![];

        if header.status == Status::Unmerged {
            self.skip_until_diff_header();
        } else {
            while self.peek("@@") {
                hunks.push(self.parse_hunk()?);
            }
        }

        Ok(FileDiff {
            range: diff_start..self.pos,
            header,
            hunks: hunks.into(),
        })
    }

    fn parse_diff_header(&mut self) -> Result<DiffHeader, ParseError<'a>> {
        let diff_header_start = self.pos;
        let mut diff_type = Status::Modified;

        let (old_file, new_file, is_conflicted) = self
            .parse_conflicted_file()
            .or_else(|_| self.parse_old_new_file_header())?;

        if is_conflicted {
            diff_type = Status::Unmerged;
        }

        if self.peek("new file") {
            diff_type = Status::Added;
            self.read_rest_of_line();
        } else if self.peek("deleted file") {
            diff_type = Status::Deleted;
            self.read_rest_of_line();
        }

        if self.read("similarity index").is_ok() {
            self.read_rest_of_line();
        }

        if self.read("dissimilarity index").is_ok() {
            self.read_rest_of_line();
        }

        if self.peek("index") {
        } else if self.peek("old mode") || self.peek("new mode") {
            self.read_rest_of_line();
        } else if self.peek("deleted file mode") {
            diff_type = Status::Deleted;
            self.read_rest_of_line();
        } else if self.peek("new file mode") {
            diff_type = Status::Added;
            self.read_rest_of_line();
        } else if self.peek("copy from") {
            diff_type = Status::Copied;
            self.read_rest_of_line();
            self.read("copy to")?;
            self.read_rest_of_line();
        } else if self.peek("rename from") {
            diff_type = Status::Renamed;
            self.read_rest_of_line();
            self.read("rename to")?;
            self.read_rest_of_line();
        }

        if self.peek("index") {
            self.read("index ")?;
            self.read_rest_of_line();
        }

        if self.peek("Binary files ") {
            self.read_rest_of_line();
        }

        if self.peek("---") {
            self.read_rest_of_line();
            self.read("+++")?;
            self.read_rest_of_line();
        }

        Ok(DiffHeader {
            range: diff_header_start..self.pos,
            old_file,
            new_file,
            status: diff_type,
        })
    }

    fn parse_unmerged_file(&mut self) -> Result<FileDiff, ParseError<'a>> {
        let unmerged_path_prefix = self.read("* Unmerged path ")?;
        let file = self.read_to_before_newline();

        Ok(FileDiff {
            range: unmerged_path_prefix.start..self.pos,
            header: DiffHeader {
                range: unmerged_path_prefix.start..self.pos,
                old_file: file.clone(),
                new_file: file,
                status: Status::Unmerged,
            },
            hunks: vec![],
        })
    }
    fn parse_old_new_file_header(
        &mut self,
    ) -> Result<(Range<usize>, Range<usize>, bool), ParseError<'a>> {
        self.read("diff --git a/")?;
        let old_file = self.read_until(" b/")?;
        let new_file = self.read_to_before_newline();
        Ok((old_file, new_file, false))
    }

    fn parse_conflicted_file(
        &mut self,
    ) -> Result<(Range<usize>, Range<usize>, bool), ParseError<'a>> {
        self.read("diff --cc ")?;
        let file = self.read_to_before_newline();
        Ok((file.clone(), file, true))
    }

    fn parse_hunk(&mut self) -> Result<Hunk, ParseError<'a>> {
        let hunk_start = self.pos;

        let header = self.parse_hunk_header()?;
        let content = self.parse_hunk_content();

        Ok(Hunk {
            range: hunk_start..self.pos,
            header,
            content,
        })
    }

    fn parse_hunk_content(&mut self) -> HunkContent {
        let hunk_content_start = self.pos;
        let mut changes = vec![];

        while self.pos < self.input.len()
            && [" ", "-", "+", "\\"]
                .into_iter()
                .any(|prefix| self.peek(prefix))
        {
            self.read_lines_while_prefixed(|parser| parser.peek(" ") || parser.peek("\\"));
            changes.push(self.parse_change());
            self.read_lines_while_prefixed(|parser| parser.peek(" ") || parser.peek("\\"));
        }

        HunkContent {
            range: hunk_content_start..self.pos,
            changes: changes.into(),
        }
    }

    fn parse_hunk_header(&mut self) -> Result<HunkHeader, ParseError<'a>> {
        let hunk_header_start = self.pos;

        self.read("@@ -")?;
        let old_line_start = self.read_number()?;
        let old_line_count = if self.read(",").is_ok() {
            self.read_number()?
        } else {
            1
        };
        self.read(" +")?;
        let new_line_start = self.read_number()?;
        let new_line_count = if self.read(",").is_ok() {
            self.read_number()?
        } else {
            1
        };
        self.read(" @@")?;
        self.read(" ").ok();

        let fn_ctx = self.read_rest_of_line();

        Ok(HunkHeader {
            range: hunk_header_start..self.pos,
            old_line_start,
            old_line_count,
            new_line_start,
            new_line_count,
            fn_ctx,
        })
    }

    fn parse_change(&mut self) -> Change {
        let removed = self.read_lines_while_prefixed(|parser| parser.peek("-"));
        let added = self.read_lines_while_prefixed(|parser| parser.peek("+"));

        Change {
            old: removed,
            new: added,
        }
    }

    fn read_lines_while_prefixed(&mut self, pred: fn(&mut Parser) -> bool) -> Range<usize> {
        let start = self.pos;
        while self.pos < self.input.len() && pred(self) {
            self.read_rest_of_line();
        }

        start..self.pos
    }

    fn read(&mut self, expected: &'static str) -> Result<Range<usize>, ParseError<'a>> {
        let start = self.pos;

        if !self.peek(expected) {
            return Err(ParseError::new(self, expected));
        }

        self.pos += expected.len();
        Ok(start..self.pos)
    }

    fn read_until(&mut self, until: &'static str) -> Result<Range<usize>, ParseError<'a>> {
        let start = self.pos;

        while !self.peek(until) {
            self.pos += 1;

            if self.pos >= self.input.len() {
                // TODO Explain "until" in another type of error message
                return Err(ParseError::new(self, until));
            }
        }

        let end = self.pos;
        self.pos += until.len();
        Ok(start..end)
    }

    fn read_to_before_newline(&mut self) -> Range<usize> {
        let start = self.pos;

        while self.pos < self.input.len() && !self.peek("\n") {
            self.pos += 1;
        }

        self.pos += 1;

        if self.input.get((self.pos - 2)..(self.pos - 1)) == Some("\r") {
            start..self.pos - 2
        } else {
            start..self.pos - 1
        }
    }

    fn read_number(&mut self) -> Result<u32, ParseError<'a>> {
        let digit_count = &self
            .input
            .get(self.pos..)
            .map(|s| s.chars().take_while(|c| c.is_ascii_digit()).count())
            .unwrap_or(0);

        if digit_count == &0 {
            return Err(ParseError::new(self, "*number*"));
        }

        self.pos += digit_count;
        Ok(self
            .input
            .get(self.pos - digit_count..self.pos)
            .ok_or(ParseError::new(self, "*number*"))?
            .parse()
            .unwrap())
    }

    fn read_rest_of_line(&mut self) -> Range<usize> {
        let start = self.pos;

        while self.pos < self.input.len() && !self.peek("\n") {
            self.pos += 1;
        }

        self.pos += 1;
        start..self.pos
    }

    fn peek(&mut self, pattern: &str) -> bool {
        self.input
            .get(self.pos..)
            .is_some_and(|s| s.starts_with(pattern))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_input() {
        let mut parser = Parser::new("");
        let diffs = parser.parse_diff().unwrap();
        assert!(diffs.is_empty(), "Expected empty vector for empty input");
    }

    #[test]
    fn parse_valid_diff() {
        let input = "diff --git a/file1.txt b/file2.txt\n\
            index 0000000..1111111 100644\n\
            --- a/file1.txt\n\
            +++ b/file2.txt\n\
            @@ -1,2 +1,2 @@ fn main() {\n\
            -foo\n\
            +bar\n";
        let mut parser = Parser::new(input);
        let diffs = parser.parse_diff().unwrap();
        assert_eq!(diffs.len(), 1, "Expected one diff block");

        let diff = &diffs[0];
        let old_file_str = &input[diff.header.old_file.clone()];
        assert_eq!(old_file_str, "file1.txt", "Old file does not match");

        let new_file_str = &input[diff.header.new_file.clone()];
        assert_eq!(new_file_str, "file2.txt", "New file does not match");

        assert_eq!(diff.hunks.len(), 1, "Expected one hunk");
        let hunk = &diff.hunks[0];

        assert_eq!(hunk.header.old_line_start, 1, "Old line start should be 1");
        assert_eq!(hunk.header.old_line_count, 2, "Old line count should be 2");
        assert_eq!(hunk.header.new_line_start, 1, "New line start should be 1");
        assert_eq!(hunk.header.new_line_count, 2, "New line count should be 2");

        let func_ctx = &input[hunk.header.fn_ctx.clone()];
        assert_eq!(func_ctx, "fn main() {\n", "Expected function context");

        assert_eq!(
            hunk.content.changes.len(),
            1,
            "Expected one change in the hunk"
        );
        let change = &hunk.content.changes[0];
        let removed_str = &input[change.old.clone()];
        assert_eq!(removed_str, "-foo\n", "Removed line does not match");
        let added_str = &input[change.new.clone()];
        assert_eq!(added_str, "+bar\n", "Added line does not match");
    }

    #[test]
    fn parse_multiple_diffs() {
        let input = "diff --git a/file1.txt b/file1.txt\n\
            index 0000000..1111111 100644\n\
            --- a/file1.txt\n\
            +++ b/file1.txt\n\
            @@ -1,1 +1,1 @@\n\
            -foo\n\
            +bar\n\
            diff --git a/file2.txt b/file2.txt\n\
            index 2222222..3333333 100644\n\
            --- a/file2.txt\n\
            +++ b/file2.txt\n\
            @@ -2,2 +2,2 @@\n\
            -baz\n\
            +qux\n";
        let mut parser = Parser::new(input);
        let diffs = parser.parse_diff().unwrap();
        assert_eq!(diffs.len(), 2, "Expected two diff blocks");

        let diff1 = &diffs[0];
        let old_file1 = &input[diff1.header.old_file.clone()];
        assert_eq!(old_file1, "file1.txt", "First diff old file mismatch");

        let diff2 = &diffs[1];
        let old_file2 = &input[diff2.header.old_file.clone()];
        assert_eq!(old_file2, "file2.txt", "Second diff old file mismatch");
    }

    #[test]
    fn parse_crlf_input() {
        let input = "diff --git a/file.txt b/file.txt\r\n\
            index 0000000..1111111 100644\r\n\
            --- a/file.txt\r\n\
            +++ b/file.txt\r\n\
            @@ -1,1 +1,1 @@\r\n\
            -foo\r\n\
            +bar\r\n";
        let mut parser = Parser::new(input);
        let diffs = parser.parse_diff().unwrap();
        assert_eq!(diffs.len(), 1, "Expected one diff block for CRLF input");
        let diff = &diffs[0];
        let old_file = &input[diff.header.old_file.clone()];
        assert_eq!(
            old_file, "file.txt",
            "Old file does not match in CRLF input"
        );
    }

    #[test]
    fn parse_malformed_input_missing_diff_header() {
        let input = "--- a/file.txt\n+++ b/file.txt\n";
        let mut parser = Parser::new(input);
        assert!(parser.parse_diff().is_err());
    }

    #[test]
    fn parse_malformed_input_missing_hunk_header() {
        let input = "diff --git a/file.txt b/file.txt\n\
            index 0000000..1111111 100644\n\
            --- a/file.txt\n\
            +++ b/file.txt\n\
            foo\n";
        let mut parser = Parser::new(input);
        assert!(parser.parse_diff().is_err());
    }

    #[test]
    fn parse_malformed_input_invalid_number() {
        let input = "diff --git a/file.txt b/file.txt\n\
            index 0000000..1111111 100644\n\
            --- a/file.txt\n\
            +++ b/file.txt\n\
            @@ -a,1 +1,1 @@\n\
            -foo\n\
            +bar\n";
        let mut parser = Parser::new(input);
        assert!(parser.parse_diff().is_err());
    }

    #[test]
    fn parse_malformed_input_extra_characters() {
        let input = "diff --git a/file.txt b/file.txt\n\
            index 0000000..1111111 100644\n\
            --- a/file.txt\n\
            +++ b/file.txt\n\
            @@ -1,1 +1,1 @@\n\
            -foo\n\
            +bar\n\
            unexpected\n";
        let mut parser = Parser::new(input);
        assert!(parser.parse_diff().is_err());
    }

    #[test]
    fn unified_diff_break() {
        let input = "diff --git a/file.txt b/file.txt\r\n\
            index 0000000..1111111 100644\r\n\
            --- a/file.txt\r\n\
            +++ b/file.txt\r\n\
            @@ -1,1 +1,1 @@\r\n\
            -foo\r\n\
            +bar\r\n";
        let mut parser = Parser::new(input);
        let _ = parser.parse_diff();
    }

    #[test]
    fn new_file() {
        let input = "diff --git a/file.txt b/file.txt\r\n\
            new file mode 100644\r\n\
            index 0000000..1111111\r\n\
            --- /dev/null\r\n\
            +++ b/file.txt\r\n\
            @@ -0,0 +1,1 @@\r\n\
            +bar\r\n";
        let mut parser = Parser::new(input);
        let diffs = parser.parse_diff().unwrap();
        assert_eq!(diffs.len(), 1, "Expected one diff block for new file test");
        let diff = &diffs[0];
        let old_file_str = &input[diff.header.old_file.clone()];
        assert_eq!(old_file_str, "file.txt",);
        let new_file_str = &input[diff.header.new_file.clone()];
        assert_eq!(new_file_str, "file.txt",);
    }

    #[test]
    fn omitted_line_count() {
        let input = "diff --git a/file.txt b/file.txt\r\n\
            index 0000000..1111111 100644\r\n\
            --- a/file.txt\r\n\
            +++ b/file.txt\r\n\
            @@ -1 +1 @@\r\n\
            -foo\r\n\
            +bar\r\n";
        let mut parser = Parser::new(input);
        let diffs = parser.parse_diff().unwrap();
        assert_eq!(
            diffs.len(),
            1,
            "Expected one diff block for omitted line count test"
        );
        let diff = &diffs[0];
        let hunk = &diff.hunks[0];
        assert_eq!(hunk.header.old_line_count, 1, "Old line count should be 1");
        assert_eq!(hunk.header.new_line_count, 1, "New line count should be 1");
    }

    #[test]
    fn new_empty_files() {
        let input = "diff --git a/file-a b/file-a\n\
             new file mode 100644\n\
             index 0000000..e69de29\n\
             diff --git a/file-b b/file-b\n\
             new file mode 100644\n\
             index 0000000..e69de29\n";
        let mut parser = Parser::new(input);
        let diffs = parser.parse_diff().unwrap();
        assert_eq!(diffs.len(), 2, "Expected two diff blocks for new files");
        assert_eq!(diffs[0].header.status, Status::Added);
        assert_eq!(diffs[0].hunks.len(), 0, "Expected no hunks in first diff");
        assert_eq!(diffs[1].header.status, Status::Added);
        assert_eq!(diffs[1].hunks.len(), 0, "Expected no hunks in second diff");
    }

    #[test]
    fn deleted_file() {
        let input = "diff --git a/Cargo.lock b/Cargo.lock\n\
            deleted file mode 100644\n\
            index 6ae58a0..0000000\n\
            --- a/Cargo.lock\n\
            +++ /dev/null\n";
        let mut parser = Parser::new(input);
        let diffs = parser.parse_diff().unwrap();
        assert_eq!(diffs.len(), 1, "Expected two diff blocks for new files");
        assert_eq!(diffs[0].header.status, Status::Deleted);
    }

    #[test]
    fn commit() {
        let input = "commit 9318f4040de9e6cf60033f21f6ae91a0f2239d38\n\
            Author: altsem <alltidsemester@pm.me>\n\
            Date:   Wed Feb 19 19:25:37 2025 +0100\n\
            \n\
                chore(release): prepare for v0.28.2\n\
            \n\
            diff --git a/.recent-changelog-entry b/.recent-changelog-entry\n\
            index 7c59f63..b3d843c 100644\n\
            --- a/.recent-changelog-entry\n\
            +++ b/.recent-changelog-entry\n\
            @@ -1,7 +1,6 @@\n\
            -## [0.28.1] - 2025-02-13\n\
            +## [0.28.2] - 2025-02-19\n ### 🐛 Bug Fixes\n \n\
            -- Change logging level to reduce inotify spam\n\
            -- Don't refresh on `gitu.log` writes (gitu --log)\n\
            +- Rebase menu opening after closing Neovim\n";
        let mut parser = Parser::new(input);
        let commit = parser.parse_commit().unwrap();
        assert!(input[commit.header.range.clone()].starts_with("commit 931"));
        assert!(input[commit.header.range.clone()].ends_with("28.2\n\n"));
        assert!(input[commit.header.hash.clone()]
            .starts_with("9318f4040de9e6cf60033f21f6ae91a0f2239d38"));
    }

    #[test]
    fn empty_commit() {
        let input = "commit 6c9991b0006b38b439605eb68baff05f0c0ebf95\nAuthor: altsem <alltidsemester@pm.me>\nDate:   Sun Jun 16 19:01:00 2024 +0200\n\n    feat: -n argument to limit log\n            \n        ";

        let mut parser = Parser::new(input);
        let commit = parser.parse_commit().unwrap();
        assert_eq!(commit.diff.len(), 0);
    }

    #[test]
    fn binary_file() {
        let input = "commit 664b2f5a3223f48d3cf38c7b517014ea98b9cb55\nAuthor: altsem <alltidsemester@pm.me>\nDate:   Sat Apr 20 13:43:23 2024 +0200\n\n    update vhs/rec\n\ndiff --git a/vhs/help.png b/vhs/help.png\nindex 876e6a1..8c46810 100644\nBinary files a/vhs/help.png and b/vhs/help.png differ\ndiff --git a/vhs/rec.gif b/vhs/rec.gif\nindex 746d957..333bc94 100644\nBinary files a/vhs/rec.gif and b/vhs/rec.gif differ\ndiff --git a/vhs/rec.tape b/vhs/rec.tape\nindex bd36591..fd56c37 100644\n--- a/vhs/rec.tape\n+++ b/vhs/rec.tape\n@@ -4,7 +4,7 @@ Set Height 800\n Set Padding 5\n \n Hide\n-Type \"git checkout 3259529\"\n+Type \"git checkout f613098b14ed99fab61bd0b78a4a41e192d90ea2\"\n Enter\n Type \"git checkout -b demo-branch\"\n Enter\n";

        let mut parser = Parser::new(input);
        let commit = parser.parse_commit().unwrap();
        // TODO assert
    }

    #[test]
    fn conflicted_file() {
        let input = "diff --cc new-file\nindex 32f95c0,2b31011..0000000\n--- a/new-file\n+++ b/new-file\n@@@ -1,1 -1,1 +1,5 @@@\n- hi\n -hey\n++<<<<<<< HEAD\n++hi\n++=======\n++hey\n++>>>>>>> other-branch\n";

        let mut parser = Parser::new(input);
        let diff = parser.parse_diff().unwrap();
        // TODO assert
    }

    #[test]
    fn unmerged_path() {
        let input = "* Unmerged path new-file\n* Unmerged path new-file-2\n";
        let mut parser = Parser::new(input);
        let diff = parser.parse_diff().unwrap();

        assert_eq!(diff.len(), 2);
        assert_eq!(diff[0].header.status, Status::Unmerged);
        assert_eq!(&input[diff[0].header.old_file.clone()], "new-file");
        assert_eq!(&input[diff[0].header.new_file.clone()], "new-file");
        assert!(diff[0].hunks.is_empty());
        assert_eq!(diff[1].header.status, Status::Unmerged);
        assert_eq!(&input[diff[1].header.old_file.clone()], "new-file-2");
        assert_eq!(&input[diff[1].header.new_file.clone()], "new-file-2");
        assert!(diff[1].hunks.is_empty());
    }

    #[test]
    fn missing_newline_before_final() {
        let input = "diff --git a/vitest.config.ts b/vitest.config.ts\nindex 97b017f..bcd28a0 100644\n--- a/vitest.config.ts\n+++ b/vitest.config.ts\n@@ -14,4 +14,4 @@ export default defineConfig({\n     globals: true,\n     setupFiles: ['./src/test/setup.ts'],\n   },\n-})\n\\ No newline at end of file\n+});";

        let mut parser = Parser::new(input);
        let diff = parser.parse_diff().unwrap();
    }
}

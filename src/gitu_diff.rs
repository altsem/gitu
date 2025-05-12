/// A git diff parser.
///
/// The aim of this module is to produce ranges that refer to the original input bytes.
/// This approach can be preferable where one would want to:
/// - Use the ranges to highlight changes in a user interface.
/// - Be sure that the original input is intact.
///
use core::ops::Range;
use std::fmt::{self, Debug};

trait ParsedRange {
    fn range(&self) -> &Range<usize>;
}

impl ParsedRange for Range<usize> {
    fn range(&self) -> &Range<usize> {
        self
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Commit {
    pub header: CommitHeader,
    pub diff: Vec<FileDiff>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CommitHeader {
    pub range: Range<usize>,
    pub hash: Range<usize>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FileDiff {
    pub range: Range<usize>,
    pub header: DiffHeader,
    pub hunks: Vec<Hunk>,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Status {
    Added,
    Deleted,
    Modified,
    Renamed,
    Copied,
    Unmerged,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DiffHeader {
    pub range: Range<usize>,
    pub old_file: Range<usize>,
    pub new_file: Range<usize>,
    pub status: Status,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Hunk {
    pub range: Range<usize>,
    pub header: HunkHeader,
    pub content: HunkContent,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HunkContent {
    pub range: Range<usize>,
    pub changes: Vec<Change>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HunkHeader {
    pub range: Range<usize>,
    pub old_line_start: u32,
    pub old_line_count: u32,
    pub new_line_start: u32,
    pub new_line_count: u32,
    pub fn_ctx: Range<usize>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Change {
    pub old: Range<usize>,
    pub new: Range<usize>,
}

pub type Result<T> = std::result::Result<T, ParseError>;

pub enum ParseError {
    Expected {
        cursor: usize,
        expected: &'static str,
    },
    NotFound {
        cursor: usize,
        expected: &'static str,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Expected { cursor, expected } => {
                write!(f, "Expected {:?} at byte {:?}", expected, cursor)
            }
            ParseError::NotFound { cursor, expected } => {
                write!(f, "Couldn't find {:?} from byte {:?}", expected, cursor)
            }
        }
    }
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{self}"))
    }
}

impl std::error::Error for ParseError {}

#[derive(Clone, Debug)]
pub struct Parser<'a> {
    input: &'a str,
    cursor: usize,
}

type ParseFn<'a, T> = fn(&mut Parser<'a>) -> Result<T>;

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, cursor: 0 }
    }

    pub fn parse_commit(&mut self) -> Result<Commit> {
        let header = self.commit_header()?;
        let diff = self.parse_diff()?;

        Ok(Commit { header, diff })
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
    /// let diff = gitu::gitu_diff::Parser::new(input).parse_diff().unwrap();
    /// assert_eq!(diff[0].header.new_file, 25..34); // "file2.txt"
    /// ```
    pub fn parse_diff(&mut self) -> Result<Vec<FileDiff>> {
        let mut diffs = vec![];

        if self.input.is_empty() {
            return Ok(vec![]);
        }

        if !diffs.is_empty() {
            return Ok(diffs);
        }

        while self.is_at_diff_header() {
            diffs.push(self.file_diff()?);
        }

        self.eof()?;

        Ok(diffs)
    }

    fn commit_header(&mut self) -> Result<CommitHeader> {
        let start = self.cursor;

        self.consume("commit ")?;
        let (hash, _) = self.consume_until(Self::newline)?;
        self.skip_until_diff_header()?;

        Ok(CommitHeader {
            range: start..self.cursor,
            hash,
        })
    }

    fn skip_until_diff_header(&mut self) -> Result<()> {
        while self.cursor < self.input.len() && !self.is_at_diff_header() {
            self.consume_until(Self::newline_or_eof)?;
        }

        Ok(())
    }

    fn is_at_diff_header(&mut self) -> bool {
        self.peek("diff") || self.peek("*")
    }

    fn file_diff(&mut self) -> Result<FileDiff> {
        let diff_start = self.cursor;
        let header = self.diff_header()?;

        let mut hunks = vec![];

        if header.status == Status::Unmerged {
            self.skip_until_diff_header()?;
        } else {
            while self.peek("@@") {
                hunks.push(self.hunk()?);
            }
        }

        Ok(FileDiff {
            range: diff_start..self.cursor,
            header,
            hunks,
        })
    }

    fn diff_header(&mut self) -> Result<DiffHeader> {
        let diff_header_start = self.cursor;
        let mut diff_type = Status::Modified;

        if let Ok(unmerged) = self.unmerged_file() {
            return Ok(unmerged);
        }

        let (old_file, new_file, is_conflicted) = self
            .conflicted_file()
            .or_else(|_| self.old_new_file_header())?;

        if is_conflicted {
            diff_type = Status::Unmerged;
        }

        if self.peek("new file") {
            diff_type = Status::Added;
            self.consume_until(Self::newline)?;
        } else if self.peek("deleted file") {
            diff_type = Status::Deleted;
            self.consume_until(Self::newline)?;
        }

        if self.consume("similarity index").is_ok() {
            self.consume_until(Self::newline)?;
        }

        if self.consume("dissimilarity index").is_ok() {
            self.consume_until(Self::newline)?;
        }

        if self.peek("index") {
        } else if self.peek("old mode") || self.peek("new mode") {
            self.consume_until(Self::newline)?;
        } else if self.peek("deleted file mode") {
            diff_type = Status::Deleted;
            self.consume_until(Self::newline)?;
        } else if self.peek("new file mode") {
            diff_type = Status::Added;
            self.consume_until(Self::newline)?;
        } else if self.peek("copy from") {
            diff_type = Status::Copied;
            self.consume_until(Self::newline)?;
            self.consume("copy to")?;
            self.consume_until(Self::newline)?;
        } else if self.peek("rename from") {
            diff_type = Status::Renamed;
            self.consume_until(Self::newline)?;
            self.consume("rename to")?;
            self.consume_until(Self::newline)?;
        }

        if self.peek("index") {
            self.consume("index ")?;
            self.consume_until(Self::newline)?;
        }

        if self.peek("Binary files ") {
            self.consume_until(Self::newline)?;
        }

        if self.peek("---") {
            self.consume_until(Self::newline)?;
            self.consume("+++")?;
            self.consume_until(Self::newline)?;
        }

        Ok(DiffHeader {
            range: diff_header_start..self.cursor,
            old_file,
            new_file,
            status: diff_type,
        })
    }

    fn unmerged_file(&mut self) -> Result<DiffHeader> {
        let unmerged_path_prefix = self.consume("* Unmerged path ")?;
        let (file, _) = self.consume_until(Self::newline_or_eof)?;

        Ok(DiffHeader {
            range: unmerged_path_prefix.start..self.cursor,
            old_file: file.clone(),
            new_file: file,
            status: Status::Unmerged,
        })
    }

    fn old_new_file_header(&mut self) -> Result<(Range<usize>, Range<usize>, bool)> {
        self.consume("diff --git")?;
        self.diff_header_path_prefix()?;
        let (old_path, _) = self.consume_until(Self::diff_header_path_prefix)?;
        let (new_path, _) = self.consume_until(Self::newline)?;

        Ok((old_path, new_path, false))
    }

    fn diff_header_path_prefix(&mut self) -> Result<Range<usize>> {
        let start = self.cursor;
        self.consume(" ")
            .and_then(|_| self.ascii_lowercase())
            .and_then(|_| self.consume("/"))
            .map_err(|_| ParseError::Expected {
                cursor: self.cursor,
                expected: "<diff header path prefix (' a/...' or ' b/...')>",
            })?;

        Ok(start..self.cursor)
    }

    fn ascii_lowercase(&mut self) -> Result<Range<usize>> {
        let start = self.cursor;
        let is_ascii_lowercase = self
            .input
            .get(self.cursor..)
            .and_then(|s| s.chars().next())
            .is_some_and(|c| c.is_ascii_lowercase());

        if is_ascii_lowercase {
            self.cursor += 1;
            Ok(start..self.cursor)
        } else {
            Err(ParseError::Expected {
                cursor: self.cursor,
                expected: "<ascii lowercase char>",
            })
        }
    }

    fn conflicted_file(&mut self) -> Result<(Range<usize>, Range<usize>, bool)> {
        self.consume("diff --cc ")?;
        let (file, _) = self.consume_until(Self::newline_or_eof)?;
        Ok((file.clone(), file, true))
    }

    fn hunk(&mut self) -> Result<Hunk> {
        let hunk_start = self.cursor;
        let header = self.hunk_header()?;
        let content = self.hunk_content()?;

        Ok(Hunk {
            range: hunk_start..self.cursor,
            header,
            content,
        })
    }

    fn hunk_content(&mut self) -> Result<HunkContent> {
        let hunk_content_start = self.cursor;
        let mut changes = vec![];

        while self.cursor < self.input.len()
            && [" ", "-", "+", "\\"]
                .into_iter()
                .any(|prefix| self.peek(prefix))
        {
            self.consume_lines_while_prefixed(|parser| parser.peek(" ") || parser.peek("\\"))?;
            changes.push(self.change()?);
            self.consume_lines_while_prefixed(|parser| parser.peek(" ") || parser.peek("\\"))?;
        }

        Ok(HunkContent {
            range: hunk_content_start..self.cursor,
            changes,
        })
    }

    fn hunk_header(&mut self) -> Result<HunkHeader> {
        let hunk_header_start = self.cursor;

        self.consume("@@ -")?;
        let old_line_start = self.number()?;
        let old_line_count = if self.consume(",").is_ok() {
            self.number()?
        } else {
            1
        };
        self.consume(" +")?;
        let new_line_start = self.number()?;
        let new_line_count = if self.consume(",").is_ok() {
            self.number()?
        } else {
            1
        };
        self.consume(" @@")?;
        self.consume(" ").ok();

        let (fn_ctx, newline) = self.consume_until(Self::newline)?;

        Ok(HunkHeader {
            range: hunk_header_start..self.cursor,
            old_line_start,
            old_line_count,
            new_line_start,
            new_line_count,
            fn_ctx: fn_ctx.start..newline.end,
        })
    }

    fn change(&mut self) -> Result<Change> {
        let removed = self.consume_lines_while_prefixed(|parser| parser.peek("-"))?;
        let removed_meta = self.consume_lines_while_prefixed(|parser| parser.peek("\\"))?;
        let added = self.consume_lines_while_prefixed(|parser| parser.peek("+"))?;
        let added_meta = self.consume_lines_while_prefixed(|parser| parser.peek("\\"))?;

        Ok(Change {
            old: removed.start..removed_meta.end,
            new: added.start..added_meta.end,
        })
    }

    fn consume_lines_while_prefixed(&mut self, pred: fn(&Parser) -> bool) -> Result<Range<usize>> {
        let start = self.cursor;
        while self.cursor < self.input.len() && pred(self) {
            self.consume_until(Self::newline_or_eof)?;
        }

        Ok(start..self.cursor)
    }

    fn number(&mut self) -> Result<u32> {
        let digit_count = &self
            .input
            .get(self.cursor..)
            .map(|s| s.chars().take_while(|c| c.is_ascii_digit()).count())
            .unwrap_or(0);

        if digit_count == &0 {
            return Err(ParseError::Expected {
                cursor: self.cursor,
                expected: "<number>",
            });
        }

        self.cursor += digit_count;
        Ok(self
            .input
            .get(self.cursor - digit_count..self.cursor)
            .ok_or(ParseError::Expected {
                cursor: self.cursor,
                expected: "<number>",
            })?
            .parse()
            .unwrap())
    }

    fn newline_or_eof(&mut self) -> Result<Range<usize>> {
        self.newline()
            .or_else(|_| self.eof())
            .map_err(|_| ParseError::Expected {
                cursor: self.cursor,
                expected: "<newline or eof>",
            })
    }

    fn newline(&mut self) -> Result<Range<usize>> {
        self.consume("\r\n")
            .or_else(|_| self.consume("\n"))
            .map_err(|_| ParseError::Expected {
                cursor: self.cursor,
                expected: "<newline>",
            })
    }

    fn eof(&mut self) -> Result<Range<usize>> {
        if self.cursor == self.input.len() {
            Ok(self.cursor..self.cursor)
        } else {
            Err(ParseError::Expected {
                cursor: self.cursor,
                expected: "<eof>",
            })
        }
    }

    /// Scans through the input, moving the cursor byte-by-byte
    /// until the provided parse_fn will succeed, or the input has been exhausted.
    /// Returns a tuple of the bytes scanned up until the match, and the match itself.
    fn consume_until<T: ParsedRange>(
        &mut self,
        parse_fn: fn(&mut Parser<'a>) -> Result<T>,
    ) -> Result<(Range<usize>, T)> {
        let start = self.cursor;
        let found = self.find(parse_fn)?;
        self.cursor = found.range().end;
        Ok((start..found.range().start, found))
    }

    /// Scans through the input byte-by-byte
    /// until the provided parse_fn will succeed, or the input has been exhausted.
    /// Returning the match. Does not step the parser.
    fn find<T: ParsedRange>(&self, parse_fn: ParseFn<'a, T>) -> Result<T> {
        let mut sub_parser = self.clone();
        let mut error = None;

        for pos in self.cursor..=self.input.len() {
            sub_parser.cursor = pos;
            match parse_fn(&mut sub_parser) {
                Ok(result) => return Ok(result),
                Err(err) => {
                    if error.is_none() {
                        error = Some(err);
                    }
                    continue;
                }
            }
        }

        Err(ParseError::NotFound {
            cursor: self.cursor,
            expected: match error.unwrap() {
                ParseError::Expected {
                    cursor: _,
                    expected,
                } => expected,
                ParseError::NotFound {
                    cursor: _,
                    expected,
                } => expected,
            },
        })
    }

    /// Consumes `expected` from the input and moves the cursor past it.
    /// Returns an error if `expected` was not found at the cursor.
    fn consume(&mut self, expected: &'static str) -> Result<Range<usize>> {
        let start = self.cursor;

        if !self.peek(expected) {
            return Err(ParseError::Expected {
                cursor: self.cursor,
                expected,
            });
        }

        self.cursor += expected.len();
        Ok(start..self.cursor)
    }

    /// Returns true if `expected` is found at the cursor.
    fn peek(&self, pattern: &str) -> bool {
        self.input
            .get(self.cursor..)
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
        assert_eq!(
            &input[commit.header.hash.clone()],
            "9318f4040de9e6cf60033f21f6ae91a0f2239d38"
        );
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
        assert_eq!(commit.diff.len(), 3);
    }

    #[test]
    fn conflicted_file() {
        let input = "diff --cc new-file\nindex 32f95c0,2b31011..0000000\n--- a/new-file\n+++ b/new-file\n@@@ -1,1 -1,1 +1,5 @@@\n- hi\n -hey\n++<<<<<<< HEAD\n++hi\n++=======\n++hey\n++>>>>>>> other-branch\n";

        let mut parser = Parser::new(input);
        let diffs = parser.parse_diff().unwrap();
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].header.status, Status::Unmerged);
        assert_eq!(&input[diffs[0].header.old_file.clone()], "new-file");
        assert_eq!(&input[diffs[0].header.new_file.clone()], "new-file");
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
        let diffs = parser.parse_diff().unwrap();
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].header.status, Status::Modified);
        assert_eq!(diffs[0].hunks.len(), 1);
        let changes = &diffs[0].hunks[0].content.changes;
        assert_eq!(changes.len(), 1);
        assert_eq!(
            &input[changes[0].old.clone()],
            "-})\n\\ No newline at end of file\n"
        );
        assert_eq!(&input[changes[0].new.clone()], "+});");
    }

    #[test]
    fn filenames_with_spaces() {
        let input = "diff --git a/file one.txt b/file two.txt\nindex 5626abf..f719efd 100644\n--- a/file one.txt	\n+++ b/file two.txt	\n@@ -1 +1 @@\n-one\n+two\n";
        let mut parser = Parser::new(input);
        let diffs = parser.parse_diff().unwrap();
        assert_eq!(&input[diffs[0].header.old_file.clone()], "file one.txt");
        assert_eq!(&input[diffs[0].header.new_file.clone()], "file two.txt");
    }

    #[test]
    fn partially_unmerged() {
        let input = "diff --git a/src/config.rs b/src/config.rs\nindex a22a438..095d9c7 100644\n--- a/src/config.rs\n+++ b/src/config.rs\n@@ -15,6 +15,7 @@ const DEFAULT_CONFIG: &str = include_str!(\"default_config.toml\");\n pub(crate) struct Config {\n     pub general: GeneralConfig,\n     pub style: StyleConfig,\n+    pub editor: EditorConfig,\n     pub bindings: BTreeMap<Menu, BTreeMap<Op, Vec<String>>>,\n }\n \n@@ -148,6 +149,13 @@ pub struct SymbolStyleConfigEntry {\n     mods: Option<Modifier>,\n }\n \n+#[derive(Default, Debug, Deserialize)]\n+pub struct EditorConfig {\n+    pub default: Option<String>,\n+    pub show: Option<String>,\n+    pub commit: Option<String>,\n+}\n+\n impl From<&StyleConfigEntry> for Style {\n     fn from(val: &StyleConfigEntry) -> Self {\n         Style {\ndiff --git a/src/default_config.toml b/src/default_config.toml\nindex eaf97e7..b5a29fa 100644\n--- a/src/default_config.toml\n+++ b/src/default_config.toml\n@@ -10,6 +10,10 @@ confirm_quit.enabled = false\n collapsed_sections = []\n refresh_on_file_change.enabled = true\n \n+[editor]\n+# show = \"zed -a\"\n+# commit = \"zile\"\n+\n [style]\n # fg / bg can be either of:\n # - a hex value: \"#707070\"\n* Unmerged path src/ops/show.rs";

        let mut parser = Parser::new(input);
        let diffs = parser.parse_diff().unwrap();
        assert_eq!(diffs.len(), 3);
        assert_eq!(diffs[2].header.status, Status::Unmerged);
        assert_eq!(&input[diffs[2].header.old_file.clone()], "src/ops/show.rs");
        assert_eq!(&input[diffs[2].header.new_file.clone()], "src/ops/show.rs");
    }

    #[test]
    fn parse_custom_prefixes() {
        let input = "diff --git i/file1.txt w/file2.txt\n\
        index 0000000..1111111 100644\n\
        --- i/file1.txt\n\
        +++ w/file2.txt\n\
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
    }
}

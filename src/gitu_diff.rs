/// A git diff parser.
///
/// The aim of this module is to produce ranges that refer to the original input bytes.
/// This approach can be preferable where one would want to:
/// - Use the ranges to highlight changes in a user interface.
/// - Be sure that the original input is intact.
///
use tinyvec::{ArrayVec, array_vec};

use core::ops::Range;
use std::{
    borrow::Cow,
    fmt::{self, Debug},
};

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
    pub old_file: FilePath,
    pub new_file: FilePath,
    pub status: Status,
}

impl FilePath {
    pub fn fmt<'a>(&'a self, input: &'a str) -> Cow<'a, str> {
        if self.is_quoted {
            Cow::Owned(
                String::from_utf8(
                    smashquote::unescape_bytes(input[self.range.clone()].as_bytes()).unwrap(),
                )
                .unwrap(),
            )
        } else {
            Cow::Borrowed(&input[self.range.clone()])
        }
    }
}

#[derive(Debug, Clone)]
pub struct FilePath {
    pub range: Range<usize>,
    pub is_quoted: bool,
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

pub type Result<'a, T> = std::result::Result<T, ParseError<'a>>;

pub struct ParseError<'a> {
    errors: ArrayVec<[ThinParseError; 4]>,
    parser: Parser<'a>,
}

/// Contains all necessary information needed to construct a descriptive error message,
/// but does not render it out.
/// This makes conditional `.or_else()` more feasible to use with parsers.
type ThinResult<T> = std::result::Result<T, ArrayVec<[ThinParseError; 4]>>;

#[derive(Default)]
pub struct ThinParseError {
    expected: &'static str,
}

impl fmt::Display for ParseError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Error parsing diff\n")?;
        for (i, error) in self.errors.iter().enumerate() {
            if i == 0 {
                f.write_fmt(format_args!("expected {:?}", error.expected))?;
            } else {
                f.write_fmt(format_args!("within   {:?}", error.expected))?;
            }

            f.write_str("\n")?;
        }

        self.parser.fmt(f)?;

        Ok(())
    }
}

impl fmt::Debug for ParseError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{self}"))
    }
}

impl fmt::Display for ThinParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Expected {}", self.expected)
    }
}

impl fmt::Debug for ThinParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{self}"))
    }
}

impl std::error::Error for ThinParseError {}

#[derive(Clone)]
pub struct Parser<'a> {
    input: &'a str,
    cursor: usize,
}

type ParseFn<'a, T> = fn(&mut Parser<'a>) -> ThinResult<T>;

impl<'a> fmt::Debug for Parser<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cursor = self.cursor;
        // TODO Need to handle invalid unicode
        let line_start = self.input[..cursor].rfind('\n').unwrap_or(0);
        let line_end = self.input[line_start..]
            .find('\n')
            .unwrap_or(self.input.len());
        let line = &self.input[line_start..line_end];
        f.write_fmt(format_args!("{}\n", line))?;
        for _ in line_start..cursor {
            f.write_str(" ")?;
        }
        f.write_str("^")?;
        Ok(())
    }
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, cursor: 0 }
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
    /// assert_eq!(diff[0].header.range, 0..97);
    /// ```
    pub fn parse_diff<'b>(&'b mut self) -> Result<'b, Vec<FileDiff>> {
        log::trace!("Parser::parse_diff\n{:?}", self);
        let mut diffs = vec![];

        if self.input.is_empty() {
            return Ok(vec![]);
        }

        if !diffs.is_empty() {
            return Ok(diffs);
        }

        self.skip_until_diff_header().map_err(|errors| ParseError {
            errors,
            parser: self.clone(),
        })?;

        while self.is_at_diff_header() {
            diffs.push(self.file_diff().map_err(|errors| ParseError {
                errors,
                parser: self.clone(),
            })?);
        }

        self.eof().map_err(|errors| ParseError {
            errors,
            parser: self.clone(),
        })?;

        Ok(diffs)
    }

    fn skip_until_diff_header(&mut self) -> ThinResult<()> {
        log::trace!("Parser::skip_until_diff_header\n{:?}", self);
        while self.cursor < self.input.len() && !self.is_at_diff_header() {
            self.consume_until(Self::newline_or_eof)?;
        }

        Ok(())
    }

    fn is_at_diff_header(&mut self) -> bool {
        self.peek("diff") || self.peek("*")
    }

    fn file_diff(&mut self) -> ThinResult<FileDiff> {
        log::trace!("Parser::file_diff\n{:?}", self);
        let diff_start = self.cursor;
        let header = self.diff_header().map_err(|mut err| {
            err.try_push(ThinParseError {
                expected: "<diff header>",
            });
            err
        })?;

        let mut hunks = vec![];

        if header.status == Status::Unmerged {
            self.skip_until_diff_header()?;
        } else {
            while self.peek("@@") {
                hunks.push(self.hunk().map_err(|mut err| {
                    err.try_push(ThinParseError { expected: "<hunk>" });
                    err
                })?);
            }
        }

        Ok(FileDiff {
            range: diff_start..self.cursor,
            header,
            hunks,
        })
    }

    fn diff_header(&mut self) -> ThinResult<DiffHeader> {
        log::trace!("Parser::diff_header\n{:?}", self);
        let diff_header_start = self.cursor;
        let mut diff_type = Status::Modified;

        if let Ok(unmerged) = self.unmerged_file() {
            return Ok(unmerged);
        }

        let (mut old_file, mut new_file, is_conflicted) = self
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
        } else if self.peek("old mode") {
            self.consume_until(Self::newline)?;
            if self.peek("new mode") {
                self.consume_until(Self::newline)?;
            }
        } else if self.peek("new mode") {
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
            self.consume_until(Self::newline_or_eof)?;
        }

        if self.peek("Binary files ") {
            self.consume_until(Self::newline_or_eof)?;
        }

        if self.consume("--- ").is_ok() {
            old_file = self.diff_header_path(Self::newline_or_eof)?;
            self.consume("+++ ")?;
            new_file = self.diff_header_path(Self::newline_or_eof)?;
        }

        Ok(DiffHeader {
            range: diff_header_start..self.cursor,
            old_file,
            new_file,
            status: diff_type,
        })
    }

    fn unmerged_file(&mut self) -> ThinResult<DiffHeader> {
        log::trace!("Parser::unmerged_file\n{:?}", self);
        let unmerged_path_prefix = self.consume("* Unmerged path ")?;
        let file = self.diff_header_path(Self::newline_or_eof)?;

        Ok(DiffHeader {
            range: unmerged_path_prefix.start..self.cursor,
            old_file: file.clone(),
            new_file: file,
            status: Status::Unmerged,
        })
    }

    fn old_new_file_header(&mut self) -> ThinResult<(FilePath, FilePath, bool)> {
        log::trace!("Parser::old_new_file_header\n{:?}", self);
        self.consume("diff --git ")?;
        let old_path = self.diff_header_path(Self::ascii_whitespace)?;
        let new_path = self.diff_header_path(Self::newline_or_eof)?;

        Ok((old_path, new_path, false))
    }

    fn diff_header_path(&mut self, end: ParseFn<'a, Range<usize>>) -> ThinResult<FilePath> {
        log::trace!("Parser::diff_header_path\n{:?}", self);
        if self.consume("\"").ok().is_some() {
            self.diff_header_path_prefix().ok();
            let quoted = self.quoted()?;
            self.ascii_whitespace().ok();
            Ok(FilePath {
                range: quoted,
                is_quoted: true,
            })
        } else {
            self.diff_header_path_prefix().ok();
            let (consumed, _) = self.consume_until(end)?;
            Ok(FilePath {
                range: consumed,
                is_quoted: false,
            })
        }
    }

    fn quoted(&mut self) -> ThinResult<Range<usize>> {
        log::trace!("Parser::quoted\n{:?}", self);
        let start = self.cursor;

        while !self.peek("\"") {
            if self.peek("\\") {
                self.escaped()?;
            } else {
                self.cursor += 1
            }
        }

        let range = start..self.cursor;
        self.consume("\"")?;
        Ok(range)
    }

    fn escaped(&mut self) -> ThinResult<Range<usize>> {
        log::trace!("Parser::escaped\n{:?}", self);
        let start = self.cursor;

        self.consume("\\")?;
        match self.input.get(self.cursor..self.cursor + 1) {
            Some("a" | "b" | "e" | "E" | "f" | "n" | "r" | "t" | "v" | "\'" | "\"" | "\\") => {
                self.cursor += 1;
            }
            Some("0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9") => {
                self.cursor += 1;
                for _ in 0..2 {
                    if !matches!(
                        self.input.get(self.cursor..self.cursor + 1),
                        Some("0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
                    ) {
                        break;
                    }

                    self.cursor += 1;
                }
            }
            _ => {
                self.cursor = start;
                return Err(array_vec![ThinParseError {
                    expected: "<escaped char>",
                }]);
            }
        }

        Ok(start..self.cursor)
    }

    fn diff_header_path_prefix(&mut self) -> ThinResult<Range<usize>> {
        log::trace!("Parser::diff_header_path_prefix\n{:?}", self);
        let start = self.cursor;
        self.ascii_lowercase()
            .and_then(|_| self.consume("/"))
            .map_err(|_| {
                self.cursor = start;
                array_vec![ThinParseError {
                    expected: "<diff header path prefix (' a/...' or ' b/...')>",
                }]
            })?;

        Ok(start..self.cursor)
    }

    fn ascii_lowercase(&mut self) -> ThinResult<Range<usize>> {
        log::trace!("Parser::ascii_lowercase\n{:?}", self);
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
            Err(array_vec![ThinParseError {
                expected: "<ascii lowercase char>",
            }])
        }
    }

    fn conflicted_file(&mut self) -> ThinResult<(FilePath, FilePath, bool)> {
        log::trace!("Parser::conflicted_file\n{:?}", self);
        self.consume("diff --cc ")?;
        let file = self.diff_header_path(Self::newline_or_eof)?;
        Ok((file.clone(), file, true))
    }

    fn hunk(&mut self) -> ThinResult<Hunk> {
        log::trace!("Parser::hunk\n{:?}", self);
        let hunk_start = self.cursor;
        let header = self.hunk_header().map_err(|mut err| {
            err.try_push(ThinParseError {
                expected: "<hunk header>",
            });
            err
        })?;
        let content = self.hunk_content().map_err(|mut err| {
            err.try_push(ThinParseError {
                expected: "<hunk content>",
            });
            err
        })?;

        Ok(Hunk {
            range: hunk_start..self.cursor,
            header,
            content,
        })
    }

    fn hunk_content(&mut self) -> ThinResult<HunkContent> {
        log::trace!("Parser::hunk_content\n{:?}", self);
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

    fn hunk_header(&mut self) -> ThinResult<HunkHeader> {
        log::trace!("Parser::hunk_header\n{:?}", self);
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

        let (fn_ctx, newline) = self.consume_until(Self::newline_or_eof)?;

        Ok(HunkHeader {
            range: hunk_header_start..self.cursor,
            old_line_start,
            old_line_count,
            new_line_start,
            new_line_count,
            fn_ctx: fn_ctx.start..newline.end,
        })
    }

    fn change(&mut self) -> ThinResult<Change> {
        log::trace!("Parser::change\n{:?}", self);
        let removed = self.consume_lines_while_prefixed(|parser| parser.peek("-"))?;
        let removed_meta = self.consume_lines_while_prefixed(|parser| parser.peek("\\"))?;
        let added = self.consume_lines_while_prefixed(|parser| parser.peek("+"))?;
        let added_meta = self.consume_lines_while_prefixed(|parser| parser.peek("\\"))?;

        Ok(Change {
            old: removed.start..removed_meta.end,
            new: added.start..added_meta.end,
        })
    }

    fn consume_lines_while_prefixed(
        &mut self,
        pred: fn(&Parser) -> bool,
    ) -> ThinResult<Range<usize>> {
        log::trace!("Parser::consume_lines_while_prefixed\n{:?}", self);
        let start = self.cursor;
        while self.cursor < self.input.len() && pred(self) {
            self.consume_until(Self::newline_or_eof)?;
        }

        Ok(start..self.cursor)
    }

    fn number(&mut self) -> ThinResult<u32> {
        log::trace!("Parser::number\n{:?}", self);
        let digit_count = &self
            .input
            .get(self.cursor..)
            .map(|s| s.chars().take_while(|c| c.is_ascii_digit()).count())
            .unwrap_or(0);

        if digit_count == &0 {
            return Err(array_vec![ThinParseError {
                expected: "<number>",
            }]);
        }

        self.cursor += digit_count;
        Ok(self
            .input
            .get(self.cursor - digit_count..self.cursor)
            .ok_or(array_vec![ThinParseError {
                expected: "<number>",
            }])?
            .parse()
            .unwrap())
    }

    fn newline_or_eof(&mut self) -> ThinResult<Range<usize>> {
        log::trace!("Parser::newline_or_eof\n{:?}", self);
        self.newline().or_else(|_| self.eof()).map_err(|_| {
            array_vec![ThinParseError {
                expected: "<newline or eof>",
            }]
        })
    }

    fn newline(&mut self) -> ThinResult<Range<usize>> {
        log::trace!("Parser::newline\n{:?}", self);
        self.consume("\r\n")
            .or_else(|_| self.consume("\n"))
            .map_err(|_| {
                array_vec![ThinParseError {
                    expected: "<newline>",
                }]
            })
    }

    fn ascii_whitespace(&mut self) -> ThinResult<Range<usize>> {
        log::trace!("Parser::ascii_whitespace\n{:?}", self);
        self.consume(" ")
            .or_else(|_| self.consume("\t"))
            .or_else(|_| self.consume("\n"))
            .or_else(|_| self.consume("\r\n"))
            .or_else(|_| self.consume("\r"))
            .or_else(|_| self.consume("\x0C"))
            .map_err(|_| {
                array_vec![ThinParseError {
                    expected: "<ascii whitespace>",
                }]
            })
    }

    fn eof(&mut self) -> ThinResult<Range<usize>> {
        log::trace!("Parser::eof\n{:?}", self);
        if self.cursor == self.input.len() {
            Ok(self.cursor..self.cursor)
        } else {
            Err(array_vec![ThinParseError { expected: "<eof>" }])
        }
    }

    /// Scans through the input, moving the cursor byte-by-byte
    /// until the provided parse_fn will succeed, or the input has been exhausted.
    /// Returns a tuple of the bytes scanned up until the match, and the match itself.
    fn consume_until<T: ParsedRange>(
        &mut self,
        parse_fn: fn(&mut Parser<'a>) -> ThinResult<T>,
    ) -> ThinResult<(Range<usize>, T)> {
        log::trace!("Parser::consume_until\n{:?}", self);
        let start = self.cursor;
        let found = self.find(parse_fn).map_err(|mut err| {
            err.try_push(ThinParseError {
                expected: "to consume the match",
            });
            err
        })?;
        self.cursor = found.range().end;
        Ok((start..found.range().start, found))
    }

    /// Scans through the input byte-by-byte
    /// until the provided parse_fn will succeed, or the input has been exhausted.
    /// Returning the match. Does not step the parser.
    fn find<T: ParsedRange>(&self, parse_fn: ParseFn<'a, T>) -> ThinResult<T> {
        log::trace!("Parser::find\n{:?}", self);
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

        let mut errors = error.unwrap();
        errors.try_push(ThinParseError {
            expected: "to find a match",
        });

        Err(errors)
    }

    /// Consumes `expected` from the input and moves the cursor past it.
    /// Returns an error if `expected` was not found at the cursor.
    fn consume(&mut self, expected: &'static str) -> ThinResult<Range<usize>> {
        let start = self.cursor;

        if !self.peek(expected) {
            return Err(array_vec![ThinParseError { expected }]);
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
        let old_file_str = diff.header.old_file.fmt(input);
        assert_eq!(old_file_str, "file1.txt", "Old file does not match");

        let new_file_str = diff.header.new_file.fmt(input);
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
        let old_file1 = &input[diff1.header.old_file.range.clone()];
        assert_eq!(old_file1, "file1.txt", "First diff old file mismatch");

        let diff2 = &diffs[1];
        let old_file2 = &input[diff2.header.old_file.range.clone()];
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
        let old_file = &input[diff.header.old_file.range.clone()];
        assert_eq!(
            old_file, "file.txt",
            "Old file does not match in CRLF input"
        );
    }

    #[test]
    fn parse_malformed_input_missing_diff_header() {
        let input = "--- a/file.txt\n+++ b/file.txt\n";
        let mut parser = Parser::new(input);
        assert_eq!(parser.parse_diff().unwrap().len(), 0);
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
        let old_file_str = diff.header.old_file.fmt(input);
        assert_eq!(old_file_str, "/dev/null",);
        let new_file_str = diff.header.new_file.fmt(input);
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
    fn mode_change() {
        let input = "diff --git a/test-file b/test-file\n\
            old mode 100644\n\
            new mode 100755\n\
            index 1234567..1234567 100644\n\
            --- a/test-file\n\
            +++ b/test-file\n";
        let mut parser = Parser::new(input);
        let diffs = parser.parse_diff().unwrap();
        assert_eq!(diffs.len(), 1, "Expected one diff block for mode change");
        assert_eq!(diffs[0].header.status, Status::Modified);
    }

    #[test]
    fn mode_change_only() {
        // Mode change without any content changes (no hunks)
        let input = "diff --git a/script.sh b/script.sh\n\
            old mode 100644\n\
            new mode 100755\n";
        let mut parser = Parser::new(input);
        let diffs = parser.parse_diff().unwrap();
        assert_eq!(
            diffs.len(),
            1,
            "Expected one diff block for mode-only change"
        );
        assert_eq!(diffs[0].header.status, Status::Modified);
        assert_eq!(
            diffs[0].hunks.len(),
            0,
            "Expected no hunks for mode-only change"
        );
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
        let diffs = parser.parse_diff().unwrap();
        assert_eq!(
            &input[diffs[0].header.old_file.range.clone()],
            ".recent-changelog-entry"
        );
        assert_eq!(
            &input[diffs[0].header.new_file.range.clone()],
            ".recent-changelog-entry"
        );
        assert_eq!(diffs[0].header.status, Status::Modified);
        assert_eq!(
            &input[diffs[0].hunks[0].header.range.clone()],
            "@@ -1,7 +1,6 @@\n"
        );
    }

    #[test]
    fn empty_commit() {
        let input = "commit 6c9991b0006b38b439605eb68baff05f0c0ebf95\nAuthor: altsem <alltidsemester@pm.me>\nDate:   Sun Jun 16 19:01:00 2024 +0200\n\n    feat: -n argument to limit log\n            \n        ";

        let mut parser = Parser::new(input);
        let diffs = parser.parse_diff().unwrap();
        assert_eq!(diffs.len(), 0);
    }

    #[test]
    fn binary_file() {
        let input = "commit 664b2f5a3223f48d3cf38c7b517014ea98b9cb55\nAuthor: altsem <alltidsemester@pm.me>\nDate:   Sat Apr 20 13:43:23 2024 +0200\n\n    update vhs/rec\n\ndiff --git a/vhs/help.png b/vhs/help.png\nindex 876e6a1..8c46810 100644\nBinary files a/vhs/help.png and b/vhs/help.png differ\ndiff --git a/vhs/rec.gif b/vhs/rec.gif\nindex 746d957..333bc94 100644\nBinary files a/vhs/rec.gif and b/vhs/rec.gif differ\ndiff --git a/vhs/rec.tape b/vhs/rec.tape\nindex bd36591..fd56c37 100644\n--- a/vhs/rec.tape\n+++ b/vhs/rec.tape\n@@ -4,7 +4,7 @@ Set Height 800\n Set Padding 5\n \n Hide\n-Type \"git checkout 3259529\"\n+Type \"git checkout f613098b14ed99fab61bd0b78a4a41e192d90ea2\"\n Enter\n Type \"git checkout -b demo-branch\"\n Enter\n";

        let mut parser = Parser::new(input);
        let diffs = parser.parse_diff().unwrap();
        assert_eq!(diffs.len(), 3);
    }

    #[test]
    fn conflicted_file() {
        let input = "diff --cc new-file\nindex 32f95c0,2b31011..0000000\n--- a/new-file\n+++ b/new-file\n@@@ -1,1 -1,1 +1,5 @@@\n- hi\n -hey\n++<<<<<<< HEAD\n++hi\n++=======\n++hey\n++>>>>>>> other-branch\n";

        let mut parser = Parser::new(input);
        let diffs = parser.parse_diff().unwrap();
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].header.status, Status::Unmerged);
        assert_eq!(&input[diffs[0].header.old_file.range.clone()], "new-file");
        assert_eq!(&input[diffs[0].header.new_file.range.clone()], "new-file");
    }

    #[test]
    fn unmerged_path() {
        let input = "* Unmerged path new-file\n* Unmerged path new-file-2\n";
        let mut parser = Parser::new(input);
        let diff = parser.parse_diff().unwrap();

        assert_eq!(diff.len(), 2);
        assert_eq!(diff[0].header.status, Status::Unmerged);
        assert_eq!(&input[diff[0].header.old_file.range.clone()], "new-file");
        assert_eq!(&input[diff[0].header.new_file.range.clone()], "new-file");
        assert!(diff[0].hunks.is_empty());
        assert_eq!(diff[1].header.status, Status::Unmerged);
        assert_eq!(&input[diff[1].header.old_file.range.clone()], "new-file-2");
        assert_eq!(&input[diff[1].header.new_file.range.clone()], "new-file-2");
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
        // This case is ambiguous, normally if there's ---/+++ headers, we can use that.
        let input = "\
            diff --git a/file one.txt b/file two.txt\n\
            index 5626abf..f719efd 100644\n\
            @@ -1 +1 @@\n\
            -one\n\
            +two\n\
            ";
        let mut parser = Parser::new(input);
        let diff = parser.parse_diff().unwrap();
        assert_eq!(diff[0].header.old_file.fmt(input), "file");
        assert_eq!(diff[0].header.new_file.fmt(input), "one.txt b/file two.txt");
    }

    #[test]
    fn utilizes_other_old_new_header_when_ambiguous() {
        let input = "\
            diff --git a/file one.txt b/file two.txt\n\
            index 5626abf..f719efd 100644\n\
            --- a/file one.txt\n\
            +++ b/file two.txt\n\
            @@ -1 +1 @@\n\
            -one\n\
            +two\n\
            ";

        let mut parser = Parser::new(input);
        let diff = parser.parse_diff().unwrap();
        assert_eq!(diff[0].header.old_file.fmt(input), "file one.txt");
        assert_eq!(diff[0].header.new_file.fmt(input), "file two.txt");
    }

    #[test]
    fn partially_unmerged() {
        let input = "diff --git a/src/config.rs b/src/config.rs\nindex a22a438..095d9c7 100644\n--- a/src/config.rs\n+++ b/src/config.rs\n@@ -15,6 +15,7 @@ const DEFAULT_CONFIG: &str = include_str!(\"default_config.toml\");\n pub(crate) struct Config {\n     pub general: GeneralConfig,\n     pub style: StyleConfig,\n+    pub editor: EditorConfig,\n     pub bindings: BTreeMap<Menu, BTreeMap<Op, Vec<String>>>,\n }\n \n@@ -148,6 +149,13 @@ pub struct SymbolStyleConfigEntry {\n     mods: Option<Modifier>,\n }\n \n+#[derive(Default, Debug, Deserialize)]\n+pub struct EditorConfig {\n+    pub default: Option<String>,\n+    pub show: Option<String>,\n+    pub commit: Option<String>,\n+}\n+\n impl From<&StyleConfigEntry> for Style {\n     fn from(val: &StyleConfigEntry) -> Self {\n         Style {\ndiff --git a/src/default_config.toml b/src/default_config.toml\nindex eaf97e7..b5a29fa 100644\n--- a/src/default_config.toml\n+++ b/src/default_config.toml\n@@ -10,6 +10,10 @@ confirm_quit.enabled = false\n collapsed_sections = []\n refresh_on_file_change.enabled = true\n \n+[editor]\n+# show = \"zed -a\"\n+# commit = \"zile\"\n+\n [style]\n # fg / bg can be either of:\n # - a hex value: \"#707070\"\n* Unmerged path src/ops/show.rs";

        let mut parser = Parser::new(input);
        let diffs = parser.parse_diff().unwrap();
        assert_eq!(diffs.len(), 3);
        assert_eq!(diffs[2].header.status, Status::Unmerged);
        assert_eq!(
            &input[diffs[2].header.old_file.range.clone()],
            "src/ops/show.rs"
        );
        assert_eq!(
            &input[diffs[2].header.new_file.range.clone()],
            "src/ops/show.rs"
        );
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
        let old_file_str = diff.header.old_file.fmt(input);
        assert_eq!(old_file_str, "file1.txt", "Old file does not match");

        let new_file_str = diff.header.new_file.fmt(input);
        assert_eq!(new_file_str, "file2.txt", "New file does not match");
    }

    #[test]
    fn parse_quoted_filenames() {
        let input = "\
            diff --git \"a/\\303\\266\" \"b/\\303\\266\"\n\
            new file mode 100644\n\
            index 0000000..e69de29\n\
            diff --git \"a/\\\"\" \"b/\\\\\"\n\
            new file mode 100644\n\
            index 0000000..e69de29\n\
            diff --git \"a/\\'\" \"b/\\v\"\n\
            new file mode 100644\n\
            index 0000000..e69de29\n\
            diff --git \"a/\\t\" \"b/\\r\"\n\
            new file mode 100644\n\
            index 0000000..e69de29\n\
            diff --git \"a/\\n\" \"b/\\f\"\n\
            new file mode 100644\n\
            index 0000000..e69de29\n\
            diff --git \"a/\\E\" \"b/\\e\"\n\
            new file mode 100644\n\
            index 0000000..e69de29\n\
            diff --git \"a/\\e\" \"b/\\b\"\n\
            new file mode 100644\n\
            index 0000000..e69de29\n\
            diff --git \"a/\\a\" \"b/\\a\"\n\
            new file mode 100644\n\
            index 0000000..e69de29";

        let mut parser = Parser::new(input);
        let diffs = parser.parse_diff().unwrap();
        assert_eq!(diffs.len(), 8);
        assert_eq!(diffs[0].header.old_file.fmt(input), "ö", "Old file");
        assert_eq!(diffs[0].header.new_file.fmt(input), "ö", "New file");
        assert_eq!(diffs[1].header.old_file.fmt(input), "\"", "Old file");
        assert_eq!(diffs[1].header.new_file.fmt(input), "\\", "New file");
        assert_eq!(diffs[2].header.old_file.fmt(input), "'", "Old file");
        assert_eq!(diffs[2].header.new_file.fmt(input), "\u{b}", "New file");
        assert_eq!(diffs[3].header.old_file.fmt(input), "\t", "Old file");
        assert_eq!(diffs[3].header.new_file.fmt(input), "\r", "New file");
        assert_eq!(diffs[4].header.old_file.fmt(input), "\n", "Old file");
        assert_eq!(diffs[4].header.new_file.fmt(input), "\u{c}", "New file");
    }

    #[test]
    fn errors_when_bad_escaped_char() {
        let input = "\
            diff --git \"a/\\y\" \"b/\\y\"\n\
            new file mode 100644\n\
        ";

        let mut parser = Parser::new(input);
        assert!(parser.parse_diff().is_err());
    }

    #[test]
    fn parse_header_noprefix() {
        let input = "\
            diff --git Cargo.lock Cargo.lock\n\
            index 1f88e5a..3b8ea64 100644\n\
            --- Cargo.lock\n\
            +++ Cargo.lock";

        let mut parser = Parser::new(input);
        let diffs = parser.parse_diff().unwrap();
        assert_eq!(diffs.len(), 1, "Expected one diff block");

        let diff = &diffs[0];
        let old_file_str = diff.header.old_file.fmt(input);
        assert_eq!(old_file_str, "Cargo.lock", "Old file does not match");

        let new_file_str = diff.header.new_file.fmt(input);
        assert_eq!(new_file_str, "Cargo.lock", "New file does not match");
    }
}

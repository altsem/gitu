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
            .map(|pos| line_start + pos)
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

        // Allow trailing content for compatibility
        if self.cursor < self.input.len() {
            log::warn!("Diff parsing stopped at position {} of {}, trailing content ignored", self.cursor, self.input.len());
        }

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
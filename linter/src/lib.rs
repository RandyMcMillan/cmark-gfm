//! # gfm-linter
//!
//! A Rust markdown linter that enforces the GitHub Flavored Markdown (GFM)
//! specification (based on CommonMark 0.29 + GFM extensions).
//!
//! ## Rule categories
//!
//! | Rule ID | Category       | Description                                           |
//! |---------|----------------|-------------------------------------------------------|
//! | MD001   | Headings       | ATX headings must have a space after `#`              |
//! | MD002   | Headings       | No trailing `#` sequences unless balanced or empty    |
//! | MD003   | Headings       | Setext headings: `===` or `---` must span full title  |
//! | MD004   | Thematic break | Consistent thematic-break character                   |
//! | MD005   | Code fence     | Fenced code block must have matching closing fence    |
//! | MD006   | Code fence     | Closing fence must not have info string               |
//! | MD007   | Lists          | Unordered list marker must be consistent (`-`,`*`,`+`)|
//! | MD008   | Lists          | Ordered list must start at 1                          |
//! | MD009   | Lists          | Task list items must use `[ ]` or `[x]`               |
//! | MD010   | Tables         | GFM table must have header separator row              |
//! | MD011   | Tables         | Table columns must be consistent across rows          |
//! | MD012   | Whitespace     | No trailing whitespace (except 2 spaces = hard break) |
//! | MD013   | Whitespace     | No tabs (use spaces)                                  |
//! | MD014   | Whitespace     | No consecutive blank lines (max 1)                    |
//! | MD015   | Links          | Inline image must have non-empty alt text             |
//! | MD016   | Links          | Reference link must have a matching definition        |

pub mod rules;

#[cfg(feature = "cmark-gfm-ffi")]
pub mod ffi;

use std::fmt;

/// A single lint diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// 1-based line number.
    pub line: usize,
    /// 1-based column number (character position).
    pub col: usize,
    /// Rule identifier, e.g. `"MD001"`.
    pub rule: &'static str,
    /// Human-readable description.
    pub message: String,
    /// Severity level.
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {} [{}] {}",
            self.line, self.col, self.severity, self.rule, self.message
        )
    }
}

/// Lint a markdown string and return all diagnostics.
pub fn lint(source: &str) -> Vec<Diagnostic> {
    let mut diags: Vec<Diagnostic> = Vec::new();

    rules::headings::check(source, &mut diags);
    rules::thematic_break::check(source, &mut diags);
    rules::code_fences::check(source, &mut diags);
    rules::lists::check(source, &mut diags);
    rules::tables::check(source, &mut diags);
    rules::whitespace::check(source, &mut diags);
    rules::links::check(source, &mut diags);

    diags.sort_by_key(|d| (d.line, d.col));
    diags
}

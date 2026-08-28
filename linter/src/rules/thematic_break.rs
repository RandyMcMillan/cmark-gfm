//! Thematic-break rule (MD004).
//!
//! ### GFM spec reference
//! - §4.1  Thematic breaks (spec line ~544)
//!
//! **MD004** – All thematic breaks in a document must use the same character
//!             (`*`, `-`, or `_`).  Mixing characters is flagged as a warning.

use crate::{Diagnostic, Severity};

pub fn check(source: &str, diags: &mut Vec<Diagnostic>) {
    let mut first_char: Option<char> = None;

    for (i, raw) in source.lines().enumerate() {
        if let Some(ch) = is_thematic_break(raw) {
            match first_char {
                None => first_char = Some(ch),
                Some(fc) if fc != ch => {
                    diags.push(Diagnostic {
                        line: i + 1,
                        col: 1,
                        rule: "MD004",
                        message: format!(
                            "Inconsistent thematic-break character: expected `{}`, found `{}`",
                            fc, ch
                        ),
                        severity: Severity::Warning,
                    });
                }
                _ => {}
            }
        }
    }
}

/// Returns `Some(char)` if the line is a valid thematic break, `None` otherwise.
///
/// Spec §4.1: A thematic break is a line of 3+ `*`, `-`, or `_` characters,
/// optionally interspersed with spaces/tabs, and with at most 3 spaces of indent.
fn is_thematic_break(line: &str) -> Option<char> {
    let indent = line.chars().take_while(|&c| c == ' ').count();
    if indent > 3 {
        return None;
    }
    let trimmed = line.trim();
    let first = trimmed.chars().next()?;
    if first != '*' && first != '-' && first != '_' {
        return None;
    }
    let mut count = 0usize;
    for c in trimmed.chars() {
        if c == first {
            count += 1;
        } else if c != ' ' && c != '\t' {
            return None;
        }
    }
    if count >= 3 { Some(first) } else { None }
}

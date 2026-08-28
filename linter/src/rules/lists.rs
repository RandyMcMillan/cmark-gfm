//! List rules (MD007, MD008, MD009).
//!
//! ### GFM spec references
//! - §5.3  Lists          (spec line ~5139)
//! - §5.2  List items     (spec line ~3962)
//! - §5.4  Task list items (extension, spec line ~5084)
//!
//! **MD007** – Unordered list markers in a document must be consistent
//!             (all `-`, all `*`, or all `+`).
//!
//! **MD008** – The first item of an ordered list must start at `1.` or `1)`.
//!
//! **MD009** – Task list items must use exactly `[ ]` (unchecked) or `[x]` /
//!             `[X]` (checked) immediately after the list marker and space.

use crate::{Diagnostic, Severity};

pub fn check(source: &str, diags: &mut Vec<Diagnostic>) {
    let mut ul_first_char: Option<char> = None;
    let lines: Vec<&str> = source.lines().collect();

    // Track whether the previous line was an ordered list item at indent 0 (so
    // we only flag the very first item of a new ordered list, not continuations).
    let mut prev_was_ordered_at_0 = false;

    for (i, raw) in lines.iter().enumerate() {
        let trimmed = raw.trim_start();

        // --- MD007: unordered list marker consistency ---
        if let Some(marker_char) = unordered_marker(trimmed) {
            match ul_first_char {
                None => ul_first_char = Some(marker_char),
                Some(fc) if fc != marker_char => {
                    let col = raw.len() - raw.trim_start().len() + 1;
                    diags.push(Diagnostic {
                        line: i + 1,
                        col,
                        rule: "MD007",
                        message: format!(
                            "Inconsistent unordered list marker: expected `{}`, found `{}`",
                            fc, marker_char
                        ),
                        severity: Severity::Warning,
                    });
                }
                _ => {}
            }
        }

        // --- MD008: ordered list starts at 1 ---
        // Only flag the *first* item of a list (i.e. not preceded by another
        // ordered list item at the same indentation level).
        let indent = raw.len() - trimmed.len();
        if let Some((start_num, _)) = ordered_marker(trimmed) {
            if indent == 0 && start_num != 1 && !prev_was_ordered_at_0 {
                diags.push(Diagnostic {
                    line: i + 1,
                    col: 1,
                    rule: "MD008",
                    message: format!(
                        "Ordered list should start at 1, found {start_num}"
                    ),
                    severity: Severity::Warning,
                });
            }
            prev_was_ordered_at_0 = indent == 0;
        } else {
            // If the line is blank, keep the ordered context; otherwise clear it.
            if !trimmed.is_empty() {
                prev_was_ordered_at_0 = false;
            }
        }

        // --- MD009: task list item syntax ---
        // A task list item: list marker + space + `[` char `]`
        // Valid chars inside brackets: ` ` (unchecked), `x`, `X` (checked).
        if let Some(after_marker) = list_item_content(trimmed) {
            if after_marker.starts_with('[') {
                let bracket_content = &after_marker[1..];
                // bracket_content is everything after the opening `[`
                // Valid: `] …` (unchecked), `x] …` / `X] …` (checked), or bare `]`/`x]`/`X]`
                let valid = bracket_content.starts_with("] ")
                    || bracket_content == "]"
                    || bracket_content.starts_with("x] ")
                    || bracket_content == "x]"
                    || bracket_content.starts_with("X] ")
                    || bracket_content == "X]"
                    // `[ ]` pattern: space then `]`
                    || bracket_content.starts_with(" ] ")
                    || bracket_content == " ]";
                // Only treat as a malformed task item if the second character is `]`
                // (i.e. it looks like an attempted single-char task marker).
                if !valid {
                    let looks_like_task = bracket_content.len() >= 2
                        && bracket_content.as_bytes().get(1) == Some(&b']');
                    if looks_like_task {
                        let col = raw.len() - raw.trim_start().len() + 1;
                        diags.push(Diagnostic {
                            line: i + 1,
                            col,
                            rule: "MD009",
                            message: format!(
                                "Malformed task list item; use `[ ]` or `[x]`, found `[{}`",
                                &bracket_content[..bracket_content.len().min(3)]
                            ),
                            severity: Severity::Error,
                        });
                    }
                }
            }
        }
    }
}

/// If `trimmed` starts with an unordered list marker followed by a space,
/// return the marker character.
fn unordered_marker(trimmed: &str) -> Option<char> {
    let c = trimmed.chars().next()?;
    if (c == '-' || c == '*' || c == '+') && trimmed.len() >= 2 {
        let second = trimmed.chars().nth(1)?;
        if second == ' ' || second == '\t' {
            return Some(c);
        }
    }
    None
}

/// If `trimmed` starts with an ordered list marker (`N.` or `N)`) followed by
/// a space, return `(N, delimiter)`.
fn ordered_marker(trimmed: &str) -> Option<(u64, char)> {
    let end = trimmed.find(|c: char| !c.is_ascii_digit())?;
    if end == 0 || end > 9 {
        return None;
    }
    let num: u64 = trimmed[..end].parse().ok()?;
    let delim = trimmed.chars().nth(end)?;
    if delim != '.' && delim != ')' {
        return None;
    }
    let after = &trimmed[end + 1..];
    if after.starts_with(' ') || after.starts_with('\t') || after.is_empty() {
        Some((num, delim))
    } else {
        None
    }
}

/// Return the content after the list marker + space, or `None` if not a list item.
fn list_item_content(trimmed: &str) -> Option<&str> {
    // Unordered
    if let Some(c) = unordered_marker(trimmed) {
        let _ = c;
        return Some(trimmed[2..].trim_start());
    }
    // Ordered
    if let Some((_, delim)) = ordered_marker(trimmed) {
        let delim_pos = trimmed.find(delim)?;
        let after = &trimmed[delim_pos + 1..];
        return Some(after.trim_start());
    }
    None
}

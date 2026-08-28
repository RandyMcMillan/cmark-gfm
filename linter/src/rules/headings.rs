//! Heading rules (MD001, MD002, MD003).
//!
//! ### GFM spec references
//! - §4.2  ATX headings  (spec line ~768)
//! - §4.3  Setext headings (spec line ~990)
//!
//! **MD001** – ATX heading `#` must be followed by a space (or end of line for
//!             an empty heading).  `##foo` is not a valid ATX heading.
//!
//! **MD002** – Trailing `#` in an ATX heading are only allowed when they are
//!             preceded by a space, or the heading content is empty.
//!             `## foo#` is fine; `## foo# ` is fine; `## foo#bar` – the
//!             trailing sequence `#bar` is text, not valid close sequence.
//!             We warn when the heading text (after stripping a valid closing
//!             `#` sequence) still ends with `#`.
//!
//! **MD003** – Setext heading underline (`===` or `---`) must be at least as
//!             long as the heading text it underlines (spec §4.3).

use crate::{Diagnostic, Severity};

pub fn check(source: &str, diags: &mut Vec<Diagnostic>) {
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let raw = lines[i];
        let trimmed = raw.trim_start();

        // --- ATX headings (MD001, MD002) ---
        if trimmed.starts_with('#') {
            let hashes: usize = trimmed.chars().take_while(|&c| c == '#').count();
            // Valid ATX: 1–6 # signs, then a space or end of line
            if hashes <= 6 {
                let after = &trimmed[hashes..];
                // MD001: after the `#` sequence must be space, or empty, or `\t`
                if !after.is_empty() && !after.starts_with(' ') && !after.starts_with('\t') {
                    let col = raw.len() - raw.trim_start().len() + 1;
                    diags.push(Diagnostic {
                        line: i + 1,
                        col,
                        rule: "MD001",
                        message: format!(
                            "ATX heading `#` must be followed by a space (got {:?})",
                            &after[..after.len().min(4)]
                        ),
                        severity: Severity::Error,
                    });
                } else {
                    // Content between leading `#` and optional trailing `#` sequence
                    let content = after.trim();
                    // Strip valid closing `#` sequence: optional spaces then one or more #
                    let content = strip_atx_closing(content);
                    // MD002: heading text must not end with unescaped `#`
                    if content.ends_with('#') && !content.ends_with("\\#") {
                        let col = raw.len() - raw.trim_start().len() + 1;
                        diags.push(Diagnostic {
                            line: i + 1,
                            col,
                            rule: "MD002",
                            message: "ATX heading text ends with `#`; add a space before any trailing `#` to make it a closing sequence".to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
            i += 1;
            continue;
        }

        // --- Setext headings (MD003) ---
        // A setext heading underline is a line of `=` or `-` (with optional
        // trailing spaces).  The previous line must be a non-blank paragraph text.
        if i + 1 < lines.len() {
            let next = lines[i + 1].trim_end();
            if is_setext_underline(next) && !trimmed.is_empty() {
                let text_len = raw.trim().chars().count();
                let underline_len = next.trim().chars().count();
                if underline_len < text_len {
                    let col = 1;
                    diags.push(Diagnostic {
                        line: i + 2,
                        col,
                        rule: "MD003",
                        message: format!(
                            "Setext heading underline ({} chars) is shorter than heading text ({} chars)",
                            underline_len, text_len
                        ),
                        severity: Severity::Warning,
                    });
                }
                i += 2;
                continue;
            }
        }

        i += 1;
    }
}

/// Strip a valid ATX closing-`#` sequence from the end of heading content.
/// Spec §4.2: the closing sequence is optional spaces followed by one or more
/// `#` at the end, with a space before those `#` (or they are the entire line).
fn strip_atx_closing(content: &str) -> &str {
    let trimmed = content.trim_end();
    // Find trailing `#`
    let hashes_end = trimmed.len();
    let hashes_start = trimmed
        .char_indices()
        .rev()
        .take_while(|(_, c)| *c == '#')
        .last()
        .map(|(i, _)| i)
        .unwrap_or(hashes_end);
    if hashes_start == hashes_end {
        return content; // no trailing hashes
    }
    // The character before the trailing `#` sequence must be a space (or nothing)
    if hashes_start == 0 {
        return ""; // the whole content is `###...`
    }
    let before = &trimmed[..hashes_start];
    if before.ends_with(' ') || before.ends_with('\t') {
        before.trim_end()
    } else {
        content // not a valid closing sequence
    }
}

fn is_setext_underline(line: &str) -> bool {
    if line.is_empty() {
        return false;
    }
    let all_eq = line.chars().all(|c| c == '=');
    let all_dash = line.chars().all(|c| c == '-');
    (all_eq || all_dash) && !line.is_empty()
}

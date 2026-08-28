//! Whitespace rules (MD012, MD013, MD014).
//!
//! ### GFM spec references
//! - §2.1  Characters and lines (spec line ~298)
//! - §9.1  Hard line breaks (spec line ~9662)
//!
//! **MD012** – No trailing whitespace, **except** exactly two trailing spaces
//!             which constitute a hard line break (spec §9.1).  A trailing tab
//!             is never valid.
//!
//! **MD013** – No tab characters used for indentation (use spaces).
//!
//! **MD014** – No more than one consecutive blank line (blank = zero non-ws
//!             characters).

use crate::{Diagnostic, Severity};

pub fn check(source: &str, diags: &mut Vec<Diagnostic>) {
    let mut consecutive_blank = 0usize;

    for (i, raw) in source.lines().enumerate() {
        let line_no = i + 1;

        // --- MD013: tabs ---
        if raw.contains('\t') {
            // Find the column of the first tab
            let col = raw.find('\t').unwrap_or(0) + 1;
            diags.push(Diagnostic {
                line: line_no,
                col,
                rule: "MD013",
                message: "Tab character found; use spaces for indentation".to_string(),
                severity: Severity::Warning,
            });
        }

        // --- MD012: trailing whitespace ---
        // Two trailing spaces are a hard-break (allowed); anything else is not.
        let len = raw.len();
        if len > 0 {
            let trimmed_end = raw.trim_end();
            let trailing_len = len - trimmed_end.len();
            if trailing_len > 0 {
                // Allow exactly two trailing spaces (hard line break)
                let is_hard_break = trailing_len == 2
                    && raw.ends_with("  ")
                    && !trimmed_end.is_empty();
                if !is_hard_break {
                    let col = trimmed_end.len() + 1;
                    let kind = if raw.ends_with('\t') {
                        "tab"
                    } else {
                        "whitespace"
                    };
                    diags.push(Diagnostic {
                        line: line_no,
                        col,
                        rule: "MD012",
                        message: format!(
                            "Trailing {kind} ({trailing_len} character(s)); remove or use exactly 2 spaces for a hard line break"
                        ),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        // --- MD014: consecutive blank lines ---
        if raw.trim().is_empty() {
            consecutive_blank += 1;
            if consecutive_blank > 1 {
                diags.push(Diagnostic {
                    line: line_no,
                    col: 1,
                    rule: "MD014",
                    message: "More than one consecutive blank line".to_string(),
                    severity: Severity::Warning,
                });
            }
        } else {
            consecutive_blank = 0;
        }
    }
}

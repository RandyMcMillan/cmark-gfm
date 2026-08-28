//! Table rules (MD010, MD011).
//!
//! ### GFM spec reference
//! - §4.10  Tables (extension, spec line ~3307)
//!
//! **MD010** – A GFM table must have a header-separator row on line 2.
//!             The separator must consist of cells with at least one `-`,
//!             optionally surrounded by colons.
//!
//! **MD011** – All rows in a table must have the same number of pipe-delimited
//!             columns as the header row.

use crate::{Diagnostic, Severity};

pub fn check(source: &str, diags: &mut Vec<Diagnostic>) {
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;
    let mut in_fence = false;
    let mut fence_char = ' ';
    let mut fence_len = 0usize;

    while i < lines.len() {
        let raw = lines[i];
        let trimmed = raw.trim_start();

        // Track fenced code blocks to skip their contents
        if !in_fence {
            if let Some((fc, fl)) = fence_open(trimmed) {
                in_fence = true;
                fence_char = fc;
                fence_len = fl;
                i += 1;
                continue;
            }
        } else {
            if fence_close(trimmed, fence_char, fence_len) {
                in_fence = false;
            }
            i += 1;
            continue;
        }

        // A table starts with a pipe-delimited header row.
        if looks_like_table_row(raw) && i + 1 < lines.len() {
            let separator = lines[i + 1];
            let header_line = i + 1;
            let sep_line = i + 2;

            if is_separator_row(separator) {
                // MD010: separator row exists – good; validate cell count
                let header_cols = col_count(raw);
                let sep_cols = col_count(separator);

                // The separator should have the same number of cells as the header.
                // The spec allows the separator to have fewer/more cells, but many
                // linters warn on this.
                if sep_cols != header_cols {
                    diags.push(Diagnostic {
                        line: sep_line,
                        col: 1,
                        rule: "MD010",
                        message: format!(
                            "Table separator has {sep_cols} column(s) but header has {header_cols}"
                        ),
                        severity: Severity::Error,
                    });
                }

                // MD011: validate body rows
                let mut j = i + 2;
                while j < lines.len() {
                    let body_row = lines[j];
                    if !looks_like_table_row(body_row) {
                        break;
                    }
                    let body_cols = col_count(body_row);
                    if body_cols != header_cols {
                        diags.push(Diagnostic {
                            line: j + 1,
                            col: 1,
                            rule: "MD011",
                            message: format!(
                                "Table row has {body_cols} column(s) but header has {header_cols}"
                            ),
                            severity: Severity::Warning,
                        });
                    }
                    j += 1;
                }

                i = j;
                continue;
            } else {
                // Table row not followed by separator – MD010
                diags.push(Diagnostic {
                    line: header_line,
                    col: 1,
                    rule: "MD010",
                    message:
                        "GFM table header must be followed by a separator row (e.g. `| --- |`)"
                            .to_string(),
                    severity: Severity::Error,
                });
            }
        }

        i += 1;
    }
}

fn looks_like_table_row(line: &str) -> bool {
    let t = line.trim();
    t.contains('|') && !t.is_empty()
}

/// Returns true if the line is a valid GFM table separator row.
/// Each cell (split by `|`) must match `\s*:?-+:?\s*`.
fn is_separator_row(line: &str) -> bool {
    let t = line.trim();
    if !t.contains('|') {
        return false;
    }
    let cells = split_table_cells(t);
    if cells.is_empty() {
        return false;
    }
    cells.iter().all(|cell| {
        let c = cell.trim();
        if c.is_empty() {
            return true; // leading/trailing empty cells are OK
        }
        let c = c.strip_prefix(':').unwrap_or(c);
        let c = c.strip_suffix(':').unwrap_or(c);
        !c.is_empty() && c.chars().all(|ch| ch == '-')
    })
}

/// Split a table row by `|`, handling leading/trailing pipes.
fn split_table_cells(row: &str) -> Vec<&str> {
    let row = row.trim();
    let row = row.strip_prefix('|').unwrap_or(row);
    let row = row.strip_suffix('|').unwrap_or(row);
    row.split('|').collect()
}

/// Count the number of non-empty cell slots in a table row.
fn col_count(row: &str) -> usize {
    split_table_cells(row)
        .iter()
        .filter(|c| !c.trim().is_empty())
        .count()
}

fn fence_open(trimmed: &str) -> Option<(char, usize)> {
    let first = trimmed.chars().next()?;
    if first != '`' && first != '~' {
        return None;
    }
    let len = trimmed.chars().take_while(|&c| c == first).count();
    if len >= 3 { Some((first, len)) } else { None }
}

fn fence_close(trimmed: &str, fence_char: char, fence_len: usize) -> bool {
    let first = trimmed.chars().next();
    if first != Some(fence_char) {
        return false;
    }
    let len = trimmed.chars().take_while(|&c| c == fence_char).count();
    len >= fence_len
}

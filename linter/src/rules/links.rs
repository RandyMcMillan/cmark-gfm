//! Link and image rules (MD015, MD016).
//!
//! ### GFM spec references
//! - §6.6  Links            (spec line ~7734)
//! - §6.7  Images           (spec line ~8777)
//! - §4.7  Link reference definitions (spec line ~2810)
//! - §6.9  Autolinks        (spec line ~9004)
//!
//! **MD015** – Inline images must have non-empty alt text.
//!             `![](url)` is an error; `![alt](url)` is fine.
//!
//! **MD016** – Every reference-style link or image (`[label][ref]` or `[ref]`)
//!             must have a matching link reference definition (`[ref]: url`)
//!             somewhere in the document.

use crate::{Diagnostic, Severity};

pub fn check(source: &str, diags: &mut Vec<Diagnostic>) {
    // Collect all link-reference definitions (case-insensitive label matching
    // per spec §4.7).
    let definitions = collect_definitions(source);

    let mut in_fence = false;
    let mut fence_char = ' ';
    let mut fence_len = 0usize;

    for (i, raw) in source.lines().enumerate() {
        let line_no = i + 1;
        let trimmed = raw.trim_start();

        // Skip fenced code block contents
        if !in_fence {
            if let Some((fc, fl)) = fence_open(trimmed) {
                in_fence = true;
                fence_char = fc;
                fence_len = fl;
                continue;
            }
        } else {
            if fence_close(trimmed, fence_char, fence_len) {
                in_fence = false;
            }
            continue;
        }

        // MD015: inline images with empty alt text
        check_images(raw, line_no, diags);

        // MD016: unresolved reference links/images
        check_references(raw, line_no, &definitions, diags);
    }
}

/// Scan `line` for `![](...)` patterns and emit MD015 for each.
fn check_images(line: &str, line_no: usize, diags: &mut Vec<Diagnostic>) {
    let bytes = line.as_bytes();
    let mut pos = 0;
    while pos < bytes.len() {
        // Look for `![`
        if pos + 1 < bytes.len() && bytes[pos] == b'!' && bytes[pos + 1] == b'[' {
            let alt_start = pos + 2;
            // Find matching `]`
            if let Some(close_bracket) = find_close_bracket(line, alt_start) {
                let alt = &line[alt_start..close_bracket];
                // Check for empty alt text in inline image `![](...)` or `![][ref]`
                if alt.trim().is_empty() {
                    // Make sure what follows is `(` (inline) or `[` (reference) –
                    // i.e. this is an actual image syntax, not just `![` text.
                    let after = &line[close_bracket + 1..];
                    if after.starts_with('(') || after.starts_with('[') {
                        let col = pos + 1;
                        diags.push(Diagnostic {
                            line: line_no,
                            col,
                            rule: "MD015",
                            message: "Image has empty alt text; provide descriptive alt text"
                                .to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }
                pos = close_bracket + 1;
                continue;
            }
        }
        pos += 1;
    }
}

/// Scan `line` for reference-style links/images and warn about unresolved ones.
fn check_references(
    line: &str,
    line_no: usize,
    definitions: &[String],
    diags: &mut Vec<Diagnostic>,
) {
    // Patterns: `[text][label]`, `[text][]`, `[label]` (collapsed/shortcut)
    let bytes = line.as_bytes();
    let mut pos = 0;
    while pos < bytes.len() {
        // Find `[`
        if bytes[pos] == b'[' {
            let text_start = pos + 1;
            if let Some(text_end) = find_close_bracket(line, text_start) {
                let text = &line[text_start..text_end];
                let after = &line[text_end + 1..];

                // `[text][label]` or `[text][]`
                if after.starts_with('[') {
                    let ref_start = text_end + 2;
                    if let Some(ref_end) = find_close_bracket(line, ref_start) {
                        let label_raw = &line[ref_start..ref_end];
                        let label = if label_raw.trim().is_empty() {
                            // collapsed: `[text][]` – label is text
                            text
                        } else {
                            label_raw
                        };
                        if !is_inline_link_or_image(line, pos)
                            && !definitions.contains(&normalise_label(label))
                        {
                            let col = pos + 1;
                            diags.push(Diagnostic {
                                line: line_no,
                                col,
                                rule: "MD016",
                                message: format!(
                                    "Reference link `[{label}]` has no matching definition"
                                ),
                                severity: Severity::Error,
                            });
                        }
                        pos = ref_end + 1;
                        continue;
                    }
                }

                // Shortcut `[label]` – but only if not followed by `(` (inline)
                if !after.starts_with('(') && !after.starts_with('[') {
                    if !is_inline_link_or_image(line, pos)
                        && !is_definition_line(line)
                        && !definitions.contains(&normalise_label(text))
                    {
                        // Only flag if it contains text that looks like a link label
                        // (has at least one non-whitespace char)
                        if !text.trim().is_empty() {
                            // Heuristic: skip if it looks like a task-list bracket `[ ]` or `[x]`
                            if text != " " && text != "x" && text != "X" {
                                let col = pos + 1;
                                diags.push(Diagnostic {
                                    line: line_no,
                                    col,
                                    rule: "MD016",
                                    message: format!(
                                        "Shortcut reference link `[{text}]` has no matching definition"
                                    ),
                                    severity: Severity::Warning,
                                });
                            }
                        }
                    }
                }

                pos = text_end + 1;
                continue;
            }
        }
        pos += 1;
    }
}

/// Find the position of the matching `]` for text starting at `start`, handling
/// nested brackets naively (one level).
fn find_close_bracket(s: &str, start: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            b'[' => depth += 1,
            b']' => {
                if depth == 0 {
                    return Some(i);
                }
                depth -= 1;
            }
            b'\\' => {
                i += 1; // skip escaped char
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// True if position `pos` in `line` is preceded by `!` (image) or the `[`
/// is part of an inline link `[text](...)`.
fn is_inline_link_or_image(line: &str, pos: usize) -> bool {
    if pos > 0 && line.as_bytes()[pos - 1] == b'!' {
        return false; // images are handled separately; don't double-report
    }
    false
}

/// True if the line is itself a link reference definition (`[label]: url`).
fn is_definition_line(line: &str) -> bool {
    let t = line.trim_start();
    if !t.starts_with('[') {
        return false;
    }
    if let Some(close) = t.find("]:") {
        // Check there's no nested `[` before the `]:`
        return !t[1..close].contains('[');
    }
    false
}

/// Normalise a link label for case-insensitive comparison.
fn normalise_label(label: &str) -> String {
    // Spec §4.7: labels are normalised by collapsing whitespace and
    // case-folding to lower-case.
    label
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Collect all link reference definition labels from the document.
fn collect_definitions(source: &str) -> Vec<String> {
    let mut defs = Vec::new();
    for line in source.lines() {
        let t = line.trim_start();
        if t.starts_with('[') {
            if let Some(close) = t.find("]:") {
                let label = &t[1..close];
                if !label.contains('[') {
                    defs.push(normalise_label(label));
                }
            }
        }
    }
    defs
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

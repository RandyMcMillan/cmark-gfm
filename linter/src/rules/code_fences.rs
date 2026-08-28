//! Code-fence rules (MD005, MD006).
//!
//! ### GFM spec reference
//! - §4.5  Fenced code blocks (spec line ~1606)
//!
//! **MD005** – Every opening fence must have a matching closing fence.
//!             An unclosed fenced code block is an error.
//!
//! **MD006** – The closing fence must not carry an info string.

use crate::{Diagnostic, Severity};

pub fn check(source: &str, diags: &mut Vec<Diagnostic>) {
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let raw = lines[i];
        if let Some((fence_char, fence_len, indent)) = opening_fence(raw) {
            let open_line = i + 1;
            i += 1;
            let mut closed = false;
            while i < lines.len() {
                let inner = lines[i];
                // Closing fence: same char, at least fence_len times, no info string,
                // indent <= 3.
                if let Some((cc, cl, ci)) = closing_fence_candidate(inner) {
                    if cc == fence_char && cl >= fence_len && ci <= indent + 3 {
                        // MD006: closing fence must not have info string
                        let rest = inner.trim().trim_start_matches(fence_char).trim();
                        if !rest.is_empty() {
                            diags.push(Diagnostic {
                                line: i + 1,
                                col: 1,
                                rule: "MD006",
                                message: "Closing code fence must not have an info string"
                                    .to_string(),
                                severity: Severity::Error,
                            });
                        }
                        closed = true;
                        i += 1;
                        break;
                    }
                }
                i += 1;
            }
            // MD005: unclosed fence
            if !closed {
                diags.push(Diagnostic {
                    line: open_line,
                    col: 1,
                    rule: "MD005",
                    message: format!(
                        "Unclosed fenced code block (opened with `{}{}`)",
                        fence_char,
                        std::iter::repeat(fence_char)
                            .take(fence_len - 1)
                            .collect::<String>()
                    ),
                    severity: Severity::Error,
                });
            }
            continue;
        }
        i += 1;
    }
}

/// Returns `(fence_char, fence_len, indent_spaces)` for an opening fence line.
fn opening_fence(line: &str) -> Option<(char, usize, usize)> {
    let indent = line.chars().take_while(|&c| c == ' ').count();
    if indent > 3 {
        return None;
    }
    let rest = &line[indent..];
    let first = rest.chars().next()?;
    if first != '`' && first != '~' {
        return None;
    }
    let fence_len = rest.chars().take_while(|&c| c == first).count();
    if fence_len < 3 {
        return None;
    }
    // Backtick fence: info string must not contain backtick
    let info = rest[fence_len..].trim();
    if first == '`' && info.contains('`') {
        return None;
    }
    Some((first, fence_len, indent))
}

/// Returns `(fence_char, fence_len, indent_spaces)` for a potential closing fence.
fn closing_fence_candidate(line: &str) -> Option<(char, usize, usize)> {
    let indent = line.chars().take_while(|&c| c == ' ').count();
    if indent > 3 {
        return None;
    }
    let rest = &line[indent..];
    let first = rest.chars().next()?;
    if first != '`' && first != '~' {
        return None;
    }
    let fence_len = rest.chars().take_while(|&c| c == first).count();
    if fence_len < 3 {
        return None;
    }
    Some((first, fence_len, indent))
}

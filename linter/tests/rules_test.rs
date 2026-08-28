//! Integration tests for gfm-linter.
//!
//! Each test function exercises one or more rules against hand-crafted
//! markdown snippets derived from the GFM spec.
//!
//! The test output uses banner-style reporting for easy diagnosis.

use gfm_linter::{Severity, lint};

// ─────────────────────────────────────────────────────────────────────────────
// helpers
// ─────────────────────────────────────────────────────────────────────────────

fn rule_ids(diags: &[gfm_linter::Diagnostic]) -> Vec<&'static str> {
    diags.iter().map(|d| d.rule).collect()
}

fn errors(diags: &[gfm_linter::Diagnostic]) -> Vec<&gfm_linter::Diagnostic> {
    diags
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect()
}

fn warnings(diags: &[gfm_linter::Diagnostic]) -> Vec<&gfm_linter::Diagnostic> {
    diags
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .collect()
}

fn banner(label: &str) {
    println!("\n╔═══════════════════════════════════════════╗");
    println!(
        "║  {}{}║",
        label,
        " ".repeat(43_usize.saturating_sub(label.len() + 2))
    );
    println!("╚═══════════════════════════════════════════╝");
}

// ─────────────────────────────────────────────────────────────────────────────
// MD001 – ATX heading must have space after #
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn md001_atx_heading_space_required() {
    banner("MD001: ATX heading space required");
    let bad = "##foo\n";
    let good = "## foo\n";
    let d_bad = lint(bad);
    let d_good = lint(good);
    println!("BAD  diags: {d_bad:#?}");
    println!("GOOD diags: {d_good:#?}");
    assert!(
        rule_ids(&d_bad).contains(&"MD001"),
        "Expected MD001 for '##foo'"
    );
    assert!(
        !rule_ids(&d_good).contains(&"MD001"),
        "No MD001 for '## foo'"
    );
    println!("  ✔  MD001 pass");
}

#[test]
fn md001_empty_heading_ok() {
    banner("MD001: empty ATX heading is OK");
    // Spec §4.2: `#` followed by end-of-line is a valid empty heading
    let src = "#\n## \n";
    let d = lint(src);
    println!("diags: {d:#?}");
    assert!(
        !rule_ids(&d).contains(&"MD001"),
        "empty ATX heading should not trigger MD001"
    );
    println!("  ✔  MD001 empty heading pass");
}

// ─────────────────────────────────────────────────────────────────────────────
// MD002 – ATX heading trailing #
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn md002_trailing_hash_in_content() {
    banner("MD002: trailing # in heading content");
    // Valid closing sequences
    let ok1 = "## foo ##\n";
    let ok2 = "## foo\n";
    let ok3 = "## foo # bar\n"; // mid-heading # is fine
    // Invalid: heading text itself ends with #
    let bad = "## foo#\n"; // the # is part of content, not closing
    let d_ok1 = lint(ok1);
    let d_ok2 = lint(ok2);
    let d_ok3 = lint(ok3);
    let d_bad = lint(bad);
    println!("ok1  diags: {d_ok1:#?}");
    println!("ok2  diags: {d_ok2:#?}");
    println!("ok3  diags: {d_ok3:#?}");
    println!("bad  diags: {d_bad:#?}");
    assert!(!rule_ids(&d_ok1).contains(&"MD002"));
    assert!(!rule_ids(&d_ok2).contains(&"MD002"));
    assert!(!rule_ids(&d_ok3).contains(&"MD002"));
    assert!(rule_ids(&d_bad).contains(&"MD002"));
    println!("  ✔  MD002 pass");
}

// ─────────────────────────────────────────────────────────────────────────────
// MD003 – Setext underline length
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn md003_setext_underline_length() {
    banner("MD003: setext heading underline length");
    let ok = "Foo Bar\n=======\n";
    let bad = "A longer heading\n---\n"; // underline shorter than title
    let d_ok = lint(ok);
    let d_bad = lint(bad);
    println!("ok  diags: {d_ok:#?}");
    println!("bad diags: {d_bad:#?}");
    assert!(!rule_ids(&d_ok).contains(&"MD003"));
    assert!(rule_ids(&d_bad).contains(&"MD003"));
    println!("  ✔  MD003 pass");
}

// ─────────────────────────────────────────────────────────────────────────────
// MD004 – Thematic break consistency
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn md004_thematic_break_consistency() {
    banner("MD004: thematic break consistency");
    let ok = "---\n\ntext\n\n---\n";
    let bad = "***\n\ntext\n\n---\n"; // mixed * and -
    let d_ok = lint(ok);
    let d_bad = lint(bad);
    println!("ok  diags: {d_ok:#?}");
    println!("bad diags: {d_bad:#?}");
    assert!(!rule_ids(&d_ok).contains(&"MD004"));
    assert!(rule_ids(&d_bad).contains(&"MD004"));
    println!("  ✔  MD004 pass");
}

// ─────────────────────────────────────────────────────────────────────────────
// MD005 – Unclosed fenced code block
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn md005_unclosed_fence() {
    banner("MD005: unclosed fenced code block");
    let ok = "```rust\nfn main() {}\n```\n";
    let bad = "```rust\nfn main() {}\n"; // no closing fence
    let d_ok = lint(ok);
    let d_bad = lint(bad);
    println!("ok  diags: {d_ok:#?}");
    println!("bad diags: {d_bad:#?}");
    assert!(!rule_ids(&d_ok).contains(&"MD005"));
    assert!(rule_ids(&d_bad).contains(&"MD005"));
    println!("  ✔  MD005 pass");
}

// ─────────────────────────────────────────────────────────────────────────────
// MD006 – Closing fence must not have info string
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn md006_closing_fence_no_info_string() {
    banner("MD006: closing fence info string");
    let bad = "```rust\ncode\n```rust\n";
    let d = lint(bad);
    println!("diags: {d:#?}");
    assert!(rule_ids(&d).contains(&"MD006"));
    println!("  ✔  MD006 pass");
}

// ─────────────────────────────────────────────────────────────────────────────
// MD007 – Unordered list marker consistency
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn md007_ul_marker_consistency() {
    banner("MD007: unordered list marker consistency");
    let ok = "- a\n- b\n- c\n";
    let bad = "- a\n* b\n"; // mixed
    let d_ok = lint(ok);
    let d_bad = lint(bad);
    println!("ok  diags: {d_ok:#?}");
    println!("bad diags: {d_bad:#?}");
    assert!(!rule_ids(&d_ok).contains(&"MD007"));
    assert!(rule_ids(&d_bad).contains(&"MD007"));
    println!("  ✔  MD007 pass");
}

// ─────────────────────────────────────────────────────────────────────────────
// MD008 – Ordered list starts at 1
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn md008_ordered_list_starts_at_1() {
    banner("MD008: ordered list starts at 1");
    let ok = "1. first\n2. second\n";
    let bad = "3. first\n4. second\n";
    let d_ok = lint(ok);
    let d_bad = lint(bad);
    println!("ok  diags: {d_ok:#?}");
    println!("bad diags: {d_bad:#?}");
    assert!(!rule_ids(&d_ok).contains(&"MD008"));
    assert!(rule_ids(&d_bad).contains(&"MD008"));
    println!("  ✔  MD008 pass");
}

// ─────────────────────────────────────────────────────────────────────────────
// MD009 – Task list item syntax
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn md009_task_list_syntax() {
    banner("MD009: task list item syntax");
    let ok = "- [ ] unchecked\n- [x] checked\n- [X] also checked\n";
    let bad = "- [y] bad marker\n";
    let d_ok = lint(ok);
    let d_bad = lint(bad);
    println!("ok  diags: {d_ok:#?}");
    println!("bad diags: {d_bad:#?}");
    assert!(!rule_ids(&d_ok).contains(&"MD009"));
    assert!(rule_ids(&d_bad).contains(&"MD009"));
    println!("  ✔  MD009 pass");
}

// ─────────────────────────────────────────────────────────────────────────────
// MD010 – Table must have separator row
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn md010_table_separator_required() {
    banner("MD010: GFM table must have separator row");
    let ok = "| A | B |\n| --- | --- |\n| 1 | 2 |\n";
    let bad = "| A | B |\n| 1 | 2 |\n"; // no separator
    let d_ok = lint(ok);
    let d_bad = lint(bad);
    println!("ok  diags: {d_ok:#?}");
    println!("bad diags: {d_bad:#?}");
    assert!(!rule_ids(&d_ok).contains(&"MD010"));
    assert!(rule_ids(&d_bad).contains(&"MD010"));
    println!("  ✔  MD010 pass");
}

// ─────────────────────────────────────────────────────────────────────────────
// MD011 – Table column count consistency
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn md011_table_column_consistency() {
    banner("MD011: table column count consistency");
    let ok = "| A | B |\n| --- | --- |\n| 1 | 2 |\n";
    let bad = "| A | B |\n| --- | --- |\n| 1 | 2 | 3 |\n"; // extra column
    let d_ok = lint(ok);
    let d_bad = lint(bad);
    println!("ok  diags: {d_ok:#?}");
    println!("bad diags: {d_bad:#?}");
    assert!(!rule_ids(&d_ok).contains(&"MD011"));
    assert!(rule_ids(&d_bad).contains(&"MD011"));
    println!("  ✔  MD011 pass");
}

// ─────────────────────────────────────────────────────────────────────────────
// MD012 – Trailing whitespace
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn md012_trailing_whitespace() {
    banner("MD012: trailing whitespace");
    // Two trailing spaces = hard break (allowed)
    let ok_hard_break = "line one  \nline two\n";
    // Three trailing spaces = not a hard break
    let bad_three = "line one   \nline two\n";
    // One trailing space
    let bad_one = "line one \nline two\n";
    let d_ok = lint(ok_hard_break);
    let d_bad3 = lint(bad_three);
    let d_bad1 = lint(bad_one);
    println!("hard-break diags: {d_ok:#?}");
    println!("3-spaces   diags: {d_bad3:#?}");
    println!("1-space    diags: {d_bad1:#?}");
    assert!(
        !rule_ids(&d_ok).contains(&"MD012"),
        "two trailing spaces is valid hard break"
    );
    assert!(
        rule_ids(&d_bad3).contains(&"MD012"),
        "3 trailing spaces should warn"
    );
    assert!(
        rule_ids(&d_bad1).contains(&"MD012"),
        "1 trailing space should warn"
    );
    println!("  ✔  MD012 pass");
}

// ─────────────────────────────────────────────────────────────────────────────
// MD013 – No tabs
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn md013_no_tabs() {
    banner("MD013: no tab characters");
    let ok = "    indented with spaces\n";
    let bad = "\tindented with tab\n";
    let d_ok = lint(ok);
    let d_bad = lint(bad);
    println!("ok  diags: {d_ok:#?}");
    println!("bad diags: {d_bad:#?}");
    assert!(!rule_ids(&d_ok).contains(&"MD013"));
    assert!(rule_ids(&d_bad).contains(&"MD013"));
    println!("  ✔  MD013 pass");
}

// ─────────────────────────────────────────────────────────────────────────────
// MD014 – No consecutive blank lines
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn md014_no_consecutive_blank_lines() {
    banner("MD014: no consecutive blank lines");
    let ok = "para 1\n\npara 2\n";
    let bad = "para 1\n\n\npara 2\n"; // two blank lines
    let d_ok = lint(ok);
    let d_bad = lint(bad);
    println!("ok  diags: {d_ok:#?}");
    println!("bad diags: {d_bad:#?}");
    assert!(!rule_ids(&d_ok).contains(&"MD014"));
    assert!(rule_ids(&d_bad).contains(&"MD014"));
    println!("  ✔  MD014 pass");
}

// ─────────────────────────────────────────────────────────────────────────────
// MD015 – Image alt text
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn md015_image_alt_text() {
    banner("MD015: image must have alt text");
    let ok = "![a cat](cat.png)\n";
    let bad = "![](cat.png)\n";
    let d_ok = lint(ok);
    let d_bad = lint(bad);
    println!("ok  diags: {d_ok:#?}");
    println!("bad diags: {d_bad:#?}");
    assert!(!rule_ids(&d_ok).contains(&"MD015"));
    assert!(rule_ids(&d_bad).contains(&"MD015"));
    println!("  ✔  MD015 pass");
}

// ─────────────────────────────────────────────────────────────────────────────
// MD016 – Reference link resolution
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn md016_reference_link_resolution() {
    banner("MD016: reference link must have a definition");
    // Reference with definition
    let ok = "[foo][bar]\n\n[bar]: https://example.com\n";
    // Reference without definition
    let bad = "[foo][baz]\n";
    let d_ok = lint(ok);
    let d_bad = lint(bad);
    println!("ok  diags: {d_ok:#?}");
    println!("bad diags: {d_bad:#?}");
    assert!(
        !rule_ids(&d_ok).contains(&"MD016"),
        "defined reference should not trigger MD016"
    );
    assert!(
        rule_ids(&d_bad).contains(&"MD016"),
        "undefined reference should trigger MD016"
    );
    println!("  ✔  MD016 pass");
}

// ─────────────────────────────────────────────────────────────────────────────
// Severity levels
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn severity_levels_correct() {
    banner("Severity: errors vs warnings");
    // MD005 (unclosed fence) is an error
    let d = lint("```\nno close\n");
    let err: Vec<_> = errors(&d);
    let warn: Vec<_> = warnings(&d);
    println!("errors:   {err:#?}");
    println!("warnings: {warn:#?}");
    assert!(!err.is_empty(), "unclosed fence should be an error");
    // MD007 (inconsistent UL marker) is a warning
    let d2 = lint("- a\n* b\n");
    let warn2: Vec<_> = warnings(&d2);
    println!("warnings: {warn2:#?}");
    assert!(
        !warn2.is_empty(),
        "inconsistent UL marker should be a warning"
    );
    println!("  ✔  Severity pass");
}

// ─────────────────────────────────────────────────────────────────────────────
// Clean document
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn clean_document_no_diags() {
    banner("Clean document: no diagnostics expected");
    let src = r#"# Hello World

A paragraph of text with [a link](https://example.com) and
![alt text](image.png).

## Section Two

- item one
- item two
- item three

1. first
2. second

```rust
fn main() {
    println!("Hello!");
}
```

| Name | Value |
| ---- | ----- |
| foo  | bar   |

---

> A blockquote.
"#;
    let d = lint(src);
    println!("diags: {d:#?}");
    assert!(
        d.is_empty(),
        "clean document should produce no diagnostics, got: {d:#?}"
    );
    println!("  ✔  Clean document pass");
}

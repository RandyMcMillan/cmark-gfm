//! Integration tests for the `cmark-gfm-ffi` feature.
//!
//! These tests exercise both the raw C FFI layer (`sys` functions) and the
//! ergonomic safe Rust wrappers.  All output uses banner-style reporting so
//! failures are immediately obvious in CI logs.
//!
//! Run with:
//!   cargo test --features cmark-gfm-ffi --test ffi_test -- --nocapture

#![cfg(feature = "cmark-gfm-ffi")]

use gfm_linter::ffi::{
    self, markdown_to_commonmark, markdown_to_html, markdown_to_html_streaming,
    markdown_to_plaintext, markdown_to_xml, version, version_string, Options,
    CMARK_OPT_DEFAULT,
};
use std::ffi::CStr;
use std::os::raw::c_char;

// ─────────────────────────────────────────────────────────────────────────────
// Banner helpers
// ─────────────────────────────────────────────────────────────────────────────

fn banner(label: &str) {
    println!("\n╔══════════════════════════════════════════════════╗");
    println!(
        "║  {}{} ║",
        label,
        " ".repeat(48_usize.saturating_sub(label.len() + 3))
    );
    println!("╚══════════════════════════════════════════════════╝");
}

fn banner_pass(label: &str) {
    println!("  ✅  PASS  {label}");
}

fn banner_section(label: &str) {
    println!("\n  ─── {label} ───");
}

// ─────────────────────────────────────────────────────────────────────────────
// version / library info
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_version_string() {
    banner("FFI: version_string()");
    let v = version_string();
    println!("  cmark-gfm version : {v}");
    assert!(!v.is_empty(), "version string must not be empty");
    assert!(
        v.contains("gfm"),
        "version string should contain 'gfm', got: {v}"
    );
    banner_pass("version_string");
}

#[test]
fn test_version_integer() {
    banner("FFI: version()");
    let v = version();
    println!("  cmark-gfm version (packed int) : 0x{v:08x} = {v}");
    // version 0.29.0.gfm.13 → (0<<24)|(29<<16)|(0<<8)|13 = 0x001d000d
    assert!(v > 0, "version integer must be > 0");
    banner_pass("version integer");
}

// ─────────────────────────────────────────────────────────────────────────────
// markdown_to_html (simple wrapper)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_html_paragraph() {
    banner("FFI: markdown_to_html – paragraph");
    let input = "Hello, *world*!\n";
    let html = markdown_to_html(input, Options::DEFAULT);
    println!("  input : {input:?}");
    println!("  html  : {html:?}");
    assert!(
        html.contains("<em>world</em>"),
        "expected <em>world</em> in:\n{html}"
    );
    assert!(html.contains("<p>"), "expected <p> tag");
    banner_pass("paragraph");
}

#[test]
fn test_html_strong() {
    banner("FFI: markdown_to_html – strong");
    let input = "**bold text**\n";
    let html = markdown_to_html(input, Options::DEFAULT);
    println!("  input : {input:?}");
    println!("  html  : {html:?}");
    assert!(
        html.contains("<strong>bold text</strong>"),
        "expected <strong>bold text</strong>:\n{html}"
    );
    banner_pass("strong");
}

#[test]
fn test_html_headings() {
    banner("FFI: markdown_to_html – headings h1-h3");
    for (n, marker) in [(1usize, "# "), (2, "## "), (3, "### ")] {
        let input = format!("{marker}Heading {n}\n");
        let html = markdown_to_html(&input, Options::DEFAULT);
        println!("  h{n}: {html:?}");
        assert!(
            html.contains(&format!("<h{n}>")),
            "expected <h{n}> in:\n{html}"
        );
    }
    banner_pass("headings h1-h3");
}

#[test]
fn test_html_code_block() {
    banner("FFI: markdown_to_html – fenced code block");
    let input = "```rust\nfn main() {}\n```\n";
    let html = markdown_to_html(input, Options::DEFAULT);
    println!("  input : {input:?}");
    println!("  html  : {html:?}");
    assert!(html.contains("<code"), "expected <code> tag");
    assert!(html.contains("fn main()"), "expected code content");
    banner_pass("fenced code block");
}

#[test]
fn test_html_github_pre_lang() {
    banner("FFI: markdown_to_html – GITHUB_PRE_LANG option");
    let input = "```python\nprint('hello')\n```\n";
    let opts = Options::DEFAULT | Options::GITHUB_PRE_LANG;
    let html = markdown_to_html(input, opts);
    println!("  html  : {html:?}");
    assert!(
        html.contains(r#"lang="python""#) || html.contains("python"),
        "expected lang attribute or python in:\n{html}"
    );
    banner_pass("GITHUB_PRE_LANG");
}

#[test]
fn test_html_blockquote() {
    banner("FFI: markdown_to_html – blockquote");
    let input = "> This is a quote.\n";
    let html = markdown_to_html(input, Options::DEFAULT);
    println!("  html  : {html:?}");
    assert!(html.contains("<blockquote>"), "expected <blockquote>");
    banner_pass("blockquote");
}

#[test]
fn test_html_ordered_list() {
    banner("FFI: markdown_to_html – ordered list");
    let input = "1. First\n2. Second\n3. Third\n";
    let html = markdown_to_html(input, Options::DEFAULT);
    println!("  html  : {html:?}");
    assert!(html.contains("<ol>"), "expected <ol>");
    assert!(html.contains("<li>First</li>") || html.contains("<li>"), "expected list items");
    banner_pass("ordered list");
}

#[test]
fn test_html_unordered_list() {
    banner("FFI: markdown_to_html – unordered list");
    let input = "- Alpha\n- Beta\n- Gamma\n";
    let html = markdown_to_html(input, Options::DEFAULT);
    println!("  html  : {html:?}");
    assert!(html.contains("<ul>"), "expected <ul>");
    banner_pass("unordered list");
}

#[test]
fn test_html_link() {
    banner("FFI: markdown_to_html – hyperlink");
    let input = "[Rust](https://www.rust-lang.org/)\n";
    let html = markdown_to_html(input, Options::DEFAULT);
    println!("  html  : {html:?}");
    assert!(
        html.contains(r#"href="https://www.rust-lang.org/""#),
        "expected href attribute"
    );
    assert!(html.contains(">Rust<"), "expected link text");
    banner_pass("hyperlink");
}

#[test]
fn test_html_image() {
    banner("FFI: markdown_to_html – image");
    let input = "![alt text](image.png)\n";
    let html = markdown_to_html(input, Options::DEFAULT);
    println!("  html  : {html:?}");
    assert!(html.contains("<img"), "expected <img> tag");
    assert!(html.contains(r#"src="image.png""#), "expected src");
    banner_pass("image");
}

#[test]
fn test_html_horizontal_rule() {
    banner("FFI: markdown_to_html – thematic break / <hr>");
    let input = "---\n";
    let html = markdown_to_html(input, Options::DEFAULT);
    println!("  html  : {html:?}");
    assert!(html.contains("<hr"), "expected <hr> tag");
    banner_pass("thematic break");
}

#[test]
fn test_html_hardbreaks_option() {
    banner("FFI: markdown_to_html – HARDBREAKS option");
    let input = "line one\nline two\n";
    let html_default = markdown_to_html(input, Options::DEFAULT);
    let html_hard = markdown_to_html(input, Options::HARDBREAKS);
    println!("  default  : {html_default:?}");
    println!("  hardbreak: {html_hard:?}");
    assert!(
        html_hard.contains("<br"),
        "expected <br> with HARDBREAKS option"
    );
    banner_pass("HARDBREAKS option");
}

#[test]
fn test_html_smart_option() {
    banner("FFI: markdown_to_html – SMART punctuation");
    let input = "\"quoted\" and 'single'\n";
    let html = markdown_to_html(input, Options::SMART);
    println!("  html: {html:?}");
    // Smart quotes should produce curly quotes (Unicode or HTML entities)
    let has_curly = html.contains('\u{201C}')
        || html.contains('\u{201D}')
        || html.contains("&ldquo;")
        || html.contains("&rdquo;");
    assert!(has_curly, "expected smart/curly quotes in:\n{html}");
    banner_pass("SMART option");
}

#[test]
fn test_html_empty_input() {
    banner("FFI: markdown_to_html – empty input");
    let html = markdown_to_html("", Options::DEFAULT);
    println!("  html: {html:?}");
    // empty markdown → empty or just a newline
    assert!(
        html.trim().is_empty(),
        "empty markdown should produce empty HTML, got: {html:?}"
    );
    banner_pass("empty input");
}

#[test]
fn test_html_unicode() {
    banner("FFI: markdown_to_html – Unicode content");
    let input = "# こんにちは 🌍\n\n日本語テスト\n";
    let html = markdown_to_html(input, Options::DEFAULT);
    println!("  html: {html:?}");
    assert!(html.contains("こんにちは"), "expected Japanese characters");
    assert!(html.contains("🌍"), "expected emoji");
    banner_pass("Unicode content");
}

// ─────────────────────────────────────────────────────────────────────────────
// markdown_to_html_streaming (parser path)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_streaming_paragraph() {
    banner("FFI: markdown_to_html_streaming – paragraph");
    let input = "Hello from the **streaming** parser!\n";
    let html = markdown_to_html_streaming(input, Options::DEFAULT);
    println!("  html: {html:?}");
    assert!(
        html.contains("<strong>streaming</strong>"),
        "expected <strong> in:\n{html}"
    );
    banner_pass("streaming paragraph");
}

#[test]
fn test_streaming_matches_simple() {
    banner("FFI: streaming output matches simple output");
    let cases = &[
        "# Title\n",
        "- item one\n- item two\n",
        "```\ncode block\n```\n",
        "> blockquote\n",
    ];
    for case in cases {
        let simple = markdown_to_html(case, Options::DEFAULT);
        let stream = markdown_to_html_streaming(case, Options::DEFAULT);
        println!("  input  : {case:?}");
        println!("  simple : {simple:?}");
        println!("  stream : {stream:?}");
        assert_eq!(
            simple, stream,
            "simple and streaming outputs differ for {case:?}"
        );
    }
    banner_pass("streaming matches simple");
}

// ─────────────────────────────────────────────────────────────────────────────
// markdown_to_commonmark
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_commonmark_roundtrip() {
    banner("FFI: markdown_to_commonmark – round-trip");
    let input = "# Hello\n\nThis is a *paragraph*.\n";
    let cm = markdown_to_commonmark(input, Options::DEFAULT, 0);
    println!("  input : {input:?}");
    println!("  cm    : {cm:?}");
    assert!(cm.contains("Hello"), "expected heading text");
    assert!(cm.contains("*paragraph*") || cm.contains("_paragraph_"),
            "expected emphasis preserved");
    banner_pass("commonmark round-trip");
}

// ─────────────────────────────────────────────────────────────────────────────
// markdown_to_plaintext
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_plaintext_strips_markup() {
    banner("FFI: markdown_to_plaintext – markup stripped");
    let input = "# Title\n\n**bold** and _italic_ text.\n";
    let plain = markdown_to_plaintext(input, Options::DEFAULT, 0);
    println!("  input : {input:?}");
    println!("  plain : {plain:?}");
    assert!(!plain.contains("<"), "plaintext must not contain HTML tags");
    assert!(plain.contains("Title"), "expected heading text");
    assert!(plain.contains("bold"), "expected bold word");
    assert!(plain.contains("italic"), "expected italic word");
    banner_pass("markup stripped");
}

// ─────────────────────────────────────────────────────────────────────────────
// markdown_to_xml
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_xml_document_structure() {
    banner("FFI: markdown_to_xml – document structure");
    let input = "# Hi\n\nParagraph.\n";
    let xml = markdown_to_xml(input, Options::DEFAULT);
    println!("  xml excerpt: {}", &xml[..xml.len().min(300)]);
    assert!(xml.contains("<?xml") || xml.contains("<document"), "expected XML prolog or <document>");
    assert!(xml.contains("heading") || xml.contains("atx_heading"),
            "expected heading element in XML");
    banner_pass("XML document structure");
}

// ─────────────────────────────────────────────────────────────────────────────
// Raw sys-layer FFI
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_raw_cmark_markdown_to_html() {
    banner("FFI (sys): cmark_markdown_to_html raw");
    let md = b"**raw** FFI call\n";
    let raw = unsafe {
        ffi::cmark_markdown_to_html(
            md.as_ptr() as *const c_char,
            md.len(),
            CMARK_OPT_DEFAULT,
        )
    };
    assert!(!raw.is_null(), "raw pointer must not be NULL");
    let s = unsafe { CStr::from_ptr(raw) }.to_str().unwrap().to_owned();
    println!("  html: {s:?}");
    assert!(s.contains("<strong>raw</strong>"), "expected <strong> tag");
    // Free via the same shim used in the safe wrappers
    unsafe {
        extern "C" {
            fn free(ptr: *mut std::ffi::c_void);
        }
        free(raw as *mut _);
    }
    banner_pass("raw cmark_markdown_to_html");
}

#[test]
fn test_raw_parser_lifecycle() {
    banner("FFI (sys): parser new / feed / finish / free");
    let md = b"Hello *parser*\n";
    unsafe {
        let parser = ffi::cmark_parser_new(CMARK_OPT_DEFAULT);
        assert!(!parser.is_null(), "parser must not be NULL");
        ffi::cmark_parser_feed(parser, md.as_ptr() as *const c_char, md.len());
        let doc = ffi::cmark_parser_finish(parser);
        assert!(!doc.is_null(), "document must not be NULL");
        let html_raw = ffi::cmark_render_html(doc, CMARK_OPT_DEFAULT, std::ptr::null_mut());
        assert!(!html_raw.is_null(), "html must not be NULL");
        let html = CStr::from_ptr(html_raw).to_str().unwrap().to_owned();
        println!("  html: {html:?}");
        assert!(html.contains("<em>parser</em>"), "expected <em> tag");
        extern "C" { fn free(ptr: *mut std::ffi::c_void); }
        free(html_raw as *mut _);
        ffi::cmark_node_free(doc);
    }
    banner_pass("raw parser lifecycle");
}

#[test]
fn test_raw_version_functions() {
    banner("FFI (sys): cmark_version / cmark_version_string");
    let v_int = unsafe { ffi::cmark_version() };
    let v_str = unsafe { CStr::from_ptr(ffi::cmark_version_string()) }
        .to_str()
        .unwrap();
    println!("  version int    : {v_int}");
    println!("  version string : {v_str}");
    assert!(v_int > 0);
    assert!(!v_str.is_empty());
    banner_pass("raw version functions");
}

// ─────────────────────────────────────────────────────────────────────────────
// Options bitwise operations
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_options_bitor() {
    banner("FFI: Options – bitwise OR");
    let combined = Options::SMART | Options::HARDBREAKS | Options::FOOTNOTES;
    let expected = Options(
        ffi::CMARK_OPT_SMART | ffi::CMARK_OPT_HARDBREAKS | ffi::CMARK_OPT_FOOTNOTES,
    );
    println!("  combined : {:?}", combined);
    println!("  expected : {:?}", expected);
    assert_eq!(combined, expected);
    banner_pass("Options bitwise OR");
}

#[test]
fn test_options_bitand() {
    banner("FFI: Options – bitwise AND");
    let combined = Options::SMART | Options::HARDBREAKS;
    let masked = combined & Options::SMART;
    println!("  masked : {:?}", masked);
    assert_eq!(masked, Options::SMART);
    banner_pass("Options bitwise AND");
}

// ─────────────────────────────────────────────────────────────────────────────
// Stress / edge cases
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_large_document() {
    banner("FFI: markdown_to_html – large document");
    let mut input = String::new();
    for i in 0..500 {
        input.push_str(&format!("## Section {i}\n\nParagraph {i} with *emphasis*.\n\n"));
    }
    let html = markdown_to_html(&input, Options::DEFAULT);
    println!("  input length : {} bytes", input.len());
    println!("  html  length : {} bytes", html.len());
    assert!(html.len() > input.len(), "HTML should be longer than raw markdown");
    assert!(html.contains("<h2>"), "expected h2 headings");
    banner_pass("large document");
}

#[test]
fn test_nested_emphasis() {
    banner("FFI: markdown_to_html – nested emphasis");
    let input = "***bold and italic***\n";
    let html = markdown_to_html(input, Options::DEFAULT);
    println!("  html: {html:?}");
    // Nested strong + em (order may vary between parsers)
    assert!(
        (html.contains("<strong>") && html.contains("<em>"))
            || html.contains("<em><strong>")
            || html.contains("<strong><em>"),
        "expected nested bold+italic in:\n{html}"
    );
    banner_pass("nested emphasis");
}

#[test]
fn test_multiple_paragraphs() {
    banner("FFI: markdown_to_html – multiple paragraphs");
    let input = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.\n";
    let html = markdown_to_html(input, Options::DEFAULT);
    println!("  html: {html:?}");
    let p_count = html.matches("<p>").count();
    assert_eq!(p_count, 3, "expected 3 <p> tags, found {p_count}");
    banner_pass("multiple paragraphs");
}

#[test]
fn test_inline_code() {
    banner("FFI: markdown_to_html – inline code");
    let input = "Use `cargo test` to run tests.\n";
    let html = markdown_to_html(input, Options::DEFAULT);
    println!("  html: {html:?}");
    assert!(html.contains("<code>cargo test</code>"), "expected inline code");
    banner_pass("inline code");
}

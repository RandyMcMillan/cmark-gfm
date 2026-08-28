//! # cmark-gfm FFI
//!
//! Low-level (`sys`) FFI declarations for the bundled `cmark-gfm` C library
//! together with ergonomic, memory-safe Rust wrappers.
//!
//! ## Feature gate
//!
//! This module is only compiled when the `cmark-gfm-ffi` Cargo feature is
//! enabled:
//!
//! ```toml
//! [dependencies]
//! gfm-linter = { version = "*", features = ["cmark-gfm-ffi"] }
//! ```
//!
//! ## Quick start
//!
//! ```rust,ignore
//! # #[cfg(feature = "cmark-gfm-ffi")] {
//! use gfm_linter::ffi::{markdown_to_html, Options};
//!
//! let html = markdown_to_html("**Hello**, _world_!", Options::DEFAULT);
//! assert!(html.contains("<strong>Hello</strong>"));
//! # }
//! ```

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

// ─────────────────────────────────────────────────────────────────────────────
// Raw C option constants (mirrors cmark-gfm.h)
// ─────────────────────────────────────────────────────────────────────────────

/// Default processing options (no flags set).
pub const CMARK_OPT_DEFAULT: c_int = 0;
/// Include source-position information in the AST.
pub const CMARK_OPT_SOURCEPOS: c_int = 1 << 1;
/// Render `\n` in paragraphs as `<br />`.
pub const CMARK_OPT_HARDBREAKS: c_int = 1 << 2;
/// (Deprecated alias – kept for API compatibility.) Same as no flag.
pub const CMARK_OPT_SAFE: c_int = 1 << 3;
/// Allow raw HTML and dangerous URLs.
pub const CMARK_OPT_UNSAFE: c_int = 1 << 17;
/// Render soft-line breaks as spaces.
pub const CMARK_OPT_NOBREAKS: c_int = 1 << 4;
/// Normalise the tree before rendering.
pub const CMARK_OPT_NORMALIZE: c_int = 1 << 8;
/// Validate UTF-8 in the input.
pub const CMARK_OPT_VALIDATE_UTF8: c_int = 1 << 9;
/// Smart punctuation (en/em-dashes, curly quotes).
pub const CMARK_OPT_SMART: c_int = 1 << 10;
/// Use GitHub-style `<pre lang="…">` for fenced code blocks.
pub const CMARK_OPT_GITHUB_PRE_LANG: c_int = 1 << 11;
/// Use `<pre>` style for fenced code blocks (liberal HTML).
pub const CMARK_OPT_LIBERAL_HTML_TAG: c_int = 1 << 12;
/// Enable footnotes.
pub const CMARK_OPT_FOOTNOTES: c_int = 1 << 13;
/// Require `~~` (double tilde) for strikethrough.
pub const CMARK_OPT_STRIKETHROUGH_DOUBLE_TILDE: c_int = 1 << 14;
/// Use `style` attributes on table cells instead of `align`.
pub const CMARK_OPT_TABLE_PREFER_STYLE_ATTRIBUTES: c_int = 1 << 15;
/// Include the full info string on fenced code blocks.
pub const CMARK_OPT_FULL_INFO_STRING: c_int = 1 << 16;

// ─────────────────────────────────────────────────────────────────────────────
// Raw C declarations (sys layer)
// ─────────────────────────────────────────────────────────────────────────────

/// Opaque C types.
#[repr(C)]
pub struct CmarkNode {
    _private: [u8; 0],
}

#[repr(C)]
pub struct CmarkParser {
    _private: [u8; 0],
}

#[repr(C)]
pub struct CmarkIter {
    _private: [u8; 0],
}

extern "C" {
    // ── simple interface ────────────────────────────────────────────────────

    /// Convert a UTF-8 markdown string of `len` bytes to a heap-allocated,
    /// null-terminated HTML string.  The caller must free the result with
    /// [`cmark_get_default_mem_allocator`] or the system free().
    ///
    /// # Safety
    /// `text` must be a valid pointer to `len` readable bytes.
    pub fn cmark_markdown_to_html(
        text: *const c_char,
        len: usize,
        options: c_int,
    ) -> *mut c_char;

    // ── streaming parser ────────────────────────────────────────────────────

    /// Create a new streaming parser.
    pub fn cmark_parser_new(options: c_int) -> *mut CmarkParser;

    /// Feed bytes to the parser.
    ///
    /// # Safety
    /// `buffer` must point to `len` readable bytes.
    pub fn cmark_parser_feed(
        parser: *mut CmarkParser,
        buffer: *const c_char,
        len: usize,
    );

    /// Finish parsing and return the document root.  The parser is consumed.
    pub fn cmark_parser_finish(parser: *mut CmarkParser) -> *mut CmarkNode;

    /// Free a streaming parser **without** finishing it.
    pub fn cmark_parser_free(parser: *mut CmarkParser);

    // ── one-shot document parse ─────────────────────────────────────────────

    /// Parse a complete markdown document and return the AST root.
    ///
    /// # Safety
    /// `buffer` must point to `len` readable bytes.
    pub fn cmark_parse_document(
        buffer: *const c_char,
        len: usize,
        options: c_int,
    ) -> *mut CmarkNode;

    // ── rendering ───────────────────────────────────────────────────────────

    /// Render a parsed AST to HTML.  Caller must free the result.
    pub fn cmark_render_html(
        root: *mut CmarkNode,
        options: c_int,
        extensions: *mut std::ffi::c_void,
    ) -> *mut c_char;

    /// Render a parsed AST to CommonMark text.  Caller must free the result.
    pub fn cmark_render_commonmark(
        root: *mut CmarkNode,
        options: c_int,
        width: c_int,
    ) -> *mut c_char;

    /// Render a parsed AST to plain text.  Caller must free the result.
    pub fn cmark_render_plaintext(
        root: *mut CmarkNode,
        options: c_int,
        width: c_int,
    ) -> *mut c_char;

    /// Render a parsed AST to XML.  Caller must free the result.
    pub fn cmark_render_xml(root: *mut CmarkNode, options: c_int) -> *mut c_char;

    /// Render a parsed AST to LaTeX.  Caller must free the result.
    pub fn cmark_render_latex(
        root: *mut CmarkNode,
        options: c_int,
        width: c_int,
    ) -> *mut c_char;

    // ── node lifecycle ──────────────────────────────────────────────────────

    /// Free the AST rooted at `root`.
    pub fn cmark_node_free(root: *mut CmarkNode);

    // ── version ─────────────────────────────────────────────────────────────

    /// Return the library version as a packed integer.
    pub fn cmark_version() -> c_int;

    /// Return the library version as a null-terminated string.
    pub fn cmark_version_string() -> *const c_char;
}

// ─────────────────────────────────────────────────────────────────────────────
// Safe Rust wrappers
// ─────────────────────────────────────────────────────────────────────────────

/// Bitfield wrapper for `CMARK_OPT_*` constants.
///
/// Individual flags can be OR-combined:
///
/// ```rust,ignore
/// # #[cfg(feature = "cmark-gfm-ffi")] {
/// use gfm_linter::ffi::Options;
/// let opts = Options::DEFAULT | Options::SMART | Options::HARDBREAKS;
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Options(pub c_int);

impl Options {
    /// Default (no flags).
    pub const DEFAULT: Options = Options(CMARK_OPT_DEFAULT);
    /// Include source position info.
    pub const SOURCEPOS: Options = Options(CMARK_OPT_SOURCEPOS);
    /// Hard line-breaks.
    pub const HARDBREAKS: Options = Options(CMARK_OPT_HARDBREAKS);
    /// Allow raw HTML / dangerous URLs.
    pub const UNSAFE: Options = Options(CMARK_OPT_UNSAFE);
    /// Smart punctuation.
    pub const SMART: Options = Options(CMARK_OPT_SMART);
    /// GitHub `<pre lang="…">` fenced blocks.
    pub const GITHUB_PRE_LANG: Options = Options(CMARK_OPT_GITHUB_PRE_LANG);
    /// Enable footnotes.
    pub const FOOTNOTES: Options = Options(CMARK_OPT_FOOTNOTES);
    /// Validate UTF-8.
    pub const VALIDATE_UTF8: Options = Options(CMARK_OPT_VALIDATE_UTF8);
    /// Include full info string on fenced code blocks.
    pub const FULL_INFO_STRING: Options = Options(CMARK_OPT_FULL_INFO_STRING);
}

impl std::ops::BitOr for Options {
    type Output = Options;
    fn bitor(self, rhs: Options) -> Options {
        Options(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for Options {
    type Output = Options;
    fn bitand(self, rhs: Options) -> Options {
        Options(self.0 & rhs.0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────

/// Convert a Rust `&str` (markdown) to an HTML `String` using the simple
/// `cmark_markdown_to_html` C function.
///
/// # Panics
/// Panics if `source` contains interior null bytes (extremely unusual in
/// practice – markdown files should not contain NUL characters).
///
/// # Example
/// ```rust,ignore
/// # #[cfg(feature = "cmark-gfm-ffi")] {
/// use gfm_linter::ffi::{markdown_to_html, Options};
/// let html = markdown_to_html("# Hello\n", Options::DEFAULT);
/// assert!(html.starts_with("<h1>"));
/// # }
/// ```
pub fn markdown_to_html(source: &str, options: Options) -> String {
    let bytes = source.as_bytes();
    // SAFETY: `bytes.as_ptr()` is valid for `bytes.len()` bytes; the C
    // function returns a heap-allocated, NUL-terminated string which we
    // immediately copy into a Rust String and then free.
    let raw = unsafe {
        cmark_markdown_to_html(bytes.as_ptr() as *const c_char, bytes.len(), options.0)
    };
    assert!(!raw.is_null(), "cmark_markdown_to_html returned NULL");
    let s = unsafe { CStr::from_ptr(raw) }
        .to_string_lossy()
        .into_owned();
    // Free the C-allocated buffer.  `libc::free` is the correct way but we
    // can use the stable `std::alloc` trick only if we know the allocator.
    // cmark uses the system malloc, so calling libc free is correct; however
    // to avoid a libc dependency we use the equivalent via a zero-cost shim.
    unsafe { c_free(raw) };
    s
}

/// Convert a Rust `&str` to HTML using the streaming parser.  This is
/// functionally equivalent to [`markdown_to_html`] but exercises the
/// `cmark_parser_new` / `cmark_parser_feed` / `cmark_parser_finish` /
/// `cmark_render_html` path.
pub fn markdown_to_html_streaming(source: &str, options: Options) -> String {
    let bytes = source.as_bytes();
    unsafe {
        let parser = cmark_parser_new(options.0);
        assert!(!parser.is_null(), "cmark_parser_new returned NULL");
        cmark_parser_feed(parser, bytes.as_ptr() as *const c_char, bytes.len());
        let doc = cmark_parser_finish(parser);
        // parser is freed by cmark_parser_finish
        assert!(!doc.is_null(), "cmark_parser_finish returned NULL");
        let raw = cmark_render_html(doc, options.0, std::ptr::null_mut());
        assert!(!raw.is_null(), "cmark_render_html returned NULL");
        let s = CStr::from_ptr(raw).to_string_lossy().into_owned();
        c_free(raw);
        cmark_node_free(doc);
        s
    }
}

/// Convert a Rust `&str` to CommonMark (round-trip normalisation).
///
/// `width` is the column wrap width (0 = no wrap).
pub fn markdown_to_commonmark(source: &str, options: Options, width: c_int) -> String {
    let bytes = source.as_bytes();
    unsafe {
        let doc =
            cmark_parse_document(bytes.as_ptr() as *const c_char, bytes.len(), options.0);
        assert!(!doc.is_null(), "cmark_parse_document returned NULL");
        let raw = cmark_render_commonmark(doc, options.0, width);
        assert!(!raw.is_null(), "cmark_render_commonmark returned NULL");
        let s = CStr::from_ptr(raw).to_string_lossy().into_owned();
        c_free(raw);
        cmark_node_free(doc);
        s
    }
}

/// Convert a Rust `&str` to plain text (all markup stripped).
///
/// `width` is the column wrap width (0 = no wrap).
pub fn markdown_to_plaintext(source: &str, options: Options, width: c_int) -> String {
    let bytes = source.as_bytes();
    unsafe {
        let doc =
            cmark_parse_document(bytes.as_ptr() as *const c_char, bytes.len(), options.0);
        assert!(!doc.is_null(), "cmark_parse_document returned NULL");
        let raw = cmark_render_plaintext(doc, options.0, width);
        assert!(!raw.is_null(), "cmark_render_plaintext returned NULL");
        let s = CStr::from_ptr(raw).to_string_lossy().into_owned();
        c_free(raw);
        cmark_node_free(doc);
        s
    }
}

/// Convert a Rust `&str` to an XML AST dump.
pub fn markdown_to_xml(source: &str, options: Options) -> String {
    let bytes = source.as_bytes();
    unsafe {
        let doc =
            cmark_parse_document(bytes.as_ptr() as *const c_char, bytes.len(), options.0);
        assert!(!doc.is_null(), "cmark_parse_document returned NULL");
        let raw = cmark_render_xml(doc, options.0);
        assert!(!raw.is_null(), "cmark_render_xml returned NULL");
        let s = CStr::from_ptr(raw).to_string_lossy().into_owned();
        c_free(raw);
        cmark_node_free(doc);
        s
    }
}

/// Return the bundled library version as a `String` (e.g. `"0.29.0.gfm.13"`).
pub fn version_string() -> String {
    unsafe {
        CStr::from_ptr(cmark_version_string())
            .to_string_lossy()
            .into_owned()
    }
}

/// Return the bundled library version as a packed `i32`.
pub fn version() -> i32 {
    unsafe { cmark_version() }
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Free a pointer allocated by the cmark C library (which uses the system
/// `malloc`/`free`).  We call the C `free()` via a minimal shim to avoid
/// adding a `libc` dependency.
unsafe fn c_free(ptr: *mut c_char) {
    extern "C" {
        fn free(ptr: *mut std::ffi::c_void);
    }
    free(ptr as *mut _);
}

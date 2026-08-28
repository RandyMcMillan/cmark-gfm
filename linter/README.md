# gfm-linter

A Rust markdown linter that enforces the
[GitHub Flavored Markdown specification](../test/spec.txt)
(CommonMark 0.29 + GFM extensions).

## Rules

| Rule ID | Category       | Severity | Description                                                         |
|---------|----------------|----------|---------------------------------------------------------------------|
| MD001   | Headings       | error    | ATX heading `#` must be followed by a space                        |
| MD002   | Headings       | warning  | ATX heading text must not end with an unescaped `#`                |
| MD003   | Headings       | warning  | Setext underline must be ≥ the length of the heading text          |
| MD004   | Thematic break | warning  | All thematic breaks must use the same character (`*`, `-`, or `_`) |
| MD005   | Code fence     | error    | Every opening fence must have a matching closing fence             |
| MD006   | Code fence     | error    | Closing fence must not carry an info string                        |
| MD007   | Lists          | warning  | Unordered list markers must be consistent across the document      |
| MD008   | Lists          | warning  | The first ordered-list item must start at `1`                      |
| MD009   | Lists          | error    | Task list items must use `[ ]` or `[x]` / `[X]`                   |
| MD010   | Tables         | error    | GFM table must have a header-separator row                         |
| MD011   | Tables         | warning  | All table rows must have the same column count as the header       |
| MD012   | Whitespace     | warning  | No trailing whitespace (two trailing spaces = hard line break)     |
| MD013   | Whitespace     | warning  | No tab characters (use spaces)                                     |
| MD014   | Whitespace     | warning  | No more than one consecutive blank line                            |
| MD015   | Links          | warning  | Inline image must have non-empty alt text                          |
| MD016   | Links          | error    | Reference link/image must have a matching link definition          |

## Usage

```
gfm-lint [--no-warnings] [--json] [--recursive] [--depth N] <PATH>...
```

### Options

| Flag            | Effect                                              |
|-----------------|-----------------------------------------------------|
| `--no-warnings` | Suppress `warning`-level diagnostics                |
| `--json`        | Output one JSON object per diagnostic (NDJSON)      |
| `--recursive`   | Recurse into directories without a depth limit      |
| `--depth N`     | Recurse into directories up to `N` levels deep      |
| `--help`        | Print help text                                     |

### Exit codes

| Code | Meaning                          |
|------|----------------------------------|
| 0    | No issues found                  |
| 1    | One or more **errors** found     |
| 2    | One or more **warnings** found   |
| 3    | Usage or I/O error               |

### Example

```
$ gfm-lint README.md
README.md:3:1: warning [MD014] More than one consecutive blank line
README.md:12:1: error [MD005] Unclosed fenced code block
```

```
$ gfm-lint --json README.md
{"file":"README.md","line":3,"col":1,"severity":"warning","rule":"MD014","message":"More than one consecutive blank line"}
```

Directories are linted at depth `0` by default, so `gfm-lint docs/` only
checks files directly inside `docs/`. Use `--depth N` to include nested
subdirectories, or `--recursive` for unlimited depth.

## Building

```sh
cd linter
cargo build --release
```

The binary will be at `target/release/gfm-lint`.

## Testing

```sh
cd linter
cargo test -- --nocapture
```

## Spec reference

Rules are derived from the GFM specification at
[`test/spec.txt`](../test/spec.txt) (version 0.29, 2019-04-06).

Relevant sections:

- **§2.1** Characters and lines → MD012, MD013
- **§4.1** Thematic breaks      → MD004
- **§4.2** ATX headings         → MD001, MD002
- **§4.3** Setext headings      → MD003
- **§4.5** Fenced code blocks   → MD005, MD006
- **§4.7** Link reference defs  → MD016
- **§4.10** Tables (extension)  → MD010, MD011
- **§5.2** List items           → MD008, MD009
- **§5.3** Lists                → MD007
- **§5.4** Task list (extension)→ MD009
- **§6.6** Links                → MD016
- **§6.7** Images               → MD015
- **§9.1** Hard line breaks     → MD012

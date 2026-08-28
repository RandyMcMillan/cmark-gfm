//! `gfm-lint` – GitHub Flavored Markdown linter CLI.
//!
//! ## Usage
//!
//! ```text
//! gfm-lint [OPTIONS] <FILE>...
//!
//! Options:
//!   --no-warnings   Suppress warning-level diagnostics
//!   --json          Output diagnostics as JSON (one object per line)
//!   --help          Print this help
//! ```
//!
//! ## Exit codes
//!
//! | Code | Meaning                                           |
//! |------|---------------------------------------------------|
//! |  0   | No errors or warnings                             |
//! |  1   | One or more errors found                          |
//! |  2   | One or more warnings found (no errors)            |
//! |  3   | Usage / I/O error                                 |

use std::path::PathBuf;
use std::process;

use gfm_linter::{lint, Severity};

fn usage() -> ! {
    eprintln!(
        "Usage: gfm-lint [--no-warnings] [--json] <FILE>...\n\
         \n\
         Options:\n\
           --no-warnings   Suppress warning-level diagnostics\n\
           --json          Output diagnostics as JSON (one object per line)\n\
           --help          Print this help\n\
         \n\
         Exit codes:\n\
           0  No issues\n\
           1  Errors found\n\
           2  Warnings found (no errors)\n\
           3  Usage / I/O error"
    );
    process::exit(3);
}

fn main() {
    let mut files: Vec<PathBuf> = Vec::new();
    let mut no_warnings = false;
    let mut json = false;

    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--help" | "-h" => usage(),
            "--no-warnings" => no_warnings = true,
            "--json" => json = true,
            s if s.starts_with('-') => {
                eprintln!("Unknown option: {s}");
                usage();
            }
            _ => files.push(PathBuf::from(&arg)),
        }
    }

    if files.is_empty() {
        eprintln!("gfm-lint: no input files");
        usage();
    }

    let mut any_error = false;
    let mut any_warning = false;

    for path in &files {
        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("gfm-lint: {}: {e}", path.display());
                process::exit(3);
            }
        };

        let diags = lint(&source);

        for d in &diags {
            if no_warnings && d.severity == Severity::Warning {
                continue;
            }
            if json {
                println!(
                    "{{\"file\":{:?},\"line\":{},\"col\":{},\"severity\":{:?},\"rule\":{:?},\"message\":{:?}}}",
                    path.display().to_string(),
                    d.line,
                    d.col,
                    d.severity.to_string(),
                    d.rule,
                    d.message,
                );
            } else {
                println!("{}:{}", path.display(), d);
            }

            match d.severity {
                Severity::Error => any_error = true,
                Severity::Warning => any_warning = true,
            }
        }
    }

    if any_error {
        process::exit(1);
    } else if any_warning {
        process::exit(2);
    } else {
        process::exit(0);
    }
}

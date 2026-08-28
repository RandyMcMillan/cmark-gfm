//! `gfm-lint` – GitHub Flavored Markdown linter CLI.
//!
//! ## Usage
//!
//! ```text
//! gfm-lint [OPTIONS] <PATH>...
//!
//! Options:
//!   --no-warnings   Suppress warning-level diagnostics
//!   --json          Output diagnostics as JSON (one object per line)
//!   --recursive     Recurse into directories without a depth limit
//!   --depth N       Recurse into directories up to N levels deep
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
        "Usage: gfm-lint [--no-warnings] [--json] [--recursive] [--depth N] <PATH>...\n\
         \n\
         Options:\n\
           --no-warnings   Suppress warning-level diagnostics\n\
           --json          Output diagnostics as JSON (one object per line)\n\
           --recursive     Recurse into directories without a depth limit\n\
           --depth N       Recurse into directories up to N levels deep\n\
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
    let mut paths: Vec<PathBuf> = Vec::new();
    let mut no_warnings = false;
    let mut json = false;
    let mut max_depth: Option<usize> = Some(0);

    let mut args = std::env::args().skip(1).peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => usage(),
            "--no-warnings" => no_warnings = true,
            "--json" => json = true,
            "--recursive" => max_depth = None,
            "--depth" => {
                let value = args.next().unwrap_or_else(|| {
                    eprintln!("gfm-lint: --depth requires a value");
                    usage();
                });
                max_depth = Some(parse_depth(&value));
            }
            s if s.starts_with("--depth=") => {
                let value = &s["--depth=".len()..];
                if value.is_empty() {
                    eprintln!("gfm-lint: --depth requires a value");
                    usage();
                }
                max_depth = Some(parse_depth(value));
            }
            s if s.starts_with('-') => {
                eprintln!("Unknown option: {s}");
                usage();
            }
            _ => paths.push(PathBuf::from(arg)),
        }
    }

    if paths.is_empty() {
        eprintln!("gfm-lint: no input files");
        usage();
    }

    let files = match collect_inputs(&paths, max_depth) {
        Ok(files) => files,
        Err(message) => {
            eprintln!("gfm-lint: {message}");
            process::exit(3);
        }
    };

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

fn parse_depth(value: &str) -> usize {
    match value.parse::<usize>() {
        Ok(depth) => depth,
        Err(_) => {
            eprintln!("gfm-lint: invalid depth: {value}");
            usage();
        }
    }
}

fn collect_inputs(
    paths: &[PathBuf],
    max_depth: Option<usize>,
) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();

    for path in paths {
        collect_input(path, max_depth, &mut files)?;
    }

    Ok(files)
}

fn collect_input(
    path: &PathBuf,
    max_depth: Option<usize>,
    files: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|e| format!("{}: {e}", path.display()))?;

    if metadata.is_file() {
        files.push(path.clone());
        return Ok(());
    }

    if metadata.is_dir() {
        let mut entries = std::fs::read_dir(path)
            .map_err(|e| format!("{}: {e}", path.display()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("{}: {e}", path.display()))?;
        entries.sort_by_key(|entry| entry.path());

        for entry in entries {
            let entry_path = entry.path();
            let entry_metadata = std::fs::symlink_metadata(&entry_path)
                .map_err(|e| format!("{}: {e}", entry_path.display()))?;

            if entry_metadata.is_file() {
                files.push(entry_path);
            } else if entry_metadata.is_dir() {
                if max_depth != Some(0) {
                    let next_depth = max_depth.map(|depth| depth.saturating_sub(1));
                    collect_input(&entry_path, next_depth, files)?;
                }
            } else {
                return Err(format!("{}: not a regular file or directory", entry_path.display()));
            }
        }

        return Ok(());
    }

    Err(format!("{}: not a regular file or directory", path.display()))
}

#[cfg(test)]
mod tests {
    use super::collect_inputs;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir() -> PathBuf {
        let mut path = std::env::temp_dir();
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX_EPOCH")
            .as_nanos();
        path.push(format!("gfm-lint-test-{}-{stamp}", std::process::id()));
        path
    }

    #[test]
    fn rejects_directory_without_recursive() {
        let dir = unique_temp_dir();
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("README.md"), "# Heading\n").unwrap();

        let files = collect_inputs(&[dir.clone()], Some(0)).unwrap();
        assert_eq!(files, vec![dir.join("README.md")]);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn recurses_into_directories_when_enabled() {
        let dir = unique_temp_dir();
        let nested = dir.join("nested");

        fs::create_dir_all(&nested).unwrap();
        fs::write(dir.join("README.md"), "# Heading\n").unwrap();
        fs::write(nested.join("doc.md"), "## Subheading\n").unwrap();

        let files = collect_inputs(&[dir.clone()], None).unwrap();
        assert_eq!(files, vec![dir.join("README.md"), nested.join("doc.md")]);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn limits_recursion_depth() {
        let dir = unique_temp_dir();
        let nested = dir.join("nested");
        let deeper = nested.join("deeper");

        fs::create_dir_all(&deeper).unwrap();
        fs::write(dir.join("README.md"), "# Heading\n").unwrap();
        fs::write(nested.join("doc.md"), "## Subheading\n").unwrap();
        fs::write(deeper.join("deep.md"), "### Deep\n").unwrap();

        let files = collect_inputs(&[dir.clone()], Some(1)).unwrap();
        assert_eq!(files, vec![dir.join("README.md"), nested.join("doc.md")]);

        let files = collect_inputs(&[dir.clone()], Some(0)).unwrap();
        assert_eq!(files, vec![dir.join("README.md")]);

        fs::remove_dir_all(&dir).unwrap();
    }
}

//! Conformance against typescript-go (tsgo) with `isolatedDeclarations` on.
//!
//! Each directory under `tests/conformance/` is a fixture project containing a
//! `tsconfig.json` and an `expected.txt` holding tsgo's diagnostics in
//! `--pretty false` format (`file(line,col): error TSxxxx: message`).
//!
//! Refresh expectations with the tsgo oracle:
//!
//! ```bash
//! OXC_CONFORMANCE_BLESS=1 cargo test -p oxc_checker --test conformance
//! # tsgo binary path override: TSGO_BIN=/path/to/tsgo
//! ```
//!
//! The committed snapshot tracks the per-case *diff* against tsgo plus an
//! aggregate score, so progress shows up as a shrinking snapshot.

use std::fmt::Write;
use std::path::{Path, PathBuf};

use oxc_checker::check_project;

const DEFAULT_TSGO: &str = "/Users/boshen/github/typescript-go/built/local/tsgo";

fn line_col(source: &str, offset: usize) -> (usize, usize) {
    let prefix = &source[..offset.min(source.len())];
    let line = prefix.matches('\n').count() + 1;
    let col = prefix.rfind('\n').map_or(offset + 1, |nl| offset - nl);
    (line, col)
}

/// Render our diagnostics for one fixture in tsgo `--pretty false` format.
fn render_ours(root: &Path) -> Vec<String> {
    let result = check_project(root).unwrap();
    let mut out = Vec::new();
    for file in &result.files {
        let Ok(rel) = file.path.strip_prefix(root) else { continue };
        let rel =
            rel.components().map(|c| c.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/");
        for diagnostic in &file.diagnostics {
            let offset: usize =
                diagnostic.labels.first().map_or(0usize, |l| l.offset().try_into().unwrap());
            let (line, col) = line_col(&file.source_text, offset);
            // Codes come either from the structured field (`ts(2322)`) or, for
            // isolated-declarations errors, as a `TS9010: ` message prefix.
            let message = diagnostic.message.as_ref();
            let (code, message) = if let Some(rest) = message.strip_prefix("TS") {
                let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
                let rest = rest[digits.len()..].strip_prefix(": ").unwrap_or(rest);
                (format!("TS{digits}"), rest.to_string())
            } else {
                let number = diagnostic.code.number.as_deref().unwrap_or("0000");
                (format!("TS{number}"), message.to_string())
            };
            out.push(format!("{rel}({line},{col}): error {code}: {message}"));
        }
    }
    out.sort();
    out
}

/// Run the tsgo oracle on a fixture and return its diagnostic lines.
fn run_tsgo(root: &Path) -> Vec<String> {
    let tsgo = std::env::var("TSGO_BIN").unwrap_or_else(|_| DEFAULT_TSGO.to_string());
    let output = std::process::Command::new(&tsgo)
        .args(["--noEmit", "--pretty", "false", "-p", "."])
        .current_dir(root)
        .output()
        .unwrap_or_else(|e| panic!("failed to run tsgo at {tsgo}: {e}"));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines: Vec<String> = stdout
        .lines()
        // Keep primary diagnostic lines; drop elaboration continuations
        // (indented) and any non-diagnostic output.
        .filter(|l| !l.starts_with(' ') && l.contains("): error TS"))
        .map(str::to_string)
        .collect();
    lines.sort();
    lines
}

#[test]
fn conformance() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/conformance");
    let bless = std::env::var("OXC_CONFORMANCE_BLESS").is_ok();

    let mut cases: Vec<PathBuf> = std::fs::read_dir(&root)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            path.join("tsconfig.json").is_file().then_some(path)
        })
        .collect();
    cases.sort();
    assert!(!cases.is_empty(), "no conformance fixtures found");

    let mut report = String::new();
    let mut total_expected = 0usize;
    let mut total_matched = 0usize;
    let mut total_extra = 0usize;
    let mut perfect_cases = 0usize;

    for case in &cases {
        let name = case.file_name().unwrap().to_string_lossy();
        let expected_path = case.join("expected.txt");

        if bless {
            let lines = run_tsgo(case);
            std::fs::write(&expected_path, lines.join("\n") + "\n").unwrap();
        }

        let expected: Vec<String> = std::fs::read_to_string(&expected_path)
            .unwrap_or_else(|_| {
                panic!("{name}: missing expected.txt — bless with OXC_CONFORMANCE_BLESS=1")
            })
            .lines()
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect();
        let actual = render_ours(case);

        let matched: Vec<&String> = actual.iter().filter(|l| expected.contains(l)).collect();
        let missing: Vec<&String> = expected.iter().filter(|l| !actual.contains(l)).collect();
        let extra: Vec<&String> = actual.iter().filter(|l| !expected.contains(l)).collect();

        total_expected += expected.len();
        total_matched += matched.len();
        total_extra += extra.len();

        if missing.is_empty() && extra.is_empty() {
            perfect_cases += 1;
            continue; // perfect cases stay out of the snapshot
        }
        let _ = writeln!(
            report,
            "## {name} — matched {}/{}, extra {}",
            matched.len(),
            expected.len(),
            extra.len()
        );
        for line in &missing {
            let _ = writeln!(report, "- {line}");
        }
        for line in &extra {
            let _ = writeln!(report, "+ {line}");
        }
        let _ = writeln!(report);
    }

    let header = format!(
        "conformance: {total_matched}/{total_expected} expected diagnostics matched, \
         {total_extra} extra, {perfect_cases}/{} cases perfect\n\n",
        cases.len()
    );
    insta::assert_snapshot!("conformance", header + &report);
}

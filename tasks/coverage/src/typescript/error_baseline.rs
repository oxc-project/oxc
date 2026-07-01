//! TypeScript `.errors.txt` conformance for oxc's parser diagnostics.
//!
//! typescript-go's `internal/testutil/tsbaseline/error_baseline.go` *generates* the full
//! `.errors.txt` (a summary block plus per-file source with `~~~~` squiggles and
//! `!!! error TSxxxx:` messages) and compares the text. oxc's parser only produces syntax
//! errors, and its wording differs from TypeScript's, so reproducing that text would score
//! ~0%. Instead we compare ERROR POSITIONS extracted from the baseline summary headers
//! (`path(line,col): error TSxxxx:`) against oxc's diagnostic positions. oxc's parser
//! attaches real TypeScript codes (`ts_error` -> `with_error_code("TS", ...)`), so we also
//! require code agreement whenever oxc has a code.
//!
//! Results are reported through the shared [`CoverageResult`] machinery (same summary /
//! snapshot format as the other suites). A file passes when oxc's diagnostics agree with the
//! baseline: no false positives, and every baseline position whose TS code oxc can emit is
//! found. Type / binder errors (`TS2xxx`, `TS18xxx`) oxc can never produce are the
//! type-checker surface and are excluded from the pass criterion.

use lazy_regex::{Lazy, Regex, lazy_regex};
use oxc::{allocator::Allocator, parser::Parser};
use rayon::prelude::*;
use rustc_hash::FxHashSet;

use super::{meta::TestCaseContent, scanner};
use crate::{CoverageResult, TestResult, TypeScriptFile};

/// A 1-based error location (matching TypeScript's `.errors.txt` summary), plus the TS code.
#[derive(Clone, PartialEq, Eq, Hash)]
struct Pos {
    file: String,
    line: u32,
    col: u32,
    /// The `TSxxxx` number (without the `TS` prefix). `None` for oxc diagnostics with no code.
    code: Option<String>,
}

// Summary header line: `path(line,col): error TSxxxx: message`. `file` is non-greedy so it can
// contain `/`; the `(\d+,\d+)` comma disambiguates filenames that themselves contain parens.
// Global `error TSxxxx:` lines (no location), `!!!`, `~~~~`, and any `Found N errors` footer do
// not match, so they are excluded automatically.
static BASELINE_HEADER: Lazy<Regex> = lazy_regex!(
    r"(?m)^(?P<file>[^\r\n(]+?)\((?P<line>\d+),(?P<col>\d+)\): error TS(?P<code>\d+): "
);

fn parse_baseline_positions(texts: &[String]) -> Vec<Pos> {
    let mut set = FxHashSet::default();
    for text in texts {
        for cap in BASELINE_HEADER.captures_iter(text) {
            set.insert(Pos {
                file: cap["file"].to_string(),
                line: cap["line"].parse().unwrap_or(0),
                col: cap["col"].parse().unwrap_or(0),
                code: Some(cap["code"].to_string()),
            });
        }
    }
    set.into_iter().collect()
}

fn oxc_positions(unit_name: &str, content: &str, source_type: oxc::span::SourceType) -> Vec<Pos> {
    let allocator = Allocator::default();
    // No `'use strict'` injection — it would shift every line and break position matching.
    let ret = Parser::new(&allocator, content, source_type).parse();
    let line_starts = scanner::compute_line_starts(content);
    ret.diagnostics
        .iter()
        .filter_map(|d| {
            let offset = d.labels.first()?.offset();
            let (line, col) = scanner::line_and_character(content, &line_starts, offset);
            let code = if d.code.scope.as_deref() == Some("TS") {
                d.code.number.as_deref().map(str::to_string)
            } else {
                None
            };
            // Convert to TypeScript's 1-based line/col.
            Some(Pos { file: unit_name.to_string(), line: line + 1, col: col + 1, code })
        })
        .collect()
}

/// Does oxc position `o` match baseline position `b`? Single-unit tests ignore the filename
/// (the header path can be absolute / renamed); multi-unit tests match by unit name. Code must
/// agree when oxc has one.
fn pos_matches(o: &Pos, b: &Pos, ignore_file: bool) -> bool {
    let file_ok = ignore_file || o.file == b.file || b.file.ends_with(&o.file);
    file_ok && o.line == b.line && o.col == b.col && (o.code.is_none() || o.code == b.code)
}

/// A sorted, human-readable position list for the `--diff` view of a mismatch.
fn positions_text(positions: &[Pos]) -> String {
    let mut lines: Vec<String> = positions
        .iter()
        .map(|p| {
            format!("{}({}, {}): TS{}", p.file, p.line, p.col, p.code.as_deref().unwrap_or("-"))
        })
        .collect();
    lines.sort();
    lines.join("\n")
}

/// Conformance runner in the shared `CoverageResult` form (mirrors `run_semantic_typescript`),
/// so `AppArgs::run_tool` prints and snapshots it exactly like the other suites.
pub fn run_errors_typescript(files: &[TypeScriptFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .filter_map(|f| {
            let texts = TestCaseContent::get_error_files(&f.path, &f.settings);
            // Only score files that ship an `.errors.txt` baseline.
            if texts.is_empty() {
                return None;
            }
            let baseline = parse_baseline_positions(&texts);
            let ignore_file = f.units.len() <= 1;

            let result = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                f.units
                    .iter()
                    .flat_map(|u| oxc_positions(&u.name, &u.content, u.source_type))
                    .collect::<Vec<Pos>>()
            })) {
                Err(_) => TestResult::ParseError("Panicked while parsing".to_string(), true),
                Ok(oxc) => {
                    // The reachable subset for this file: baseline positions whose TS code oxc
                    // actually emitted here (i.e. the syntax errors oxc is capable of reporting).
                    let oxc_codes: FxHashSet<&str> =
                        oxc.iter().filter_map(|p| p.code.as_deref()).collect();
                    let no_false_positives =
                        oxc.iter().all(|o| baseline.iter().any(|b| pos_matches(o, b, ignore_file)));
                    let found_reachable = baseline
                        .iter()
                        .filter(|b| b.code.as_deref().is_some_and(|c| oxc_codes.contains(c)))
                        .all(|b| oxc.iter().any(|o| pos_matches(o, b, ignore_file)));
                    if no_false_positives && found_reachable {
                        TestResult::Passed
                    } else {
                        TestResult::Mismatch(
                            "Error Mismatch",
                            positions_text(&oxc),
                            positions_text(&baseline),
                        )
                    }
                }
            };
            Some(CoverageResult { path: f.path.clone(), should_fail: false, result })
        })
        .collect()
}

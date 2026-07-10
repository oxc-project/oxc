use std::{
    fmt::Write,
    panic::{AssertUnwindSafe, catch_unwind},
};

use oxc::{
    allocator::Allocator,
    diagnostics::OxcDiagnostic,
    parser::{Parser, config::TokensParserConfig},
    span::SourceType,
};
use rayon::prelude::*;

use crate::{
    BabelFile, CoverageResult, MiscFile, Test262File, TestResult, TypeScriptFile, test262::TestFlag,
};

fn lexer_stream(
    code: &str,
    source_type: SourceType,
) -> (Vec<(u32, u32)>, Vec<oxc_lexer::Diagnostic>) {
    let n = code.len();
    // `lex_utf8` requires >= n + 64 bytes of trailing padding.
    let mut buf = Vec::with_capacity(n + 64);
    buf.extend_from_slice(code.as_bytes());
    buf.resize(n + 64, 0);

    let mut options = oxc_lexer::default_options();
    options.source_type_module = source_type.is_module();
    options.jsx = source_type.is_jsx();
    options.ts = source_type.is_typescript();
    // Keep the stream fully contiguous so `starts[i + 1]` is token `i`'s true end.
    options.emit_whitespace = true;
    options.emit_comments = true;

    let (result, arena) = oxc_lexer::lex_utf8(&buf, n as u32, options);
    let kinds = result.tok_kinds(&arena);
    let starts = result.tok_starts(&arena);

    let mut spans = Vec::with_capacity(kinds.len());
    for (i, &kind) in kinds.iter().enumerate() {
        if kind == oxc_lexer::token_kind::EOF || oxc_lexer::is_trivia(kind) {
            continue;
        }
        let start = oxc_lexer::token::offset(starts[i]);
        let end = oxc_lexer::token::offset(starts[i + 1]);
        spans.push((start, end));
    }
    let mut diags = result.diagnostics().to_vec();
    diags.sort_by_key(|d| d.off);
    (spans, diags)
}

fn oracle_stream(
    code: &str,
    source_type: SourceType,
) -> Option<(Vec<(u32, u32)>, Vec<OxcDiagnostic>)> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, code, source_type).with_config(TokensParserConfig).parse();
    if ret.panicked {
        return None;
    }
    let spans = ret.tokens.iter().map(|t| (t.start(), t.end())).collect();
    let errors = ret.diagnostics.errors().cloned().collect();
    Some((spans, errors))
}

fn diag_key(d: &OxcDiagnostic) -> (String, Vec<(u32, u32)>) {
    let labels = d
        .labels
        .as_ref()
        .iter()
        .map(|l| (l.offset(), l.offset().saturating_add(l.len())))
        .collect();
    (d.message.to_string(), labels)
}

fn label_start(d: &OxcDiagnostic) -> u32 {
    d.labels.as_ref().first().map_or(0, |l| l.offset())
}

fn prefix_matches(ours: &[(u32, u32)], oracle: &[(u32, u32)], err_start: u32) -> bool {
    for (a, b) in ours.iter().zip(oracle.iter()) {
        if a.1 > err_start || b.1 > err_start {
            break;
        }
        if a != b {
            return false;
        }
    }
    true
}

fn fmt_spans(spans: &[(u32, u32)], code: &str) -> String {
    let mut out = String::new();
    for &(start, end) in spans {
        let slice = code.get(start as usize..end as usize).unwrap_or("<out-of-bounds>");
        let _ = writeln!(out, "{start}..{end} {slice:?}");
    }
    out
}

/// One-line rendering of a diagnostic for mismatch output.
fn fmt_diag(d: &OxcDiagnostic) -> String {
    let (message, labels) = diag_key(d);
    let spans = labels.iter().map(|(s, e)| format!("{s}..{e}")).collect::<Vec<_>>().join(", ");
    format!("{message:?} @ [{spans}]")
}

/// Token-stream mismatch, with each side's first diagnostic appended so a
/// failed error-case fallback shows *why* diagnostic parity did not hold.
fn token_mismatch(
    code: &str,
    ours: &[(u32, u32)],
    oracle: &[(u32, u32)],
    ours_first: Option<&OxcDiagnostic>,
    parser_first: Option<&OxcDiagnostic>,
) -> TestResult {
    let mut actual = fmt_spans(ours, code);
    let mut expected = fmt_spans(oracle, code);
    let none = || "(none)".to_string();
    let _ = writeln!(actual, "first diagnostic: {}", ours_first.map_or_else(none, fmt_diag));
    let _ = writeln!(expected, "first diagnostic: {}", parser_first.map_or_else(none, fmt_diag));
    TestResult::Mismatch("Tokens", actual, expected)
}

fn lex_diff(code: &str, source_type: SourceType) -> TestResult {
    let (oracle, parser_errors) =
        match catch_unwind(AssertUnwindSafe(|| oracle_stream(code, source_type))) {
            Ok(Some(stream)) => stream,
            Ok(None) | Err(_) => return TestResult::Passed,
        };

    let (ours, our_diags) = match catch_unwind(AssertUnwindSafe(|| lexer_stream(code, source_type)))
    {
        Ok(stream) => stream,
        Err(_) => return TestResult::ParseError("oxc_lexer panicked".to_string(), true),
    };

    let ours_first = our_diags.first().map(|d| oxc_lexer::diagnostics::to_oxc_diagnostic(d, code));
    let parser_first =
        parser_errors.iter().enumerate().min_by_key(|(i, e)| (label_start(e), *i)).map(|(_, e)| e);

    if ours == oracle {
        return match (&parser_first, &ours_first) {
            (None, Some(diag)) => TestResult::Mismatch(
                "Diagnostics",
                format!("first diagnostic: {}\n", fmt_diag(diag)),
                "(parser reported no errors)\n".to_string(),
            ),
            _ => TestResult::Passed,
        };
    }
    if let (Some(parser_diag), Some(our_diag)) = (parser_first, &ours_first) {
        if diag_key(our_diag) == diag_key(parser_diag)
            && prefix_matches(&ours, &oracle, label_start(parser_diag))
        {
            return TestResult::Passed;
        }
    }

    token_mismatch(code, &ours, &oracle, ours_first.as_ref(), parser_first)
}

pub fn run_lexer_test262(files: &[Test262File]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .map(|f| {
            let mut source_type = SourceType::cjs().with_script(true);
            if f.meta.flags.contains(&TestFlag::Module) {
                source_type = source_type.with_module(true);
            }
            let result = lex_diff(&f.code, source_type);
            CoverageResult { path: f.path.clone(), should_fail: false, result }
        })
        .collect()
}

pub fn run_lexer_babel(files: &[BabelFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .map(|f| {
            let result = lex_diff(&f.code, f.source_type);
            CoverageResult { path: f.path.clone(), should_fail: false, result }
        })
        .collect()
}

pub fn run_lexer_typescript(files: &[TypeScriptFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .map(|f| {
            let source_type = SourceType::from_path(&f.path).unwrap_or_default();
            let result = lex_diff(&f.code, source_type);
            CoverageResult { path: f.path.clone(), should_fail: false, result }
        })
        .collect()
}

pub fn run_lexer_misc(files: &[MiscFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .map(|f| {
            let result = lex_diff(&f.code, f.source_type);
            CoverageResult { path: f.path.clone(), should_fail: false, result }
        })
        .collect()
}

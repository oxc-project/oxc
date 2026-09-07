//! Differential test of the incubating `oxc_lexer` crate against the parser's own lexer.
//!
//! Each fixture is lexed twice. The parser is the oracle: its token stream is what `oxc_lexer`
//! must reproduce. Trivia and `Eof` are dropped from `oxc_lexer`'s output first, because the
//! parser never surfaces them as tokens.
//!
//! Sources that both sides reject identically still count as a pass - see [`lex_diff`].

use std::{
    fmt::Write,
    iter,
    panic::{AssertUnwindSafe, catch_unwind},
};

use oxc::{
    allocator::Allocator,
    diagnostics::{LabeledSpan, OxcDiagnostic},
    parser::{Parser, Token, config::TokensParserConfig},
    span::{SourceType, Span},
};
use oxc_lexer::{LexOptions, PAD, TokenKind, diagnostics::to_oxc_diagnostic};
use rayon::prelude::*;

use crate::{
    BabelFile, CoverageResult, MiscFile, Test262File, TestResult, TypeScriptFile, test262::TestFlag,
};

pub fn run_lexer_test262(files: &[Test262File]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .map(|file| {
            let mut source_type = SourceType::script();
            if file.meta.flags.contains(&TestFlag::Module) {
                source_type = source_type.with_module(true);
            }
            let result = lex_diff(&file.code, source_type);
            CoverageResult { path: file.path.clone(), should_fail: false, result }
        })
        .collect()
}

pub fn run_lexer_babel(files: &[BabelFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .map(|file| {
            let result = lex_diff(&file.code, file.source_type);
            CoverageResult { path: file.path.clone(), should_fail: false, result }
        })
        .collect()
}

pub fn run_lexer_typescript(files: &[TypeScriptFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .map(|file| {
            let source_type = SourceType::from_path(&file.path).unwrap_or_default();
            let result = lex_diff(&file.code, source_type);
            CoverageResult { path: file.path.clone(), should_fail: false, result }
        })
        .collect()
}

pub fn run_lexer_misc(files: &[MiscFile]) -> Vec<CoverageResult> {
    files
        .par_iter()
        .map(|file| {
            let result = lex_diff(&file.code, file.source_type);
            CoverageResult { path: file.path.clone(), should_fail: false, result }
        })
        .collect()
}

fn lex_diff(code: &str, source_type: SourceType) -> TestResult {
    // If parser cannot parse `code`, there is nothing to compare against, so skip fixture
    let Ok(Some((old_spans, old_error))) =
        catch_unwind(AssertUnwindSafe(|| parser_stream(code, source_type)))
    else {
        return TestResult::Passed;
    };

    let Ok((new_spans, new_error)) =
        catch_unwind(AssertUnwindSafe(|| lexer_stream(code, source_type)))
    else {
        return TestResult::ParseError("`oxc_lexer` panicked".to_string(), true);
    };

    if new_spans == old_spans {
        return match (&old_error, &new_error) {
            (None, Some(new_error)) => TestResult::Mismatch(
                "Diagnostics",
                format!("first error: {}\n", fmt_diagnostic(new_error)),
                "(parser reported no errors)\n".to_string(),
            ),
            _ => TestResult::Passed,
        };
    }

    if let (Some(old_error), Some(new_error)) = (&old_error, &new_error)
        && diagnostic_key(new_error) == diagnostic_key(old_error)
        && prefix_matches(&new_spans, &old_spans, label_start(old_error))
    {
        return TestResult::Passed;
    }

    token_mismatch(code, &new_spans, &old_spans, new_error.as_ref(), old_error.as_ref())
}

/// Get token stream from `oxc_lexer`, plus the first error (if any).
fn lexer_stream(code: &str, source_type: SourceType) -> (Vec<Span>, Option<OxcDiagnostic>) {
    let len = code.len();
    let len_u32 = u32::try_from(len).expect("source longer than `u32::MAX`");

    // `lex_utf8` requires at least `PAD` bytes of zeroed padding past the end of the source
    let mut buf = Vec::with_capacity(len + PAD);
    buf.extend_from_slice(code.as_bytes());
    buf.resize(len + PAD, 0);

    let options = LexOptions {
        source_type_module: source_type.is_module(),
        jsx: source_type.is_jsx(),
        ts: source_type.is_typescript(),
        ..Default::default()
    };

    let (result, arena) = oxc_lexer::lex_utf8(&buf, len_u32, options);
    let kinds = result.tok_kinds(&arena);
    let token_spans = result.tok_spans(&arena);
    assert_eq!(kinds.len(), token_spans.len());

    let spans = iter::zip(kinds, token_spans)
        .filter_map(|(&kind, &span)| match kind {
            TokenKind::Eof => None,
            _ if kind.is_trivia() => None,
            _ => Some(span),
        })
        .collect();

    let error = result
        .diagnostics()
        .iter()
        .min_by_key(|error| error.off)
        .map(|error| to_oxc_diagnostic(error, code));

    (spans, error)
}

/// Get token stream from `oxc_parser`, plus the first error (if any).
fn parser_stream(
    code: &str,
    source_type: SourceType,
) -> Option<(Vec<Span>, Option<OxcDiagnostic>)> {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, code, source_type).with_config(TokensParserConfig).parse();
    if ret.panicked {
        return None;
    }

    let spans = ret.tokens.iter().map(Token::span).collect();
    let error = ret.diagnostics.into_iter().min_by_key(label_start);

    Some((spans, error))
}

fn diagnostic_key(error: &OxcDiagnostic) -> (String, Vec<Span>) {
    let labels = error.labels.iter().map(LabeledSpan::span).collect();
    (error.message.to_string(), labels)
}

fn label_start(error: &OxcDiagnostic) -> u32 {
    error.labels.as_ref().first().map_or(0, LabeledSpan::offset)
}

fn prefix_matches(new_spans: &[Span], old_spans: &[Span], err_start: u32) -> bool {
    for (new_span, old_span) in iter::zip(new_spans, old_spans) {
        if new_span.end > err_start || old_span.end > err_start {
            break;
        }
        if new_span != old_span {
            return false;
        }
    }
    true
}

fn fmt_spans(spans: &[Span], code: &str) -> String {
    let mut out = String::new();
    for &span in spans {
        let slice = code.get(span.start as usize..span.end as usize).unwrap_or("<out-of-bounds>");
        writeln!(out, "{}..{} {:?}", span.start, span.end, slice).unwrap();
    }
    out
}

/// One-line rendering of a diagnostic for mismatch output.
fn fmt_diagnostic(error: &OxcDiagnostic) -> String {
    let (message, labels) = diagnostic_key(error);
    let spans = labels
        .iter()
        .map(|&span| format!("{}..{}", span.start, span.end))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{message:?} @ [{spans}]")
}

/// Token-stream mismatch, with each side's first diagnostic appended so a
/// failed error-case fallback shows *why* diagnostic parity did not hold.
fn token_mismatch(
    code: &str,
    new_spans: &[Span],
    old_spans: &[Span],
    new_error: Option<&OxcDiagnostic>,
    old_error: Option<&OxcDiagnostic>,
) -> TestResult {
    let mut actual = fmt_spans(new_spans, code);
    let mut expected = fmt_spans(old_spans, code);
    let none = || "(none)".to_string();
    writeln!(actual, "first error: {}", new_error.map_or_else(none, fmt_diagnostic)).unwrap();
    writeln!(expected, "first error: {}", old_error.map_or_else(none, fmt_diagnostic)).unwrap();
    TestResult::Mismatch("Tokens", actual, expected)
}

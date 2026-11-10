//! Differential test of the incubating `oxc_lexer` crate against the parser's own lexer.
//!
//! Each fixture is lexed by the parser, and by `oxc_lexer` with each CPU feature set
//! this CPU supports, one straight after the other (e.g. AVX2 then generic on x86_64).
//! `oxc_lexer`'s output must be identical with every CPU feature set.
//!
//! The parser is the oracle. Its token stream and comments are what `oxc_lexer` must reproduce.
//! Trivia and `Eof` are dropped from `oxc_lexer`'s tokens first, because the parser
//! never surfaces them as tokens.
//!
//! Sources that both sides reject identically still count as a pass - see [`lex_diff`].

use std::{
    fmt::{Debug, Write},
    iter,
    panic::{AssertUnwindSafe, catch_unwind},
};

use oxc::{
    allocator::Allocator,
    ast::{Comment, CommentContent, CommentPosition},
    diagnostics::{LabeledSpan, OxcDiagnostic},
    parser::{ParseOptions, Parser, Token, config::TokensParserConfig},
    span::{SourceType, Span},
};
use oxc_lexer::{
    LexOptions, PAD, TokenKind, Tokens, diagnostics::to_oxc_diagnostic, testing::lex_with_cpu,
};
use oxc_simd::DynCpu;
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
    const SKIP: &[&str] = &[
        // Both contain `\ declare`. The parser reads `\`, the space and `declare` as one token,
        // and reports an escaped keyword. That's probably a parser bug, so don't try to match it.
        "slashBeforeVariableDeclaration1.ts",
        "parserSkippedTokens19.ts",
        // Unclear whether `\u` escapes should be allowed in JSX identifiers.
        // JSX spec seems to suggest yes, but Babel and TypeScript say no,
        // and have tests specifically checking that it produces errors.
        // `oxc_parser` has only partial support, and it looks accidental. In `oxc_parser`:
        // * Any escape after a `-` ends the name.
        //   `<a data-\u0076ideo="x" />` is parsed as 2 attributes (`data-` and `\u0076ideo`).
        //   `<a-\u0063></a-\u0063>` is rejected.
        // * Escapes aren't decoded. `JSXIdentifier.name` is the raw source text
        //   (`\u0061`, not `a`), unlike `Identifier.name`.
        //   It's also marked `json_safe`, so ESTree JSON for `<\u{61} />` is invalid.
        // We need to make `oxc_parser` do the right thing before trying to replicate it.
        "unicodeEscapesInJsxtags.tsx",
    ];

    files
        .par_iter()
        .filter(|file| !SKIP.iter().any(|skip| file.path.ends_with(skip)))
        .map(|file| {
            let source_type = SourceType::from_path(&file.path).unwrap_or_default();
            let result = lex_diff(&file.code, source_type);
            CoverageResult { path: file.path.clone(), should_fail: false, result }
        })
        .collect()
}

pub fn run_lexer_misc(files: &[MiscFile]) -> Vec<CoverageResult> {
    // Both need unambiguous mode, which `oxc_lexer` doesn't support. The parser switches
    // to module mode when it finds `export`, and that turns `await /x/` into a regex.
    const SKIP: &[&str] = &["babel-16776-m.js", "unambiguous-await-regex.js"];

    files
        .par_iter()
        .filter(|file| !SKIP.iter().any(|skip| file.path.ends_with(skip)))
        .map(|file| {
            let result = lex_diff(&file.code, file.source_type);
            CoverageResult { path: file.path.clone(), should_fail: false, result }
        })
        .collect()
}

fn lex_diff(code: &str, source_type: SourceType) -> TestResult {
    // If parser panics, there is nothing to compare against, so skip fixture
    let Ok((old_spans, mut old_comments, mut old_ws, old_error, old_last_error_start)) =
        catch_unwind(AssertUnwindSafe(|| parser_stream(code, source_type)))
    else {
        return TestResult::Passed;
    };

    let tokens = match lex_with_each_cpu(code, source_type) {
        Ok(output) => output,
        Err(failure) => return failure,
    };
    let (new_spans, mut new_comments, mut new_ws, new_error) = lexer_stream(code, &tokens);

    // Only compare comments which start before the parser's last error.
    //
    // This isn't ideal, but the parser can stop lexing at an error without reporting a fatal error.
    // A lexer error at a statement boundary, for example, ends parsing as if it was EOF,
    // and `fatal_error` is `false`. The parser's output contains no comments after that point,
    // and nothing in `ParserReturn` says that lexing stopped early.
    // An unterminated block comment is a similar case. The parser doesn't record it at all.
    //
    // The cost is that comments after the last error are never checked,
    // even when all errors are recoverable and the parser did lex to the end.
    if let Some(end) = old_last_error_start {
        old_comments.retain(|comment| comment.span.start < end);
        new_comments.retain(|comment| comment.span.start < end);
        old_ws.retain(|span| span.start < end);
        new_ws.retain(|span| span.start < end);
    }
    let trivia_diff = || match comment_diff(code, &new_comments, &old_comments) {
        TestResult::Passed if new_ws != old_ws => TestResult::Mismatch(
            "IrregularWhitespaces",
            fmt_spans(&new_ws, code),
            fmt_spans(&old_ws, code),
        ),
        result => result,
    };

    // A fatal error discards the parser's tokens, so only comments can be compared
    let Some(old_spans) = old_spans else {
        return trivia_diff();
    };

    if new_spans == old_spans {
        return match (&old_error, &new_error) {
            (None, Some(new_error)) => TestResult::Mismatch(
                "Diagnostics",
                format!("first error: {}\n", fmt_diagnostic(new_error)),
                "(parser reported no errors)\n".to_string(),
            ),
            _ => trivia_diff(),
        };
    }

    if let (Some(old_error), Some(new_error)) = (&old_error, &new_error)
        && diagnostic_key(new_error) == diagnostic_key(old_error)
        && prefix_matches(&new_spans, &old_spans, label_start(old_error))
    {
        return trivia_diff();
    }

    token_mismatch(code, &new_spans, &old_spans, new_error.as_ref(), old_error.as_ref())
}

/// Get token stream and comments from `oxc_lexer`'s output, plus the first error (if any).
fn lexer_stream(
    code: &str,
    tokens: &Tokens,
) -> (Vec<Span>, Vec<Comment>, Vec<Span>, Option<OxcDiagnostic>) {
    let kinds = tokens.kinds();
    let token_spans = tokens.spans();
    assert_eq!(kinds.len(), token_spans.len());

    let spans = iter::zip(kinds, token_spans)
        .filter_map(|(&kind, &span)| match kind {
            TokenKind::Eof => None,
            _ if kind.is_trivia() => None,
            _ => Some(span),
        })
        .collect();

    let comments = tokens.comments().to_vec();
    let irregular_whitespaces = tokens.irregular_whitespaces().to_vec();

    let error = tokens
        .diagnostics()
        .iter()
        .min_by_key(|error| error.off)
        .map(|error| to_oxc_diagnostic(error, code));

    (spans, comments, irregular_whitespaces, error)
}

/// Lex `code` with `oxc_lexer`, with each CPU feature set this CPU supports,
/// one straight after the other.
///
/// Returns the output from the first CPU feature set.
/// Returns `Err` if lexing panicked, or the outputs from each CPU feature set differ.
fn lex_with_each_cpu(code: &str, source_type: SourceType) -> Result<Tokens, TestResult> {
    let len = code.len();
    let len_u32 = u32::try_from(len).expect("source longer than `u32::MAX`");

    // lex_with_cpu requires at least PAD bytes of zeroed padding past the end of the source
    let mut buf = Vec::with_capacity(len + PAD);
    buf.extend_from_slice(code.as_bytes());
    buf.resize(len + PAD, 0);

    let options = LexOptions {
        source_type_module: source_type.is_module(),
        jsx: source_type.is_jsx(),
        ts: source_type.is_typescript(),
        ..Default::default()
    };

    let mut outputs = vec![];
    let mut panicked = vec![];
    DynCpu::for_each_supported(|cpu| {
        match catch_unwind(AssertUnwindSafe(|| lex_with_cpu(&buf, len_u32, options, cpu))) {
            Ok(output) => outputs.push((cpu, output)),
            Err(_) => panicked.push(cpu.name()),
        }
    });

    if !panicked.is_empty() {
        let message = if outputs.is_empty() {
            "`oxc_lexer` panicked".to_string()
        } else {
            let lexed = outputs.iter().map(|(cpu, _)| cpu.name()).collect::<Vec<_>>();
            format!(
                "`oxc_lexer` panicked with {}, but not with {}",
                panicked.join(", "),
                lexed.join(", ")
            )
        };
        return Err(TestResult::ParseError(message, true));
    }

    let mut outputs = outputs.into_iter();
    let (first_cpu, first) =
        outputs.next().expect("`for_each_supported` calls its closure at least once");
    for (cpu, output) in outputs {
        if let Some((expected, actual)) = first_difference(&first, &output) {
            return Err(TestResult::Mismatch(
                "CPU feature sets",
                format!("{}: {actual}", cpu.name()),
                format!("{}: {expected}", first_cpu.name()),
            ));
        }
    }

    Ok(first)
}

/// Find the first difference between 2 outputs from `oxc_lexer`.
///
/// Returns a description of the first entry which differs, from each output.
fn first_difference(a: &Tokens, b: &Tokens) -> Option<(String, String)> {
    // Compare bits, so `NaN`s compare equal, and `0.0` and `-0.0` don't
    let numbers = |tokens: &Tokens| {
        tokens.numbers().iter().map(|number| number.to_bits()).collect::<Vec<_>>()
    };

    lane_difference("tokens", a.kinds(), b.kinds())
        .or_else(|| lane_difference("spans", a.spans(), b.spans()))
        .or_else(|| lane_difference("diagnostics", a.diagnostics(), b.diagnostics()))
        .or_else(|| lane_difference("numbers", &numbers(a), &numbers(b)))
        .or_else(|| lane_difference("strings", a.strings(), b.strings()))
        .or_else(|| lane_difference("templates", a.templates(), b.templates()))
        .or_else(|| lane_difference("atoms", a.atoms(), b.atoms()))
        .or_else(|| lane_difference("cooked bytes", a.cooked_bytes(), b.cooked_bytes()))
        .or_else(|| lane_difference("regex flags", a.regex_flags(), b.regex_flags()))
        .or_else(|| lane_difference("comments", a.comments(), b.comments()))
        .or_else(|| {
            let (a_ws, b_ws) = (a.irregular_whitespaces(), b.irregular_whitespaces());
            lane_difference("irregular whitespaces", a_ws, b_ws)
        })
}

/// If lanes `a` and `b` differ, describe the first entry which differs, from each lane.
fn lane_difference<T: PartialEq + Debug>(name: &str, a: &[T], b: &[T]) -> Option<(String, String)> {
    if a == b {
        return None;
    }

    let index = iter::zip(a, b).position(|(a, b)| a != b).unwrap_or(a.len().min(b.len()));
    let describe =
        |lane: &[T]| format!("{name}[{index}] = {:?} ({} entries)\n", lane.get(index), lane.len());
    Some((describe(a), describe(b)))
}

/// Token spans, comments, irregular whitespaces, first error and start of the last error.
type ParserStream =
    (Option<Vec<Span>>, Vec<Comment>, Vec<Span>, Option<OxcDiagnostic>, Option<u32>);

/// Get token stream and comments from `oxc_parser`,
/// plus the first error and the start of the last error (if any).
/// Token stream is `None` if parser hit a fatal error.
fn parser_stream(code: &str, source_type: SourceType) -> ParserStream {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, code, source_type)
        .with_options(ParseOptions { enable_ident_hashes: false, ..ParseOptions::default() })
        .with_config(TokensParserConfig)
        .parse();

    let spans =
        if ret.fatal_error { None } else { Some(ret.tokens.iter().map(Token::span).collect()) };
    let comments = ret.program.comments.iter().map(conform_comment).collect();
    let irregular_whitespaces = ret.irregular_whitespaces.to_vec();
    let last_error_start = ret.diagnostics.iter().map(label_start).max();
    let error = ret.diagnostics.into_iter().min_by_key(label_start);

    (spans, comments, irregular_whitespaces, error, last_error_start)
}

/// Copy of a comment from the parser, with the fields which only the parser sets reset to the values
/// `Comment::new` gives them. `oxc_lexer` creates its comments with `Comment::new`.
/// All other fields are compared.
#[expect(clippy::default_trait_access)]
fn conform_comment(comment: &Comment) -> Comment {
    let mut comment = comment.clone();
    comment.attached_to = Default::default();
    comment.node_id = Default::default();
    comment.placement = Default::default();
    // Not the default - `Comment::new` sets `Trailing`
    comment.position = CommentPosition::Trailing;
    // The parser marks `#__PURE__` and `#__NO_SIDE_EFFECTS__` comments "not applied",
    // until it finds the call or function they annotate
    match comment.content {
        CommentContent::PureNotApplied => comment.content = CommentContent::Pure,
        CommentContent::NoSideEffectsNotApplied => comment.content = CommentContent::NoSideEffects,
        _ => {}
    }
    comment
}

fn comment_diff(code: &str, new_comments: &[Comment], old_comments: &[Comment]) -> TestResult {
    if new_comments == old_comments {
        return TestResult::Passed;
    }
    TestResult::Mismatch(
        "Comments",
        fmt_comments(new_comments, code),
        fmt_comments(old_comments, code),
    )
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

fn fmt_comments(comments: &[Comment], code: &str) -> String {
    const MAX_TEXT_LEN: usize = 60;

    let mut out = String::new();
    for comment in comments {
        let Span { start, end, .. } = comment.span;
        let mut text = code.get(start as usize..end as usize).unwrap_or("<out-of-bounds>");
        if text.len() > MAX_TEXT_LEN {
            let mut len = MAX_TEXT_LEN;
            while !text.is_char_boundary(len) {
                len -= 1;
            }
            text = &text[..len];
        }
        writeln!(
            out,
            "{start}..{end} {:?} {:?} newline_before={} newline_after={} {text:?}",
            comment.kind,
            comment.content,
            comment.preceded_by_newline(),
            comment.followed_by_newline(),
        )
        .unwrap();
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

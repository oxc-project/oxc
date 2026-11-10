#![cfg(target_endian = "little")]
#![expect(clippy::cast_possible_truncation, reason = "test helpers: lengths fit u32")]

use oxc_lexer::{DiagCode, LexOptions, PAD, TokenKind, testing::lex_with_cpu};
use oxc_simd::{DynCpu, test_cpus};

fn kinds_of(code: &str, module: bool, cpu: DynCpu) -> Vec<TokenKind> {
    kinds_with(code, LexOptions { source_type_module: module, ..Default::default() }, cpu)
}

fn kinds_with(code: &str, opts: LexOptions, cpu: DynCpu) -> Vec<TokenKind> {
    let mut buf = code.as_bytes().to_vec();
    let n = buf.len();
    buf.resize(n + PAD, 0);
    let res = lex_with_cpu(&buf, n as u32, opts, cpu);
    let count = res.token_count();
    res.kinds()[..count].iter().copied().filter(|kk| !kk.is_trivia()).collect()
}

fn diag_codes(code: &str, module: bool, cpu: DynCpu) -> Vec<DiagCode> {
    diag_codes_with(code, LexOptions { source_type_module: module, ..Default::default() }, cpu)
}

fn diag_codes_with(code: &str, opts: LexOptions, cpu: DynCpu) -> Vec<DiagCode> {
    let mut buf = code.as_bytes().to_vec();
    let n = buf.len() as u32;
    buf.resize(buf.len() + PAD, 0);
    let res = lex_with_cpu(&buf, n, opts, cpu);
    res.diagnostics().iter().map(|d| d.code).collect()
}

#[test_cpus]
fn script_html_open_comment_anywhere(cpu: DynCpu) {
    let ks = kinds_of("x <!-- y\nz;", false, cpu);
    assert!(
        !ks.contains(&TokenKind::Lt) && !ks.contains(&TokenKind::Bang),
        "script comment: {ks:?}"
    );
    assert_eq!(diag_codes("x <!-- y\nz;", false, cpu), []);
}

#[test_cpus]
fn html_comment_content(cpu: DynCpu) {
    let source = "<!--a\n-->\n<!--";
    let mut buf = source.as_bytes().to_vec();
    buf.resize(buf.len() + PAD, 0);
    let options = LexOptions { source_type_module: false, ..Default::default() };
    let result = lex_with_cpu(&buf, source.len() as u32, options, cpu);
    assert_eq!(result.diagnostics(), []);
    let comments = result.comments();
    assert_eq!(comments.len(), 3);
    for (comment, content) in comments.iter().zip(["a", "", ""]) {
        assert!(comment.is_line());
        assert_eq!(comment.content_span().source_text(source), content);
    }
}

#[test_cpus]
fn module_html_open_comment_mid_expression_is_lt(cpu: DynCpu) {
    let ks = kinds_of("x <!-- y;", true, cpu);
    assert!(ks.contains(&TokenKind::Lt), "expected `<` operator: {ks:?}");
    assert!(ks.contains(&TokenKind::Bang), "expected `!` operator: {ks:?}");
    assert!(ks.contains(&TokenKind::MinusMinus), "expected `--` operator: {ks:?}");
    assert_eq!(diag_codes("x <!-- y;", true, cpu), []);
}

#[test_cpus]
fn module_html_open_comment_at_line_start_diagnosed(cpu: DynCpu) {
    let src = "<!-- c\nx;";
    let ks = kinds_of(src, true, cpu);
    assert!(!ks.contains(&TokenKind::Lt), "line-start form stays a comment: {ks:?}");
    assert_eq!(diag_codes(src, true, cpu), [DiagCode::HtmlCommentInModule]);
    let src = "x;\n<!-- c\ny;";
    assert_eq!(diag_codes(src, true, cpu), [DiagCode::HtmlCommentInModule]);
    assert!(diag_codes(src, false, cpu).is_empty(), "script form is silent");
}

#[test_cpus]
fn module_html_close_comment_is_operators(cpu: DynCpu) {
    let src = "a;\n--> b;";
    let script = kinds_of(src, false, cpu);
    assert!(!script.contains(&TokenKind::MinusMinus), "script comment: {script:?}");
    let module = kinds_of(src, true, cpu);
    assert!(module.contains(&TokenKind::MinusMinus), "expected `--`: {module:?}");
    assert!(module.contains(&TokenKind::Gt), "expected `>`: {module:?}");
    assert_eq!(diag_codes(src, true, cpu), []);
}

#[test_cpus]
fn mid_line_close_comment_unchanged(cpu: DynCpu) {
    for module in [false, true] {
        let ks = kinds_of("a --> b;", module, cpu);
        assert!(ks.contains(&TokenKind::MinusMinus), "module={module}: {ks:?}");
        assert!(ks.contains(&TokenKind::Gt), "module={module}: {ks:?}");
    }
}

fn kinds_of_jsx(code: &str, module: bool, cpu: DynCpu) -> Vec<TokenKind> {
    kinds_with(
        code,
        LexOptions { source_type_module: module, jsx: true, ..Default::default() },
        cpu,
    )
}

#[test_cpus]
fn jsx_script_html_comments(cpu: DynCpu) {
    // Annex B applies to a script whatever its JSX setting.
    for src in ["x <!-- y\nz;", "<!-- c\nx;", "x = <a>{y}</a> <!-- c\nz;"] {
        let ks = kinds_of_jsx(src, false, cpu);
        assert!(!ks.contains(&TokenKind::Lt) && !ks.contains(&TokenKind::Bang), "{src:?}: {ks:?}");
    }
    // `-->` closes a comment a `<!--` opened: in a JSX script it is recognised after one.
    for src in ["<!--\na;\n--> b;", "<!--\nx = 1;\n  --> c\ny;", "<!--\nx = 1;\n/* */ --> c\ny;"] {
        let ks = kinds_of_jsx(src, false, cpu);
        assert!(!ks.contains(&TokenKind::MinusMinus), "{src:?}: {ks:?}");
        let ks = kinds_of_jsx(src, true, cpu);
        assert!(ks.contains(&TokenKind::MinusMinus), "{src:?}: {ks:?}");
    }
    // Without a `<!--` before it, a JSX script keeps `-->` as operators: the finder that would
    // stop at every `>` is only switched on by the opener.
    let ks = kinds_of_jsx("a;\n--> b;", false, cpu);
    assert!(ks.contains(&TokenKind::MinusMinus), "{ks:?}");
    let ks = kinds_of_jsx("x <!-- y;", true, cpu);
    assert!(ks.contains(&TokenKind::Lt) && ks.contains(&TokenKind::Bang), "{ks:?}");
}

#[test_cpus]
fn tsx_script_lone_html_close_comment_stays_operators(cpu: DynCpu) {
    // tsc has no HTML comments: the line is a decrement and a comparison, and tsc rejects it.
    for src in ["a;\n--> b;", "f((\n--> c\n) => 1);", "x = <a/>;\n  --> c\ny;"] {
        for module in [false, true] {
            let opts = LexOptions {
                source_type_module: module,
                jsx: true,
                ts: true,
                ..Default::default()
            };
            let ks = kinds_with(src, opts, cpu);
            assert!(
                ks.contains(&TokenKind::MinusMinus) && ks.contains(&TokenKind::Gt),
                "{src:?} module={module}: {ks:?}"
            );
            assert!(diag_codes_with(src, opts, cpu).is_empty(), "{src:?} module={module}");
        }
    }
}

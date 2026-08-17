#![cfg(target_endian = "little")]
#![expect(clippy::cast_possible_truncation, reason = "test helpers: lengths fit u32")]

use oxc_lexer::{Lexer, PAD, TokenKind, default_options, diag_code, lex_utf8};

fn kinds_of(code: &str, module: bool) -> Vec<TokenKind> {
    let mut buf = code.as_bytes().to_vec();
    let n = buf.len();
    buf.resize(n + PAD, 0);
    let mut opts = default_options();
    opts.source_type_module = module;
    let mut lx = Lexer::new();
    let count = lx.lex(&buf, n, opts);
    lx.kinds()[..count].iter().copied().filter(|kk| !kk.is_trivia()).collect()
}

fn diag_codes(code: &str, module: bool) -> Vec<u16> {
    let mut buf = code.as_bytes().to_vec();
    let n = buf.len() as u32;
    buf.resize(buf.len() + PAD, 0);
    let mut opts = default_options();
    opts.source_type_module = module;
    let (res, _arena) = lex_utf8(&buf, n, opts);
    res.diagnostics().iter().map(|d| d.code).collect()
}

#[test]
fn script_html_open_comment_anywhere() {
    let ks = kinds_of("x <!-- y\nz;", false);
    assert!(
        !ks.contains(&TokenKind::Lt) && !ks.contains(&TokenKind::Bang),
        "script comment: {ks:?}"
    );
    assert!(diag_codes("x <!-- y\nz;", false).is_empty());
}

#[test]
fn module_html_open_comment_mid_expression_is_lt() {
    let ks = kinds_of("x <!-- y;", true);
    assert!(ks.contains(&TokenKind::Lt), "expected `<` operator: {ks:?}");
    assert!(ks.contains(&TokenKind::Bang), "expected `!` operator: {ks:?}");
    assert!(ks.contains(&TokenKind::MinusMinus), "expected `--` operator: {ks:?}");
    assert!(diag_codes("x <!-- y;", true).is_empty());
}

#[test]
fn module_html_open_comment_at_line_start_diagnosed() {
    let src = "<!-- c\nx;";
    let ks = kinds_of(src, true);
    assert!(!ks.contains(&TokenKind::Lt), "line-start form stays a comment: {ks:?}");
    assert_eq!(diag_codes(src, true), vec![diag_code::HTML_COMMENT_IN_MODULE]);
    let src = "x;\n<!-- c\ny;";
    assert_eq!(diag_codes(src, true), vec![diag_code::HTML_COMMENT_IN_MODULE]);
    assert!(diag_codes(src, false).is_empty(), "script form is silent");
}

#[test]
fn module_html_close_comment_is_operators() {
    let src = "a;\n--> b;";
    let script = kinds_of(src, false);
    assert!(!script.contains(&TokenKind::MinusMinus), "script comment: {script:?}");
    let module = kinds_of(src, true);
    assert!(module.contains(&TokenKind::MinusMinus), "expected `--`: {module:?}");
    assert!(module.contains(&TokenKind::Gt), "expected `>`: {module:?}");
    assert!(diag_codes(src, true).is_empty());
}

#[test]
fn mid_line_close_comment_unchanged() {
    for module in [false, true] {
        let ks = kinds_of("a --> b;", module);
        assert!(ks.contains(&TokenKind::MinusMinus), "module={module}: {ks:?}");
        assert!(ks.contains(&TokenKind::Gt), "module={module}: {ks:?}");
    }
}

#![cfg(target_endian = "little")]
#![expect(clippy::cast_possible_truncation, reason = "test helpers: lengths fit u32")]

use oxc_lexer::{Lexer, PAD, TokenKind, default_options, lex_utf8};

fn kinds_of(code: &str, ts: bool, jsx: bool) -> Vec<TokenKind> {
    let mut buf = code.as_bytes().to_vec();
    let n = buf.len();
    buf.resize(n + PAD, 0);
    let mut opts = default_options();
    opts.ts = ts;
    opts.jsx = jsx;
    let mut lx = Lexer::new();
    let count = lx.lex(&buf, n, opts);
    lx.kinds()[..count].iter().copied().filter(|kk| !kk.is_trivia()).collect()
}

#[test]
fn one_probe_per_family() {
    let ks = kinds_of("a++ / b;", false, false);
    assert!(!ks.contains(&TokenKind::RegExp) && ks.contains(&TokenKind::Slash), "{ks:?}");
    let ks = kinds_of("x = ++/re/.lastIndex;", false, false);
    assert!(ks.contains(&TokenKind::RegExp), "{ks:?}");
    let ks = kinds_of("export default /^x$/;", false, false);
    assert!(ks.contains(&TokenKind::RegExp), "{ks:?}");
    let ks = kinds_of("if (x) /re/.test(y);", false, false);
    assert!(ks.contains(&TokenKind::RegExp), "{ks:?}");
    let ks = kinds_of("(a + b) / 2;", false, false);
    assert!(!ks.contains(&TokenKind::RegExp), "{ks:?}");
    let ks = kinds_of("(class {} / 2);", false, false);
    assert!(!ks.contains(&TokenKind::RegExp), "{ks:?}");
    let ks = kinds_of("var yield = 1; var r = yield /2/g;", false, false);
    assert!(!ks.contains(&TokenKind::RegExp), "{ks:?}");
    let ks = kinds_of("function* g() { yield /re/; }", false, false);
    assert!(ks.contains(&TokenKind::RegExp), "{ks:?}");
    let ks = kinds_of("export default <App/>;", false, true);
    assert!(ks.contains(&TokenKind::JsxLt), "{ks:?}");
}

#[test]
fn arena_api_matches_and_stays_clean() {
    let code = "x = /a/g / /b/g; if (y) /re/.test(z); var yield = 1; var r = yield /2/g;";
    let mut buf = code.as_bytes().to_vec();
    let n = buf.len() as u32;
    buf.resize(buf.len() + PAD, 0);
    let (res, arena) = lex_utf8(&buf, n, default_options());
    assert!(res.diagnostics().is_empty(), "{:?}", res.diagnostics());
    let count = res.token_count as usize;
    let kinds = res.tok_kinds(&arena)[..count].to_vec();
    let re = kinds.iter().filter(|&&kk| kk == TokenKind::RegExp).count();
    assert_eq!(re, 3, "two regex literals and one regex argument: {kinds:?}");
}

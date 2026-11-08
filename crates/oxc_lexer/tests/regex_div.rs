#![cfg(target_endian = "little")]
#![expect(clippy::cast_possible_truncation, reason = "test helpers: lengths fit u32")]

use oxc_lexer::{LexOptions, PAD, TokenKind, testing::lex_utf8_with_cpu};
use oxc_simd::{DynCpu, test_cpus};

fn kinds_of(code: &str, ts: bool, jsx: bool, cpu: DynCpu) -> Vec<TokenKind> {
    let mut buf = code.as_bytes().to_vec();
    let n = buf.len();
    buf.resize(n + PAD, 0);
    let opts = LexOptions { jsx, ts, ..Default::default() };
    let (res, arena) = lex_utf8_with_cpu(&buf, n as u32, opts, cpu);
    let count = res.token_count as usize;
    res.tok_kinds(&arena)[..count].iter().copied().filter(|kk| !kk.is_trivia()).collect()
}

#[test_cpus]
fn one_probe_per_family(cpu: DynCpu) {
    let ks = kinds_of("a++ / b;", false, false, cpu);
    assert!(!ks.contains(&TokenKind::RegExp) && ks.contains(&TokenKind::Slash), "{ks:?}");
    let ks = kinds_of("x = ++/re/.lastIndex;", false, false, cpu);
    assert!(ks.contains(&TokenKind::RegExp), "{ks:?}");
    let ks = kinds_of("export default /^x$/;", false, false, cpu);
    assert!(ks.contains(&TokenKind::RegExp), "{ks:?}");
    let ks = kinds_of("if (x) /re/.test(y);", false, false, cpu);
    assert!(ks.contains(&TokenKind::RegExp), "{ks:?}");
    let ks = kinds_of("(a + b) / 2;", false, false, cpu);
    assert!(!ks.contains(&TokenKind::RegExp), "{ks:?}");
    let ks = kinds_of("(class {} / 2);", false, false, cpu);
    assert!(!ks.contains(&TokenKind::RegExp), "{ks:?}");
    let ks = kinds_of("var yield = 1; var r = yield /2/g;", false, false, cpu);
    assert!(!ks.contains(&TokenKind::RegExp), "{ks:?}");
    let ks = kinds_of("function* g() { yield /re/; }", false, false, cpu);
    assert!(ks.contains(&TokenKind::RegExp), "{ks:?}");
    let ks = kinds_of("export default <App/>;", false, true, cpu);
    assert!(ks.contains(&TokenKind::JsxLt), "{ks:?}");
}

#[test_cpus]
fn arena_api_matches_and_stays_clean(cpu: DynCpu) {
    let code = "x = /a/g / /b/g; if (y) /re/.test(z); var yield = 1; var r = yield /2/g;";
    let mut buf = code.as_bytes().to_vec();
    let n = buf.len() as u32;
    buf.resize(buf.len() + PAD, 0);
    let (res, arena) = lex_utf8_with_cpu(&buf, n, LexOptions::default(), cpu);
    assert!(res.diagnostics().is_empty(), "{:?}", res.diagnostics());
    let count = res.token_count as usize;
    let kinds = res.tok_kinds(&arena)[..count].to_vec();
    let re = kinds.iter().filter(|&&kk| kk == TokenKind::RegExp).count();
    assert_eq!(re, 3, "two regex literals and one regex argument: {kinds:?}");
}

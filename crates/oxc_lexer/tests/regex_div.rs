#![cfg(target_endian = "little")]

use oxc_lexer::{LexOptions, TokenKind, testing::lex_with_cpu};
use oxc_simd::{DynCpu, test_cpus};

fn kinds_of(code: &str, ts: bool, jsx: bool, cpu: DynCpu) -> Vec<TokenKind> {
    let opts = LexOptions { jsx, ts, ..Default::default() };
    let res = lex_with_cpu(code, opts, cpu);
    let count = res.token_count();
    res.kinds()[..count].iter().copied().filter(|kk| !kk.is_trivia()).collect()
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
fn tokens_api_matches_and_stays_clean(cpu: DynCpu) {
    let code = "x = /a/g / /b/g; if (y) /re/.test(z); var yield = 1; var r = yield /2/g;";
    let res = lex_with_cpu(code, LexOptions::default(), cpu);
    assert!(res.diagnostics().is_empty(), "{:?}", res.diagnostics());
    let count = res.token_count();
    let kinds = res.kinds()[..count].to_vec();
    let re = kinds.iter().filter(|&&kk| kk == TokenKind::RegExp).count();
    assert_eq!(re, 3, "two regex literals and one regex argument: {kinds:?}");
}

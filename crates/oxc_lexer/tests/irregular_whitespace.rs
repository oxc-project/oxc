#![cfg(target_endian = "little")]

use oxc_lexer::{LexOptions, testing::lex_with_cpu};
use oxc_simd::{DynCpu, test_cpus};

fn irregular_whitespaces(source: &str, jsx: bool, cpu: DynCpu) -> Vec<(u32, u32)> {
    let options = LexOptions { jsx, ..LexOptions::default() };
    let tokens = lex_with_cpu(source, options, cpu);
    assert!(tokens.diagnostics().is_empty(), "{source:?}");
    tokens.irregular_whitespaces().iter().map(|span| (span.start, span.end)).collect()
}

#[test_cpus]
fn irregular_whitespace_between_tokens(cpu: DynCpu) {
    let cases: &[(&str, &[(u32, u32)])] = &[
        ("a\u{b}=\u{c}b;", &[(1, 2), (3, 4)]),
        ("a \u{b}\t= b;", &[(2, 3)]),
        ("a\u{a0}= b;", &[(1, 3)]),
        ("\u{feff}a;", &[(0, 3)]),
        ("a;\u{2028}b;\u{2029}", &[(2, 5), (7, 10)]),
        ("a\u{3000}\u{205f}= b;", &[(1, 4), (4, 7)]),
        ("`${\u{a0}a}`;", &[(3, 5)]),
        ("// c\u{2028}a;", &[(4, 7)]),
        ("a /* c */\u{a0}= b;", &[(9, 11)]),
    ];
    let mut mismatches = Vec::new();
    for &(source, want) in cases {
        let actual = irregular_whitespaces(source, false, cpu);
        if actual != want {
            mismatches.push(format!("{source:?}: got {actual:?}, want {want:?}"));
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

#[test_cpus]
fn irregular_whitespace_inside_literals_and_comments(cpu: DynCpu) {
    for source in [
        "'\u{b}\u{a0}';",
        "`\u{c}\u{3000}`;",
        "`${a}\u{a0}`;",
        "/\u{a0}\u{b}/;",
        "// \u{a0}\u{c}\n",
        "/* \u{2028}\u{b} */",
        "#!/usr/bin/env\u{a0}node\na;",
    ] {
        assert_eq!(irregular_whitespaces(source, false, cpu), [], "{source:?}");
    }
}

#[test_cpus]
fn irregular_whitespace_in_jsx(cpu: DynCpu) {
    assert_eq!(irregular_whitespaces("<a\u{a0}b='\u{a0}'>\u{a0}c\u{a0}</a>;", true, cpu), [(2, 4)]);
    assert_eq!(irregular_whitespaces("<a>{\u{a0}b}</a>;", true, cpu), [(4, 6)]);
}

#![cfg(target_endian = "little")]
#![expect(clippy::cast_possible_truncation, reason = "test helpers: lengths fit u32")]

use oxc_lexer::{LexOptions, PAD, lex_utf8};

fn irregular_whitespaces(source: &str, jsx: bool) -> Vec<(u32, u32)> {
    let mut buf = source.as_bytes().to_vec();
    buf.resize(source.len() + PAD, 0);
    let options = LexOptions { jsx, ..LexOptions::default() };
    let (result, arena) = lex_utf8(&buf, source.len() as u32, options);
    assert!(result.diagnostics().is_empty(), "{source:?}");
    result.irregular_whitespaces(&arena).iter().map(|span| (span.start, span.end)).collect()
}

#[test]
fn irregular_whitespace_between_tokens() {
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
        let actual = irregular_whitespaces(source, false);
        if actual != want {
            mismatches.push(format!("{source:?}: got {actual:?}, want {want:?}"));
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

#[test]
fn irregular_whitespace_inside_literals_and_comments() {
    for source in [
        "'\u{b}\u{a0}';",
        "`\u{c}\u{3000}`;",
        "`${a}\u{a0}`;",
        "/\u{a0}\u{b}/;",
        "// \u{a0}\u{c}\n",
        "/* \u{2028}\u{b} */",
        "#!/usr/bin/env\u{a0}node\na;",
    ] {
        assert_eq!(irregular_whitespaces(source, false), [], "{source:?}");
    }
}

#[test]
fn irregular_whitespace_in_jsx() {
    assert_eq!(irregular_whitespaces("<a\u{a0}b='\u{a0}'>\u{a0}c\u{a0}</a>;", true), [(2, 4)]);
    assert_eq!(irregular_whitespaces("<a>{\u{a0}b}</a>;", true), [(4, 6)]);
}

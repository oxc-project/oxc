#![cfg(target_endian = "little")]
#![expect(clippy::cast_possible_truncation, reason = "test helpers: lengths fit u32")]

use oxc_lexer::{LexOptions, PAD, testing::lex_utf8_with_cpu};
use oxc_simd::{DynCpu, test_cpus};

fn comment_flags(source: &str, cpu: DynCpu) -> Vec<(bool, bool)> {
    let mut buf = source.as_bytes().to_vec();
    buf.resize(source.len() + PAD, 0);
    let (result, arena) = lex_utf8_with_cpu(&buf, source.len() as u32, LexOptions::default(), cpu);
    assert!(result.diagnostics().is_empty(), "{source:?}");
    result
        .comments(&arena)
        .iter()
        .map(|comment| (comment.preceded_by_newline(), comment.followed_by_newline()))
        .collect()
}

// Cases from https://github.com/oxc-project/oxc/issues/28318, plus a leading BOM.
#[test_cpus]
fn comment_newline_flags_ls_ps_and_unicode_ws(cpu: DynCpu) {
    let cases: &[(&str, bool, bool)] = &[
        ("x;\u{2029}/* a */ y;", true, false),
        ("x;\n\u{00A0}/* a */ y;", true, false),
        ("x;\n\u{FEFF}/* a */ y;", true, false),
        ("x; /* a */\u{2028}y;", false, true),
        ("x; /* a */\u{00A0}\ny;", false, true),
        ("\u{FEFF}/* a */ y;", true, false),
    ];
    let mut mismatches = Vec::new();
    for &(source, preceded, followed) in cases {
        let actual = comment_flags(source, cpu);
        if actual != [(preceded, followed)] {
            mismatches.push(format!("{source:?}: got {actual:?}, want [({preceded}, {followed})]"));
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

#[test_cpus]
fn comment_newline_flags_ascii_still_work(cpu: DynCpu) {
    assert_eq!(comment_flags("x;\n/* a */ y;", cpu), [(true, false)]);
    assert_eq!(comment_flags("x; /* a */\ny;", cpu), [(false, true)]);
    assert_eq!(comment_flags("x; /* a */ y;", cpu), [(false, false)]);
}

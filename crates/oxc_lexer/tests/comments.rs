#![cfg(target_endian = "little")]
#![expect(clippy::cast_possible_truncation, reason = "test helpers: lengths fit u32")]

use oxc_ast::CommentKind::{self, HtmlClose, MultiLineBlock, SingleLineBlock};
use oxc_lexer::{DiagCode, LexOptions, PAD, lex_utf8};

/// Lex `source` as a script. Returns the kinds of its comments and the codes of its diagnostics.
fn lex(source: &str) -> (Vec<CommentKind>, Vec<DiagCode>) {
    let mut buf = source.as_bytes().to_vec();
    buf.resize(source.len() + PAD, 0);
    let (result, arena) = lex_utf8(&buf, source.len() as u32, LexOptions::default());
    let kinds = result.comments(&arena).iter().map(|comment| comment.kind).collect();
    let codes = result.diagnostics().iter().map(|diagnostic| diagnostic.code).collect();
    (kinds, codes)
}

fn comment_kinds(source: &str) -> Vec<CommentKind> {
    let (kinds, codes) = lex(source);
    assert!(codes.is_empty(), "{source:?}: {codes:?}");
    kinds
}

/// LS and PS are line terminators, so a block comment containing one is multi-line,
/// the same as one containing LF or CR. Other chars encoded with the same lead byte (0xE2) are not.
#[test]
fn block_comment_line_terminators() {
    let cases = [
        ("", SingleLineBlock),
        ("\n", MultiLineBlock),
        ("\r", MultiLineBlock),
        ("\u{2028}", MultiLineBlock),  // LS: E2 80 A8
        ("\u{2029}", MultiLineBlock),  // PS: E2 80 A9
        ("\u{2026}", SingleLineBlock), // E2 80 A6
        ("\u{2027}", SingleLineBlock), // E2 80 A7
        ("\u{202F}", SingleLineBlock), // E2 80 AF
        ("\u{2128}", SingleLineBlock), // E2 84 A8
        ("\u{2026}\u{2028}", MultiLineBlock),
        ("\u{2026}\u{2029}", MultiLineBlock),
        ("\u{2026}\n", MultiLineBlock),
        ("\u{2026}\r", MultiLineBlock),
        ("\u{2026}\u{2027}", SingleLineBlock),
    ];

    // Code after the comment keeps the scanners in their chunked loops until the comment ends,
    // and varying the text before the char moves it across chunk boundaries.
    let code_after = "x;".repeat(32);
    for len in 0..64 {
        let text = "x".repeat(len);
        for (char, kind) in cases {
            let comment = format!("/*{text}{char}*/");
            for source in [comment.clone(), format!("{comment}{code_after}")] {
                assert_eq!(comment_kinds(&source), [kind], "{source:?}");
            }

            // A line terminator after the comment doesn't make it multi-line.
            if matches!(char, "\n" | "\r" | "\u{2028}" | "\u{2029}") {
                let source = format!("/*{text}*/{char}{code_after}");
                assert_eq!(comment_kinds(&source), [SingleLineBlock], "{source:?}");
            }
        }
    }
}

/// The kind of an unterminated block comment is found by another route, after the scan.
#[test]
fn unterminated_block_comment_line_terminators() {
    for (char, kind) in [
        ("\n", MultiLineBlock),
        ("\r", MultiLineBlock),
        ("\u{2028}", MultiLineBlock),
        ("\u{2029}", MultiLineBlock),
        ("\u{2027}", SingleLineBlock),
    ] {
        for source in [format!("/*{char}"), format!("x /*{char} unterminated")] {
            assert_eq!(
                lex(&source),
                (vec![kind], vec![DiagCode::UnterminatedBlockComment]),
                "{source:?}"
            );
        }
    }
}

/// In a script, a `-->` after a block comment containing a line terminator is at the start of
/// a line, so it begins an HTML close comment.
#[test]
fn html_close_comment_after_multi_line_block_comment() {
    for char in ["\n", "\r", "\u{2028}", "\u{2029}"] {
        let source = format!("a /*{char}*/ --> b");
        assert_eq!(comment_kinds(&source), [MultiLineBlock, HtmlClose], "{source:?}");
        let source = format!("a /*{char}*/ /**/ --> b");
        assert_eq!(
            comment_kinds(&source),
            [MultiLineBlock, SingleLineBlock, HtmlClose],
            "{source:?}"
        );
    }

    // `--` and `>`
    let source = "a /*\u{2027}*/ --> b";
    assert_eq!(comment_kinds(source), [SingleLineBlock], "{source:?}");
}

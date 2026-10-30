#![cfg(target_endian = "little")]
#![expect(clippy::cast_possible_truncation, reason = "test helpers: lengths fit u32")]

use oxc_ast::CommentContent;
use oxc_lexer::{LexOptions, PAD, lex_utf8};

fn comment_contents(source: &str) -> Vec<CommentContent> {
    let mut buf = source.as_bytes().to_vec();
    buf.resize(source.len() + PAD, 0);
    let (result, arena) = lex_utf8(&buf, source.len() as u32, LexOptions::default());
    assert!(result.diagnostics().is_empty(), "{source:?}");
    result.comments(&arena).iter().map(|comment| comment.content).collect()
}

/// `@license` or `@preserve` anywhere in a comment makes it legal, even right at its end.
#[test]
fn license_or_preserve_at_end() {
    // spellchecker:off
    let cases = [
        ("//", "@license", CommentContent::Legal),
        ("//", "@license\u{2028}", CommentContent::Legal),
        ("//", "@preserve", CommentContent::Legal),
        ("//", "@licens", CommentContent::None),
        ("/*", "@license*/", CommentContent::Legal),
        ("/*", "@preserve*/", CommentContent::Legal),
        ("/*", "@preserv*/", CommentContent::None),
        ("/**", "@license*/", CommentContent::JsdocLegal),
        ("/**", "@licens*/", CommentContent::Jsdoc),
    ];
    // spellchecker:on

    // Code after the comment keeps the scanners in their chunked loops until the comment ends,
    // and varying the text before the tag moves the end of the comment across chunk boundaries.
    let code_after = format!("\n{}", "x;".repeat(32));
    for len in 0..40 {
        let text = "x".repeat(len);
        for (open, close, content) in cases {
            let comment = format!("{open} {text} {close}");
            for source in [comment.clone(), format!("{comment}{code_after}")] {
                assert_eq!(comment_contents(&source), [content], "{source:?}");
            }
        }
    }
}

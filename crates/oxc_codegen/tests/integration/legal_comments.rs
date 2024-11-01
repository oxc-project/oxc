use crate::snapshot;

fn cases() -> Vec<&'static str> {
    vec![
        "/* @license */\n/* @license */\nfoo;bar;",
        "/* @license */\n/* @preserve */\nfoo;bar;",
        "/* @license */\n//! KEEP\nfoo;bar;",
        "/* @license */\n/*! KEEP */\nfoo;bar;",
    ]
}

#[test]
fn legal_inline_comment() {
    snapshot("legal_inline_comments", &cases());
}

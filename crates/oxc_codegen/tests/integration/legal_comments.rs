use oxc_codegen::{CodegenOptions, LegalComment};

use crate::{snapshot, snapshot_options};

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

#[test]
fn legal_eof_comment() {
    let options = CodegenOptions { legal_comments: LegalComment::Eof, ..Default::default() };
    snapshot_options("legal_eof_comments", &cases(), &options);
}

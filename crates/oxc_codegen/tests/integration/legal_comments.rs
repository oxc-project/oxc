use oxc_codegen::{CodegenOptions, LegalComment};

use crate::{codegen_options, snapshot, snapshot_options};

fn cases() -> Vec<&'static str> {
    vec![
        "/* @license */\n/* @license */\nfoo;bar;",
        "/* @license */\n/* @preserve */\nfoo;bar;",
        "/* @license */\n//! KEEP\nfoo;bar;",
        "/* @license */\n/*! KEEP */\nfoo;bar;",
        "/* @license *//*! KEEP */\nfoo;bar;",
        "function () {
    /*
    * @license
    * Copyright notice 2
    */
    bar;
}",
        "function bar() { var foo; /*! #__NO_SIDE_EFFECTS__ */ function () { } }",
        "function foo() {
	(() => {
		/**
		 * @preserve
		 */
	})();
	/**
	 * @preserve
	 */
}
/**
 * @preserve
 */",
        "/**
* @preserve
*/
",
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

#[test]
fn legal_linked_comment() {
    let options = CodegenOptions {
        legal_comments: LegalComment::Linked(String::from("test.js")),
        ..Default::default()
    };
    snapshot_options("legal_linked_comments", &cases(), &options);
}

#[test]
fn legal_external_comment() {
    let options = CodegenOptions { legal_comments: LegalComment::External, ..Default::default() };
    let code = "/* @license */\n/* @preserve */\nfoo;\n";
    let ret = codegen_options(code, &options);
    assert_eq!(ret.code, "foo;\n");
    assert_eq!(ret.legal_comments[0].content_span().source_text(code), " @license ");
    assert_eq!(ret.legal_comments[1].content_span().source_text(code), " @preserve ");
}

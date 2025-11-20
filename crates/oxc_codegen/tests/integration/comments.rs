use crate::tester::{test, test_same};

#[test]
fn test_comment_at_top_of_file() {
    use oxc_allocator::Allocator;
    use oxc_ast::CommentPosition;
    use oxc_codegen::Codegen;
    use oxc_parser::Parser;
    use oxc_span::SourceType;
    let source_type = SourceType::mjs();
    let allocator = Allocator::default();
    let mut ret = Parser::new(&allocator, "export{} /** comment */", source_type).parse();
    // Move comment to top of the file.
    ret.program.comments[0].attached_to = 0;
    ret.program.comments[0].position = CommentPosition::Leading;
    let code = Codegen::new().build(&ret.program).code;
    assert_eq!(code, "/** comment */ export {};\n");
}

#[test]
fn unit() {
    test_same("<div>{/* Hello */}</div>;\n");
    // https://lingui.dev/ref/macro#definemessage
    test("const message = /*i18n*/{};", "const message = (/*i18n*/ {});\n");
    test(
        "function foo() { return /*i18n*/ {} }",
        "function foo() {\n\treturn (\t/*i18n*/ {});\n}\n",
    );
}

pub mod jsdoc {
    use crate::snapshot;

    #[test]
    fn comment() {
        let cases = vec![
            r"
/**
 * Top level
 *
 * @module
 */

/** This is a description of the foo function. */
function foo() {
}

/**
 * Preserve newline
 */

/**
 * Represents a book.
 * @constructor
 * @param {string} title - The title of the book.
 * @param {string} author - The author of the book.
 */
function Book(title, author) {
}

/** Class representing a point. */
class Point {
    /**
     * Preserve newline
     */

    /**
     * Create a point.
     * @param {number} x - The x value.
     * @param {number} y - The y value.
     */
    constructor(x, y) {
    }

    /**
     * Get the x value.
     * @return {number} The x value.
     */
    getX() {
    }

    /**
     * Get the y value.
     * @return {number} The y value.
     */
    getY() {
    }

    /**
     * Convert a string containing two comma-separated numbers into a point.
     * @param {string} str - The string containing two comma-separated numbers.
     * @return {Point} A Point object.
     */
    static fromString(str) {
    }
}

/** Class representing a point. */
const Point = class {
}

/**
 * Shirt module.
 * @module my/shirt
 */

/** Button the shirt. */
exports.button = function() {
};

/** Unbutton the shirt. */
exports.unbutton = function() {
};

this.Book = function(title) {
    /** The title of the book. */
    this.title = title;
}
// https://github.com/oxc-project/oxc/issues/6006
export enum DefinitionKind {
  /**
   * Definition is a referenced variable.
   *
   * @example defineSomething(foo)
   */
  Reference = 'Reference',
  /**
   * Definition is a `ObjectExpression`.
   *
   * @example defineSomething({ ... })
   */
  Object = 'Object',
  /**
   * Definition is TypeScript interface.
   *
   * @example defineSomething<{ ... }>()
   */
  TS = 'TS',
}
export type TSTypeLiteral = {
    /**
     * Comment
     */
    foo: string
}
",
        ];

        snapshot("jsdoc", &cases);
    }
}

pub mod coverage {
    use crate::snapshot;

    #[test]
    fn comment() {
        let cases = vec![
            "/* v8 ignore next */ x",
            "/* v8 ignore next 2 */ x",
            "/* v8 ignore start */ x",
            "/* v8 ignore stop */ x",
            "/* v8 ignore if */ x",
            "/* v8 ignore else */ x",
            "/* v8 ignore file */ x",
            "/* c8 ignore next */ x",
            "/* c8 ignore next 2 */x ",
            "/* c8 ignore start */ x",
            "/* c8 ignore stop */ x",
            "/* node:coverage disable */ x",
            "/* node:coverage enable */ x",
            "/* node:coverage ignore next */ x",
            "/* node:coverage ignore next 2 */ x",
            "/* istanbul ignore if */ x",
            "/* istanbul ignore else */ x",
            "/* istanbul ignore next */ x",
            "/* istanbul ignore file */ x",
            "try { something(); }
/* istanbul ignore next */
catch(e) {
  // should never happen
}
",
        ];

        snapshot("coverage", &cases);
    }
}

pub mod legal {
    use oxc_codegen::{CodegenOptions, CommentOptions, LegalComment};

    use crate::{codegen_options, snapshot, snapshot_options};

    fn cases() -> Vec<&'static str> {
        vec![
            "/* @license */\n/* @license */\nfoo;bar;",
            "/* @license */\n/* @preserve */\nfoo;bar;",
            "/* @license */\n//! KEEP\nfoo;bar;",
            "/* @license */\n/*! KEEP */\nfoo;bar;",
            "/* @license *//*! KEEP */\nfoo;bar;",
            "function test() {
    /*
    * @license
    * Copyright notice 2
    */
    bar;
}",
            "function bar() { var foo; /*! #__NO_SIDE_EFFECTS__ */ function baz() { } }",
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
            // Issue #14953: legal comments above directives
            "/*!\n * legal comment\n */\n\n\"use strict\";\n\nexport const foo = 'foo';",
        ]
    }

    #[test]
    fn legal_inline_comment() {
        snapshot("legal_inline_comments", &cases());
    }

    #[test]
    fn legal_eof_comment() {
        let options = CodegenOptions {
            comments: CommentOptions { legal: LegalComment::Eof, ..CommentOptions::default() },
            ..CodegenOptions::default()
        };
        snapshot_options("legal_eof_comments", &cases(), &options);
    }

    #[test]
    fn legal_eof_minify_comment() {
        let options = CodegenOptions {
            minify: true,
            comments: CommentOptions { legal: LegalComment::Eof, ..CommentOptions::default() },
            ..CodegenOptions::default()
        };
        snapshot_options("legal_eof_minify_comments", &cases(), &options);
    }

    #[test]
    fn legal_linked_comment() {
        let options = CodegenOptions {
            comments: CommentOptions {
                legal: LegalComment::Linked(String::from("test.js")),
                ..CommentOptions::default()
            },
            ..CodegenOptions::default()
        };
        snapshot_options("legal_linked_comments", &cases(), &options);
    }

    #[test]
    fn legal_external_comment() {
        let options = CodegenOptions {
            comments: CommentOptions { legal: LegalComment::External, ..CommentOptions::default() },
            ..CodegenOptions::default()
        };
        let code = "/* @license */\n/* @preserve */\nfoo;\n";
        let ret = codegen_options(code, &options);
        assert_eq!(ret.code, "foo;\n");
        assert_eq!(ret.legal_comments[0].content_span().source_text(code), " @license ");
        assert_eq!(ret.legal_comments[1].content_span().source_text(code), " @preserve ");
    }
}

pub mod pure {
    use crate::snapshot;

    #[test]
    fn annotate_comment() {
        let cases = vec![
            r"
x([
  /* #__NO_SIDE_EFFECTS__ */ function() {},
  /* #__NO_SIDE_EFFECTS__ */ function y() {},
  /* #__NO_SIDE_EFFECTS__ */ function*() {},
  /* #__NO_SIDE_EFFECTS__ */ function* y() {},
  /* #__NO_SIDE_EFFECTS__ */ async function() {},
  /* #__NO_SIDE_EFFECTS__ */ async function y() {},
  /* #__NO_SIDE_EFFECTS__ */ async function*() {},
  /* #__NO_SIDE_EFFECTS__ */ async function* y() {},
])",
            r"
x([
  /* #__NO_SIDE_EFFECTS__ */ y => y,
  /* #__NO_SIDE_EFFECTS__ */ () => {},
  /* #__NO_SIDE_EFFECTS__ */ (y) => (y),
  /* #__NO_SIDE_EFFECTS__ */ async y => y,
  /* #__NO_SIDE_EFFECTS__ */ async () => {},
  /* #__NO_SIDE_EFFECTS__ */ async (y) => (y),
])",
            r"
x([
  /* #__NO_SIDE_EFFECTS__ */ y => y,
  /* #__NO_SIDE_EFFECTS__ */ () => {},
  /* #__NO_SIDE_EFFECTS__ */ (y) => (y),
  /* #__NO_SIDE_EFFECTS__ */ async y => y,
  /* #__NO_SIDE_EFFECTS__ */ async () => {},
  /* #__NO_SIDE_EFFECTS__ */ async (y) => (y),
])",
            r"
// #__NO_SIDE_EFFECTS__
function a() {}
// #__NO_SIDE_EFFECTS__
function* b() {}
// #__NO_SIDE_EFFECTS__
async function c() {}
// #__NO_SIDE_EFFECTS__
async function* d() {}
        ",
            r"
// #__NO_SIDE_EFFECTS__
function a() {}
// #__NO_SIDE_EFFECTS__
function* b() {}
// #__NO_SIDE_EFFECTS__
async function c() {}
// #__NO_SIDE_EFFECTS__
async function* d() {}
        ",
            r"
/* @__NO_SIDE_EFFECTS__ */ export function a() {}
/* @__NO_SIDE_EFFECTS__ */ export function* b() {}
/* @__NO_SIDE_EFFECTS__ */ export async function c() {}
/* @__NO_SIDE_EFFECTS__ */ export async function* d() {}",
            r"/* @__NO_SIDE_EFFECTS__ */ export function a() {}
/* @__NO_SIDE_EFFECTS__ */ export function* b() {}
/* @__NO_SIDE_EFFECTS__ */ export async function c() {}
/* @__NO_SIDE_EFFECTS__ */ export async function* d() {}
export default /* @__NO_SIDE_EFFECTS__ */ async function() {}
export default /* @__NO_SIDE_EFFECTS__ */ function() {}
        ",
            // Only "c0" and "c2" should have "no side effects" (Rollup only respects "const" and only for the first one)
            r"
/* #__NO_SIDE_EFFECTS__ */ export var v0 = function() {}, v1 = function() {}
/* #__NO_SIDE_EFFECTS__ */ export let l0 = function() {}, l1 = function() {}
/* #__NO_SIDE_EFFECTS__ */ export const c0 = function() {}, c1 = function() {}
/* #__NO_SIDE_EFFECTS__ */ export var v2 = () => {}, v3 = () => {}
/* #__NO_SIDE_EFFECTS__ */ export let l2 = () => {}, l3 = () => {}
/* #__NO_SIDE_EFFECTS__ */ export const c2 = () => {}, c3 = () => {}
        ",
            // Only "c0" and "c2" should have "no side effects" (Rollup only respects "const" and only for the first one)
            r"
/* #__NO_SIDE_EFFECTS__ */ var v0 = function() {}, v1 = function() {}
/* #__NO_SIDE_EFFECTS__ */ let l0 = function() {}, l1 = function() {}
/* #__NO_SIDE_EFFECTS__ */ const c0 = function() {}, c1 = function() {}
/* #__NO_SIDE_EFFECTS__ */ var v2 = () => {}, v3 = () => {}
/* #__NO_SIDE_EFFECTS__ */ let l2 = () => {}, l3 = () => {}
/* #__NO_SIDE_EFFECTS__ */ const c2 = () => {}, c3 = () => {}
        ",
            r"
isFunction(options)
? // #8326: extend call and options.name access are considered side-effects
  // by Rollup, so we have to wrap it in a pure-annotated IIFE.
  /*#__PURE__*/ (() =>
    extend({ name: options.name }, extraOptions, { setup: options }))()
: options
                ",
            r"isFunction(options) ? /*#__PURE__*/ (() => extend({ name: options.name }, extraOptions, { setup: options }))() : options;
        ",
            r"
const obj = {
  props: /*#__PURE__*/ extend({}, TransitionPropsValidators, {
    tag: String,
    moveClass: String,
  }),
};
const p = /*#__PURE__*/ Promise.resolve();
        ",
            r"
const staticCacheMap = /*#__PURE__*/ new WeakMap()
        ",
            r#"
const builtInSymbols = new Set(
  /*#__PURE__*/
  Object.getOwnPropertyNames(Symbol)
    .filter(key => key !== "arguments" && key !== "caller")
)
        "#,
            "(/* @__PURE__ */ new Foo()).bar();\n",
            "(/* @__PURE__ */ Foo()).bar();\n",
            "(/* @__PURE__ */ new Foo())['bar']();\n",
            "(/* @__PURE__ */ Foo())['bar']();\n",
            // https://github.com/oxc-project/oxc/issues/4843
            r"
/* #__NO_SIDE_EFFECTS__ */
const defineSSRCustomElement = /* @__NO_SIDE_EFFECTS__ */ (
  options,
  extraOptions,
) => {
  return /* @__PURE__ */ defineCustomElement(options, extraOptions, hydrate);
};
        ",
            // Range leading comments
            r"
const defineSSRCustomElement = () => {
  return /* @__PURE__ */ /* @__NO_SIDE_EFFECTS__ */ /* #__NO_SIDE_EFFECTS__ */ defineCustomElement(options, extraOptions, hydrate);
};
        ",
            "
        const Component = // #__PURE__
        React.forwardRef((props, ref) => {});
        ",
            // Copy from <https://github.com/rolldown-rs/rolldown/blob/v0.14.0/crates/rolldown/tests/esbuild/dce/remove_unused_pure_comment_calls/entry.js>
            "
        function bar() {}
        let bare = foo(bar);

        let at_yes = /* @__PURE__ */ foo(bar);
        let at_no = /* @__PURE__ */ foo(bar());
        let new_at_yes = /* @__PURE__ */ new foo(bar);
        let new_at_no = /* @__PURE__ */ new foo(bar());

        let nospace_at_yes = /*@__PURE__*/ foo(bar);
        let nospace_at_no = /*@__PURE__*/ foo(bar());
        let nospace_new_at_yes = /*@__PURE__*/ new foo(bar);
        let nospace_new_at_no = /*@__PURE__*/ new foo(bar());

        let num_yes = /* #__PURE__ */ foo(bar);
        let num_no = /* #__PURE__ */ foo(bar());
        let new_num_yes = /* #__PURE__ */ new foo(bar);
        let new_num_no = /* #__PURE__ */ new foo(bar());

        let nospace_num_yes = /*#__PURE__*/ foo(bar);
        let nospace_num_no = /*#__PURE__*/ foo(bar());
        let nospace_new_num_yes = /*#__PURE__*/ new foo(bar);
        let nospace_new_num_no = /*#__PURE__*/ new foo(bar());

        let dot_yes = /* @__PURE__ */ foo(sideEffect()).dot(bar);
        let dot_no = /* @__PURE__ */ foo(sideEffect()).dot(bar());
        let new_dot_yes = /* @__PURE__ */ new foo(sideEffect()).dot(bar);
        let new_dot_no = /* @__PURE__ */ new foo(sideEffect()).dot(bar());

        let nested_yes = [1, /* @__PURE__ */ foo(bar), 2];
        let nested_no = [1, /* @__PURE__ */ foo(bar()), 2];
        let new_nested_yes = [1, /* @__PURE__ */ new foo(bar), 2];
        let new_nested_no = [1, /* @__PURE__ */ new foo(bar()), 2];

        let single_at_yes = // @__PURE__
                foo(bar);
        let single_at_no = // @__PURE__
                foo(bar());
        let new_single_at_yes = // @__PURE__
                new foo(bar);
        let new_single_at_no = // @__PURE__
                new foo(bar());

        let single_num_yes = // #__PURE__
                foo(bar);
        let single_num_no = // #__PURE__
                foo(bar());
        let new_single_num_yes = // #__PURE__
                new foo(bar);
        let new_single_num_no = // #__PURE__
                new foo(bar());

        let bad_no = /* __PURE__ */ foo(bar);
        let new_bad_no = /* __PURE__ */ new foo(bar);

        let parens_no = (/* @__PURE__ */ foo)(bar);
        let new_parens_no = new (/* @__PURE__ */ foo)(bar);

        let exp_no = /* @__PURE__ */ foo() ** foo();
        let new_exp_no = /* @__PURE__ */ new foo() ** foo();
        ",
            "{ /* @__PURE__ */ (function() {})(); }",
            "{ /* @__PURE__ */ (() => {})(); }",
            "
void /* @__PURE__ */ function() {}();
typeof /* @__PURE__ */ function() {}();
! /* @__PURE__ */ function() {}();
delete /* @__PURE__ */ (() => {})();",
            "const Foo = /* @__PURE__ */ (((() => {})()))",
            "const Foo = /* @__PURE__ */ (() => { })() as unknown as { new (): any }",
            "const Foo = /* @__PURE__ */ (() => {})() satisfies X",
            "const Foo = /* @__PURE__ */ (() => {})()<X>",
            "const Foo = /* @__PURE__ */ <Foo>(() => {})()!",
            "const Foo = /* @__PURE__ */ <Foo>(() => {})()! as X satisfies Y",
        ];

        snapshot("pure_comments", &cases);
    }
}

pub mod options {
    use oxc_codegen::{CodegenOptions, CommentOptions, LegalComment};

    use crate::codegen_options;

    #[test]
    fn test() {
        let code = "
//! Top Legal Comment
function foo() {
    /** JSDoc Comment */
    function bar() {
        /* #__PURE__ */ x();
    }
    function baz() {
        //! Function Legal Comment
    }
    x(/* Normal Comment */);
    x(/** Call Expression Jsdoc Comment */ token);
}";

        for normal in [true, false] {
            for jsdoc in [true, false] {
                for annotation in [true, false] {
                    for legal in [LegalComment::Inline, LegalComment::Eof, LegalComment::None] {
                        let options = CodegenOptions {
                            comments: CommentOptions {
                                normal,
                                jsdoc,
                                annotation,
                                legal: legal.clone(),
                            },
                            ..CodegenOptions::default()
                        };
                        let printed = codegen_options(code, &options).code;

                        if normal {
                            assert!(printed.contains("Normal Comment"));
                        } else {
                            assert!(!printed.contains("Normal Comment"));
                        }

                        if jsdoc {
                            assert!(printed.contains("JSDoc Comment"));
                            assert!(printed.contains("Call Expression Jsdoc Comment"));
                        } else {
                            assert!(!printed.contains("JSDoc Comment"));
                            assert!(!printed.contains("Call Expression Jsdoc Comment"));
                        }

                        if annotation {
                            assert!(printed.contains("__PURE__"));
                        } else {
                            assert!(!printed.contains("__PURE__"));
                        }

                        if legal.is_none() {
                            assert!(!printed.contains("Top Legal Comment"));
                            assert!(!printed.contains("Function Legal Comment"));
                        } else {
                            assert!(printed.contains("Top Legal Comment"));
                            assert!(printed.contains("Function Legal Comment"));
                        }
                    }
                }
            }
        }
    }
}

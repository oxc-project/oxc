use itertools::Itertools;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use phf::phf_set;
use serde::Deserialize;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jsdoc(check-tag-names): Invalid tag name found.")]
#[diagnostic(severity(warning), help("{1}"))]
struct CheckTagNamesDiagnostic(#[label] pub Span, String);

#[derive(Debug, Default, Clone)]
pub struct CheckTagNames(Box<CheckTagnamesConfig>);

declare_oxc_lint!(
    /// ### What it does
    /// Reports invalid block tag names.
    /// Additionally checks for tag names that are redundant when using a type checker such as TypeScript.
    ///
    /// ### Why is this bad?
    /// Using invalid tags can lead to confusion and make the documentation harder to read.
    ///
    /// ### Example
    /// ```javascript
    /// // Passing
    /// /** @param */
    ///
    /// // Failing
    /// /** @Param */
    /// /** @foo */
    ///
    /// /**
    ///  * This is redundant when typed.
    ///  * @type {string}
    ///  */
    /// ```
    CheckTagNames,
    correctness
);

#[derive(Debug, Default, Clone, Deserialize)]
struct CheckTagnamesConfig {
    #[serde(default, rename = "definedTags")]
    defined_tags: Vec<String>,
    #[serde(default, rename = "jsxTags")]
    jsx_tags: bool,
    #[serde(default)]
    typed: bool,
}

const VALID_BLOCK_TAGS: phf::Set<&'static str> = phf_set! {
"abstract",
"access",
"alias",
"async",
"augments",
"author",
"borrows",
"callback",
"class",
"classdesc",
"constant",
"constructs",
"copyright",
"default",
"deprecated",
"description",
"enum",
"event",
"example",
"exports",
"external",
"file",
"fires",
"function",
"generator",
"global",
"hideconstructor",
"ignore",
"implements",
"inheritdoc",
"inner",
"instance",
"interface",
"kind",
"lends",
"license",
"listens",
"member",
"memberof",
"memberof!",
"mixes",
"mixin",
"module",
"name",
"namespace",
"override",
"package",
"param",
"private",
"property",
"protected",
"public",
"readonly",
"requires",
"returns",
"see",
"since",
"static",
"summary",
"this",
"throws",
"todo",
"tutorial",
"type",
"typedef",
"variation",
"version",
"yields"
};

impl Rule for CheckTagNames {
    fn from_configuration(value: serde_json::Value) -> Self {
        value
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .map_or_else(Self::default, |value| Self(Box::new(value)))
    }

    fn run_once(&self, ctx: &LintContext) {
        let settings = &ctx.settings().jsdoc;
        let config = &self.0;
        println!("👻 {config:?}");
        println!("👻 {settings:?}");

        let user_preferred_tags = settings.list_preferred_tag_names();
        let is_valid_tag = |tag_name: &str| {
            if config.defined_tags.contains(&tag_name.to_string())
                || user_preferred_tags.contains(&tag_name.to_string())
            {
                return true;
            }

            if VALID_BLOCK_TAGS.contains(&settings.resolve_tag_name(tag_name)) {
                return true;
            }
            false
        };

        // TODO: typed, d.ts
        for jsdoc in ctx.semantic().jsdoc().iter_all() {
            for tag in jsdoc.tags() {
                let tag_name = tag.kind.parsed();
                println!("👻 {tag_name}");

                if let Some(message) = settings.is_blocked_tag_name(tag_name) {
                    ctx.diagnostic(CheckTagNamesDiagnostic(tag.kind.span, message));
                    continue;
                }
                println!("  => not blocked");

                if !is_valid_tag(tag_name) {
                    ctx.diagnostic(CheckTagNamesDiagnostic(
                        tag.kind.span,
                        format!("`@{tag_name}` is invalid tag name."),
                    ));
                    continue;
                }
                println!("  => is valid");
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
//         ("
// 			        /** @default 0 */
// 			        let a;
// 			      ", Some(serde_json::json!([
//         {
//           "typed": true,
//         },
//       ])), None),
// ("
// 			        /** @default 0 */
// 			        declare let a;
// 			      ", Some(serde_json::json!([
//         {
//           "typed": true,
//         },
//       ])), None),
// ("
// 			        /** @abstract */
// 			        let a;
// 			      ", Some(serde_json::json!([
//         {
//           "typed": true,
//         },
//       ])), None),
// ("
// 			        /** @abstract */
// 			        declare let a;
// 			      ", Some(serde_json::json!([
//         {
//           "typed": true,
//         },
//       ])), None),
// ("
// 			        /** @abstract */
// 			        { declare let a; }
// 			      ", Some(serde_json::json!([
//         {
//           "typed": true,
//         },
//       ])), None),
// ("
// 			        function test() {
// 			          /** @abstract */
// 			          declare let a;
// 			        }
// 			      ", Some(serde_json::json!([
//         {
//           "typed": true,
//         },
//       ])), None),
// ("
// 			        /** @template name */
// 			        let a;
// 			      ", Some(serde_json::json!([
//         {
//           "typed": true,
//         },
//       ])), None),
// ("
// 			        /** @param param - takes information */
// 			        function takesOne(param) {}
// 			      ", Some(serde_json::json!([
//         {
//           "typed": true,
//         },
//       ])), None),
("
			          /**
			           * @param foo
			           */
			          function quux (foo) {
			
			          }
			      ", None, None),
("
			          /**
			           * @memberof! foo
			           */
			          function quux (foo) {
			
			          }
			      ", None, None),
("
			          /**
			           * @arg foo
			           */
			          function quux (foo) {
			
			          }
			      ", None, Some(serde_json::json!({
        "jsdoc": {
          "tagNamePreference": {
            "param": "arg",
          },
        },
      }))),
("
			          /**
			           * @bar foo
			           */
			          function quux (foo) {
			
			          }
			      ", Some(serde_json::json!([
        {
          "definedTags": [
            "bar",
          ],
        },
      ])), None),
("
			          /**
			           * @baz @bar foo
			           */
			          function quux (foo) {
			
			          }
			      ", Some(serde_json::json!([
        {
          "definedTags": [
            "baz", "bar",
          ],
        },
      ])), None),
("
			          /**
			           * @baz @bar foo
			           */
			          function quux (foo) {
			
			          }
			      ", None, Some(serde_json::json!({
        "jsdoc": {
          "tagNamePreference": {
            "param": "baz",
            "returns": {
              "message": "Prefer `bar`",
              "replacement": "bar",
            },
            "todo": false,
          },
        },
      }))),
("
			      /**
			       * @returns
			       */
			      function quux (foo) {}
			      ", None, None),
("", None, None),
("
			          /**
			           *
			           */
			          function quux (foo) {
			
			          }
			      ", None, None),
("
			          /**
			           * @todo
			           */
			          function quux () {
			
			          }
			      ", None, None),
("
			          /**
			           * @extends Foo
			           */
			          function quux () {
			
			          }
			      ", None, Some(serde_json::json!({
        "jsdoc": {
          "tagNamePreference": {
            "augments": {
              "message": "@extends is to be used over @augments.",
              "replacement": "extends",
            },
          },
        },
      }))),
("
			          /**
			           * (Set tag name preference to itself to get aliases to
			           *   work along with main tag name.)
			           * @augments Bar
			           * @extends Foo
			           */
			          function quux () {
			          }
			      ", None, Some(serde_json::json!({
        "jsdoc": {
          "tagNamePreference": {
            "extends": "extends",
          },
        },
      }))),
("
			      /**
			       * Registers the `target` class as a transient dependency; each time the dependency is resolved a new instance will be created.
			       *
			       * @param target - The class / constructor function to register as transient.
			       *
			       * @example ```ts
			      @transient()
			      class Foo { }
			      ```
			       * @param Time for a new tag
			       */
			      export function transient<T>(target?: T): T {
			        // ...
			      }
			", None, None),
("
			        /** @jsx h */
			        /** @jsxFrag Fragment */
			        /** @jsxImportSource preact */
			        /** @jsxRuntime automatic */
			      ", Some(serde_json::json!([
        {
          "jsxTags": true,
        },
      ])), None),
("
			      /**
			       * @internal
			       */
			      ", None, Some(serde_json::json!({
        "jsdoc": {
        },
      }))),
("
			        interface WebTwain {
			          /**
			           * Converts the images specified by the indices to base64 synchronously.
			           * @function WebTwain#ConvertToBase64
			           * @returns {Base64Result}
			
			          ConvertToBase64(): Base64Result;
			          */
			
			          /**
			           * Converts the images specified by the indices to base64 asynchronously.
			           * @function WebTwain#ConvertToBase64
			           * @returns {boolean}
			           */
			          ConvertToBase64(): boolean;
			        }
			      ", None, None),
("
			        /**
			         * @overload
			         * @satisfies
			         */
			      ", None, Some(serde_json::json!({
        "jsdoc": {
        },
      }))),
// ("
// 			        /**
// 			         * @module
// 			         * A comment related to the module
// 			         */
// 			      ", Some(serde_json::json!([
//         {
//           "typed": true,
//         },
//       ])), None)
    ];

    let fail = vec![
        // (
        //     "/** @type {string} */let a;
        // 			      ",
        //     Some(serde_json::json!([
        //       {
        //         "typed": true,
        //       },
        //     ])),
        //     None,
        // ),
        // (
        //     "
        // 			        /**
        // 			         * Existing comment.
        // 			         *  @type {string}
        // 			         */
        // 			        let a;
        // 			      ",
        //     Some(serde_json::json!([
        //       {
        //         "typed": true,
        //       },
        //     ])),
        //     None,
        // ),
        // (
        //     "
        // 			        /** @abstract */
        // 			        let a;
        // 			      ",
        //     Some(serde_json::json!([
        //       {
        //         "typed": true,
        //       },
        //     ])),
        //     None,
        // ),
        // (
        //     "
        // 			        const a = {
        // 			          /** @abstract */
        // 			          b: true,
        // 			        };
        // 			      ",
        //     Some(serde_json::json!([
        //       {
        //         "typed": true,
        //       },
        //     ])),
        //     None,
        // ),
        // (
        //     "
        // 			        /** @template */
        // 			        let a;
        // 			      ",
        //     Some(serde_json::json!([
        //       {
        //         "typed": true,
        //       },
        //     ])),
        //     None,
        // ),
        // (
        //     "
        // 			        /**
        // 			         * Prior description.
        // 			         *
        // 			         * @template
        // 			         */
        // 			        let a;
        // 			      ",
        //     Some(serde_json::json!([
        //       {
        //         "typed": true,
        //       },
        //     ])),
        //     None,
        // ),
        (
            "
        			        /** @typoo {string} */
        			        let a;
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           * @Param
        			           */
        			          function quux () {
			
        			          }
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           * @foo
        			           */
        			          function quux () {
			
        			          }
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           * @arg foo
        			           */
        			          function quux (foo) {
			
        			          }
        			      ",
            None,
            None,
        ),
        // (
        //     "
        // 			          /**
        // 			           * @param foo
        // 			           */
        // 			          function quux (foo) {

        // 			          }
        // 			      ",
        //     None,
        //     Some(serde_json::json!({
        //       "jsdoc": {
        //         "tagNamePreference": {
        //           "param": "arg",
        //         },
        //       },
        //     })),
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @constructor foo
        // 			           */
        // 			          function quux (foo) {

        // 			          }
        // 			      ",
        //     None,
        //     Some(serde_json::json!({
        //       "jsdoc": {
        //         "tagNamePreference": {
        //           "constructor": "cons",
        //         },
        //       },
        //     })),
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @arg foo
        // 			           */
        // 			          function quux (foo) {

        // 			          }
        // 			      ",
        //     None,
        //     Some(serde_json::json!({
        //       "jsdoc": {
        //         "tagNamePreference": {
        //           "arg": "somethingDifferent",
        //         },
        //       },
        //     })),
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @param foo
        // 			           */
        // 			          function quux (foo) {

        // 			          }
        // 			      ",
        //     None,
        //     Some(serde_json::json!({
        //       "jsdoc": {
        //         "tagNamePreference": {
        //           "param": "parameter",
        //         },
        //       },
        //     })),
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @bar foo
        // 			           */
        // 			          function quux (foo) {

        // 			          }
        // 			      ",
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @baz @bar foo
        // 			           */
        // 			          function quux (foo) {

        // 			          }
        // 			      ",
        //     Some(serde_json::json!([
        //       {
        //         "definedTags": [
        //           "bar",
        //         ],
        //       },
        //     ])),
        //     None,
        // ),
        // (
        //     "
        // 			            /**
        // 			             * @bar
        // 			             * @baz
        // 			             */
        // 			            function quux (foo) {

        // 			            }
        // 			        ",
        //     Some(serde_json::json!([
        //       {
        //         "definedTags": [
        //           "bar",
        //         ],
        //       },
        //     ])),
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @todo
        // 			           */
        // 			          function quux () {

        // 			          }
        // 			      ",
        //     None,
        //     Some(serde_json::json!({
        //       "jsdoc": {
        //         "tagNamePreference": {
        //           "todo": false,
        //         },
        //       },
        //     })),
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @todo
        // 			           */
        // 			          function quux () {

        // 			          }
        // 			      ",
        //     None,
        //     Some(serde_json::json!({
        //       "jsdoc": {
        //         "tagNamePreference": {
        //           "todo": {
        //             "message": "Please resolve to-dos or add to the tracker",
        //           },
        //         },
        //       },
        //     })),
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @todo
        // 			           */
        // 			          function quux () {

        // 			          }
        // 			      ",
        //     None,
        //     Some(serde_json::json!({
        //       "jsdoc": {
        //         "tagNamePreference": {
        //           "todo": {
        //             "message": "Please use x-todo instead of todo",
        //             "replacement": "x-todo",
        //           },
        //         },
        //       },
        //     })),
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @todo
        // 			           */
        // 			          function quux () {

        // 			          }
        // 			      ",
        //     None,
        //     Some(serde_json::json!({
        //       "jsdoc": {
        //         "tagNamePreference": {
        //           "todo": 55,
        //         },
        //       },
        //     })),
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @property {object} a
        // 			           * @prop {boolean} b
        // 			           */
        // 			          function quux () {

        // 			          }
        // 			      ",
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @abc foo
        // 			           * @abcd bar
        // 			           */
        // 			          function quux () {

        // 			          }
        // 			      ",
        //     Some(serde_json::json!([
        //       {
        //         "definedTags": [
        //           "abcd",
        //         ],
        //       },
        //     ])),
        //     Some(serde_json::json!({
        //       "jsdoc": {
        //         "tagNamePreference": {
        //           "abc": "abcd",
        //         },
        //       },
        //     })),
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @abc
        // 			           * @abcd
        // 			           */
        // 			          function quux () {

        // 			          }
        // 			      ",
        //     None,
        //     Some(serde_json::json!({
        //       "jsdoc": {
        //         "tagNamePreference": {
        //           "abc": "abcd",
        //         },
        //       },
        //     })),
        // ),
        ("", None, None),
        // (
        //     "
        // 			        /** @jsx h */
        // 			        /** @jsxFrag Fragment */
        // 			        /** @jsxImportSource preact */
        // 			        /** @jsxRuntime automatic */
        // 			      ",
        //     None,
        //     None,
        // ),
        // (
        //     "
        // 			      /**
        // 			       * @constructor
        // 			       */
        // 			      function Test() {
        // 			        this.works = false;
        // 			      }
        // 			      ",
        //     None,
        //     Some(serde_json::json!({
        //       "jsdoc": {
        //         "tagNamePreference": {
        //           "returns": "return",
        //         },
        //       },
        //     })),
        // ),
        // (
        //     "
        // 			      /** @typedef {Object} MyObject
        // 			       * @property {string} id - my id
        // 			       */
        // 			      ",
        //     Some(serde_json::json!([
        //       {
        //         "typed": true,
        //       },
        //     ])),
        //     None,
        // ),
        // (
        //     "
        // 			      /**
        // 			       * @property {string} id - my id
        // 			       */
        // 			      ",
        //     Some(serde_json::json!([
        //       {
        //         "typed": true,
        //       },
        //     ])),
        //     None,
        // ),
        // (
        //     "
        // 			      /** @typedef {Object} MyObject */
        // 			      ",
        //     Some(serde_json::json!([
        //       {
        //         "typed": true,
        //       },
        //     ])),
        //     None,
        // ),
        // (
        //     "
        // 			      /** @typedef {Object} MyObject
        // 			       */
        // 			      ",
        //     Some(serde_json::json!([
        //       {
        //         "typed": true,
        //       },
        //     ])),
        //     None,
        // ),
        // (
        //     "
        // 			          /**
        // 			           * @todo
        // 			           */
        // 			          function quux () {

        // 			          }
        // 			      ",
        //     None,
        //     Some(serde_json::json!({
        //       "jsdoc": {
        //         "tagNamePreference": {
        //           "todo": {
        //             "message": "Please don\"t use todo",
        //           },
        //         },
        //       },
        //     })),
        // ),
    ];

    Tester::new(CheckTagNames::NAME, pass, fail).test_and_snapshot();
}

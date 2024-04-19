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
// Undocumented, but exists
// https://github.com/jsdoc/jsdoc/blob/a08ac18a11f5b0d93421d1e8ecf632468db2d045/packages/jsdoc-tag/lib/definitions/core.js#L374
"modifies",
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
"yields",
// TypeScript specific
"import",
"internal",
"overload",
"satisfies",
"template",
};

const JSX_TAGS: phf::Set<&'static str> = phf_set! {
"jsx",
"jsxFrag",
"jsxImportSource",
"jsxRuntime",
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

        let user_preferred_tags = settings.list_preferred_tag_names();

        // TODO: typed, d.ts
        // TODO: Bundle multiple diagnostics into one?
        // TODO: Test for all tags...?
        for jsdoc in ctx.semantic().jsdoc().iter_all() {
            for tag in jsdoc.tags() {
                let tag_name = tag.kind.parsed();

                // If explicitly blocked, report
                if let Some(reason) = settings.check_blocked_tag_name(tag_name) {
                    ctx.diagnostic(CheckTagNamesDiagnostic(tag.kind.span, reason));
                    continue;
                }

                // If valid JSX tags, skip
                if config.jsx_tags && JSX_TAGS.contains(tag_name) {
                    continue;
                }

                let is_valid = config.defined_tags.contains(&tag_name.to_string())
                    || VALID_BLOCK_TAGS.contains(tag_name);

                // If valid
                if is_valid {
                    // But preferred, report to use it
                    let preferred_name = settings.resolve_tag_name(tag_name);
                    if preferred_name != tag_name {
                        ctx.diagnostic(CheckTagNamesDiagnostic(
                            tag.kind.span,
                            format!("Replace tag `@{tag_name}` with `@{preferred_name}`."),
                        ));
                        continue;
                    }

                    continue;
                }

                // If invalid but user preferred, skip
                if user_preferred_tags.contains(&tag_name.to_string()) {
                    continue;
                }

                // Otherwise, report
                ctx.diagnostic(CheckTagNamesDiagnostic(
                    tag.kind.span,
                    format!("`@{tag_name}` is invalid tag name."),
                ));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
("
			          /**
			           * @param foo (pass: valid name)
			           */
			          function quux (foo) {
			
			          }
			      ", None, None),
("
			          /**
			           * @memberof! foo (pass: valid name)
			           */
			          function quux (foo) {
			
			          }
			      ", None, None),
("
			          /**
			           * @arg foo (pass: invalid name but user preferred)
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
			           * @bar foo (pass: invalid name but defined)
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
			           * @baz @bar foo (pass: invalid names but defined)
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
			           * @baz @bar foo (pass: invalid names but user preferred)
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
			       * @returns (pass: valid name)
			       */
			      function quux (foo) {}
			      ", None, None),
("", None, None),
("
			          /**
			           * (pass: no tag)
			           */
			          function quux (foo) {
			
			          }
			      ", None, None),
("
			          /**
			           * @todo (pass: valid name)
			           */
			          function quux () {
			
			          }
			      ", None, None),
("
			          /**
			           * @extends Foo (pass: invalid name but user preferred)
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
			           * @extends Foo (pass: invalid name but user preferred)
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
			       * @param Time for a new tag (pass: valid names)
			       */
			      export function transient<T>(target?: T): T {
			        // ...
			      }
			", None, None),
("
			        /** @jsx h */
			        /** @jsxFrag Fragment */
			        /** @jsxImportSource preact */
			        /** @jsxRuntime automatic (pass: valid jsx names)*/
			      ", Some(serde_json::json!([
        {
          "jsxTags": true,
        },
      ])), None),
("
			      /**
			       * @internal (pass: valid name)
			       */
			      ", None, Some(serde_json::json!({
        "jsdoc": {
        },
      }))),
("
			        /**
			         * @overload
			         * @satisfies (pass: valid names)
			         */
			      ", None, Some(serde_json::json!({
        "jsdoc": {
        },
      }))),
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
        (
            "
        			        /** @typoo {string} (fail: invalid name) */
        			        let a;
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           * @Param (fail: invalid name)
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
        			           * @foo (fail: invalid name)
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
        			           * @arg foo (fail: invalid name, default aliased)
        			           */
        			          function quux (foo) {
			
        			          }
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           * @param foo (fail: valid name but user preferred)
        			           */
        			          function quux (foo) {

        			          }
        			      ",
            None,
            Some(serde_json::json!({
              "jsdoc": {
                "tagNamePreference": {
                  "param": "arg",
                },
              },
            })),
        ),
        (
            "
        			          /**
        			           * @constructor foo (fail: invalid name and user preferred)
        			           */
        			          function quux (foo) {

        			          }
        			      ",
            None,
            Some(serde_json::json!({
              "jsdoc": {
                "tagNamePreference": {
                  "constructor": "cons",
                },
              },
            })),
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
            Some(serde_json::json!({
              "jsdoc": {
                "tagNamePreference": {
                  "arg": "somethingDifferent",
                },
              },
            })),
        ),
        (
            "
        			          /**
        			           * @param foo
        			           */
        			          function quux (foo) {

        			          }
        			      ",
            None,
            Some(serde_json::json!({
              "jsdoc": {
                "tagNamePreference": {
                  "param": "parameter",
                },
              },
            })),
        ),
        (
            "
        			          /**
        			           * @bar foo
        			           */
        			          function quux (foo) {

        			          }
        			      ",
            None,
            None,
        ),
        (
            "
        			          /**
        			           * @baz @bar foo
        			           */
        			          function quux (foo) {

        			          }
        			      ",
            Some(serde_json::json!([
              {
                "definedTags": [
                  "bar",
                ],
              },
            ])),
            None,
        ),
        (
            "
        			            /**
        			             * @bar
        			             * @baz
        			             */
        			            function quux (foo) {

        			            }
        			        ",
            Some(serde_json::json!([
              {
                "definedTags": [
                  "bar",
                ],
              },
            ])),
            None,
        ),
        (
            "
        			          /**
        			           * @todo
        			           */
        			          function quux () {

        			          }
        			      ",
            None,
            Some(serde_json::json!({
              "jsdoc": {
                "tagNamePreference": {
                  "todo": false,
                },
              },
            })),
        ),
        (
            "
        			          /**
        			           * @todo
        			           */
        			          function quux () {

        			          }
        			      ",
            None,
            Some(serde_json::json!({
              "jsdoc": {
                "tagNamePreference": {
                  "todo": {
                    "message": "Please resolve to-dos or add to the tracker",
                  },
                },
              },
            })),
        ),
        (
            "
        			          /**
        			           * @todo
        			           */
        			          function quux () {

        			          }
        			      ",
            None,
            Some(serde_json::json!({
              "jsdoc": {
                "tagNamePreference": {
                  "todo": {
                    "message": "Please use x-todo instead of todo",
                    "replacement": "x-todo",
                  },
                },
              },
            })),
        ),
        (
            "
        			          /**
        			           * @todo
        			           */
        			          function quux () {

        			          }
        			      ",
            None,
            Some(serde_json::json!({
              "jsdoc": {
                "tagNamePreference": {
                  "todo": "55",
                },
              },
            })),
        ),
        (
            "
        			          /**
        			           * @property {object} a
        			           * @prop {boolean} b
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
        			           * @abc foo
        			           * @abcd bar
        			           */
        			          function quux () {

        			          }
        			      ",
            Some(serde_json::json!([
              {
                "definedTags": [
                  "abcd",
                ],
              },
            ])),
            Some(serde_json::json!({
              "jsdoc": {
                "tagNamePreference": {
                  "abc": "abcd",
                },
              },
            })),
        ),
        (
            "
        			          /**
        			           * @abc
        			           * @abcd
        			           */
        			          function quux () {

        			          }
        			      ",
            None,
            Some(serde_json::json!({
              "jsdoc": {
                "tagNamePreference": {
                  "abc": "abcd",
                },
              },
            })),
        ),
        (
            "
        			        /** @jsx h */
        			        /** @jsxFrag Fragment */
        			        /** @jsxImportSource preact */
        			        /** @jsxRuntime automatic */
        			      ",
            None,
            None,
        ),
        (
            "
        			      /**
        			       * @constructor
        			       */
        			      function Test() {
        			        this.works = false;
        			      }
        			      ",
            None,
            Some(serde_json::json!({
              "jsdoc": {
                "tagNamePreference": {
                  "returns": "return",
                },
              },
            })),
        ),
        (
            "
        			          /**
        			           * @todo
        			           */
        			          function quux () {

        			          }
        			      ",
            None,
            Some(serde_json::json!({
              "jsdoc": {
                "tagNamePreference": {
                  "todo": {
                    "message": "Please don\"t use todo",
                  },
                },
              },
            })),
        ),
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
    ];

    Tester::new(CheckTagNames::NAME, pass, fail).test_and_snapshot();
}

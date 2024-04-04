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
#[error("eslint-plugin-jsdoc(empty-tags): Expects the void tags to be empty of any content.")]
#[diagnostic(severity(warning), help("`@{1}` tag should be empty."))]
struct EmptyTagsDiagnostic(#[label] Span, String);

#[derive(Debug, Default, Clone)]
pub struct EmptyTags(Box<EmptyTagsConfig>);

declare_oxc_lint!(
    /// ### What it does
    /// Expects the following tags to be empty of any content:
    /// - `@abstract`
    /// - `@async`
    /// - `@generator`
    /// - `@global`
    /// - `@hideconstructor`
    /// - `@ignore`
    /// - `@inner`
    /// - `@instance`
    /// - `@override`
    /// - `@readonly`
    /// - `@inheritDoc`
    /// - `@internal`
    /// - `@overload`
    /// - `@package`
    /// - `@private`
    /// - `@protected`
    /// - `@public`
    /// - `@static`
    ///
    /// ### Why is this bad?
    /// The void tags should be empty.
    ///
    /// ### Example
    /// ```javascript
    /// // Passing
    /// /** @async */
    ///
    /// /** @private */
    ///
    /// // Failing
    /// /** @async foo */
    ///
    /// /** @private bar */
    /// ```
    EmptyTags,
    restriction
);

const EMPTY_TAGS: phf::Set<&'static str> = phf_set! {
    "abstract",
    "async",
    "generator",
    "global",
    "hideconstructor",
    "ignore",
    "inner",
    "instance",
    "override",
    "readonly",
    "inheritDoc",
    "internal",
    "overload",
    "package",
    "private",
    "protected",
    "public",
    "static",
};

#[derive(Debug, Default, Clone, Deserialize)]
struct EmptyTagsConfig {
    #[serde(default)]
    tags: Vec<String>,
}

impl Rule for EmptyTags {
    fn from_configuration(value: serde_json::Value) -> Self {
        value
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .map_or_else(Self::default, |value| Self(Box::new(value)))
    }

    fn run_once(&self, ctx: &LintContext) {
        let is_empty_tag_kind = |kind: &str| {
            if EMPTY_TAGS.contains(kind) {
                return true;
            }
            if !self.0.tags.is_empty() && self.0.tags.contains(&kind.to_string()) {
                return true;
            }
            false
        };

        for jsdoc in ctx.semantic().jsdoc().iter_all() {
            for (span, tag) in jsdoc.tags() {
                if !is_empty_tag_kind(tag.kind) {
                    continue;
                }
                if tag.comment().is_empty() {
                    continue;
                }

                ctx.diagnostic(EmptyTagsDiagnostic(*span, tag.kind.to_string()));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "
			          /**
			           * @abstract
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
			           *
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
			           * @param aName
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
			           * @abstract
			           * @inheritdoc
			           * @async
			           */
			          function quux () {
			
			          }
			      ",
            None,
            None,
        ),
        // (
        //     "
        // 			      /**
        // 			       * @private {someType}
        // 			       */
        // 			      function quux () {

        // 			      }
        // 			      ",
        //     None,
        //     Some(serde_json::json!({
        //       "jsdoc": {
        //         "mode": "closure",
        //       },
        //     })),
        // ),
        (
            "
			      /**
			       * @private
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
			       * @internal
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
			       * Create an array.
			       *
			       * @private
			       *
			       * @param {string[]} [elem] - Elements to make an array of.
			       * @param {boolean} [clone] - Optionally clone nodes.
			       * @returns {string[]} The array of nodes.
			       */
			      function quux () {}
			      ",
            None,
            None,
        ),
    ];

    let fail = vec![
        (
            "
			          /**
			           * @abstract extra text
			           */
			          function quux () {
			
			          }
			      ",
            None,
            None,
        ),
        // (
        //     "
        // 			          /**
        // 			           * @interface extra text
        // 			           */
        // 			      ",
        //     None,
        //     Some(serde_json::json!({
        //       "jsdoc": {
        //         "mode": "closure",
        //       },
        //     })),
        // ),
        (
            "
			      class Test {
			          /**
			           * @abstract extra text
			           */
			          quux () {
			
			          }
			      }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @abstract extra text
			           * @inheritdoc
			           * @async out of place
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
			           * @event anEvent
			           */
			          function quux () {
			
			          }
			      ",
            Some(serde_json::json!([
              {
                "tags": [
                  "event",
                ],
              },
            ])),
            None,
        ),
        (
            "
			      /**
			       * @private {someType}
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
			       * @internal {someType}
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
			       * @private {someType}
			       */
			      function quux () {
			
			      }
			      ",
            None,
            Some(serde_json::json!({
              "jsdoc": {
                "ignorePrivate": true,
              },
            })),
        ),
    ];

    Tester::new(EmptyTags::NAME, pass, fail).test_and_snapshot();
}

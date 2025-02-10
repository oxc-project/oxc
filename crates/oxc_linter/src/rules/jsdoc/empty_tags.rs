use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use phf::phf_set;
use serde::Deserialize;

use crate::{context::LintContext, rule::Rule, utils::should_ignore_as_private};

fn empty_tags_diagnostic(span: Span, x1: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expects the void tags to be empty of any content.")
        .with_help(format!("`@{x1}` tag should not have body."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct EmptyTags(Box<EmptyTagsConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
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
    ///
    /// The void tags should be empty.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// /** @async foo */
    ///
    /// /** @private bar */
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// /** @async */
    ///
    /// /** @private */
    /// ```
    EmptyTags,
    jsdoc,
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
        let settings = &ctx.settings().jsdoc;

        let is_empty_tag_kind = |tag_name: &str| {
            if EMPTY_TAGS.contains(tag_name) {
                return true;
            }
            if !self.0.tags.is_empty() && self.0.tags.contains(&tag_name.to_string()) {
                return true;
            }
            false
        };

        for jsdoc in ctx
            .semantic()
            .jsdoc()
            .iter_all()
            .filter(|jsdoc| !should_ignore_as_private(jsdoc, settings))
        {
            for tag in jsdoc.tags() {
                let tag_name = tag.kind.parsed();

                if !is_empty_tag_kind(tag_name) {
                    continue;
                }

                let comment = tag.comment();
                if comment.parsed().is_empty() {
                    continue;
                }

                ctx.diagnostic(empty_tags_diagnostic(comment.span_trimmed_first_line(), tag_name));
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
			       * @private foo
			       * bar
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
                   * foo
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

    Tester::new(EmptyTags::NAME, EmptyTags::PLUGIN, pass, fail).test_and_snapshot();
}

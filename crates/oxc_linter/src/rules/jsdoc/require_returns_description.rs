use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    ast_util::is_function_node,
    context::LintContext,
    rule::Rule,
    utils::{get_function_nearest_jsdoc_node, should_ignore_as_internal, should_ignore_as_private},
    AstNode,
};

fn missing_description_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing JSDoc `@returns` description.")
        .with_help("Add description comment to `@returns` tag.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireReturnsDescription;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires that the `@returns` tag has a description value.
    /// The error will not be reported if the return value is `void `or `undefined` or if it is `Promise<void>` or `Promise<undefined>`.
    ///
    /// ### Why is this bad?
    ///
    /// A `@returns` tag should have a description value.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// /** @returns */
    /// function quux (foo) {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// /** @returns Foo. */
    /// function quux (foo) {}
    /// ```
    RequireReturnsDescription,
    jsdoc,
    pedantic,
);

impl Rule for RequireReturnsDescription {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if !is_function_node(node) {
            return;
        }

        // If no JSDoc is found, skip
        let Some(jsdocs) = get_function_nearest_jsdoc_node(node, ctx)
            .and_then(|node| ctx.jsdoc().get_all_by_node(node))
        else {
            return;
        };

        let settings = &ctx.settings().jsdoc;
        let resolved_returns_tag_name = settings.resolve_tag_name("returns");
        for jsdoc in jsdocs
            .iter()
            .filter(|jsdoc| !should_ignore_as_internal(jsdoc, settings))
            .filter(|jsdoc| !should_ignore_as_private(jsdoc, settings))
        {
            for tag in jsdoc.tags() {
                if tag.kind.parsed() != resolved_returns_tag_name {
                    continue;
                }

                let (type_part, comment_part) = tag.type_comment();

                // If returns type is marked as nothing, skip
                if let Some(type_part) = type_part {
                    if matches!(
                        type_part.parsed(),
                        "void" | "undefined" | "Promise<void>" | "Promise<undefined>"
                    ) {
                        continue;
                    }
                }

                // If description exists, skip
                if !comment_part.parsed().is_empty() {
                    continue;
                }

                ctx.diagnostic(missing_description_diagnostic(tag.kind.span));
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
			           * @returns Foo.
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
			           * @returns Foo.
			           */
			          function quux () {
			
			          }
			      ",
            Some(serde_json::json!([
              {
                "contexts": [
                  "any",
                ],
              },
            ])),
            None,
        ),
        (
            "
			          /**
			           * @returns {undefined}
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
			           * @returns {void}
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
			           * @returns {Promise<void>}
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
			           * @returns {Promise<undefined>}
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
			           * @function
			           * @returns
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @callback
			           * @returns
			           */
			      ",
            None,
            None,
        ),
    ];

    let fail = vec![
        (
            "
			          /**
			           * @returns
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
			           * @returns {string}
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
			           * @return
			           */
			          function quux (foo) {
			
			          }
			      ",
            None,
            Some(serde_json::json!({ "settings": {
        "jsdoc": {
          "tagNamePreference": {
            "returns": "return",
          },
        },
      } })),
        ),
    ];

    Tester::new(RequireReturnsDescription::NAME, RequireReturnsDescription::PLUGIN, pass, fail)
        .test_and_snapshot();
}

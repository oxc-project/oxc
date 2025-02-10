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

fn missing_type_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing JSDoc `@returns` type.")
        .with_help("Add {type} to `@returns` tag.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireReturnsType;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires that `@returns` tag has a type value (in curly brackets).
    ///
    /// ### Why is this bad?
    ///
    /// A `@returns` tag should have a type value.
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
    /// /** @returns {string} */
    /// function quux (foo) {}
    /// ```
    RequireReturnsType,
    jsdoc,
    pedantic,
);

impl Rule for RequireReturnsType {
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

                // If type exists, skip
                if let (Some(_), _) = tag.type_comment() {
                    continue;
                };

                ctx.diagnostic(missing_type_diagnostic(tag.kind.span));
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
			           * @returns {number}
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
			           * @returns Foo.
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @callback
			           * @returns Foo.
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
			           * @return Foo.
			           */
			          function quux () {
			
			          }
			      ",
            None,
            Some(serde_json::json!({ "settings": { "jsdoc": {
                    "tagNamePreference": { "returns": "return", },
                }, } })),
        ),
    ];

    Tester::new(RequireReturnsType::NAME, RequireReturnsType::PLUGIN, pass, fail)
        .test_and_snapshot();
}

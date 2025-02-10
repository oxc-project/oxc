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

fn missing_name_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing JSDoc `@param` name.")
        .with_help("Add name to `@param` tag.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireParamName;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires that all `@param` tags have names.
    ///
    /// ### Why is this bad?
    ///
    /// The name of a param should be documented.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// /** @param {SomeType} */
    /// function quux (foo) {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// /** @param {SomeType} foo */
    /// function quux (foo) {}
    /// ```
    RequireParamName,
    jsdoc,
    pedantic,
);

impl Rule for RequireParamName {
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
        let resolved_param_tag_name = settings.resolve_tag_name("param");

        for jsdoc in jsdocs
            .iter()
            .filter(|jsdoc| !should_ignore_as_internal(jsdoc, settings))
            .filter(|jsdoc| !should_ignore_as_private(jsdoc, settings))
        {
            for tag in jsdoc.tags() {
                if tag.kind.parsed() != resolved_param_tag_name {
                    continue;
                }

                let (_, name_part, _) = tag.type_name_comment();

                // If name exists, skip
                if name_part.is_some() {
                    continue;
                }

                ctx.diagnostic(missing_name_diagnostic(tag.kind.span));
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
			           * @param foo
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
			           * @param {string} foo
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
			           * @function
			           * @param
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @callback
			           * @param
			           */
			      ",
            None,
            None,
        ),
        (
            "
			      /**
			       * @param {Function} [processor=data => data] A function to run
			       */
			      function processData(processor) {
			        return processor(data)
			      }
			      ",
            None,
            None,
        ),
        (
            "
			      /** Example with multi-line param type.
			      *
			      * @param {function(
			      *   number
			      * )} cb Callback.
			      */
			      function example(cb) {
			        cb(42);
			      }
			      ",
            None,
            None,
        ),
    ];

    let fail = vec![
        (
            "
			          /**
			           * @param
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
			           * @param {string}
			           */
			          function quux (foo) {
			
			          }
			      ",
            None,
            None,
        ),
    ];

    Tester::new(RequireParamName::NAME, RequireParamName::PLUGIN, pass, fail).test_and_snapshot();
}

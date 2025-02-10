use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        collect_params, get_function_nearest_jsdoc_node, should_ignore_as_internal,
        should_ignore_as_private, ParamKind,
    },
    AstNode,
};

fn missing_type_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing JSDoc `@param` description.")
        .with_help("Add description to `@param` tag.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireParamDescription;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires that each `@param` tag has a description value.
    ///
    /// ### Why is this bad?
    ///
    /// The description of a param should be documented.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// /** @param foo */
    /// function quux (foo) {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// /** @param foo Foo. */
    /// function quux (foo) {}
    /// ```
    RequireParamDescription,
    jsdoc,
    pedantic,
);

impl Rule for RequireParamDescription {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // Collected targets from `FormalParameters`
        let params_to_check = match node.kind() {
            AstKind::Function(func) if !func.is_typescript_syntax() => collect_params(&func.params),
            AstKind::ArrowFunctionExpression(arrow_func) => collect_params(&arrow_func.params),
            // If not a function, skip
            _ => return,
        };

        // If no JSDoc is found, skip
        let Some(jsdocs) = get_function_nearest_jsdoc_node(node, ctx)
            .and_then(|node| ctx.jsdoc().get_all_by_node(node))
        else {
            return;
        };

        let settings = &ctx.settings().jsdoc;
        let resolved_param_tag_name = settings.resolve_tag_name("param");

        let mut root_count = 0;
        for jsdoc in jsdocs
            .iter()
            .filter(|jsdoc| !should_ignore_as_internal(jsdoc, settings))
            .filter(|jsdoc| !should_ignore_as_private(jsdoc, settings))
        {
            for tag in jsdoc.tags() {
                if tag.kind.parsed() != resolved_param_tag_name {
                    continue;
                }

                let (_, name_part, comment_part) = tag.type_name_comment();

                if name_part.is_some_and(|name_part| !name_part.parsed().contains('.')) {
                    root_count += 1;
                }
                if settings.exempt_destructured_roots_from_checks {
                    // -1 for count to idx conversion
                    if let Some(ParamKind::Nested(_)) = params_to_check.get(root_count - 1) {
                        continue;
                    }
                }

                // If description exists, skip
                if !comment_part.parsed().is_empty() {
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
			           *
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
			           * @param foo Foo.
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
			           * @param foo
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @callback
			           * @param foo
			           */
			      ",
            None,
            None,
        ),
        (
            "
			      /**
			       * Checks if the XML document sort of equals another XML document.
			       * @param {Object} obj The other object.
			       * @param {{includeWhiteSpace: (boolean|undefined),
			       *    ignoreElementOrder: (boolean|undefined)}} [options] The options.
			       * @return {expect.Assertion} The assertion.
			       */
			      expect.Assertion.prototype.xmleql = function (obj, options) {
			      }
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @param {number} foo Foo description
			           * @param {object} root
			           * @param {boolean} baz Baz description
			           */
			          function quux (foo, {bar}, baz) {
			
			          }
			      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "jsdoc": {          "exemptDestructuredRootsFromChecks": true,        },      } }),
            ),
        ),
        (
            "
			          /**
			           * @param {number} foo Foo description
			           * @param {object} root
			           * @param {object} root.bar
			           */
			          function quux (foo, {bar: {baz}}) {
			
			          }
			      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "jsdoc": {          "exemptDestructuredRootsFromChecks": true,        },      } }),
            ),
        ),
    ];

    let fail = vec![
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
			           * @arg foo
			           */
			          function quux (foo) {
			
			          }
			      ",
            None,
            Some(
                serde_json::json!({ "settings": {        "jsdoc": {          "tagNamePreference": {            "param": "arg",          },        },      } }),
            ),
        ),
        (
            "
			          /**
			           * @param {number} foo Foo description
			           * @param {object} root
			           * @param {boolean} baz Baz description
			           */
			          function quux (foo, {bar}, baz) {
			
			          }
			      ",
            Some(
                serde_json::json!([        {          "setDefaultDestructuredRootDescription": true,        },      ]),
            ),
            None,
        ),
        (
            "
			          /**
			           * @param {number} foo Foo description
			           * @param {object} root
			           * @param {boolean} baz Baz description
			           */
			          function quux (foo, {bar}, baz) {
			
			          }
			      ",
            Some(
                serde_json::json!([        {          "defaultDestructuredRootDescription": "Root description",          "setDefaultDestructuredRootDescription": true,        },      ]),
            ),
            None,
        ),
        (
            "
			          /**
			           * @param {number} foo Foo description
			           * @param {object} root
			           * @param {boolean} baz Baz description
			           */
			          function quux (foo, {bar}, baz) {
			
			          }
			      ",
            Some(
                serde_json::json!([        {          "setDefaultDestructuredRootDescription": false,        },      ]),
            ),
            None,
        ),
    ];

    Tester::new(RequireParamDescription::NAME, RequireParamDescription::PLUGIN, pass, fail)
        .test_and_snapshot();
}

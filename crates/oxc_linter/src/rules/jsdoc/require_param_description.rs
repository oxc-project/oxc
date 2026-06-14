use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_str::CompactStr;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::{
        ParamKind, collect_params, get_function_nearest_jsdoc_node, should_ignore_as_internal,
        should_ignore_as_private,
    },
};

fn missing_type_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing JSDoc `@param` description.")
        .with_help("Add description to `@param` tag.")
        .with_label(span)
}

fn missing_root_description_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing root description for @param.")
        .with_help("Add root description to `@param`.")
        .with_label(span)
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct RequireParamDescriptionConfig {
    /// The description string to set by default for destructured roots. Defaults to "The root object".
    default_destructured_root_description: CompactStr,
    /// Whether to set a default destructured root description.
    /// For example, you may wish to avoid manually having to set the description for a @param corresponding to a destructured root object as it should always be the same type of object.
    /// Uses `defaultDestructuredRootDescription` for the description string. Defaults to `false`.
    set_default_destructured_root_description: bool,
}

impl Default for RequireParamDescriptionConfig {
    fn default() -> Self {
        Self {
            default_destructured_root_description: CompactStr::from("The root object"),
            set_default_destructured_root_description: false,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct RequireParamDescription(Box<RequireParamDescriptionConfig>);

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
    fix,
    config = RequireParamDescriptionConfig,
    version = "0.4.4",
    short_description = "Requires that each `@param` tag has a description value.",
);

impl Rule for RequireParamDescription {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<RequireParamDescriptionConfig>>(value)
            .map(|config| Self(Box::new(config.into_inner())))
    }

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
            .and_then(|node| ctx.jsdoc().get_all_by_node(ctx.nodes(), node))
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
                }

                let is_destructured_root =
                    matches!(params_to_check.get(root_count - 1), Some(ParamKind::Nested(_)));

                if self.0.set_default_destructured_root_description
                    && is_destructured_root
                    && let Some(name_part) = name_part
                {
                    ctx.diagnostic_with_fix(
                        missing_root_description_diagnostic(tag.kind.span),
                        |fixer| {
                            fixer.insert_text_after_range(
                                name_part.span,
                                format!(" {}", self.0.default_destructured_root_description),
                            )
                        },
                    );
                } else {
                    ctx.diagnostic(missing_type_diagnostic(tag.kind.span));
                }
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
                serde_json::json!([        {          "setDefaultDestructuredRootDescription": false,        },      ]),
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
            Some(serde_json::json!([{ "setDefaultDestructuredRootDescription": true }])),
            None,
        ),
    ];

    let fix = vec![
        (
            "/** @param {object} root */\nfunction quux ({bar}) {}",
            "/** @param {object} root The root object */\nfunction quux ({bar}) {}",
            Some(serde_json::json!([{ "setDefaultDestructuredRootDescription": true }])),
        ),
        (
            "/** @param {object} root */\nfunction quux ({bar}) {}",
            "/** @param {object} root Custom root description */\nfunction quux ({bar}) {}",
            Some(serde_json::json!([{
                "defaultDestructuredRootDescription": "Custom root description",
                "setDefaultDestructuredRootDescription": true
            }])),
        ),
        (
            "/** @param {number} foo\n * @param {object} root */\nfunction quux (foo, {bar}) {}",
            "/** @param {number} foo\n * @param {object} root The root object */\nfunction quux (foo, {bar}) {}",
            Some(serde_json::json!([{ "setDefaultDestructuredRootDescription": true }])),
        ),
    ];

    Tester::new(RequireParamDescription::NAME, RequireParamDescription::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

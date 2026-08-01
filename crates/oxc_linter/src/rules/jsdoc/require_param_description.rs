use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_str::CompactStr;

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
    pending,
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
            AstKind::Function(func) if !func.is_typescript_syntax() => {
                collect_params(&func.params, self.0.set_default_destructured_root_description)
            }
            AstKind::ArrowFunctionExpression(arrow_func) => {
                collect_params(&arrow_func.params, self.0.set_default_destructured_root_description)
            }
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

        let mut root_names = Vec::new();
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

                let (current_param, is_current_root_tag) =
                    name_part.map_or((None, false), |name_part| {
                        let name = name_part.parsed();
                        let root_name = name
                            .split_once('.')
                            .map_or(name, |(root_name, _)| root_name)
                            .trim_end_matches("[]");
                        let is_current_root_tag = name == root_name;
                        let root_index = root_names
                            .iter()
                            .position(|&name| name == root_name)
                            .unwrap_or_else(|| {
                                root_names.push(root_name);
                                root_names.len() - 1
                            });
                        (params_to_check.get(root_index), is_current_root_tag)
                    });

                if settings.exempt_destructured_roots_from_checks
                    && matches!(current_param, Some(ParamKind::Nested(_)))
                {
                    continue;
                }

                // If description exists, skip
                if !comment_part.parsed().is_empty() {
                    continue;
                }

                let is_destructured_root =
                    is_current_root_tag && matches!(current_param, Some(ParamKind::Nested(_)));

                if self.0.set_default_destructured_root_description
                    && is_destructured_root
                    && name_part.is_some()
                {
                    ctx.diagnostic(missing_root_description_diagnostic(tag.kind.span));
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
        ("/** @param input.userId */\nfunction quux (input) {}", None, None),
        (
            "/** @param input.userId User ID.\n * @param options */\nfunction quux (input, {flag}) {}",
            Some(serde_json::json!([{ "setDefaultDestructuredRootDescription": true }])),
            None,
        ),
        (
            "/** @param employees Employees.\n * @param employees[].name Employee name.\n * @param options */\nfunction quux (employees, {flag}) {}",
            Some(serde_json::json!([{ "setDefaultDestructuredRootDescription": true }])),
            None,
        ),
        (
            "/** @param root.foo */\nfunction quux ({foo}) {}",
            Some(serde_json::json!([{ "setDefaultDestructuredRootDescription": true }])),
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

    Tester::new(RequireParamDescription::NAME, RequireParamDescription::PLUGIN, pass, fail)
        .test_and_snapshot();
}

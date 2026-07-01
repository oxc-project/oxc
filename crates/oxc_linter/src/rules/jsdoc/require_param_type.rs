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
    OxcDiagnostic::warn("Missing JSDoc `@param` type.")
        .with_help("Add {type} to `@param` tag.")
        .with_label(span)
}

fn missing_root_type_diagnostic(span: Span, default_type: &CompactStr) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing root type for @param.")
        .with_help(format!("Add {{{default_type}}} to `@param` tag."))
        .with_label(span)
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct RequireParamTypeConfig {
    /// The type string to set by default for destructured roots. Defaults to "object".
    default_destructured_root_type: CompactStr,
    /// Whether to set a default destructured root type. For example, you may wish to avoid manually having to set the type for a `@param` corresponding to a destructured root object as it is always going to be an object. Uses `defaultDestructuredRootType` for the type string. Defaults to `false`.
    set_default_destructured_root_type: bool,
}

impl Default for RequireParamTypeConfig {
    fn default() -> Self {
        Self {
            default_destructured_root_type: CompactStr::from("object"),
            set_default_destructured_root_type: false,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct RequireParamType(Box<RequireParamTypeConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires that each `@param` tag has a type value (within curly brackets).
    ///
    /// ### Why is this bad?
    ///
    /// The type of a parameter should be documented.
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
    /// /** @param {SomeType} foo */
    /// function quux (foo) {}
    /// ```
    RequireParamType,
    jsdoc,
    pedantic,
    fix,
    config = RequireParamTypeConfig,
    version = "0.4.4",
    short_description = "Requires that each `@param` tag has a type value (within curly brackets).",
);

impl Rule for RequireParamType {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<RequireParamTypeConfig>>(value)
            .map(|config| Self(Box::new(config.into_inner())))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // Collected targets from `FormalParameters`
        let params_to_check = match node.kind() {
            AstKind::Function(func) if !func.is_typescript_syntax() => {
                collect_params(&func.params, self.0.set_default_destructured_root_type)
            }
            AstKind::ArrowFunctionExpression(arrow_func) => {
                collect_params(&arrow_func.params, self.0.set_default_destructured_root_type)
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

                let (type_part, name_part, _) = tag.type_name_comment();

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

                // If type exists, skip
                if type_part.is_some() {
                    continue;
                }

                let is_destructured_root =
                    is_current_root_tag && matches!(current_param, Some(ParamKind::Nested(_)));

                if self.0.set_default_destructured_root_type
                    && is_destructured_root
                    && let Some(name_part) = name_part
                {
                    ctx.diagnostic_with_fix(
                        missing_root_type_diagnostic(
                            tag.kind.span,
                            &self.0.default_destructured_root_type,
                        ),
                        |fixer| {
                            fixer.insert_text_before_range(
                                name_part.span,
                                format!("{{{}}} ", self.0.default_destructured_root_type),
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
			           * @param {number} foo
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
			           * @param {number} foo
			           * @param root
			           * @param {boolean} baz
			           */
			          function quux (foo, {bar}, baz) {

			          }
			      ",
            None,
            Some(
                serde_json::json!({ "settings": { "jsdoc": { "exemptDestructuredRootsFromChecks": true } } }),
            ),
        ),
        (
            "
			          /**
			           * @param {number} foo
			           * @param root
			           * @param root.bar
			           */
			          function quux (foo, {bar: {baz}}) {

			          }
			      ",
            None,
            Some(
                serde_json::json!({ "settings": { "jsdoc": { "exemptDestructuredRootsFromChecks": true } } }),
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
			       * @param {a xxx
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
            Some(
                serde_json::json!({ "settings": { "jsdoc": { "tagNamePreference": { "param": "arg" } } } }),
            ),
        ),
        (
            "/** @param input.userId - The ID of the user to update. */\nfunction quux (input) {}",
            None,
            None,
        ),
        (
            "/** @param input.userId - The ID of the user to update.\n * @param options */\nfunction quux (input, {flag}) {}",
            Some(serde_json::json!([{ "setDefaultDestructuredRootType": true }])),
            None,
        ),
        (
            "/** @param {object[]} employees\n * @param {string} employees[].name\n * @param options */\nfunction quux (employees, {flag}) {}",
            Some(serde_json::json!([{ "setDefaultDestructuredRootType": true }])),
            None,
        ),
        (
            "/** @param root.foo */\nfunction quux ({foo}) {}",
            Some(serde_json::json!([{ "setDefaultDestructuredRootType": true }])),
            None,
        ),
        (
            "
			          /**
			           * @param {number} foo
			           * @param root
			           * @param {boolean} baz
			           */
			          function quux (foo, {bar}, baz) {

			          }
			      ",
            Some(serde_json::json!([ { "setDefaultDestructuredRootType": true } ])),
            None,
        ),
        (
            "
			          /**
			           * @param {number} foo
			           * @param root
			           * @param {boolean} baz
			           */
			          function quux (foo, {bar}, baz) {

			          }
			      ",
            Some(serde_json::json!([ { "setDefaultDestructuredRootType": false } ])),
            None,
        ),
    ];

    let fix = vec![
        (
            "/** @param root */\nfunction quux ({bar}) {}",
            "/** @param {object} root */\nfunction quux ({bar}) {}",
            Some(serde_json::json!([{ "setDefaultDestructuredRootType": true }])),
        ),
        (
            "/** @param root */\nfunction quux ({bar}) {}",
            "/** @param {CustomType} root */\nfunction quux ({bar}) {}",
            Some(serde_json::json!([{
                "defaultDestructuredRootType": "CustomType",
                "setDefaultDestructuredRootType": true
            }])),
        ),
        (
            "/** @param {number} foo\n * @param root */\nfunction quux (foo, {bar}) {}",
            "/** @param {number} foo\n * @param {object} root */\nfunction quux (foo, {bar}) {}",
            Some(serde_json::json!([{ "setDefaultDestructuredRootType": true }])),
        ),
    ];

    Tester::new(RequireParamType::NAME, RequireParamType::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

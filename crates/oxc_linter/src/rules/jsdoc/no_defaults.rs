use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::{get_function_nearest_jsdoc_node, should_ignore_as_internal, should_ignore_as_private},
};

fn no_defaults_diagnostic(span: Span, x1: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn("Defaults are not permitted.").with_help(x1.to_string()).with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoDefaults(Box<NoDefaultsConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule reports defaults being used on the relevant portion of `@param` or `@default`.
    /// It also optionally reports the presence of the square-bracketed optional arguments at all.
    ///
    /// ### Why is this bad?
    ///
    /// The rule is intended to prevent the indication of defaults on tags
    /// where this would be redundant with ES2015 default parameters.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// /** @param {number} [foo="7"] */
    /// function quux (foo) {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// /** @param {number} foo */
    /// function quux (foo) {}
    ///
    /// /** @param foo */
    /// function quux (foo) {}
    /// ```
    NoDefaults,
    jsdoc,
    correctness,
    config = NoDefaultsConfig,
);

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
struct NoDefaultsConfig {
    /// If true, report the presence of optional param names (square brackets) on `@param` tags.
    no_optional_param_names: bool,
}

impl Rule for NoDefaults {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<NoDefaults>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::Function(f) if f.is_function_declaration() || f.is_expression() => {}
            AstKind::ArrowFunctionExpression(_) => {}
            _ => return,
        }

        let Some(jsdocs) = get_function_nearest_jsdoc_node(node, ctx)
            .and_then(|node| ctx.jsdoc().get_all_by_node(ctx.nodes(), node))
        else {
            return;
        };

        let settings = &ctx.settings().jsdoc;
        let resolved_param_tag_name = settings.resolve_tag_name("param");
        let config = &self.0;

        for jsdoc in jsdocs
            .iter()
            .filter(|jsdoc| !should_ignore_as_internal(jsdoc, settings))
            .filter(|jsdoc| !should_ignore_as_private(jsdoc, settings))
        {
            for tag in jsdoc.tags() {
                let tag_name = tag.kind.parsed();

                if tag_name != resolved_param_tag_name {
                    continue;
                }
                let (_, Some(name_part), _) = tag.type_name_comment() else {
                    continue;
                };

                if !name_part.optional {
                    continue;
                }

                match (config.no_optional_param_names, name_part.default) {
                    (true, _) => ctx.diagnostic(no_defaults_diagnostic(
                        name_part.span,
                        &format!("Optional param names are not permitted on `@{tag_name}` tag."),
                    )),
                    (false, true) => ctx.diagnostic(no_defaults_diagnostic(
                        name_part.span,
                        &format!("Defaults are not permitted on `@{tag_name}` tag."),
                    )),
                    (false, false) => {}
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
			           * @param {number} foo
			           * @param {number} [bar]
			           */
			          function quux (foo) {

			          }
			      ",
            None,
            None,
        ),
        // (
        //     "
        // 			          /**
        // 			           * @param foo
        // 			           */
        // 			      ",
        //     Some(serde_json::json!([
        //       {
        //         "contexts": [
        //           "any",
        //         ],
        //       },
        //     ])),
        //     None,
        // ),
        (
            "
			          /**
			           * @function
			           * @param {number} foo
			           */
			      ",
            None,
            None,
        ),
        (
            "
			          /**
			           * @callback
			           * @param {number} foo
			           */
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
            Some(serde_json::json!([
              {
                "noOptionalParamNames": true,
              },
            ])),
            None,
        ),
        // (
        //     "
        // 			          /**
        // 			           * @default
        // 			           */
        // 			          const a = {};
        // 			      ",
        //     Some(serde_json::json!([
        //       {
        //         "contexts": [
        //           "any",
        //         ],
        //       },
        //     ])),
        //     None,
        // ),
    ];

    let fail = vec![
        (
            r#"
			          /**
			           * @param {number} [foo="7"]
			           */
			          function quux (foo) {

			          }
			      "#,
            None,
            None,
        ),
        (
            "
			          /**
			           * @param {number} foo
			           * @param {number} [bar = 1]
			           */
			          const quux = (foo, bar = 1) => {

			          }
			      ",
            None,
            None,
        ),
        (
            r#"
			      class Test {
			          /**
			           * @param {number} [foo="7"]
			           */
			          quux (foo) {

			          }
			      }
			      "#,
            None,
            None,
        ),
        (
            r#"
			          /**
			           * @param {number} [foo="7"]
			           */
			          function quux (foo) {

			          }
			      "#,
            Some(serde_json::json!([
              {
                "noOptionalParamNames": true,
              },
            ])),
            None,
        ),
        (
            "
			          /**
			           * @param {number} [foo]
			           */
			          function quux (foo) {

			          }
			      ",
            Some(serde_json::json!([
              {
                "noOptionalParamNames": true,
              },
            ])),
            None,
        ),
        (
            r#"
			          /**
			           * @arg {number} [foo="7"]
			           */
			          function quux (foo) {

			          }
			      "#,
            None,
            Some(serde_json::json!({ "settings": {
        "jsdoc": {
          "tagNamePreference": {
            "param": "arg",
          },
        },
      } })),
        ),
        //   (
        //       r#"
        // 			          /**
        // 			           * @param {number} [foo="7"]
        // 			           */
        // 			          function quux (foo) {

        // 			          }
        // 			      "#,
        //       Some(serde_json::json!([
        //         {
        //           "contexts": [
        //             "any",
        //           ],
        //         },
        //       ])),
        //       None,
        //   ),
        //   (
        //       r#"
        // 			          /**
        // 			           * @function
        // 			           * @param {number} [foo="7"]
        // 			           */
        // 			      "#,
        //       Some(serde_json::json!([
        //         {
        //           "contexts": [
        //             "any",
        //           ],
        //         },
        //       ])),
        //       None,
        //   ),
        //   (
        //       r#"
        // 			          /**
        // 			           * @callback
        // 			           * @param {number} [foo="7"]
        // 			           */
        // 			      "#,
        //       Some(serde_json::json!([
        //         {
        //           "contexts": [
        //             "any",
        //           ],
        //         },
        //       ])),
        //       None,
        //   ),
        //   (
        //       "
        // 			          /**
        // 			           * @default {}
        // 			           */
        // 			          const a = {};
        // 			      ",
        //       Some(serde_json::json!([
        //         {
        //           "contexts": [
        //             "any",
        //           ],
        //         },
        //       ])),
        //       None,
        //   ),
        //   (
        //       "
        // 			          /**
        // 			           * @defaultvalue {}
        // 			           */
        // 			          const a = {};
        // 			      ",
        //       Some(serde_json::json!([
        //         {
        //           "contexts": [
        //             "any",
        //           ],
        //         },
        //       ])),
        //       Some(serde_json::json!({ "settings": {
        //   "jsdoc": {
        //     "tagNamePreference": {
        //       "default": "defaultvalue",
        //     },
        //   },
        // } })),
        //   ),
    ];

    Tester::new(NoDefaults::NAME, NoDefaults::PLUGIN, pass, fail).test_and_snapshot();
}

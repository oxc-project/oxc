use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde::Deserialize;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jsdoc(no-defaults): Defaults are not permitted.")]
#[diagnostic(severity(warning), help("{1}"))]
struct NoDefaultsDiagnostic(#[label] pub Span, String);

#[derive(Debug, Default, Clone)]
pub struct NoDefaults(Box<NoDefaultsConfig>);

declare_oxc_lint!(
    /// ### What it does
    /// This rule reports defaults being used on the relevant portion of `@param` or `@default`.
    /// It also optionally reports the presence of the square-bracketed optional arguments at all.
    ///
    /// ### Why is this bad?
    /// The rule is intended to prevent the indication of defaults on tags
    /// where this would be redundant with ES6 default parameters.
    ///
    /// ### Example
    /// ```javascript
    /// // Passing
    /// /** @param {number} foo */
    /// function quux (foo) {}
    /// /** @param foo */
    /// function quux (foo) {}
    ///
    /// // Failing
    /// /** @param {number} [foo="7"] */
    /// function quux (foo) {}
    /// ```
    NoDefaults,
    correctness
);

#[derive(Debug, Default, Clone, Deserialize)]
struct NoDefaultsConfig {
    #[serde(default, rename = "noOptionalParamNames")]
    no_optional_param_names: bool,
}

/// Get the definition root node of a function.
/// JSDoc often appears on the parent node of a function.
///
/// ```js
/// /** FunctionDeclaration */
/// function foo() {}
///
/// /** VariableDeclaration > VariableDeclarator > FunctionExpression */
/// const bar = function() {}
///
/// /** VariableDeclaration > VariableDeclarator > ArrowFunctionExpression */
/// const baz = () => {}
///
/// /** MethodDefinition > FunctionExpression */
/// class X { quux() {} }
///
/// /** PropertyDefinition > ArrowFunctionExpression */
/// class X { quux = () => {} }
/// ```
fn get_function_definition_node<'a, 'b>(
    node: &'b AstNode<'a>,
    ctx: &'b LintContext<'a>,
) -> Option<&'b AstNode<'a>> {
    match node.kind() {
        AstKind::Function(f) if f.is_function_declaration() => return Some(node),
        AstKind::Function(f) if f.is_expression() => {}
        AstKind::ArrowFunctionExpression(_) => {}
        _ => return None,
    };

    let mut current_node = node;
    while let Some(parent_node) = ctx.nodes().parent_node(current_node.id()) {
        match parent_node.kind() {
            AstKind::VariableDeclarator(_) | AstKind::ParenthesizedExpression(_) => {
                current_node = parent_node;
            }
            AstKind::MethodDefinition(_)
            | AstKind::PropertyDefinition(_)
            | AstKind::VariableDeclaration(_) => return Some(parent_node),
            _ => return None,
        }
    }

    None
}

impl Rule for NoDefaults {
    fn from_configuration(value: serde_json::Value) -> Self {
        value
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .map_or_else(Self::default, |value| Self(Box::new(value)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let Some(jsdocs) = get_function_definition_node(node, ctx)
            .and_then(|node| ctx.jsdoc().get_all_by_node(node))
        else {
            return;
        };

        let settings = &ctx.settings().jsdoc;
        let resolved_param_tag_name = settings.resolve_tag_name("param");
        let config = &self.0;

        for jsdoc in jsdocs {
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
                    (true, _) => ctx.diagnostic(NoDefaultsDiagnostic(
                        name_part.span,
                        format!("Optional param names are not permitted on `@{tag_name}` tag."),
                    )),
                    (false, true) => ctx.diagnostic(NoDefaultsDiagnostic(
                        name_part.span,
                        format!("Defaults are not permitted on `@{tag_name}` tag."),
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

    Tester::new(NoDefaults::NAME, pass, fail).test_and_snapshot();
}

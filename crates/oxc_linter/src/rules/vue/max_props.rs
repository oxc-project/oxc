use oxc_ast::{
    AstKind,
    ast::{ExportDefaultDeclarationKind, Expression, ObjectPropertyKind, TSType, TSTypeName},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde_json::Value;

use crate::{
    AstNode, ast_util::get_declaration_from_reference_id, context::LintContext,
    frameworks::FrameworkOptions, rule::Rule,
};

fn max_props_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct MaxProps {
    max_props: usize,
}

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    MaxProps,
    vue,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
    pending,
);

impl Rule for MaxProps {
    fn from_configuration(value: Value) -> Self {
        Self {
            max_props: value
                .get(0)
                .and_then(|v| v.get("maxProps"))
                .and_then(Value::as_u64)
                .map_or(1, |n| n as usize),
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if ctx.frameworks_options() == FrameworkOptions::VueSetup {
            self.run_on_setup(node, ctx);
        } else {
            self.run_on_options(node, ctx);
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_path().extension().is_some_and(|ext| ext == "vue")
    }
}

impl MaxProps {
    fn run_on_setup<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(ident) = call_expr.callee.get_identifier_reference() else {
            return;
        };
        if ident.name != "defineProps" {
            return;
        }
        if let Some(first_arg) = call_expr.arguments.first() {
            match first_arg.as_expression() {
                Some(Expression::ObjectExpression(obj_expr))
                    if obj_expr.properties.len() > self.max_props =>
                {
                    ctx.diagnostic(max_props_diagnostic(call_expr.span));
                }
                Some(Expression::ArrayExpression(arr_expr))
                    if arr_expr.elements.len() > self.max_props =>
                {
                    ctx.diagnostic(max_props_diagnostic(call_expr.span));
                }
                _ => {}
            }
        } else {
            let Some(type_arguments) = call_expr.type_arguments.as_ref() else {
                return;
            };
            let Some(first_type_argument) = type_arguments.params.first() else {
                return;
            };

            if is_type_argument_length_exceeded(ctx, first_type_argument, self.max_props) {
                ctx.diagnostic(max_props_diagnostic(call_expr.span));
            }
        }
    }

    fn run_on_options<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ExportDefaultDeclaration(export_default_decl) = node.kind() else {
            return;
        };
        let ExportDefaultDeclarationKind::ObjectExpression(obj_expr) =
            &export_default_decl.declaration
        else {
            return;
        };

        // 优化后的props属性查找逻辑
        let Some(props_obj_expr) = obj_expr.properties.iter()
            // 查找key为"props"的ObjectProperty
            .find_map(|item| {
                if let ObjectPropertyKind::ObjectProperty(obj_prop) = item
                    && let Some(key) = obj_prop.key.static_name()
                    && key == "props"
                    // 直接获取并检查props的值是否为ObjectExpression
                    && let Expression::ObjectExpression(props_expr) = obj_prop.value.get_inner_expression()
                {
                    Some(props_expr)
                } else {
                    None
                }
            })
        else {
            return;
        };

        // 添加props数量检查逻辑（与run_on_setup保持一致）
        if props_obj_expr.properties.len() > self.max_props {
            ctx.diagnostic(max_props_diagnostic(props_obj_expr.span));
        }
    }
}

fn is_type_argument_length_exceeded(
    ctx: &LintContext,
    type_argument: &TSType,
    max_props: usize,
) -> bool {
    match type_argument {
        TSType::TSTypeReference(type_ref) => {
            let TSTypeName::IdentifierReference(ident_ref) = &type_ref.type_name else {
                return false;
            };
            // we need to find the reference of type_ref
            let reference = ctx.scoping().get_reference(ident_ref.reference_id());
            if !reference.is_type() {
                return false;
            }
            let Some(reference_node) =
                get_declaration_from_reference_id(ident_ref.reference_id(), ctx.semantic())
            else {
                return false;
            };
            let AstKind::TSInterfaceDeclaration(interface_decl) = reference_node.kind() else {
                return false;
            };
            let interface_body = &interface_decl.body;
            interface_body.body.len() > max_props
        }
        TSType::TSTypeLiteral(type_literal) => type_literal.members.len() > max_props,
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    // let pass = vec![];

    // let fail = vec![
    //     (
    //         r#"
    // 		      <script setup lang="ts">
    // 		      defineProps<{ prop1: string, prop2: string, prop3: string }>();
    // 		      </script>
    // 		      "#,
    //         Some(serde_json::json!([{ "maxProps": 2 }])),
    //         None,
    //         Some(PathBuf::from("test.vue")),
    //     ),
    // ];

    let pass = vec![
        (
            "
			      <script setup>
			      defineProps({ prop1: '', prop2: '' })
			      </script>
			      ",
            Some(serde_json::json!([{ "maxProps": 5 }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script setup>
			      defineProps(['prop1', 'prop2'])
			      </script>
			      ",
            Some(serde_json::json!([{ "maxProps": 5 }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      export default {
			        props: {
			          prop1: String,
			          prop2: String
			        }
			      }
			      </script>
			      ",
            Some(serde_json::json!([{ "maxProps": 5 }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      export default {
			        props: {
			          prop1: String,
			          prop2: String,
			          prop3: String,
			          prop4: String,
			          prop5: String
			        }
			      }
			      </script>
			      ",
            Some(serde_json::json!([{ "maxProps": 5 }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      export default {
			        props: {}
			      }
			      </script>
			      ",
            Some(serde_json::json!([{ "maxProps": 5 }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script setup>
			      defineProps({})
			      </script>
			      ",
            Some(serde_json::json!([{ "maxProps": 5 }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      </script>
			      ",
            Some(serde_json::json!([{ "maxProps": 5 }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
			      <script setup lang="ts">
			      defineProps<{ prop1: string, prop2: string }>();
			      </script>
			      "#,
            Some(serde_json::json!([{ "maxProps": 5 }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parser": require("vue-eslint-parser"),        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
        (
            r#"
			      <script setup lang="ts">
			      defineProps<{prop1: string, prop2: string} | {prop1: number}>()
			      </script>
			      "#,
            Some(serde_json::json!([{ "maxProps": 2 }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      }
    ];

    let fail = vec![
        (
            "
			      <script setup>
			      defineProps({ prop1: '', prop2: '' })
			      </script>
			      ",
            Some(serde_json::json!([{ "maxProps": 1 }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script>
			      export default {
			        props: {
			          prop1: String,
			          prop2: String
			        }
			      }
			      </script>
			      ",
            Some(serde_json::json!([{ "maxProps": 1 }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
			      <script setup lang="ts">
			      defineProps<{ prop1: string, prop2: string, prop3: string }>();
			      </script>
			      "#,
            Some(serde_json::json!([{ "maxProps": 2 }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
			      <script setup lang="ts">
			      defineProps<{prop1: string, prop2: string} | {prop1: number, prop3: string}>()
			      </script>
			      "#,
            Some(serde_json::json!([{ "maxProps": 2 }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
        (
            r#"
			      <script setup lang="ts">
			      defineProps<{
			        prop1: string
			      } & {
			        prop2?: true;
			        prop3?: never;
			      } | {
			        prop2?: false;
			        prop3?: boolean;
			      }>()
			      </script>
			      "#,
            Some(serde_json::json!([{ "maxProps": 2 }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      }
    ];

    Tester::new(MaxProps::NAME, MaxProps::PLUGIN, pass, fail).test_and_snapshot();
}

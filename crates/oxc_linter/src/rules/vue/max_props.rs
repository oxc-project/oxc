use oxc_ast::{
    AstKind,
    ast::{ExportDefaultDeclarationKind, Expression, TSSignature, TSType},
};

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_str::CompactStr;
use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    AstNode,
    context::LintContext,
    frameworks::FrameworkOptions,
    rule::{DefaultRuleConfig, Rule},
    utils::{find_property, for_each_define_props_type_signature},
};

fn max_props_diagnostic(span: Span, cur: u32, limit: u32) -> OxcDiagnostic {
    let msg = format!("This component has too many props ({cur}). Maximum allowed is {limit}.");
    OxcDiagnostic::warn(msg)
        .with_help(
            "Consider refactoring the component to reduce the number of props that are needed.",
        )
        .with_label(span)
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct MaxProps {
    /// The maximum number of props allowed in a Vue SFC.
    max_props: u32,
}

impl Default for MaxProps {
    fn default() -> Self {
        Self { max_props: 1 }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce a maximum number of props defined for a given Vue component.
    ///
    /// ### Why is this bad?
    ///
    /// A large number of props on a component can indicate that it is trying
    /// to do too much and may be difficult to maintain or understand.
    ///
    /// By limiting the number of props, developers are encouraged to avoid
    /// overly complex components and instead create smaller, more focused
    /// components that are easier to reason about.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with the default `{ "maxProps": 1 }` option:
    /// ```js
    /// <script setup>
    /// defineProps({
    ///   prop1: String,
    ///   prop2: String,
    /// })
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule with the default `{ "maxProps": 1 }` option:
    /// ```js
    /// <script setup>
    /// defineProps({
    ///   prop1: String,
    /// })
    /// </script>
    /// ```
    MaxProps,
    vue,
    restriction,
    config = MaxProps,
    version = "1.19.0",
);

impl Rule for MaxProps {
    fn from_configuration(value: Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if ctx.frameworks_options() == FrameworkOptions::VueSetup {
            self.run_on_setup(node, ctx);
        } else {
            self.run_on_options(node, ctx);
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
    }
}

impl MaxProps {
    #[expect(clippy::cast_possible_truncation)] // the length of properties/arrays can't be over u32::MAX, because the source code is already limited by u32::MAX.
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
                    if obj_expr.properties.len() as u32 > self.max_props =>
                {
                    ctx.diagnostic(max_props_diagnostic(
                        call_expr.span,
                        obj_expr.properties.len() as u32,
                        self.max_props,
                    ));
                }
                Some(Expression::ArrayExpression(arr_expr))
                    if arr_expr.elements.len() as u32 > self.max_props =>
                {
                    ctx.diagnostic(max_props_diagnostic(
                        call_expr.span,
                        arr_expr.elements.len() as u32,
                        self.max_props,
                    ));
                }
                _ => {}
            }
        } else {
            // e.g defineProps<A>();
            let Some(type_arguments) = call_expr.type_arguments.as_ref() else {
                return;
            };
            let Some(first_type_argument) = type_arguments.params.first() else {
                return;
            };

            let all_key_len = get_type_argument_keys(ctx, first_type_argument).len() as u32;
            if all_key_len > self.max_props {
                ctx.diagnostic(max_props_diagnostic(call_expr.span, all_key_len, self.max_props));
            }
        }
    }

    #[expect(clippy::cast_possible_truncation)] // the length of properties can't be over u32::MAX, because the source code is already limited by u32::MAX.
    fn run_on_options<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ExportDefaultDeclaration(export_default_decl) = node.kind() else {
            return;
        };
        let ExportDefaultDeclarationKind::ObjectExpression(obj_expr) =
            &export_default_decl.declaration
        else {
            return;
        };

        let Some(props_prop) = find_property(obj_expr, "props") else {
            return;
        };
        let Expression::ObjectExpression(props_obj_expr) = props_prop.value.get_inner_expression()
        else {
            return;
        };

        if props_obj_expr.properties.len() as u32 > self.max_props {
            ctx.diagnostic(max_props_diagnostic(
                props_obj_expr.span,
                props_obj_expr.properties.len() as u32,
                self.max_props,
            ));
        }
    }
}

fn get_type_argument_keys<'a>(
    ctx: &LintContext<'a>,
    type_argument: &TSType<'a>,
) -> FxHashSet<CompactStr> {
    let mut keys = FxHashSet::default();
    for_each_define_props_type_signature(type_argument, ctx, &mut |signature| {
        let name = match signature {
            TSSignature::TSPropertySignature(prop) => prop.key.static_name(),
            TSSignature::TSMethodSignature(method) => method.key.static_name(),
            _ => return,
        };
        if let Some(name) = name {
            keys.insert(CompactStr::from(name));
        }
    });
    keys
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

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
        ), // { "parser": require("vue-eslint-parser"), "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") } },
        (
            r#"
			      <script setup lang="ts">
			      defineProps<{prop1: string, prop2: string} | {prop1: number}>()
			      </script>
			      "#,
            Some(serde_json::json!([{ "maxProps": 2 }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") } }
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
        ), // { "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") } }
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
        ), // { "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") } }
        (
            r#"
			      <script setup lang="ts">
			      type Props = { prop1: string, prop2: string, prop3: string };
			      defineProps<Props>();
			      </script>
			      "#,
            Some(serde_json::json!([{ "maxProps": 2 }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // { "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") } }
    ];

    Tester::new(MaxProps::NAME, MaxProps::PLUGIN, pass, fail).test_and_snapshot();
}

use oxc_allocator::Vec;
use oxc_ast::{
    AstKind,
    ast::{
        ExportDefaultDeclarationKind, Expression, ObjectPropertyKind, TSSignature, TSType,
        TSTypeName, TSTypeReference,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use rustc_hash::FxHashSet;
use serde_json::Value;

use crate::{
    AstNode, ast_util::get_declaration_from_reference_id, context::LintContext,
    frameworks::FrameworkOptions, rule::Rule,
};

fn max_props_diagnostic(span: Span, cur: usize, limit: usize) -> OxcDiagnostic {
    let msg = format!("Component has too many props ({cur}). Maximum allowed is {limit}.");
    OxcDiagnostic::warn(msg)
        .with_help("Consider refactoring the component by reducing the number of props.")
        .with_label(span)
}

#[derive(Debug, Clone)]
pub struct MaxProps {
    max_props: usize,
}

impl Default for MaxProps {
    fn default() -> Self {
        Self { max_props: 1 }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce maximum number of props in Vue component.
    ///
    /// ### Why is this bad?
    ///
    /// This rule enforces a maximum number of props in a Vue SFC,
    /// in order to aid in maintainability and reduce complexity.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with the default `{ maxProps: 1 }` option:
    /// ```js
    /// <script setup>
    /// defineProps({
    ///   prop1: String,
    ///   prop2: String,
    /// })
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule with the default `{ maxProps: 1 }` option:
    /// ```js
    /// <script setup>
    /// defineProps({
    ///   prop1: String,
    /// })
    /// </script>
    /// ```
    /// ### Options
    ///
    /// This rule takes an object, where you can specify the maximum number of props allowed in a Vue SFC.
    ///
    /// ```json
    /// {
    ///     "vue/max-props": [
    ///         "error",
    ///         {
    ///             "maxProps": 1
    ///         }
    ///     ]
    /// }
    /// ```
    MaxProps,
    vue,
    restriction,
);

impl Rule for MaxProps {
    #[expect(clippy::cast_possible_truncation)]
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
        ctx.file_extension().is_some_and(|ext| ext == "vue")
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
                    ctx.diagnostic(max_props_diagnostic(
                        call_expr.span,
                        obj_expr.properties.len(),
                        self.max_props,
                    ));
                }
                Some(Expression::ArrayExpression(arr_expr))
                    if arr_expr.elements.len() > self.max_props =>
                {
                    ctx.diagnostic(max_props_diagnostic(
                        call_expr.span,
                        arr_expr.elements.len(),
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

            let all_key_len = get_type_argument_keys(ctx, first_type_argument).len();
            if all_key_len > self.max_props {
                ctx.diagnostic(max_props_diagnostic(call_expr.span, all_key_len, self.max_props));
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

        let Some(props_obj_expr) = obj_expr.properties.iter().find_map(|item| {
            if let ObjectPropertyKind::ObjectProperty(obj_prop) = item
                && let Some(key) = obj_prop.key.static_name()
                && key == "props"
                && let Expression::ObjectExpression(props_expr) =
                    obj_prop.value.get_inner_expression()
            {
                Some(props_expr)
            } else {
                None
            }
        }) else {
            return;
        };

        if props_obj_expr.properties.len() > self.max_props {
            ctx.diagnostic(max_props_diagnostic(
                props_obj_expr.span,
                props_obj_expr.properties.len(),
                self.max_props,
            ));
        }
    }
}

fn get_type_argument_keys(ctx: &LintContext, type_argument: &TSType) -> FxHashSet<CompactStr> {
    match type_argument {
        // e.g defineProps<A | B>();
        TSType::TSUnionType(union_type) => {
            union_type.types.iter().fold(FxHashSet::default(), |mut all_keys, args| {
                let type_arg_keys = get_type_argument_keys(ctx, args);
                all_keys.extend(type_arg_keys);
                all_keys
            })
        }
        // e.g defineProps<A & B>();
        TSType::TSIntersectionType(intersection_type) => {
            intersection_type.types.iter().fold(FxHashSet::default(), |mut all_keys, args| {
                let type_arg_keys = get_type_argument_keys(ctx, args);
                all_keys.extend(type_arg_keys);
                all_keys
            })
        }
        // e.g defineProps<A>();
        TSType::TSTypeReference(type_ref) => collect_key_from_type_reference(ctx, type_ref),
        // e.g defineProps<{ a: string }>();
        TSType::TSTypeLiteral(type_literal) => collect_keys_from_signatures(&type_literal.members),
        _ => FxHashSet::default(),
    }
}

fn collect_key_from_type_reference(
    ctx: &LintContext,
    type_ref: &TSTypeReference,
) -> FxHashSet<CompactStr> {
    let TSTypeName::IdentifierReference(ident_ref) = &type_ref.type_name else {
        return FxHashSet::default();
    };
    // we need to find the reference of type_ref
    let reference = ctx.scoping().get_reference(ident_ref.reference_id());
    if !reference.is_type() {
        return FxHashSet::default();
    }
    let Some(reference_node) =
        get_declaration_from_reference_id(ident_ref.reference_id(), ctx.semantic())
    else {
        return FxHashSet::default();
    };
    let AstKind::TSInterfaceDeclaration(interface_decl) = reference_node.kind() else {
        return FxHashSet::default();
    };
    let interface_body = &interface_decl.body;
    collect_keys_from_signatures(&interface_body.body)
}

fn collect_keys_from_signatures(signatures: &Vec<TSSignature<'_>>) -> FxHashSet<CompactStr> {
    signatures
        .iter()
        .filter_map(|member| match member {
            TSSignature::TSPropertySignature(prop_signature) => {
                prop_signature.key.static_name().map(CompactStr::from)
            }
            TSSignature::TSMethodSignature(method_signature) => {
                method_signature.key.static_name().map(CompactStr::from)
            }
            _ => None,
        })
        .collect()
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

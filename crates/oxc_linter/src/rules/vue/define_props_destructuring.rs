use oxc_ast::{AstKind, ast::BindingPatternKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    frameworks::FrameworkOptions,
    rule::{DefaultRuleConfig, Rule},
};

fn prefer_destructuring_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer destructuring from `defineProps` directly.").with_label(span)
}

fn avoid_destructuring_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Avoid destructuring from `defineProps`.").with_label(span)
}

fn avoid_with_defaults_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Avoid using `withDefaults` with destructuring.").with_label(span)
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum Destructure {
    /// Requires destructuring when using `defineProps` and warns against using `withDefaults` with destructuring
    #[default]
    Always,
    /// Requires using a variable to store props and prohibits destructuring
    Never,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct DefinePropsDestructuring {
    /// Require or prohibit destructuring.
    destructure: Destructure,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces a consistent style for handling Vue 3 Composition API props,
    /// allowing you to choose between requiring destructuring or prohibiting it.
    ///
    /// ### Why is this bad?
    ///
    /// By default, the rule requires you to use destructuring syntax when using `defineProps`
    /// instead of storing props in a variable and warns against combining `withDefaults` with destructuring.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script setup lang="ts">
    ///   const props = defineProps(['foo']);
    ///   const propsWithDefaults = withDefaults(defineProps(['foo']), { foo: 'default' });
    ///   const { baz } = withDefaults(defineProps(['baz']), { baz: 'default' });
    ///   const props = defineProps<{ foo?: string }>()
    ///   const propsWithDefaults = withDefaults(defineProps<{ foo?: string }>(), { foo: 'default' })
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// <script setup lang="ts">
    ///   const { foo } = defineProps(['foo'])
    ///   const { bar = 'default' } = defineProps(['bar'])
    ///   const { foo } = defineProps<{ foo?: string }>()
    ///   const { bar = 'default' } = defineProps<{ bar?: string }>()
    /// </script>
    /// ```
    DefinePropsDestructuring,
    vue,
    style,
    config = DefinePropsDestructuring,
);

impl Rule for DefinePropsDestructuring {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<DefinePropsDestructuring>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };

        // only check call Expression which is `defineProps`
        if call_expr
            .callee
            .get_identifier_reference()
            .is_none_or(|reference| reference.name != "defineProps")
        {
            return;
        }

        if call_expr.arguments.is_empty() && call_expr.type_arguments.is_none() {
            return;
        }

        let parent = &ctx.nodes().parent_node(node.id());
        let with_defaults_span = get_parent_with_defaults_call_expression_span(parent);
        let has_destructuring = is_parent_destructuring_variable(parent, ctx);

        if self.destructure == Destructure::Never {
            if has_destructuring {
                ctx.diagnostic(avoid_destructuring_diagnostic(call_expr.span));
            }
        } else if !has_destructuring {
            ctx.diagnostic(prefer_destructuring_diagnostic(call_expr.span));
        } else if let Some(span) = with_defaults_span {
            ctx.diagnostic(avoid_with_defaults_diagnostic(span));
        }
    }

    fn should_run(&self, ctx: &ContextHost<'_>) -> bool {
        ctx.frameworks_options() == FrameworkOptions::VueSetup
    }
}

fn get_parent_with_defaults_call_expression_span(parent: &AstNode<'_>) -> Option<Span> {
    let AstKind::CallExpression(call_expr) = parent.kind() else { return None };

    call_expr.callee.get_identifier_reference().and_then(|reference| {
        if reference.name == "withDefaults" { Some(reference.span) } else { None }
    })
}

fn is_parent_destructuring_variable(parent: &AstNode<'_>, ctx: &LintContext<'_>) -> bool {
    let Some(declarator) = (match parent.kind() {
        AstKind::VariableDeclarator(var_decl) => Some(var_decl),
        _ => ctx.nodes().ancestor_kinds(parent.id()).find_map(|kind| {
            if let AstKind::VariableDeclarator(var_decl) = kind { Some(var_decl) } else { None }
        }),
    }) else {
        return false;
    };

    matches!(declarator.id.kind, BindingPatternKind::ObjectPattern(_))
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        (
            "
                  <script setup>
                  const props = defineProps()
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script setup>
                  const { foo = 'default' } = defineProps(['foo'])
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
                  <script setup lang="ts">
                  const { foo = 'default' } = defineProps<{ foo?: string }>()
                  </script>
                  "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") }      },
        (
            "
                  <script setup>
                  const props = defineProps(['foo'])
                  </script>
                  ",
            Some(serde_json::json!([{ "destructure": "never" }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script setup>
                  const props = withDefaults(defineProps(['foo']), { foo: 'default' })
                  </script>
                  ",
            Some(serde_json::json!([{ "destructure": "never" }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
                  <script setup lang="ts">
                  const props = defineProps<{ foo?: string }>()
                  </script>
                  "#,
            Some(serde_json::json!([{ "destructure": "never" }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") }      }
    ];

    let fail = vec![
        (
            "
                  <script setup>
                  const props = defineProps(['foo'])
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script setup>
                  const props = withDefaults(defineProps(['foo']), { foo: 'default' })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script setup>
                  const { foo } = withDefaults(defineProps(['foo']), { foo: 'default' })
                  </script>
                  ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
                  <script setup lang="ts">
                  const props = withDefaults(defineProps<{ foo?: string }>(), { foo: 'default' })
                  </script>
                  "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") }      },
        (
            r#"
                  <script setup lang="ts">
                  const { foo } = withDefaults(defineProps<{ foo?: string }>(), { foo: 'default' })
                  </script>
                  "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") }      },
        (
            "
                  <script setup>
                  const { foo } = defineProps(['foo'])
                  </script>
                  ",
            Some(serde_json::json!([{ "destructure": "never" }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                  <script setup>
                  const { foo } = withDefaults(defineProps(['foo']), { foo: 'default' })
                  </script>
                  ",
            Some(serde_json::json!([{ "destructure": "never" }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
                  <script setup lang="ts">
                  const { foo } = defineProps<{ foo?: string }>()
                  </script>
                  "#,
            Some(serde_json::json!([{ "destructure": "never" }])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": { "parser": require.resolve("@typescript-eslint/parser") }      }
    ];

    Tester::new(DefinePropsDestructuring::NAME, DefinePropsDestructuring::PLUGIN, pass, fail)
        .test_and_snapshot();
}

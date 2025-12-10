use oxc_ast::{
    AstKind,
    ast::{TSSignature, TSType},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    frameworks::FrameworkOptions,
    rule::{DefaultRuleConfig, Rule},
};

fn has_arg_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use type based declaration instead of runtime declaration")
        .with_label(span)
}

fn has_type_arg_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use runtime declaration instead of type based declaration")
        .with_label(span)
}

fn has_type_call_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Use new type literal declaration instead of the old call signature declaration",
    )
    .with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum DeclarationStyle {
    /// Enforces the use of a named TypeScript type or interface as the
    /// argument to `defineEmits`, e.g. `defineEmits<MyEmits>()`.
    #[default]
    TypeBased,
    /// Enforces the use of an inline type literal as the argument to
    /// `defineEmits`, e.g. `defineEmits<{ (event: string): void }>()`.
    TypeLiteral,
    /// Enforces the use of runtime declaration, where emits are declared
    /// using an array or object, e.g. `defineEmits(['event1', 'event2'])`.
    Runtime,
}

#[derive(Debug, Default, Clone)]
pub struct DefineEmitsDeclaration(DeclarationStyle);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces `defineEmits` typing style which you should use `type-based`, strict `type-literal` (introduced in Vue 3.3), or `runtime` declaration.
    /// This rule only works in setup script and `lang="ts"`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// // "vue/define-emits-declaration": ["error", "type-based"]
    /// <script setup lang="ts">
    /// const emit = defineEmits(['change', 'update']);
    /// const emit2 = defineEmits({
    ///   change: (id) => typeof id === 'number',
    ///   update: (value) => typeof value === 'string'
    /// });
    /// </script>
    ///
    /// // "vue/define-emits-declaration": ["error", "type-literal"]
    /// <script setup lang="ts">
    /// const emit = defineEmits<{
    ///  (e: 'change', id: number): void
    ///  (e: 'update', value: string): void
    /// }>();
    /// </script>
    ///
    /// // "vue/define-emits-declaration": ["error", "runtime"]
    /// <script setup lang="ts">
    /// const emit = defineEmits<{
    ///   (e: 'change', id: number): void
    ///   (e: 'update', value: string): void
    /// }>()
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// // "vue/define-emits-declaration": ["error", "type-based"]
    /// <script setup lang="ts">
    /// const emit = defineEmits<{
    ///   (e: 'change', id: number): void
    ///   (e: 'update', value: string): void
    /// }>()
    /// const emit2 = defineEmits<{
    ///   change: [id: number]
    ///   update: [value: string]
    /// }>()
    /// </script>
    ///
    /// // "vue/define-emits-declaration": ["error", "type-literal"]
    /// <script setup lang="ts">
    /// const emit = defineEmits<{
    ///   change: [id: number]
    ///   update: [value: string]
    /// }>()
    /// </script>
    ///
    /// // "vue/define-emits-declaration": ["error", "runtime"]
    /// <script setup lang="ts">
    /// const emit = defineEmits<{
    ///   (e: 'change', id: number): void
    ///   (e: 'update', value: string): void
    /// }>()
    /// const emit2 = defineEmits({
    ///   change: (id) => typeof id === 'number',
    ///   update: (value) => typeof value === 'string'
    /// });
    /// </script>
    /// ```
    DefineEmitsDeclaration,
    vue,
    style,
    pending, // TODO: transform it to the other declaration (if possible)
    config = DeclarationStyle,
);

impl Rule for DefineEmitsDeclaration {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(
            serde_json::from_value::<DefaultRuleConfig<DeclarationStyle>>(value)
                .unwrap_or_default()
                .into_inner(),
        )
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };

        // only check call Expression which is `defineEmits`
        if call_expr
            .callee
            .get_identifier_reference()
            .is_none_or(|reference| reference.name != "defineEmits")
        {
            return;
        }

        match self.0 {
            DeclarationStyle::TypeBased => {
                if !call_expr.arguments.is_empty() {
                    ctx.diagnostic(has_arg_diagnostic(call_expr.span));
                }
            }
            DeclarationStyle::TypeLiteral => {
                if !call_expr.arguments.is_empty() {
                    ctx.diagnostic(has_arg_diagnostic(call_expr.span));
                    return;
                }
                let Some(type_arguments) = &call_expr.type_arguments else {
                    return;
                };

                for param in &type_arguments.params {
                    match param {
                        TSType::TSTypeLiteral(literal) => {
                            for member in &literal.members {
                                if !matches!(member, TSSignature::TSPropertySignature(_)) {
                                    ctx.diagnostic(has_type_call_diagnostic(member.span()));
                                }
                            }
                        }
                        TSType::TSFunctionType(function) => {
                            ctx.diagnostic(has_type_call_diagnostic(function.span));
                        }
                        _ => {}
                    }
                }
            }
            DeclarationStyle::Runtime => {
                if call_expr.type_arguments.as_ref().is_some_and(|args| !args.params.is_empty()) {
                    ctx.diagnostic(has_type_arg_diagnostic(call_expr.span));
                }
            }
        }
    }

    fn should_run(&self, ctx: &ContextHost<'_>) -> bool {
        ctx.frameworks_options() == FrameworkOptions::VueSetup && ctx.source_type().is_typescript()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        (
            "
			        <script setup>
			          const emit = defineEmits(['change', 'update'])
			        </script>
			       ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
			        <script setup lang="ts">
			        const emit = defineEmits<{
			          (e: 'change', id: number): void
			          (e: 'update', value: string): void
			        }>()
			        </script>
			       "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
        (
            r#"
			        <script setup lang="ts">
			        const emit = defineEmits<{
			          (e: 'change', id: number): void
			          (e: 'update', value: string): void
			        }>()
			        </script>
			       "#,
            Some(serde_json::json!(["type-based"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
        (
            r#"
			       <script setup lang="ts">
			       const emit = defineEmits(['change', 'update'])
			       </script>
			       "#,
            Some(serde_json::json!(["runtime"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
			        <script setup lang="ts">
			        const emit = defineEmits<{
			          change: [id: number]
			          update: [value: string]
			        }>()
			        </script>
			       "#,
            Some(serde_json::json!(["type-based"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
        (
            r#"
			        <script setup lang="ts">
			        const emit = defineEmits<{
			          change: [id: number]
			          update: [value: string]
			        }>()
			        </script>
			       "#,
            Some(serde_json::json!(["type-literal"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
        (
            r#"
			        <script setup lang="ts">
			        const props = defineProps({
			          kind: { type: String },
			        })
			        </script>
			       "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
        (
            r#"
			         <script lang="ts">
			         import { PropType } from 'vue'

			         export default {
			           props: {
			             kind: { type: String as PropType<'primary' | 'secondary'> },
			           },
			           emits: ['check']
			         }
			         </script>
			       "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      }
    ];

    let fail = vec![
        (
            r#"
			       <script setup lang="ts">
			       const emit = defineEmits(['change', 'update'])
			       </script>
			       "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
			       <script setup lang="ts">
			       const emit = defineEmits(['change', 'update'])
			       </script>
			       "#,
            Some(serde_json::json!(["type-based"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
			       <script setup lang="ts">
			       const emit = defineEmits(['change', 'update'])
			       </script>
			       "#,
            Some(serde_json::json!(["type-literal"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
        (
            r#"
			        <script setup lang="ts">
			        const emit = defineEmits<{
			          (e: 'change', id: number): void
			          (e: 'update', value: string): void
			        }>()
			        </script>
			       "#,
            Some(serde_json::json!(["runtime"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
        (
            r#"
			        <script setup lang="ts">
			        const emit = defineEmits<{
			          (e: 'change', id: number): void
			          (e: 'update', value: string): void
			        }>()
			        </script>
			       "#,
            Some(serde_json::json!(["type-literal"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
        (
            r#"
			        <script setup lang="ts">
			        const emit = defineEmits<{
			          'change': [id: number]
			          (e: 'update', value: string): void
			        }>()
			        </script>
			       "#,
            Some(serde_json::json!(["type-literal"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      },
        (
            r#"
			        <script setup lang="ts">
			        const emit = defineEmits<(e: 'change', id: number) => void>()
			        </script>
			        "#,
            Some(serde_json::json!(["type-literal"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      }
    ];

    Tester::new(DefineEmitsDeclaration::NAME, DefineEmitsDeclaration::PLUGIN, pass, fail)
        .test_and_snapshot();
}

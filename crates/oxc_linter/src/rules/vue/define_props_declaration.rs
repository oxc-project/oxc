use oxc_ast::AstKind;
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

fn use_runtime_declaration_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use runtime declaration instead of type-based declaration")
        .with_label(span)
}

fn use_type_based_declaration_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use type-based declaration instead of runtime declaration")
        .with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum DeclarationStyle {
    /// Enforce type-based declaration.
    #[default]
    TypeBased,
    /// Enforce runtime declaration.
    Runtime,
}

#[derive(Debug, Default, Clone)]
pub struct DefinePropsDeclaration(DeclarationStyle);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces `defineProps` typing style which you should use `type-based` or `runtime` declaration.
    /// This rule only works in setup script and `lang="ts"`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// // "vue/define-props-declaration": ["error", "type-based"]
    /// <script setup lang="ts">
    ///	const props = defineProps({
    ///	  kind: { type: String },
    ///	})
    ///	</script>
    ///
    /// // "vue/define-props-declaration": ["error", "runtime"]
    /// <script setup lang="ts">
    /// const props = defineProps<{
    ///   kind: string;
    /// }>()
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```vue
    /// // "vue/define-props-declaration": ["error", "type-based"]
    /// <script setup lang="ts">
    /// const props = defineProps<{
    ///   kind: string;
    /// }>()
    /// </script>
    ///
    /// // "vue/define-props-declaration": ["error", "runtime"]
    /// <script setup lang="ts">
    ///	const props = defineProps({
    ///	  kind: { type: String },
    ///	})
    ///	</script>
    /// ```
    DefinePropsDeclaration,
    vue,
    style,
    config = DeclarationStyle,
);

impl Rule for DefinePropsDeclaration {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(
            serde_json::from_value::<DefaultRuleConfig<DeclarationStyle>>(value)
                .unwrap_or_default()
                .into_inner(),
        )
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

        match self.0 {
            DeclarationStyle::TypeBased => {
                if !call_expr.arguments.is_empty() {
                    ctx.diagnostic(use_type_based_declaration_diagnostic(call_expr.span));
                }
            }
            DeclarationStyle::Runtime => {
                if call_expr.type_arguments.is_some() {
                    ctx.diagnostic(use_runtime_declaration_diagnostic(call_expr.span));
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
            r#"
			      <script setup lang="ts">
			      const props = defineProps<{
			        kind: string;
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
			      const props = defineProps<{
			        kind: string;
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
			      const props = defineProps({
			        kind: { type: String },
			      })
			      </script>
			      "#,
            Some(serde_json::json!(["runtime"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
			      <script setup>
			      const props = defineProps({
			        kind: { type: String },
			      })
			      </script>
			      ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
			      <script setup lang="ts">
			      const emit = defineEmits({
			        click: (event: PointerEvent) => !!event
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
			      const props = defineProps({
			        kind: { type: String },
			      })
			      </script>
			      "#,
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
			      <script setup lang="ts">
			      const props = defineProps({
			        kind: { type: String },
			      })
			      </script>
			      "#,
            Some(serde_json::json!(["type-based"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
			      <script setup lang="ts">
			      const props = defineProps<{
			        kind: string;
			      }>()
			      </script>
			      "#,
            Some(serde_json::json!(["runtime"])),
            None,
            Some(PathBuf::from("test.vue")),
        ), // {        "parserOptions": {          "parser": require.resolve("@typescript-eslint/parser")        }      }
    ];

    Tester::new(DefinePropsDeclaration::NAME, DefinePropsDeclaration::PLUGIN, pass, fail)
        .test_and_snapshot();
}

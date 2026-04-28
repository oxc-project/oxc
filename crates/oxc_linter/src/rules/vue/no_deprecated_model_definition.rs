use oxc_str::Str;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, MemberExpression, ObjectExpression, ObjectPropertyKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn deprecated_model_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("`model` definition is deprecated.").with_label(span)
}

fn vue3_compat_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "`model` definition is deprecated. You may use the Vue 3-compatible `modelValue`/`update:modelValue` though.",
    )
    .with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct NoDeprecatedModelDefinitionConfig {
    /// Allow `model: { prop: 'modelValue', event: 'update:modelValue' }` (or
    /// the kebab-case `model-value` variant) which is forwards-compatible with
    /// Vue 3's `v-model`.
    allow_vue3_compat: bool,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NoDeprecatedModelDefinition(NoDeprecatedModelDefinitionConfig);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow deprecated `model` definition (in Vue.js 3.0.0+).
    ///
    /// ### Why is this bad?
    ///
    /// Vue 3 removed the per-component `model` option. Instead, `v-model`
    /// works through the `modelValue` prop and the `update:modelValue` event,
    /// so a `model: { prop, event }` block is no longer needed.
    ///
    /// With `{ "allowVue3Compat": true }`, a `model` block is allowed if it
    /// already uses the Vue 3-compatible `modelValue` / `update:modelValue`
    /// (or kebab-case `model-value` / `update:model-value`) pair, easing
    /// migration.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```vue
    /// <script>
    /// export default {
    ///   model: {
    ///     prop: 'foo',
    ///     event: 'update'
    ///   }
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule with the
    /// `{ "allowVue3Compat": true }` option:
    /// ```vue
    /// <script>
    /// export default {
    ///   model: {
    ///     prop: 'modelValue',
    ///     event: 'update:modelValue'
    ///   }
    /// }
    /// </script>
    /// ```
    NoDeprecatedModelDefinition,
    vue,
    correctness,
    pending,
    config = NoDeprecatedModelDefinition,
    version = "next",
);

impl Rule for NoDeprecatedModelDefinition {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext == "vue")
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ObjectProperty(prop) = node.kind() else { return };

        if !prop.key.is_specific_static_name("model") {
            return;
        }

        let Expression::ObjectExpression(model_value) = prop.value.get_inner_expression() else {
            return;
        };

        let mut ancestors = ctx.nodes().ancestors(node.id());
        let Some(parent) = ancestors.next() else { return };
        if !matches!(parent.kind(), AstKind::ObjectExpression(_)) {
            return;
        }

        let Some(grand) = ancestors.next() else { return };
        if !is_vue_component_root(grand.kind()) {
            return;
        }

        if !self.0.allow_vue3_compat {
            ctx.diagnostic(deprecated_model_diagnostic(prop.span));
            return;
        }

        let prop_name = find_string_property_value(model_value, "prop");
        let event_name = find_string_property_value(model_value, "event");

        let is_vue3_compat = matches!(
            (prop_name.as_deref(), event_name.as_deref()),
            (Some("modelValue"), Some("update:modelValue"))
                | (Some("model-value"), Some("update:model-value"))
        );

        if !is_vue3_compat {
            ctx.diagnostic(vue3_compat_diagnostic(prop.span));
        }
    }
}

fn is_vue_component_root(grand_kind: AstKind<'_>) -> bool {
    match grand_kind {
        AstKind::ExportDefaultDeclaration(_) => true,
        AstKind::CallExpression(call) => is_vue_component_definition_call(call),
        AstKind::NewExpression(new_expr) => {
            new_expr.callee.get_identifier_reference().is_some_and(|ident| ident.name == "Vue")
        }
        _ => false,
    }
}

fn is_vue_component_definition_call(call: &CallExpression<'_>) -> bool {
    let callee = call.callee.get_inner_expression();

    if let Expression::Identifier(ident) = callee {
        return matches!(
            ident.name.as_str(),
            "defineComponent" | "component" | "createApp" | "defineNuxtComponent"
        );
    }

    let Some(MemberExpression::StaticMemberExpression(static_member)) =
        callee.as_member_expression()
    else {
        return false;
    };
    let prop_name = static_member.property.name.as_str();
    if let Expression::Identifier(obj_ident) = static_member.object.get_inner_expression()
        && obj_ident.name == "Vue"
    {
        return matches!(prop_name, "component" | "mixin" | "extend");
    }
    matches!(prop_name, "component" | "mixin")
}

fn find_string_property_value(obj: &ObjectExpression<'_>, key: &str) -> Option<String> {
    for property_kind in &obj.properties {
        let ObjectPropertyKind::ObjectProperty(p) = property_kind else { continue };
        if !p.key.is_specific_static_name(key) {
            continue;
        }
        return string_literal_value(p.value.get_inner_expression());
    }
    None
}

fn string_literal_value(expr: &Expression<'_>) -> Option<String> {
    match expr {
        Expression::StringLiteral(lit) => Some(lit.value.to_string()),
        Expression::TemplateLiteral(tpl) => tpl.single_quasi().map(Str::into_string),
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        (
            "
                <script>
                export default { name: 'test' }
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                export default {
                  model: {
                    prop: 'modelValue',
                    event: 'update:modelValue'
                  }
                }
                </script>
            ",
            Some(serde_json::json!([{ "allowVue3Compat": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                export default defineComponent({
                  model: {
                    prop: 'model-value',
                    event: 'update:model-value'
                  }
                })
                </script>
            ",
            Some(serde_json::json!([{ "allowVue3Compat": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                export default defineComponent({
                  model: {
                    prop: `model-value`,
                    event: `update:model-value`
                  }
                })
                </script>
            ",
            Some(serde_json::json!([{ "allowVue3Compat": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    let fail = vec![
        (
            "
                <script>
                export default {
                  model: {
                    prop: 'foo',
                  }
                }
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                export default Vue.extend({
                  model: {
                    event: 'update'
                  }
                })
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                export default defineComponent({
                  model: {
                    prop: 'foo',
                    event: 'update'
                  }
                })
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                export default {
                  model: {
                    prop: 'foo',
                  }
                }
                </script>
            ",
            Some(serde_json::json!([{ "allowVue3Compat": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                export default Vue.extend({
                  model: {
                    event: 'update'
                  }
                })
                </script>
            ",
            Some(serde_json::json!([{ "allowVue3Compat": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                export default defineComponent({
                  model: {
                    prop: 'foo',
                    event: 'update'
                  }
                })
                </script>
            ",
            Some(serde_json::json!([{ "allowVue3Compat": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            r#"
                <script>
                export default {
                  model: {
                    prop: "fooBar",
                    event: "update:fooBar"
                  }
                }
                </script>
            "#,
            Some(serde_json::json!([{ "allowVue3Compat": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                export default defineComponent({
                  model: {
                    prop: 'foo-bar',
                    event: 'update:foo-bar'
                  }
                })
                </script>
            ",
            Some(serde_json::json!([{ "allowVue3Compat": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                <script>
                export default defineComponent({
                  model: {
                    prop: `foo-bar`,
                    event: `update:foo-bar`
                  }
                })
                </script>
            ",
            Some(serde_json::json!([{ "allowVue3Compat": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(NoDeprecatedModelDefinition::NAME, NoDeprecatedModelDefinition::PLUGIN, pass, fail)
        .test_and_snapshot();
}

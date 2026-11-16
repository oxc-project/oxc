use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind,
    ast::{Expression, ObjectExpression, ObjectPropertyKind},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_str::JSStr;

use crate::{
    AstNode,
    context::LintContext,
    fixer::RuleFixer,
    rule::{DefaultRuleConfig, Rule},
    utils::is_vue_component_options_object,
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
    recommended,
    conditional_suggestion,
    config = NoDeprecatedModelDefinition,
    version = "1.63.0",
    short_description = "Disallow deprecated `model` definition (in Vue.js 3.0.0+).",
);

impl Rule for NoDeprecatedModelDefinition {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
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

        let parent = ctx.nodes().parent_node(node.id());
        if !matches!(parent.kind(), AstKind::ObjectExpression(_)) {
            return;
        }

        if !is_vue_component_options_object(parent, ctx) {
            return;
        }

        if !self.0.allow_vue3_compat {
            ctx.diagnostic(deprecated_model_diagnostic(prop.span));
            return;
        }

        let prop_name = find_string_property_value(model_value, "prop");
        let event_name = find_string_property_value(model_value, "event");

        let is_vue3_compat = matches!(
            (prop_name.map(|(_, value)| value), event_name.map(|(_, value)| value)),
            (Some("modelValue"), Some("update:modelValue"))
                | (Some("model-value"), Some("update:model-value"))
        );

        if !is_vue3_compat {
            let diagnostic = vue3_compat_diagnostic(prop.span);
            let (Some((prop_span, _)), Some((event_span, _))) = (prop_name, event_name) else {
                ctx.diagnostic(diagnostic);
                return;
            };

            let fixer = RuleFixer::new(FixKind::Suggestion, ctx).for_multifix();
            let prop_span = Span::new(prop_span.start + 1, prop_span.end - 1);
            let event_span = Span::new(event_span.start + 1, event_span.end - 1);

            ctx.diagnostic_with_suggestions(
                diagnostic,
                [
                    fixer
                        .replace(prop_span, "modelValue")
                        .extend(fixer.replace(event_span, "update:modelValue"))
                        .with_message("Change to `modelValue`/`update:modelValue`."),
                    fixer
                        .replace(prop_span, "model-value")
                        .extend(fixer.replace(event_span, "update:model-value"))
                        .with_message("Change to `model-value`/`update:model-value`."),
                ],
            );
        }
    }
}

fn find_string_property_value<'a>(
    obj: &'a ObjectExpression<'_>,
    key: &str,
) -> Option<(Span, &'a str)> {
    for property_kind in &obj.properties {
        let ObjectPropertyKind::ObjectProperty(p) = property_kind else { continue };
        if !p.key.is_specific_static_name(key) {
            continue;
        }
        let value = p.value.get_inner_expression();
        let text = match value {
            Expression::StringLiteral(lit) => lit.value.as_str(),
            Expression::TemplateLiteral(tpl) => tpl.single_quasi().and_then(JSStr::as_str),
            _ => None,
        }?;
        return Some((value.span(), text));
    }
    None
}

#[test]
fn test() {
    use crate::tester::{ExpectFixTestCase, Tester};
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
        (
            "
                <script>
                new Vue({
                  model: {
                    prop: 'checked',
                    event: 'change'
                  }
                })
                </script>
            ",
            None,
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>export default { model: { prop: name, event: 'change' } }</script>",
            Some(serde_json::json!([{ "allowVue3Compat": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>export default { model: { prop: 'checked', event: `update:${name}` } }</script>",
            Some(serde_json::json!([{ "allowVue3Compat": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "<script>export default { model: { prop: 'checked', event: 1 } }</script>",
            Some(serde_json::json!([{ "allowVue3Compat": true }])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    let fix: Vec<ExpectFixTestCase> = vec![
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
            (
                r"
                <script>
                export default defineComponent({
                  model: {
                    prop: 'modelValue',
                    event: 'update:modelValue'
                  }
                })
                </script>
            ",
                r"
                <script>
                export default defineComponent({
                  model: {
                    prop: 'model-value',
                    event: 'update:model-value'
                  }
                })
                </script>
            ",
            ),
            Some(serde_json::json!([{ "allowVue3Compat": true }])),
        )
            .into(),
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
            (
                r#"
                <script>
                export default {
                  model: {
                    prop: "modelValue",
                    event: "update:modelValue"
                  }
                }
                </script>
            "#,
                r#"
                <script>
                export default {
                  model: {
                    prop: "model-value",
                    event: "update:model-value"
                  }
                }
                </script>
            "#,
            ),
            Some(serde_json::json!([{ "allowVue3Compat": true }])),
        )
            .into(),
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
            (
                r"
                <script>
                export default defineComponent({
                  model: {
                    prop: 'modelValue',
                    event: 'update:modelValue'
                  }
                })
                </script>
            ",
                r"
                <script>
                export default defineComponent({
                  model: {
                    prop: 'model-value',
                    event: 'update:model-value'
                  }
                })
                </script>
            ",
            ),
            Some(serde_json::json!([{ "allowVue3Compat": true }])),
        )
            .into(),
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
            (
                r"
                <script>
                export default defineComponent({
                  model: {
                    prop: `modelValue`,
                    event: `update:modelValue`
                  }
                })
                </script>
            ",
                r"
                <script>
                export default defineComponent({
                  model: {
                    prop: `model-value`,
                    event: `update:model-value`
                  }
                })
                </script>
            ",
            ),
            Some(serde_json::json!([{ "allowVue3Compat": true }])),
        )
            .into(),
        (
            r#"<script>export default { model: { prop: (/* prop */ "old"), event: (`change`) } }</script>"#,
            (
                r#"<script>export default { model: { prop: (/* prop */ "modelValue"), event: (`update:modelValue`) } }</script>"#,
                r#"<script>export default { model: { prop: (/* prop */ "model-value"), event: (`update:model-value`) } }</script>"#,
            ),
            Some(serde_json::json!([{ "allowVue3Compat": true }])),
        )
            .into(),
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
        )
            .into(),
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
        )
            .into(),
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
        )
            .into(),
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
        )
            .into(),
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
        )
            .into(),
        (
            "
                <script>
                new Vue({
                  model: {
                    prop: 'checked',
                    event: 'change'
                  }
                })
                </script>
            ",
            "
                <script>
                new Vue({
                  model: {
                    prop: 'checked',
                    event: 'change'
                  }
                })
                </script>
            ",
            None,
        )
            .into(),
        (
            "<script>export default { model: { prop: name, event: 'change' } }</script>",
            "<script>export default { model: { prop: name, event: 'change' } }</script>",
            Some(serde_json::json!([{ "allowVue3Compat": true }])),
        )
            .into(),
        (
            "<script>export default { model: { prop: 'checked', event: `update:${name}` } }</script>",
            "<script>export default { model: { prop: 'checked', event: `update:${name}` } }</script>",
            Some(serde_json::json!([{ "allowVue3Compat": true }])),
        )
            .into(),
        (
            "<script>export default { model: { prop: 'checked', event: 1 } }</script>",
            "<script>export default { model: { prop: 'checked', event: 1 } }</script>",
            Some(serde_json::json!([{ "allowVue3Compat": true }])),
        )
            .into(),
    ];

    Tester::new(NoDeprecatedModelDefinition::NAME, NoDeprecatedModelDefinition::PLUGIN, pass, fail)
        .change_rule_path_extension("vue")
        .expect_fix(fix)
        .test_and_snapshot();
}

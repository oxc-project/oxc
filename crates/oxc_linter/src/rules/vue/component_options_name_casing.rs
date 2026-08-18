use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind,
    ast::{Expression, ObjectPropertyKind, PropertyKey},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::{find_property, is_vue_component_options_object, vue_casing},
};

fn component_options_name_casing_diagnostic(
    span: Span,
    name: &str,
    case_type: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Component name \"{name}\" is not {case_type}.")).with_label(span)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
enum CaseType {
    #[default]
    #[serde(rename = "PascalCase")]
    Pascal,
    #[serde(rename = "kebab-case")]
    Kebab,
    #[serde(rename = "camelCase")]
    Camel,
}

impl CaseType {
    fn as_str(self) -> &'static str {
        match self {
            CaseType::Pascal => "PascalCase",
            CaseType::Kebab => "kebab-case",
            CaseType::Camel => "camelCase",
        }
    }

    fn check(self, s: &str) -> bool {
        match self {
            CaseType::Pascal => vue_casing::is_pascal_case(s),
            CaseType::Kebab => vue_casing::is_kebab_case(s),
            CaseType::Camel => vue_casing::is_camel_case(s),
        }
    }

    fn convert(self, s: &str) -> String {
        match self {
            CaseType::Pascal => vue_casing::pascal_case(s),
            CaseType::Kebab => vue_casing::kebab_case(s),
            CaseType::Camel => vue_casing::camel_case(s),
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct ComponentOptionsNameCasing(CaseType);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce the casing of component names in the `components` option.
    ///
    /// ### Why is this bad?
    ///
    /// Registering components under inconsistently cased keys makes templates
    /// harder to read and harder to grep. Picking one casing (`PascalCase` by
    /// default) and sticking with it keeps component registration uniform
    /// across the codebase.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule (default `PascalCase`):
    /// ```vue
    /// <script>
    /// export default {
    ///   components: {
    ///     fooBar
    ///   }
    /// }
    /// </script>
    /// ```
    ///
    /// Examples of **correct** code for this rule (default `PascalCase`):
    /// ```vue
    /// <script>
    /// export default {
    ///   components: {
    ///     FooBar
    ///   }
    /// }
    /// </script>
    /// ```
    ComponentOptionsNameCasing,
    vue,
    style,
    conditional_fix_suggestion,
    config = CaseType,
    version = "next",
    short_description = "Enforce the casing of component names in the `components` option.",
);

impl Rule for ComponentOptionsNameCasing {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ObjectExpression(obj) = node.kind() else { return };
        if !is_vue_component_options_object(node, ctx) {
            return;
        }
        let Some(components) = find_property(obj, "components") else { return };
        let Expression::ObjectExpression(components_obj) = components.value.get_inner_expression()
        else {
            return;
        };

        let case_type = self.0;
        for prop_kind in &components_obj.properties {
            let ObjectPropertyKind::ObjectProperty(prop) = prop_kind else { continue };
            if prop.computed {
                continue;
            }
            let name = match &prop.key {
                PropertyKey::StaticIdentifier(ident) => ident.name.as_str(),
                PropertyKey::StringLiteral(lit) => lit.value.as_str(),
                _ => continue,
            };
            if case_type.check(name) {
                continue;
            }

            let diagnostic =
                component_options_name_casing_diagnostic(prop.key.span(), name, case_type.as_str());
            let converted = case_type.convert(name);

            // Mirrors upstream: only PascalCase converts to a bare identifier
            // in every case, so only it is safe to auto-fix; the other casings
            // are offered as suggestions (kebab-case needs a quoted key).
            if case_type == CaseType::Pascal {
                ctx.diagnostic_with_fix(diagnostic, |fixer| {
                    if prop.shorthand {
                        fixer.replace(prop.span, format!("{converted}: {name}"))
                    } else {
                        fixer.replace(prop.key.span(), converted)
                    }
                });
            } else {
                ctx.diagnostic_with_suggestion(diagnostic, |fixer| {
                    let key = if case_type == CaseType::Kebab {
                        format!("'{converted}'")
                    } else {
                        converted
                    };
                    if prop.shorthand {
                        fixer.replace(prop.span, format!("{key}: {name}"))
                    } else {
                        fixer.replace(prop.key.span(), key)
                    }
                });
            }
        }
    }
}

#[test]
fn test() {
    use std::path::PathBuf;

    use crate::tester::Tester;
    // ref: https://github.com/vuejs/eslint-plugin-vue/blob/master/tests/lib/rules/component-options-name-casing.test.ts

    let pass = vec![
        (
            "
                    <script>
                    export default {
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
                    export default {
                      ...components
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
                    export default {
                      components: {
                        FooBar
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
                    export default {
                      components: {
                        FooBar: fooBar
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
                    export default {
                      components: {
                        FooBar
                      }
                    }
                    </script>
                  ",
            Some(serde_json::json!(["PascalCase"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    export default {
                      components: {
                        fooBar
                      }
                    }
                    </script>
                  ",
            Some(serde_json::json!(["camelCase"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    export default {
                      components: {
                        fooBar: FooBar
                      }
                    }
                    </script>
                  ",
            Some(serde_json::json!(["camelCase"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    export default {
                      components: {
                        'foo-bar': fooBar
                      }
                    }
                    </script>
                  ",
            Some(serde_json::json!(["kebab-case"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    export default {
                      components: {
                        'foo-bar': FooBar
                      }
                    }
                    </script>
                  ",
            Some(serde_json::json!(["kebab-case"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    let fail = vec![
        (
            "
                    <script>
                    export default {
                      components: {
                        fooBar
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
                    export default {
                      components: {
                        fooBar: FooBar
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
                    export default {
                      components: {
                        fooBar
                      }
                    }
                    </script>
                  ",
            Some(serde_json::json!(["PascalCase"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    export default {
                      components: {
                        fooBar: FooBar
                      }
                    }
                    </script>
                  ",
            Some(serde_json::json!(["PascalCase"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    export default {
                      components: {
                        'foo-bar': FooBar
                      }
                    }
                    </script>
                  ",
            Some(serde_json::json!(["PascalCase"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    export default {
                      components: {
                        FooBar
                      }
                    }
                    </script>
                  ",
            Some(serde_json::json!(["camelCase"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    export default {
                      components: {
                        FooBar: fooBar
                      }
                    }
                    </script>
                  ",
            Some(serde_json::json!(["camelCase"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    export default {
                      components: {
                        'foo-bar': fooBar
                      }
                    }
                    </script>
                  ",
            Some(serde_json::json!(["camelCase"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    export default {
                      components: {
                        FooBar
                      }
                    }
                    </script>
                  ",
            Some(serde_json::json!(["kebab-case"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    export default {
                      components: {
                        FooBar: fooBar
                      }
                    }
                    </script>
                  ",
            Some(serde_json::json!(["kebab-case"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    export default {
                      components: {
                        fooBar
                      }
                    }
                    </script>
                  ",
            Some(serde_json::json!(["kebab-case"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    export default {
                      components: {
                        fooBar: FooBar
                      }
                    }
                    </script>
                  ",
            Some(serde_json::json!(["kebab-case"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    export default {
                      components: {
                        FooBar,
                        'my-component': MyComponent
                      }
                    }
                    </script>
                  ",
            Some(serde_json::json!(["kebab-case"])),
            None,
            Some(PathBuf::from("test.vue")),
        ),
    ];

    let fix = vec![
        (
            "
                    <script>
                    export default {
                      components: {
                        fooBar
                      }
                    }
                    </script>
                  ",
            "
                    <script>
                    export default {
                      components: {
                        FooBar: fooBar
                      }
                    }
                    </script>
                  ",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    export default {
                      components: {
                        fooBar: FooBar
                      }
                    }
                    </script>
                  ",
            "
                    <script>
                    export default {
                      components: {
                        FooBar: FooBar
                      }
                    }
                    </script>
                  ",
            None,
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    export default {
                      components: {
                        fooBar
                      }
                    }
                    </script>
                  ",
            "
                    <script>
                    export default {
                      components: {
                        FooBar: fooBar
                      }
                    }
                    </script>
                  ",
            Some(serde_json::json!(["PascalCase"])),
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    export default {
                      components: {
                        fooBar: FooBar
                      }
                    }
                    </script>
                  ",
            "
                    <script>
                    export default {
                      components: {
                        FooBar: FooBar
                      }
                    }
                    </script>
                  ",
            Some(serde_json::json!(["PascalCase"])),
            Some(PathBuf::from("test.vue")),
        ),
        (
            "
                    <script>
                    export default {
                      components: {
                        'foo-bar': FooBar
                      }
                    }
                    </script>
                  ",
            "
                    <script>
                    export default {
                      components: {
                        FooBar: FooBar
                      }
                    }
                    </script>
                  ",
            Some(serde_json::json!(["PascalCase"])),
            Some(PathBuf::from("test.vue")),
        ),
    ];

    Tester::new(ComponentOptionsNameCasing::NAME, ComponentOptionsNameCasing::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

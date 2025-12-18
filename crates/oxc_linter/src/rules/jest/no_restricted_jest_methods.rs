use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::{JestFnKind, JestGeneralFnKind, PossibleJestNode, is_type_of_jest_fn_call},
};

fn restricted_jest_method(method_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Use of `{method_name}` is not allowed")).with_label(span)
}

fn restricted_jest_method_with_message(message: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(message.to_string()).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoRestrictedJestMethods(Box<NoRestrictedJestMethodsConfig>);

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NoRestrictedJestMethodsConfig {
    /// A mapping of restricted Jest method names to custom messages - or
    /// `null`, for a generic message.
    #[serde(flatten)]
    restricted_jest_methods: FxHashMap<String, Option<String>>,
}

impl JsonSchema for NoRestrictedJestMethodsConfig {
    fn schema_name() -> String {
        "NoRestrictedJestMethodsConfig".to_string()
    }

    fn json_schema(_gen: &mut schemars::r#gen::SchemaGenerator) -> schemars::schema::Schema {
        use schemars::schema::*;

        Schema::Object(SchemaObject {
            instance_type: Some(InstanceType::Object.into()),
            metadata: Some(Box::new(Metadata {
                description: Some(
                    "A map of restricted Jest method names to custom error messages.".to_string(),
                ),
                ..Default::default()
            })),
            object: Some(Box::new(ObjectValidation {
                additional_properties: Some(Box::new(Schema::Object(SchemaObject {
                    instance_type: Some(vec![InstanceType::String, InstanceType::Null].into()),
                    ..Default::default()
                }))),
                ..Default::default()
            })),
            ..Default::default()
        })
    }
}

impl std::ops::Deref for NoRestrictedJestMethods {
    type Target = NoRestrictedJestMethodsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Restrict the use of specific `jest` and `vi` methods.
    ///
    /// ### Why is this bad?
    ///
    /// Certain Jest or Vitest methods may be deprecated, discouraged in specific
    /// contexts, or incompatible with your testing environment. Restricting
    /// them helps maintain consistent and reliable test practices.
    ///
    /// By default, no methods are restricted by this rule.
    /// You must configure the rule for it to disable anything.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// jest.useFakeTimers();
    /// it('calls the callback after 1 second via advanceTimersByTime', () => {
    ///   // ...
    ///
    ///   jest.advanceTimersByTime(1000);
    ///
    ///   // ...
    /// });
    ///
    /// test('plays video', () => {
    ///   const spy = jest.spyOn(video, 'play');
    ///
    ///   // ...
    /// });
    /// ```
    ///
    /// This rule is compatible with [eslint-plugin-vitest](https://github.com/vitest-dev/eslint-plugin-vitest/blob/main/docs/rules/no-restricted-vi-methods.md),
    /// to use it, add the following configuration to your `.oxlintrc.json`:
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///      "vitest/no-restricted-vi-methods": ["error", { "badFunction": "Don't use `badFunction`, it is bad." }]
    ///   }
    /// }
    /// ```
    NoRestrictedJestMethods,
    jest,
    style,
    config = NoRestrictedJestMethodsConfig,
);

impl Rule for NoRestrictedJestMethods {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config =
            serde_json::from_value::<DefaultRuleConfig<NoRestrictedJestMethodsConfig>>(value)
                .unwrap_or_default()
                .into_inner();

        Self(Box::new(config))
    }

    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        self.run(jest_node, ctx);
    }
}

impl NoRestrictedJestMethods {
    fn contains(&self, key: &str) -> bool {
        self.restricted_jest_methods.contains_key(key)
    }

    fn run<'a>(&self, possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_type_of_jest_fn_call(
            call_expr,
            possible_jest_node,
            ctx,
            &[
                JestFnKind::General(JestGeneralFnKind::Jest),
                JestFnKind::General(JestGeneralFnKind::Vitest),
            ],
        ) {
            return;
        }

        let Some(mem_expr) = call_expr.callee.as_member_expression() else {
            return;
        };
        let Some(property_name) = mem_expr.static_property_name() else {
            return;
        };
        let Some((span, _)) = mem_expr.static_property_info() else {
            return;
        };

        if self.contains(property_name) {
            match self.restricted_jest_methods.get(property_name).and_then(|m| m.as_deref()) {
                None | Some("") => {
                    ctx.diagnostic(restricted_jest_method(property_name, span));
                }
                Some(message) => {
                    ctx.diagnostic(restricted_jest_method_with_message(message, span));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let mut pass = vec![
        ("jest", None),
        ("jest()", None),
        ("jest.mock()", None),
        ("expect(a).rejects;", None),
        ("expect(a);", None),
        (
            "
                import { jest } from '@jest/globals';

                jest;
            ",
            None,
        ),
    ];

    let mut fail = vec![
        ("jest.fn()", Some(serde_json::json!([{ "fn": null }]))),
        ("jest[\"fn\"]()", Some(serde_json::json!([{ "fn": null }]))),
        ("jest.mock()", Some(serde_json::json!([{ "mock": "Do not use mocks" }]))),
        ("jest[\"mock\"]()", Some(serde_json::json!([{ "mock": "Do not use mocks" }]))),
        (
            "
                import { jest } from '@jest/globals';
                jest.advanceTimersByTime();
            ",
            Some(serde_json::json!([{ "advanceTimersByTime": null }])),
        ),
    ];

    let pass_vitest = vec![
        ("vi", None),
        ("vi()", None),
        ("vi.mock()", None),
        ("expect(a).rejects;", None),
        ("expect(a);", None),
        (
            "
			     import { vi } from 'vitest';
			     vi;
			    ",
            None,
        ), // { "parserOptions": { "sourceType": "module" } }
    ];

    let fail_vitest = vec![
        ("vi.fn()", Some(serde_json::json!([{ "fn": null }]))),
        ("vi.mock()", Some(serde_json::json!([{ "mock": "Do not use mocks" }]))),
        (
            "
			     import { vi } from 'vitest';
			     vi.advanceTimersByTime();
			    ",
            Some(serde_json::json!([{ "advanceTimersByTime": null }])),
        ), // { "parserOptions": { "sourceType": "module" } },
        (r#"vi["fn"]()"#, Some(serde_json::json!([{ "fn": null }]))),
    ];

    pass.extend(pass_vitest);
    fail.extend(fail_vitest);

    Tester::new(NoRestrictedJestMethods::NAME, NoRestrictedJestMethods::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}

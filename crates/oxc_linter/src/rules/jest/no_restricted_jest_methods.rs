use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use rustc_hash::FxHashMap;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        collect_possible_jest_call_node, is_type_of_jest_fn_call, JestFnKind, JestGeneralFnKind,
        PossibleJestNode,
    },
};

fn restricted_jest_method(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Disallow specific `jest.` methods")
        .with_help(format!("Use of `{x0:?}` is disallowed"))
        .with_label(span1)
}

fn restricted_jest_method_with_message(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Disallow specific `jest.` methods")
        .with_help(format!("{x0:?}"))
        .with_label(span1)
}

#[derive(Debug, Default, Clone)]
pub struct NoRestrictedJestMethods(Box<NoRestrictedJestMethodsConfig>);

#[derive(Debug, Default, Clone)]
pub struct NoRestrictedJestMethodsConfig {
    restricted_jest_methods: FxHashMap<String, String>,
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
    /// Restrict the use of specific `jest` methods.
    ///
    /// ### Example
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
    ///
    NoRestrictedJestMethods,
    style,
);

impl Rule for NoRestrictedJestMethods {
    fn from_configuration(value: serde_json::Value) -> Self {
        let restricted_jest_methods = &value
            .get(0)
            .and_then(serde_json::Value::as_object)
            .and_then(Self::compile_restricted_jest_methods)
            .unwrap_or_default();

        Self(Box::new(NoRestrictedJestMethodsConfig {
            restricted_jest_methods: restricted_jest_methods.clone(),
        }))
    }

    fn run_once(&self, ctx: &LintContext) {
        for possible_jest_node in &collect_possible_jest_call_node(ctx) {
            self.run(possible_jest_node, ctx);
        }
    }
}

impl NoRestrictedJestMethods {
    fn contains(&self, key: &str) -> bool {
        self.restricted_jest_methods.contains_key(key)
    }

    fn get_message(&self, name: &str) -> Option<String> {
        self.restricted_jest_methods.get(name).cloned()
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
            &[JestFnKind::General(JestGeneralFnKind::Jest)],
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
            self.get_message(property_name).map_or_else(
                || {
                    ctx.diagnostic(restricted_jest_method(property_name, span));
                },
                |message| {
                    if message.trim() == "" {
                        ctx.diagnostic(restricted_jest_method(property_name, span));
                    } else {
                        ctx.diagnostic(restricted_jest_method_with_message(&message, span));
                    }
                },
            );
        }
    }

    #[allow(clippy::unnecessary_wraps)]
    pub fn compile_restricted_jest_methods(
        matchers: &serde_json::Map<String, serde_json::Value>,
    ) -> Option<FxHashMap<String, String>> {
        Some(
            matchers
                .iter()
                .map(|(key, value)| {
                    (String::from(key), String::from(value.as_str().unwrap_or_default()))
                })
                .collect(),
        )
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
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

    let fail = vec![
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

    Tester::new(NoRestrictedJestMethods::NAME, pass, fail)
        .with_jest_plugin(true)
        .test_and_snapshot();
}

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        collect_possible_jest_call_node, is_type_of_jest_fn_call, JestFnKind, JestGeneralFnKind,
        PossibleJestNode,
    },
};

use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use rustc_hash::{FxHashMap, FxHasher};
use std::{collections::HashMap, hash::BuildHasherDefault};

#[derive(Debug, Error, Diagnostic)]
enum NoRestrictedMatchersDiagnostic {
    #[error("eslint-plugin-jest(no-restricted-matchers): Disallow specific matchers & modifiers")]
    #[diagnostic(severity(warning), help("Use of `{0:?}` is disallowed`"))]
    RestrictedChain(String, #[label] Span),
    #[error("eslint-plugin-jest(no-restricted-matchers): Disallow specific matchers & modifiers")]
    #[diagnostic(severity(warning), help("{0:?}"))]
    RestrictedChainWithMessage(String, #[label] Span),
}

#[derive(Debug, Default, Clone)]
pub struct NoRestrictedMatchers(Box<NoRestrictedMatchersConfig>);

#[derive(Debug, Default, Clone)]
pub struct NoRestrictedMatchersConfig {
    restricted_matchers: FxHashMap<String, String>,
}

impl std::ops::Deref for NoRestrictedMatchers {
    type Target = NoRestrictedMatchersConfig;

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
    ///
    /// jest.useFakeTimers();
    /// it('calls the callback after 1 second via advanceTimersByTime', () => {
    ///   ...
    ///   jest.advanceTimersByTime(1000);
    ///   ...
    /// });
    ///
    /// test('plays video', () => {
    ///   const spy = jest.spyOn(video, 'play');
    ///   ...
    /// });
    /// ```
    NoRestrictedMatchers,
    style,
);

impl Rule for NoRestrictedMatchers {
    fn from_configuration(value: serde_json::Value) -> Self {
        let restricted_matchers = &value
            .get(0)
            .and_then(serde_json::Value::as_object)
            .and_then(Self::compile_restricted_matchers)
            .unwrap_or_default();

        Self(Box::new(NoRestrictedMatchersConfig {
            restricted_matchers: restricted_matchers.clone(),
        }))
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        for possible_jest_node in &collect_possible_jest_call_node(ctx) {
            self.run(possible_jest_node, ctx);
        }
    }
}

impl NoRestrictedMatchers {
    fn contains(&self, key: &str) -> bool {
        self.restricted_matchers.contains_key(key)
    }

    fn get_message(&self, name: &str) -> Option<String> {
        self.restricted_matchers.get(name).cloned()
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
            &[JestFnKind::Expect, JestFnKind::General(JestGeneralFnKind::Jest)],
        ) {
            return;
        }

        let Expression::MemberExpression(mem_expr) = &call_expr.callee else {
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
                    ctx.diagnostic(NoRestrictedMatchersDiagnostic::RestrictedChain(
                        property_name.to_string(),
                        span,
                    ));
                },
                |message| {
                    if message.trim() == "" {
                        ctx.diagnostic(NoRestrictedMatchersDiagnostic::RestrictedChain(
                            property_name.to_string(),
                            span,
                        ));
                    } else {
                        ctx.diagnostic(NoRestrictedMatchersDiagnostic::RestrictedChainWithMessage(
                            message, span,
                        ));
                    }
                },
            );
        }
    }

    #[allow(clippy::unnecessary_wraps)]
    pub fn compile_restricted_matchers(
        matchers: &serde_json::Map<String, serde_json::Value>,
    ) -> Option<HashMap<String, String, BuildHasherDefault<FxHasher>>> {
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
        ("jest.fn()", Some(serde_json::json!([ { "fn": null }]))),
        ("jest['fn']()", Some(serde_json::json!([ { "fn": null }]))),
        ("jest.mock()", Some(serde_json::json!([ { "mock": "Do not use mocks" }]))),
        ("jest['mock']()", Some(serde_json::json!([ { "mock": "Do not use mocks" }]))),
        (
            "
                import { jest } from '@jest/globals';
                jest.advanceTimersByTime();
            ",
            Some(serde_json::json!([ { "advanceTimersByTime": null }])),
        ),
    ];

    Tester::new(NoRestrictedMatchers::NAME, pass, fail).with_jest_plugin(true).test_and_snapshot();
}

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        collect_possible_jest_call_node, is_type_of_jest_fn_call, parse_expect_jest_fn_call,
        JestFnKind, KnownMemberExpressionProperty, PossibleJestNode,
    },
};

use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use phf::phf_set;
use rustc_hash::{FxHashMap, FxHasher};
use std::{collections::HashMap, hash::BuildHasherDefault, path::Path};

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
    /// Ban specific matchers & modifiers from being used, and can suggest alternatives.
    ///
    /// ### Example
    /// ```javascript
    ///
    /// it('is false', () => {
    ///   if this has a modifier (i.e. `not.toBeFalsy`), it would be considered fine
    ///   expect(a).toBeFalsy();
    /// });
    ///
    /// it('resolves', async () => {
    ///   // all uses of this modifier are disallowed, regardless of matcher
    ///   await expect(myPromise()).resolves.toBe(true);
    /// });
    ///
    /// describe('when an error happens', () => {
    ///   it('does not upload the file', async () => {
    ///     // all uses of this matcher are disallowed
    ///     expect(uploadFileMock).not.toHaveBeenCalledWith('file.name');
    ///   });
    /// });
    ///
    NoRestrictedMatchers,
    style,
);

const MODIFIER_NAME: phf::Set<&'static str> = phf_set!["not", "rejects", "resolves"];

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
    fn run<'a>(&self, possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_type_of_jest_fn_call(call_expr, possible_jest_node, ctx, &[JestFnKind::Expect]) {
            return;
        }

        let Some(jest_fn_call) = parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };

        let members = &jest_fn_call.members;

        if members.is_empty() {
            return;
        }

        let chain_call = members
            .iter()
            .filter_map(KnownMemberExpressionProperty::name)
            .collect::<Vec<_>>()
            .join(".");

        let span = Span::new(members.first().unwrap().span.start, members.last().unwrap().span.end);

        for (restriction, message) in &self.restricted_matchers {
            if Self::check_restriction(chain_call.as_str(), restriction.as_str()) {
                if message.is_empty() {
                    ctx.diagnostic(NoRestrictedMatchersDiagnostic::RestrictedChain(
                        chain_call.clone(),
                        span,
                    ));
                } else {
                    ctx.diagnostic(NoRestrictedMatchersDiagnostic::RestrictedChainWithMessage(
                        message.to_string(),
                        span,
                    ));
                }
            }
        }
    }

    fn check_restriction(chain_call: &str, restriction: &str) -> bool {
        if MODIFIER_NAME.contains(restriction)
            || Path::new(restriction)
                .extension()
                .map_or(false, |ext| ext.eq_ignore_ascii_case("not"))
        {
            return chain_call.starts_with(restriction);
        }

        chain_call == restriction
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
        ("expect(a).toHaveBeenCalled()", None),
        ("expect(a).not.toHaveBeenCalled()", None),
        ("expect(a).toHaveBeenCalledTimes()", None),
        ("expect(a).toHaveBeenCalledWith()", None),
        ("expect(a).toHaveBeenLastCalledWith()", None),
        ("expect(a).toHaveBeenNthCalledWith()", None),
        ("expect(a).toHaveReturned()", None),
        ("expect(a).toHaveReturnedTimes()", None),
        ("expect(a).toHaveReturnedWith()", None),
        ("expect(a).toHaveLastReturnedWith()", None),
        ("expect(a).toHaveNthReturnedWith()", None),
        ("expect(a).toThrow()", None),
        ("expect(a).rejects;", None),
        ("expect(a);", None),
        ("expect(a).resolves", Some(serde_json::json!([{ "not": null }]))),
        ("expect(a).toBe(b)", Some(serde_json::json!([{ "not.toBe": null }]))),
        ("expect(a).toBeUndefined(b)", Some(serde_json::json!([{ "toBe": null }]))),
        ("expect(a)[\"toBe\"](b)", Some(serde_json::json!([{ "not.toBe": null }]))),
        ("expect(a).resolves.not.toBe(b)", Some(serde_json::json!([{ "not": null }]))),
        ("expect(a).resolves.not.toBe(b)", Some(serde_json::json!([{ "not.toBe": null }]))),
        (
            "expect(uploadFileMock).resolves.toHaveBeenCalledWith('file.name')",
            Some(
                serde_json::json!([{ "not.toHaveBeenCalledWith": "Use not.toHaveBeenCalled instead" }]),
            ),
        ),
        (
            "expect(uploadFileMock).resolves.not.toHaveBeenCalledWith('file.name')",
            Some(
                serde_json::json!([{ "not.toHaveBeenCalledWith": "Use not.toHaveBeenCalled instead" }]),
            ),
        ),
    ];

    let fail = vec![
        ("expect(a).toBe(b)", Some(serde_json::json!([{ "toBe": null }]))),
        ("expect(a)[\"toBe\"](b)", Some(serde_json::json!([{ "toBe": null }]))),
        ("expect(a).not[x]()", Some(serde_json::json!([{ "not": null }]))),
        ("expect(a).not.toBe(b)", Some(serde_json::json!([{ "not": null }]))),
        ("expect(a).resolves.toBe(b)", Some(serde_json::json!([{ "resolves": null }]))),
        ("expect(a).resolves.not.toBe(b)", Some(serde_json::json!([{ "resolves": null }]))),
        ("expect(a).resolves.not.toBe(b)", Some(serde_json::json!([{ "resolves.not": null }]))),
        ("expect(a).not.toBe(b)", Some(serde_json::json!([{ "not.toBe": null }]))),
        (
            "expect(a).resolves.not.toBe(b)",
            Some(serde_json::json!([{ "resolves.not.toBe": null }])),
        ),
        (
            "expect(a).toBe(b)",
            Some(serde_json::json!([{ "toBe": "Prefer `toStrictEqual` instead" }])),
        ),
        (
            "
                test('some test', async () => {
                    await expect(Promise.resolve(1)).resolves.toBe(1);
                });
            ",
            Some(serde_json::json!([{ "resolves": "Use `expect(await promise)` instead." }])),
        ),
        (
            "expect(Promise.resolve({})).rejects.toBeFalsy()",
            Some(serde_json::json!([{ "rejects.toBeFalsy": null }])),
        ),
        (
            "expect(uploadFileMock).not.toHaveBeenCalledWith('file.name')",
            Some(serde_json::json!([
                { "not.toHaveBeenCalledWith": "Use not.toHaveBeenCalled instead" },
            ])),
        ),
    ];

    Tester::new(NoRestrictedMatchers::NAME, pass, fail).with_jest_plugin(true).test_and_snapshot();
}

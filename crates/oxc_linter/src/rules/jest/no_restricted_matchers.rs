use std::path::Path;

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
    utils::{
        JestFnKind, KnownMemberExpressionProperty, PossibleJestNode, is_type_of_jest_fn_call,
        parse_expect_jest_fn_call,
    },
};

fn restricted_chain(chain_call: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Use of `{chain_call}` is disallowed")).with_label(span)
}

fn restricted_chain_with_message(chain_call: &str, message: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Use of `{chain_call}` is disallowed"))
        .with_help(message.to_string())
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoRestrictedMatchers(Box<NoRestrictedMatchersConfig>);

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(transparent)]
pub struct NoRestrictedMatchersConfig(FxHashMap<String, Option<String>>);

impl JsonSchema for NoRestrictedMatchersConfig {
    fn schema_name() -> String {
        "NoRestrictedMatchersConfig".to_string()
    }

    fn json_schema(_gen: &mut schemars::r#gen::SchemaGenerator) -> schemars::schema::Schema {
        use schemars::schema::*;

        Schema::Object(SchemaObject {
            instance_type: Some(InstanceType::Object.into()),
            metadata: Some(Box::new(Metadata {
                description: Some(
                    "A map of restricted matcher/modifier names to custom error messages."
                        .to_string(),
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

impl std::ops::Deref for NoRestrictedMatchers {
    type Target = NoRestrictedMatchersConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::Deref for NoRestrictedMatchersConfig {
    type Target = FxHashMap<String, Option<String>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ban specific matchers & modifiers from being used, and can suggest alternatives.
    ///
    /// ### Why is this bad?
    ///
    /// Some matchers or modifiers might be discouraged in your codebase for various reasons:
    /// they might be deprecated, cause confusion, have performance implications, or there
    /// might be better alternatives available. This rule allows you to enforce consistent
    /// testing patterns by restricting certain Jest matchers and providing guidance on
    /// preferred alternatives.
    ///
    /// ### Examples
    ///
    /// Bans are expressed in the form of a map, with the value being either a string message to be shown,
    /// or null if only the default rule message should be used. Bans are checked against the start of
    /// the expect chain - this means that to ban a specific matcher entirely you must specify all
    /// six permutations, but allows you to ban modifiers as well. By default, this map is empty, meaning
    /// no matchers or modifiers are banned.
    ///
    /// Example configuration:
    /// ```json
    /// {
    ///   "jest/no-restricted-matchers": [
    ///     "error",
    ///     {
    ///       "toBeFalsy": null,
    ///       "resolves": "Use `expect(await promise)` instead.",
    ///       "toHaveBeenCalledWith": null,
    ///       "not.toHaveBeenCalledWith": null,
    ///       "resolves.toHaveBeenCalledWith": null,
    ///       "rejects.toHaveBeenCalledWith": null,
    ///       "resolves.not.toHaveBeenCalledWith": null,
    ///       "rejects.not.toHaveBeenCalledWith": null
    ///     }
    ///   ]
    /// }
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with the above configuration:
    /// ```javascript
    /// it('is false', () => {
    ///   // if this has a modifier (i.e. `not.toBeFalsy`), it would be considered fine
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
    /// ```
    ///
    /// This rule is compatible with [eslint-plugin-vitest](https://github.com/vitest-dev/eslint-plugin-vitest/blob/main/docs/rules/no-restricted-matchers.md),
    /// to use it, add the following configuration to your `.oxlintrc.json`:
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///      "vitest/no-restricted-matchers": "error"
    ///   }
    /// }
    /// ```
    NoRestrictedMatchers,
    jest,
    style,
    config = NoRestrictedMatchersConfig,
);

const MODIFIER_NAME: [&str; 3] = ["not", "rejects", "resolves"];

impl Rule for NoRestrictedMatchers {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = serde_json::from_value::<DefaultRuleConfig<NoRestrictedMatchersConfig>>(value)
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

        for (restriction, message) in self.iter() {
            if Self::check_restriction(chain_call.as_str(), restriction.as_str()) {
                match message.as_deref() {
                    None | Some("") => {
                        ctx.diagnostic(restricted_chain(&chain_call, span));
                    }
                    Some(message) => {
                        ctx.diagnostic(restricted_chain_with_message(&chain_call, message, span));
                    }
                }
            }
        }
    }

    fn check_restriction(chain_call: &str, restriction: &str) -> bool {
        if MODIFIER_NAME.contains(&restriction)
            || Path::new(restriction).extension().is_some_and(|ext| ext.eq_ignore_ascii_case("not"))
        {
            chain_call.starts_with(restriction)
        } else {
            chain_call == restriction
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    // Note: Both Jest and Vitest share the same unit tests

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

    Tester::new(NoRestrictedMatchers::NAME, NoRestrictedMatchers::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .test_and_snapshot();
}

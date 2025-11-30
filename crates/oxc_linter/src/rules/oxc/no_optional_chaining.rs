use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

fn no_optional_chaining_diagnostic(span: Span, help: &str) -> OxcDiagnostic {
    if help.is_empty() {
        OxcDiagnostic::warn("Optional chaining is not allowed.").with_label(span)
    } else {
        OxcDiagnostic::warn("Optional chaining is not allowed.")
            .with_help(help.to_owned())
            .with_label(span)
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoOptionalChaining(Box<NoOptionalChainingConfig>);

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NoOptionalChainingConfig {
    /// A custom help message to display when optional chaining is found.
    /// For example, "Our output target is ES2016, and optional chaining results in verbose
    /// helpers and should be avoided."
    message: String,
}

impl std::ops::Deref for NoOptionalChaining {
    type Target = NoOptionalChainingConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow [optional chaining](https://github.com/tc39/proposal-optional-chaining).
    ///
    /// ### Why is this bad?
    ///
    /// Optional chaining is a relatively new JavaScript feature that may not be
    /// supported in older environments. In some cases, transpiling optional
    /// chaining can result in verbose helper code that impacts bundle size or
    /// performance. This rule is useful when you need to maintain compatibility
    /// with older JavaScript targets or want to avoid the overhead of transpiled
    /// optional chaining.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const foo = obj?.foo;
    /// obj.fn?.();
    /// ```
    NoOptionalChaining,
    oxc,
    restriction,
    config = NoOptionalChainingConfig,
);

impl Rule for NoOptionalChaining {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<NoOptionalChaining>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::ChainExpression(expr) = node.kind() {
            ctx.diagnostic(no_optional_chaining_diagnostic(expr.span, &self.message));
        }
    }
}

// Test cases port from: https://github.com/mysticatea/eslint-plugin-es/blob/v4.1.0/tests/lib/rules/no-optional-chaining.js
#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![("var x = a.b", None), ("var x = a[b]", None), ("foo()", None)];

    let fail = vec![
        ("var x = a?.b", None),
        ("var x = a?.[b]", None),
        ("foo?.()", None),
        ("var x = ((a?.b)?.c)?.()", None),
        ("var x = a/*?.*/?.b", None),
        ("var x = '?.'?.['?.']", None),
        ("var x = '?.'?.['?.']", None),
        ("a?.c?.b<c>", None),
        ("foo?.bar!", None),
        ("foo?.[bar]!", None),
        ("x?.f<T>();", None),
        ("x?.f?.<T>();", None),
        ("f?.<Q>();", None),
        (
            "var x = a?.b",
            Some(serde_json::json!([{
                "message": "Our output target is ES2016, and optional chaining results in verbose helpers and should be avoided."
            }])),
        ),
    ];

    Tester::new(NoOptionalChaining::NAME, NoOptionalChaining::PLUGIN, pass, fail)
        .test_and_snapshot();
}

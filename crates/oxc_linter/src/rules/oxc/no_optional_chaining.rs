use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_optional_chaining_diagnostic(span0: Span, x1: &str) -> OxcDiagnostic {
    if x1.is_empty() {
        OxcDiagnostic::warn("oxc(no-optional-chaining): Optional chaining is not allowed.")
            .with_label(span0)
    } else {
        OxcDiagnostic::warn("oxc(no-optional-chaining): Optional chaining is not allowed.")
            .with_help(x1)
            .with_label(span0)
    }
}

#[derive(Debug, Default, Clone)]
pub struct NoOptionalChaining(Box<NoOptionalChainingConfig>);

#[derive(Debug, Default, Clone)]
pub struct NoOptionalChainingConfig {
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
    /// ### Example
    ///
    /// ```javascript
    /// const foo = obj?.foo;
    /// obj.fn?.();
    /// ```
    ///
    /// ### Options
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///     "no-optional-chaining": [
    ///         "error",
    ///         {
    ///             "message": "Our output target is ES2016, and optional chaining results in verbose
    ///             helpers and should be avoided.",
    ///         }
    ///     ]
    ///   }
    /// }
    /// ```
    ///
    /// - `message`: A custom help message to display when optional chaining is found.
    ///
    NoOptionalChaining,
    restriction,
);

impl Rule for NoOptionalChaining {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value.get(0);
        let message = config
            .and_then(|v| v.get("message"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();

        Self(Box::new(NoOptionalChainingConfig { message: message.to_string() }))
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
        (
            "var x = a?.b",
            Some(serde_json::json!([{
                "message": "Our output target is ES2016, and optional chaining results in verbose helpers and should be avoided."
            }])),
        ),
    ];

    Tester::new(NoOptionalChaining::NAME, pass, fail).test_and_snapshot();
}

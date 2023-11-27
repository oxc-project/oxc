use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
enum ExplicitLengthCheckDiagnostic {
    #[error("eslint-plugin-unicorn(explicit-length-check): Use `.{1} {2}` when checking {1} is not zero.")]
    #[diagnostic(severity(warning))]
    NoneZero(#[label] Span, Atom, Atom, #[help] Option<String>),
    #[error(
        "eslint-plugin-unicorn(explicit-length-check): Use `.{1} {2}` when checking {1} is zero."
    )]
    #[diagnostic(severity(warning))]
    Zero(#[label] Span, Atom, Atom, #[help] Option<String>),
}
#[derive(Debug, Default, Clone)]
enum NonZero {
    #[default]
    GreaterThan,
    NotEqual,
}
impl NonZero {
    pub fn from(raw: &str) -> Self {
        match raw {
            "not-equal" => Self::NotEqual,
            _ => Self::GreaterThan,
        }
    }
}
#[derive(Debug, Default, Clone)]
pub struct ExplicitLengthCheck {
    non_zero: NonZero,
}

declare_oxc_lint!(
    /// ### What it does
    /// Enforce explicitly comparing the length or size property of a value.
    ///
    /// The non-zero option can be configured with one of the following:
    /// greater-than (default)
    ///     Enforces non-zero to be checked with: foo.length > 0
    /// not-equal
    ///     Enforces non-zero to be checked with: foo.length !== 0
    /// ### Example
    /// ```javascript
    /// // fail
    /// const isEmpty = !foo.length;
    /// const isEmpty = foo.length == 0;
    /// const isEmpty = foo.length < 1;
    /// const isEmpty = 0 === foo.length;
    /// const isEmpty = 0 == foo.length;
    /// const isEmpty = 1 > foo.length;
    /// // Negative style is disallowed too
    /// const isEmpty = !(foo.length > 0);
    /// const isEmptySet = !foo.size;
    /// // pass
    /// const isEmpty = foo.length === 0;
    /// ```
    ExplicitLengthCheck,
    pedantic
);

impl Rule for ExplicitLengthCheck {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
    fn from_configuration(value: serde_json::Value) -> Self {
        Self {
            non_zero: value
                .get(0)
                .and_then(serde_json::Value::as_str)
                .map(NonZero::from)
                .unwrap_or_default(),
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Not `.length`
        ("if (foo.notLength) {}", None),
        ("if (length) {}", None),
        ("if (foo[length]) {}", None),
        (r#"if (foo["length"]) {}"#, None),
        // Already in wanted style
        ("foo.length === 0", None),
        ("foo.length > 0", None),
        // Not boolean
        ("const bar = foo.length", None),
        ("const bar = +foo.length", None),
        ("const x = Boolean(foo.length, foo.length)", None),
        ("const x = new Boolean(foo.length)", None),
        ("const x = NotBoolean(foo.length)", None),
        ("const length = foo.length ?? 0", None),
        ("if (foo.length ?? bar) {}", None),
        // Checking 'non-zero'
        ("if (foo.length > 0) {}", None),
        ("if (foo.length > 0) {}", Some(serde_json::json!([{"non-zero": "greater-than"}]))),
        ("if (foo.length !== 0) {}", Some(serde_json::json!([{"non-zero": "not-equal"}]))),
        // Checking "non-zero"
        ("if (foo.length === 0) {}", None),
        // `ConditionalExpression`
        ("const bar = foo.length === 0 ? 1 : 2", None),
        ("while (foo.length > 0) { foo.pop(); }", None),
        ("do { foo.pop(); } while (foo.length > 0);", None),
        // `ForStatement`
        ("for (; foo.length > 0; foo.pop());", None),
        ("if (foo.length !== 1) {}", None),
        ("if (foo.length > 1) {}", None),
        ("if (foo.length < 2) {}", None),
        // With known static length value
        (r#"const foo = { size: "small" }; if (foo.size) {}"#, None), // Not a number
        ("const foo = { length: -1 }; if (foo.length) {}", None), // Array lengths cannot be negative
        ("const foo = { length: 1.5 }; if (foo.length) {}", None), // Array lengths must be integers
        ("const foo = { length: NaN }; if (foo.length) {}", None), // Array lengths cannot be NaN
        ("const foo = { length: Infinity }; if (foo.length) {}", None), // Array lengths cannot be Infinity
        // Logical OR
        ("const x = foo.length || 2", None),
        ("const A_NUMBER = 2; const x = foo.length || A_NUMBER", None),
    ];

    let fail = vec![("const x = foo.length || bar()", None)];
    let fixes = vec![
        ("const x = foo.length || bar()", "const x = foo.length > 0 || bar()", None),
        ("", "", None),
    ];
    Tester::new(ExplicitLengthCheck::NAME, pass, fail).expect_fix(fixes).test_and_snapshot();
}

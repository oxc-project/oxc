use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::UnaryOperator;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_void_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected `void` operator")
        .with_help("Use `undefined` instead")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoVoid {
    pub allow_as_statement: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow `void` operators.
    ///
    /// Why is this bad
    ///
    /// The `void` operator is often used to obtain the `undefined` primitive value, but it is unnecessary. You can use `undefined` directly instead.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    /// void 0;
    /// var foo = void 0;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// "var foo = bar()";
    /// "foo.void()";
    /// "foo.void = bar";
    /// ```
    NoVoid,
    eslint,
    restriction,
    pending // TODO: suggestion
);

impl Rule for NoVoid {
    fn from_configuration(value: serde_json::Value) -> Self {
        let allow_as_statement = value
            .get(0)
            .and_then(|config| config.get("allowAsStatement"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        Self { allow_as_statement }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::UnaryExpression(unary_expr) = node.kind() else {
            return;
        };

        if let Some(kind) = ctx.nodes().parent_kind(node.id()) {
            if self.allow_as_statement && matches!(kind, AstKind::ExpressionStatement(_)) {
                return;
            }
        };

        if unary_expr.operator == UnaryOperator::Void {
            ctx.diagnostic(no_void_diagnostic(Span::new(
                unary_expr.span.start,
                unary_expr.span.start + 4,
            )));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var foo = bar()", None),
        ("foo.void()", None),
        ("foo.void = bar", None),
        ("delete foo;", None),
        ("void 0", Some(serde_json::json!([{ "allowAsStatement": true }]))),
        ("void(0)", Some(serde_json::json!([{ "allowAsStatement": true }]))),
    ];

    let fail = vec![
        ("void 0", None),
        ("void 0", Some(serde_json::json!([{}]))),
        ("void 0", Some(serde_json::json!([{ "allowAsStatement": false }]))),
        ("void(0)", None),
        ("var foo = void 0", None),
        ("var foo = void 0", Some(serde_json::json!([{ "allowAsStatement": true }]))),
    ];

    Tester::new(NoVoid::NAME, NoVoid::PLUGIN, pass, fail).test_and_snapshot();
}

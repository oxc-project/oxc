use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::UnaryOperator;

use crate::{AstNode, ast_util::outermost_paren_parent, context::LintContext, rule::Rule};

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
    /// Disallows the use of the `void` operator.
    ///
    /// ### Why is this bad?
    ///
    /// The `void` operator is often used to get `undefined`, but this is
    /// unnecessary because `undefined` can be used directly instead.
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
    ///
    /// ### Options
    ///
    /// #### allowAsStatement
    ///
    /// `{ type: boolean, default: false }`
    ///
    /// If set to `true`, using `void` as a standalone statement is allowed.
    NoVoid,
    eslint,
    restriction,
    suggestion
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

        if let Some(node) = outermost_paren_parent(node, ctx)
            && self.allow_as_statement
            && matches!(node.kind(), AstKind::ExpressionStatement(_))
        {
            return;
        }

        if unary_expr.operator == UnaryOperator::Void {
            ctx.diagnostic_with_suggestion(no_void_diagnostic(unary_expr.span), |fixer| {
                fixer.replace(unary_expr.span, "undefined")
            });
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
        ("(void 0)", Some(serde_json::json!([{ "allowAsStatement": true }]))),
    ];

    let fail = vec![
        ("void 0", None),
        ("void 0", Some(serde_json::json!([{}]))),
        ("void 0", Some(serde_json::json!([{ "allowAsStatement": false }]))),
        ("void(0)", None),
        ("var foo = void 0", None),
        ("var foo = void 0", Some(serde_json::json!([{ "allowAsStatement": true }]))),
    ];

    let fix = vec![
        ("void 0", "undefined", None),
        ("void 0", "undefined", Some(serde_json::json!([{}]))),
        ("void 0", "undefined", Some(serde_json::json!([{ "allowAsStatement": false }]))),
        ("void(0)", "undefined", None),
        ("var foo = void 0", "var foo = undefined", None),
        (
            "var foo = void 0",
            "var foo = undefined",
            Some(serde_json::json!([{ "allowAsStatement": true }])),
        ),
    ];

    Tester::new(NoVoid::NAME, NoVoid::PLUGIN, pass, fail).expect_fix(fix).test_and_snapshot();
}

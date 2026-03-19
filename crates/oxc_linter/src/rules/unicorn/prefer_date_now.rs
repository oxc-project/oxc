use oxc_ast::{
    AstKind,
    ast::{Argument, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, UnaryOperator};

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_date_now(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `Date.now()` over `new Date()`")
        .with_help("Change to `Date.now()`.")
        .with_label(span)
}

fn prefer_date_now_over_methods(span: Span, bad_method: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `Date.now()` over `new Date().{bad_method}()`"))
        .with_help("Change to `Date.now()`.")
        .with_label(span)
}

fn prefer_date_now_over_number_date_object(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `Date.now()` over `Number(new Date())`")
        .with_help("Change to `Date.now()`.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferDateNow;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers use of `Date.now()` over `new Date().getTime()` or `new Date().valueOf()`.
    ///
    /// ### Why is this bad?
    ///
    /// Using `Date.now()` is shorter and nicer than `new Date().getTime()`, and avoids unnecessary instantiation of `Date` objects.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const ts = new Date().getTime();
    /// const ts = new Date().valueOf();
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const ts = Date.now();
    /// ```
    PreferDateNow,
    unicorn,
    pedantic,
    fix
);

impl Rule for PreferDateNow {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CallExpression(call_expr) => {
                // `new Date().{getTime,valueOf}()`
                if let Some(member_expr) =
                    call_expr.callee.get_inner_expression().as_member_expression()
                    && call_expr.arguments.is_empty()
                    && !member_expr.is_computed()
                    && matches!(member_expr.static_property_name(), Some("getTime" | "valueOf"))
                    && is_new_date(member_expr.object().get_inner_expression())
                {
                    ctx.diagnostic_with_fix(
                        prefer_date_now_over_methods(
                            call_expr.span,
                            member_expr.static_property_name().unwrap(),
                        ),
                        |fixer| fixer.replace(call_expr.span, "Date.now()"),
                    );
                }

                // `{Number,BigInt}(new Date())`
                if let Expression::Identifier(ident) = &call_expr.callee
                    && matches!(ident.name.as_str(), "Number" | "BigInt")
                    && call_expr.arguments.len() == 1
                    && let Some(expr) =
                        call_expr.arguments.first().and_then(Argument::as_expression)
                    && is_new_date(expr.get_inner_expression())
                {
                    ctx.diagnostic_with_fix(
                        prefer_date_now_over_number_date_object(call_expr.span),
                        |fixer| fixer.replace(call_expr.span, "Date.now()"),
                    );
                }
            }
            AstKind::UnaryExpression(unary_expr) => {
                if !matches!(
                    unary_expr.operator,
                    UnaryOperator::UnaryPlus | UnaryOperator::UnaryNegation,
                ) {
                    return;
                }
                if is_new_date(&unary_expr.argument) {
                    ctx.diagnostic_with_fix(prefer_date_now(unary_expr.argument.span()), |fixer| {
                        fixer.replace(unary_expr.span, "Date.now()")
                    });
                }
            }
            AstKind::AssignmentExpression(assignment_expr) => {
                if !matches!(
                    assignment_expr.operator,
                    AssignmentOperator::Subtraction
                        | AssignmentOperator::Multiplication
                        | AssignmentOperator::Division
                        | AssignmentOperator::Remainder
                        | AssignmentOperator::Exponential
                ) {
                    return;
                }

                if is_new_date(&assignment_expr.right) {
                    ctx.diagnostic(prefer_date_now(assignment_expr.right.span()));
                }
            }
            AstKind::BinaryExpression(bin_expr) => {
                if !matches!(
                    bin_expr.operator,
                    BinaryOperator::Subtraction
                        | BinaryOperator::Multiplication
                        | BinaryOperator::Division
                        | BinaryOperator::Remainder
                        | BinaryOperator::Exponential
                ) {
                    return;
                }

                if is_new_date(&bin_expr.left) {
                    ctx.diagnostic(prefer_date_now(bin_expr.left.span()));
                }
                if is_new_date(&bin_expr.right) {
                    ctx.diagnostic(prefer_date_now(bin_expr.right.span()));
                }
            }
            _ => {}
        }
    }
}

fn is_new_date(expr: &Expression) -> bool {
    let Expression::NewExpression(new_expr) = expr.get_inner_expression() else {
        return false;
    };

    if let Expression::Identifier(ident) = &new_expr.callee {
        return ident.name == "Date" && new_expr.arguments.is_empty();
    }
    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const ts = Date.now()",
        "+Date()",
        "+ Date",
        "+ new window.Date()",
        "+ new Moments()",
        "+ new Date(0)",
        "+ new Date(...[])",
        "new Date.getTime()",
        "valueOf()",
        "new Date()[getTime]()",
        r#"new Date()["valueOf"]()"#,
        "new Date().notListed(0)",
        "new Date().getTime(0)",
        "new Date().valueOf(...[])",
        "new Number(new Date())",
        "window.BigInt(new Date())",
        "toNumber(new Date())",
        "BigInt()",
        "Number(new Date(), extraArgument)",
        "BigInt([...new Date()])",
        "throw new Date()",
        "typeof new Date()",
        "const foo = () => {return new Date()}",
        "foo += new Date()",
        "function * foo() {yield new Date()}",
        "new Date() + new Date()",
        "foo = new Date() | 0",
        "foo &= new Date()",
        "foo = new Date() >> 0",
    ];

    let fail = vec![
        "const ts = new Date().getTime();",
        "const ts = (new Date).getTime();",
        "const ts = (new Date()).getTime();",
        "const ts = new Date().valueOf();",
        "const ts = (new Date).valueOf();",
        "const ts = (new Date()).valueOf();",
        "const ts = /* 1 */ Number(/* 2 */ new /* 3 */ Date( /* 4 */ ) /* 5 */) /* 6 */",
        "const tsBigInt = /* 1 */ BigInt(/* 2 */ new /* 3 */ Date( /* 4 */ ) /* 5 */) /* 6 */",
        "const ts = + /* 1 */ new Date;",
        "const ts = - /* 1 */ new Date();",
        "const ts = new Date() - 0",
        "const foo = bar - new Date",
        "const foo = new Date() * bar",
        "const ts = new Date() / 1",
        "const ts = new Date() % Infinity",
        "const ts = new Date() ** 1",
        "const zero = (new Date(/* 1 */) /* 2 */) /* 3 */ - /* 4 */new Date",
        "foo -= new Date()",
        "foo *= new Date()",
        "foo /= new Date",
        "foo %= new Date()",
        "foo **= new Date()",
        "foo **= (new Date())",
        "function foo(){return+new Date}",
        "function foo(){return-new Date}",
    ];

    let fix = vec![
        ("new Date().getTime()", "Date.now()"),
        ("new Date().valueOf()", "Date.now()"),
        ("Number(new Date())", "Date.now()"),
        ("BigInt(new Date())", "Date.now()"),
        ("(new Date() as number).getTime()", "Date.now()"),
        ("(new Date().valueOf() as string)", "(Date.now() as string)"),
        ("(new Date()     ).     getTime()", "Date.now()"),
        ("(new Date().valueOf()       )", "(Date.now()       )"),
        ("Number(new Date()        )", "Date.now()"),
        ("BigInt(new             Date());", "Date.now();"),
        ("BigInt(new Date());", "Date.now();"),
    ];

    Tester::new(PreferDateNow::NAME, PreferDateNow::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

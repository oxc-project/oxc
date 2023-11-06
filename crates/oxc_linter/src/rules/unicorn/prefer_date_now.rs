use oxc_ast::{
    ast::{Argument, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, GetSpan, Span};
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, UnaryOperator};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[allow(clippy::enum_variant_names)]
enum PreferDateNowDiagnostic {
    #[error("eslint-plugin-unicorn(prefer-date-now): Prefer `Date.now()` over `new Date()`")]
    #[diagnostic(severity(warning), help("Change to `Date.now()`."))]
    PreferDateNow(#[label] Span),

    #[error("eslint-plugin-unicorn(prefer-date-now): Prefer `Date.now()` over `new Date().{1}()`")]
    #[diagnostic(severity(warning), help("Change to `Date.now()`."))]
    PreferDateNowOverMethods(#[label] Span, Atom),

    #[error(
        "eslint-plugin-unicorn(prefer-date-now): Prefer `Date.now()` over `Number(new Date())`"
    )]
    #[diagnostic(severity(warning), help("Change to `Date.now()`."))]
    PreferDateNowOverNumberDateObject(#[label] Span),
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
    ///
    /// ### Example
    /// ```javascript
    /// // bad
    /// const ts = new Date().getTime();
    /// const ts = new Date().valueOf();
    ///
    /// // good
    /// const ts = Date.now();
    /// ```
    PreferDateNow,
    correctness
);

impl Rule for PreferDateNow {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CallExpression(call_expr) => {
                // `new Date().{getTime,valueOf}()`
                if let Expression::MemberExpression(member_expr) =
                    &call_expr.callee.without_parenthesized()
                {
                    if call_expr.arguments.is_empty()
                        && !member_expr.is_computed()
                        && matches!(member_expr.static_property_name(), Some("getTime" | "valueOf"))
                        && is_new_date(member_expr.object().without_parenthesized())
                    {
                        ctx.diagnostic(PreferDateNowDiagnostic::PreferDateNowOverMethods(
                            call_expr.span,
                            member_expr.static_property_name().unwrap().into(),
                        ));
                    }
                }

                // `{Number,BigInt}(new Date())`
                if let Expression::Identifier(ident) = &call_expr.callee {
                    if matches!(ident.name.as_str(), "Number" | "BigInt")
                        && call_expr.arguments.len() == 1
                    {
                        if let Some(Argument::Expression(expr)) = call_expr.arguments.get(0) {
                            if is_new_date(expr.without_parenthesized()) {
                                ctx.diagnostic(
                                    PreferDateNowDiagnostic::PreferDateNowOverNumberDateObject(
                                        call_expr.span,
                                    ),
                                );
                            }
                        }
                    }
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
                    ctx.diagnostic(PreferDateNowDiagnostic::PreferDateNow(
                        unary_expr.argument.span(),
                    ));
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
                    ctx.diagnostic(PreferDateNowDiagnostic::PreferDateNow(
                        assignment_expr.right.span(),
                    ));
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
                    ctx.diagnostic(PreferDateNowDiagnostic::PreferDateNow(bin_expr.left.span()));
                }
                if is_new_date(&bin_expr.right) {
                    ctx.diagnostic(PreferDateNowDiagnostic::PreferDateNow(bin_expr.right.span()));
                }
            }
            _ => {}
        }
    }
}

fn is_new_date(expr: &Expression) -> bool {
    let Expression::NewExpression(new_expr) = expr else { return false };

    if let Expression::Identifier(ident) = &new_expr.callee {
        return ident.name == "Date" && new_expr.arguments.is_empty();
    }
    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"const ts = Date.now()"#,
        r#"+Date()"#,
        r#"+ Date"#,
        r#"+ new window.Date()"#,
        r#"+ new Moments()"#,
        r#"+ new Date(0)"#,
        r#"+ new Date(...[])"#,
        r#"new Date.getTime()"#,
        r#"valueOf()"#,
        r#"new Date()[getTime]()"#,
        r#"new Date()["valueOf"]()"#,
        r#"new Date().notListed(0)"#,
        r#"new Date().getTime(0)"#,
        r#"new Date().valueOf(...[])"#,
        r#"new Number(new Date())"#,
        r#"window.BigInt(new Date())"#,
        r#"toNumber(new Date())"#,
        r#"BigInt()"#,
        r#"Number(new Date(), extraArgument)"#,
        r#"BigInt([...new Date()])"#,
        r#"throw new Date()"#,
        r#"typeof new Date()"#,
        r#"const foo = () => {return new Date()}"#,
        r#"foo += new Date()"#,
        r#"function * foo() {yield new Date()}"#,
        r#"new Date() + new Date()"#,
        r#"foo = new Date() | 0"#,
        r#"foo &= new Date()"#,
        r#"foo = new Date() >> 0"#,
    ];

    let fail = vec![
        r#"const ts = new Date().getTime();"#,
        r#"const ts = (new Date).getTime();"#,
        r#"const ts = (new Date()).getTime();"#,
        r#"const ts = new Date().valueOf();"#,
        r#"const ts = (new Date).valueOf();"#,
        r#"const ts = (new Date()).valueOf();"#,
        r#"const ts = /* 1 */ Number(/* 2 */ new /* 3 */ Date( /* 4 */ ) /* 5 */) /* 6 */"#,
        r#"const tsBigInt = /* 1 */ BigInt(/* 2 */ new /* 3 */ Date( /* 4 */ ) /* 5 */) /* 6 */"#,
        r#"const ts = + /* 1 */ new Date;"#,
        r#"const ts = - /* 1 */ new Date();"#,
        r#"const ts = new Date() - 0"#,
        r#"const foo = bar - new Date"#,
        r#"const foo = new Date() * bar"#,
        r#"const ts = new Date() / 1"#,
        r#"const ts = new Date() % Infinity"#,
        r#"const ts = new Date() ** 1"#,
        r#"const zero = (new Date(/* 1 */) /* 2 */) /* 3 */ - /* 4 */new Date"#,
        r#"foo -= new Date()"#,
        r#"foo *= new Date()"#,
        r#"foo /= new Date"#,
        r#"foo %= new Date()"#,
        r#"foo **= new Date()"#,
        r#"function foo(){return+new Date}"#,
        r#"function foo(){return-new Date}"#,
    ];

    Tester::new_without_config(PreferDateNow::NAME, pass, fail).test_and_snapshot();
}

use oxc_ast::{
    ast::{AssignmentOperator, Expression, LogicalOperator},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_throw_literal_diagnostic(span: Span, is_undef: bool) -> OxcDiagnostic {
    let message =
        if is_undef { "Do not throw undefined" } else { "Expected an error object to be thrown" };

    OxcDiagnostic::warn(message).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoThrowLiteral;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows throwing literals or non-Error objects as exceptions.
    ///
    /// ### Why is this bad?
    ///
    /// It is considered good practice to only throw the Error object itself or an object using
    /// the Error object as base objects for user-defined exceptions. The fundamental benefit of
    /// Error objects is that they automatically keep track of where they were built and originated.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// throw "error";
    ///
    /// throw 0;
    ///
    /// throw undefined;
    ///
    /// throw null;
    ///
    /// var err = new Error();
    /// throw "an " + err;
    /// // err is recast to a string literal
    ///
    /// var err = new Error();
    /// throw `${err}`
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// throw new Error();
    ///
    /// throw new Error("error");
    ///
    /// var e = new Error("error");
    /// throw e;
    ///
    /// try {
    ///     throw new Error("error");
    /// } catch (e) {
    ///     throw e;
    /// }
    /// ```
    NoThrowLiteral,
    correctness,
);

impl Rule for NoThrowLiteral {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ThrowStatement(stmt) = node.kind() else {
            return;
        };

        let expr = &stmt.argument;

        if !Self::could_be_error(expr) {
            ctx.diagnostic(no_throw_literal_diagnostic(expr.span(), false));
        } else if matches!(expr, Expression::Identifier(id) if id.name == "undefined") {
            ctx.diagnostic(no_throw_literal_diagnostic(expr.span(), true));
        };
    }
}

impl NoThrowLiteral {
    fn could_be_error(expr: &Expression) -> bool {
        match expr.get_inner_expression() {
            Expression::Identifier(_)
            | Expression::NewExpression(_)
            | Expression::AwaitExpression(_)
            | Expression::CallExpression(_)
            | Expression::ChainExpression(_)
            | Expression::YieldExpression(_)
            | Expression::PrivateFieldExpression(_)
            | Expression::StaticMemberExpression(_)
            | Expression::ComputedMemberExpression(_)
            | Expression::TaggedTemplateExpression(_) => true,
            Expression::AssignmentExpression(expr) => {
                if matches!(
                    expr.operator,
                    AssignmentOperator::Assign | AssignmentOperator::LogicalAnd
                ) {
                    return Self::could_be_error(&expr.right);
                }

                if matches!(
                    expr.operator,
                    AssignmentOperator::LogicalOr | AssignmentOperator::LogicalNullish
                ) {
                    return expr
                        .left
                        .get_expression()
                        .map_or(true, |expr| Self::could_be_error(expr))
                        || Self::could_be_error(&expr.right);
                }

                false
            }
            Expression::SequenceExpression(expr) => {
                if expr.expressions.len() > 0 {
                    return Self::could_be_error(expr.expressions.last().unwrap());
                }

                false
            }
            Expression::LogicalExpression(expr) => {
                if matches!(expr.operator, LogicalOperator::And) {
                    return Self::could_be_error(&expr.right);
                }

                Self::could_be_error(&expr.left) || Self::could_be_error(&expr.right)
            }
            Expression::ConditionalExpression(expr) => {
                Self::could_be_error(&expr.consequent) || Self::could_be_error(&expr.alternate)
            }
            _ => false,
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "throw new Error();",
        "throw new Error('error');",
        "throw Error('error');",
        "var e = new Error(); throw e;",
        "try {throw new Error();} catch (e) {throw e;};",
        "throw a;",
        "throw foo();",
        "throw new foo();",
        "throw foo.bar;",
        "throw foo[bar];",
        "class C { #field; foo() { throw foo.#field; } }", // { "ecmaVersion": 2022 },
        "throw foo = new Error();",
        "throw foo.bar ||= 'literal'",  // { "ecmaVersion": 2021 },
        "throw foo[bar] ??= 'literal'", // { "ecmaVersion": 2021 },
        "throw 1, 2, new Error();",
        "throw 'literal' && new Error();",
        "throw new Error() || 'literal';",
        "throw foo ? new Error() : 'literal';",
        "throw foo ? 'literal' : new Error();",
        "throw tag `${foo}`;", // { "ecmaVersion": 6 },
        "function* foo() { var index = 0; throw yield index++; }", // { "ecmaVersion": 6 },
        "async function foo() { throw await bar; }", // { "ecmaVersion": 8 },
        "throw obj?.foo",      // { "ecmaVersion": 2020 },
        "throw obj?.foo()",    // { "ecmaVersion": 2020 }
        "throw obj?.foo() as string",
        "throw obj?.foo() satisfies Direction",
    ];

    let fail = vec![
        "throw 'error';",
        "throw 0;",
        "throw false;",
        "throw null;",
        "throw {};",
        "throw undefined;",
        "throw 'a' + 'b';",
        "var b = new Error(); throw 'a' + b;",
        "throw foo = 'error';",
        "throw foo += new Error();",
        "throw foo &= new Error();",
        "throw foo &&= 'literal'", // { "ecmaVersion": 2021 },
        "throw new Error(), 1, 2, 3;",
        "throw 'literal' && 'not an Error';",
        "throw foo && 'literal'",
        "throw foo ? 'not an Error' : 'literal';",
        "throw `${err}`;", // { "ecmaVersion": 6 }
        "throw 0 as number",
        "throw 'error' satisfies Error",
    ];

    Tester::new(NoThrowLiteral::NAME, pass, fail).test_and_snapshot();
}

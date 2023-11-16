use oxc_ast::{
    ast::{Expression, Statement},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(no-negated-condition): Unexpected negated condition.")]
#[diagnostic(
    severity(warning),
    help("Remove the negation operator and switch the consequent and alternate branches.")
)]
struct NoNegatedConditionDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoNegatedCondition;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow negated conditions.
    ///
    /// ### Why is this bad?
    ///
    /// Negated conditions are more difficult to understand. Code can be made more readable by inverting the condition.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    ///
    /// if (!a) {
    /// 	doSomethingC();
    /// } else {
    /// 	doSomethingB();
    /// }
    ///
    /// !a ? doSomethingC() : doSomethingB()
    ///
    /// // Good
    ///
    /// if (a) {
    /// 	doSomethingB();
    /// } else {
    /// 	doSomethingC();
    /// }
    ///
    /// a ? doSomethingB() : doSomethingC()
    /// ```
    NoNegatedCondition,
    pedantic
);

impl Rule for NoNegatedCondition {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let stmt_test = match node.kind() {
            AstKind::IfStatement(if_stmt) => {
                let Some(if_stmt_alternate) = &if_stmt.alternate else { return };

                if matches!(if_stmt_alternate, Statement::IfStatement(_)) {
                    return;
                }

                if_stmt.test.without_parenthesized()
            }
            AstKind::ConditionalExpression(conditional_expr) => {
                conditional_expr.test.without_parenthesized()
            }
            _ => {
                return;
            }
        };

        match stmt_test {
            Expression::UnaryExpression(unary_expr) => {
                if unary_expr.operator != UnaryOperator::LogicalNot {
                    return;
                }
            }
            Expression::BinaryExpression(binary_expr) => {
                if !matches!(
                    binary_expr.operator,
                    BinaryOperator::Inequality | BinaryOperator::StrictInequality
                ) {
                    return;
                }
            }
            _ => {
                return;
            }
        }

        ctx.diagnostic(NoNegatedConditionDiagnostic(stmt_test.span()));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"if (a) {}",
        r"if (a) {} else {}",
        r"if (!a) {}",
        r"if (!a) {} else if (b) {}",
        r"if (!a) {} else if (b) {} else {}",
        r"if (a == b) {}",
        r"if (a == b) {} else {}",
        r"if (a != b) {}",
        r"if (a != b) {} else if (b) {}",
        r"if (a != b) {} else if (b) {} else {}",
        r"if (a !== b) {}",
        r"if (a === b) {} else {}",
        r"a ? b : c",
    ];

    let fail = vec![
        r"if (!a) {;} else {;}",
        r"if (a != b) {;} else {;}",
        r"if (a !== b) {;} else {;}",
        r"!a ? b : c",
        r"a != b ? c : d",
        r"a !== b ? c : d",
        r"(( !a )) ? b : c",
        r"!(( a )) ? b : c",
        r"if(!(( a ))) b(); else c();",
        r"if((( !a ))) b(); else c();",
        r"function a() {return!a ? b : c}",
        r"function a() {return!(( a )) ? b : c}",
        r"!a ? b : c ? d : e",
        r"!a ? b : (( c ? d : e ))",
        r"if(!a) b(); else c()",
        r"if(!a) {b()} else {c()}",
        r"if(!!a) b(); else c();",
        r"(!!a) ? b() : c();",
    ];

    Tester::new_without_config(NoNegatedCondition::NAME, pass, fail).test_and_snapshot();
}

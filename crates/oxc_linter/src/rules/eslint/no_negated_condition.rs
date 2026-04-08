use oxc_ast::{
    AstKind,
    ast::{Expression, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_negated_condition_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected negated condition.")
        .with_help("Remove the negation operator and switch the consequent and alternate branches.")
        .with_label(span)
}

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
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// if (!a) {
    /// 	doSomethingC();
    /// } else {
    /// 	doSomethingB();
    /// }
    ///
    /// !a ? doSomethingC() : doSomethingB()
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// if (a) {
    /// 	doSomethingB();
    /// } else {
    /// 	doSomethingC();
    /// }
    ///
    /// a ? doSomethingB() : doSomethingC()
    /// ```
    NoNegatedCondition,
    eslint,
    pedantic,
    suggestion
);

impl Rule for NoNegatedCondition {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::IfStatement(if_stmt) => {
                let Some(if_stmt_alternate) = &if_stmt.alternate else {
                    return;
                };

                if matches!(if_stmt_alternate, Statement::IfStatement(_)) {
                    return;
                }

                let test = if_stmt.test.without_parentheses();
                if is_negated_expression(test) {
                    let consequent_span = if_stmt.consequent.span();
                    let alternate_span = if_stmt_alternate.span();
                    let test_span = test.span();
                    ctx.diagnostic_with_suggestion(
                        no_negated_condition_diagnostic(test_span),
                        |fixer| {
                            let inverted_condition =
                                get_inverted_condition(test, fixer.source_text());
                            let consequent_text = fixer.source_range(consequent_span).to_string();
                            let alternate_text = fixer.source_range(alternate_span).to_string();
                            let mut fix = fixer.new_fix_with_capacity(3);
                            fix.push(fixer.replace(test_span, inverted_condition));
                            fix.push(fixer.replace(consequent_span, alternate_text));
                            fix.push(fixer.replace(alternate_span, consequent_text));
                            fix.with_message("Invert the condition and swap the branches")
                        },
                    );
                }
            }
            AstKind::ConditionalExpression(conditional_expr) => {
                let test = conditional_expr.test.without_parentheses();
                if is_negated_expression(test) {
                    let consequent_span = conditional_expr.consequent.span();
                    let alternate_span = conditional_expr.alternate.span();
                    let test_span = test.span();
                    ctx.diagnostic_with_suggestion(
                        no_negated_condition_diagnostic(test_span),
                        |fixer| {
                            let inverted_condition =
                                get_inverted_condition(test, fixer.source_text());
                            let consequent_text = fixer.source_range(consequent_span).to_string();
                            let alternate_text = fixer.source_range(alternate_span).to_string();
                            let mut fix = fixer.new_fix_with_capacity(3);
                            fix.push(fixer.replace(test_span, inverted_condition));
                            fix.push(fixer.replace(consequent_span, alternate_text));
                            fix.push(fixer.replace(alternate_span, consequent_text));
                            fix.with_message("Invert the condition and swap the branches")
                        },
                    );
                }
            }
            _ => {}
        }
    }
}

fn get_inverted_condition(expr: &Expression, source_text: &str) -> String {
    match expr {
        Expression::UnaryExpression(unary_expr) => {
            // !a -> a, !!a -> !a (remove one level of negation)
            unary_expr.argument.span().source_text(source_text).to_string()
        }
        Expression::BinaryExpression(binary_expr) => {
            let left = binary_expr.left.span().source_text(source_text);
            let right = binary_expr.right.span().source_text(source_text);
            let new_op = match binary_expr.operator {
                BinaryOperator::Inequality => "==",
                BinaryOperator::StrictInequality => "===",
                _ => return expr.span().source_text(source_text).to_string(),
            };
            format!("{left} {new_op} {right}")
        }
        _ => expr.span().source_text(source_text).to_string(),
    }
}

fn is_negated_expression(expr: &Expression) -> bool {
    match expr {
        Expression::UnaryExpression(unary_expr) => unary_expr.operator == UnaryOperator::LogicalNot,
        Expression::BinaryExpression(binary_expr) => matches!(
            binary_expr.operator,
            BinaryOperator::Inequality | BinaryOperator::StrictInequality
        ),
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "if (a) {}",
        "if (a) {} else {}",
        "if (!a) {}",
        "if (!a) {} else if (b) {}",
        "if (!a) {} else if (b) {} else {}",
        "if (a == b) {}",
        "if (a == b) {} else {}",
        "if (a != b) {}",
        "if (a != b) {} else if (b) {}",
        "if (a != b) {} else if (b) {} else {}",
        "if (a !== b) {}",
        "if (a === b) {} else {}",
        "a ? b : c",
        // Test cases from eslint-plugin-unicorn
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
        "if (!a) {;} else {;}",
        "if (a != b) {;} else {;}",
        "if (a !== b) {;} else {;}",
        "!a ? b : c",
        "a != b ? c : d",
        "a !== b ? c : d",
        // Test cases from eslint-plugin-unicorn
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

    let fix = vec![
        ("if (!a) {;} else {;}", "if (a) {;} else {;}"),
        ("if (a != b) {;} else {;}", "if (a == b) {;} else {;}"),
        ("if (a !== b) {;} else {;}", "if (a === b) {;} else {;}"),
        ("!a ? b : c", "a ? c : b"),
        ("a != b ? c : d", "a == b ? d : c"),
        ("a !== b ? c : d", "a === b ? d : c"),
    ];

    Tester::new(NoNegatedCondition::NAME, NoNegatedCondition::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

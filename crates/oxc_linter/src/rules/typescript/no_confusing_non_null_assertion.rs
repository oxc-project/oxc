use oxc_ast::{
    AstKind,
    ast::{ChainElement, Expression, SimpleAssignmentTarget},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

#[derive(Debug, Default, Clone)]
pub struct NoConfusingNonNullAssertion;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow non-null assertion in locations that may be confusing.
    ///
    /// ### Why is this bad?
    ///
    /// Using a non-null assertion (`!`) next to an assignment or equality check (`=` or `==` or
    /// `===`) creates code that is confusing as it looks similar to an inequality check (`!=` or
    /// `!==`). Using one next to an `in` or `instanceof` check is also confusing because it may
    /// look like the operator is negated.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```ts
    ///    a! == b; // a non-null assertions(`!`) and an equals test(`==`)
    ///    a !== b; // not equals test(`!==`)
    ///    a! === b; // a non-null assertions(`!`) and an triple equals test(`===`)
    ///    a! in b;
    ///    a! instanceof b;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// a == b;
    /// a !== b;
    /// a === b;
    /// ```
    NoConfusingNonNullAssertion,
    typescript,
    suspicious,
    suggestion,
    version = "0.6.1",
    short_description = "Disallow non-null assertion in locations that may be confusing.",
);

fn not_need_no_confusing_non_null_assertion_diagnostic(op_str: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        r"Confusing combinations of non-null assertion and equal test like `a! {op_str} b`, which looks very similar to not equal `a !{op_str} b`."
    ))
    .with_help(r"Remove the `!`, or prefix the `=` with it.")
    .with_label(span)
}

fn wrap_up_no_confusing_non_null_assertion_diagnostic(op_str: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        r"Confusing combinations of non-null assertion and equal test like `a! {op_str} b`, which looks very similar to not equal `a !{op_str} b`."
    ))
    .with_help(
        r"Wrap left-hand side in parentheses to avoid putting non-null assertion `!` and `=` together.",
    )
    .with_label(span)
}

fn confusing_non_null_assignment_assertion_diagnostic(op_str: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        r"Confusing combinations of non-null assertion and assignment like `a! {op_str} b`, which looks very similar to not equal `a !{op_str} b`."
    ))
    .with_help(r"Remove the `!`, or wrap the left-hand side in parentheses.")
    .with_label(span)
}

fn confusing_non_null_operator_diagnostic(op_str: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Confusing combination of non-null assertion and `{op_str}` operator like `a! {op_str} b`, which might be misinterpreted as `!(a {op_str} b)`."
    ))
    .with_help("Remove the `!`, or wrap the left-hand side in parentheses.")
    .with_label(span)
}

fn get_depth_ends_in_bang(expr: &Expression<'_>) -> Option<u32> {
    match expr {
        Expression::TSNonNullExpression(_) => Some(0),
        Expression::ChainExpression(chain_expr) => {
            matches!(&chain_expr.expression, ChainElement::TSNonNullExpression(_)).then_some(0)
        }
        Expression::BinaryExpression(binary_expr) => {
            get_depth_ends_in_bang(&binary_expr.right).map(|x| x + 1)
        }
        Expression::UnaryExpression(unary_expr) => {
            get_depth_ends_in_bang(&unary_expr.argument).map(|x| x + 1)
        }
        Expression::AssignmentExpression(assignment_expr) => {
            get_depth_ends_in_bang(&assignment_expr.right).map(|x| x + 1)
        }
        _ => None,
    }
}

fn is_confusable_operator(operator: BinaryOperator) -> bool {
    matches!(
        operator,
        BinaryOperator::Equality
            | BinaryOperator::StrictEquality
            | BinaryOperator::In
            | BinaryOperator::Instanceof
    )
}

impl Rule for NoConfusingNonNullAssertion {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::BinaryExpression(binary_expr)
                if is_confusable_operator(binary_expr.operator) =>
            {
                let Some(bang_depth) = get_depth_ends_in_bang(&binary_expr.left) else {
                    return;
                };
                let operator = binary_expr.operator.as_str();
                let left_span = binary_expr.left.span();
                if matches!(binary_expr.operator, BinaryOperator::In | BinaryOperator::Instanceof) {
                    let diagnostic =
                        confusing_non_null_operator_diagnostic(operator, binary_expr.span);
                    if bang_depth == 0 {
                        let fixer = RuleFixer::new(FixKind::Suggestion, ctx);
                        ctx.diagnostic_with_suggestions(
                            diagnostic,
                            [
                                remove_non_null_assertion_fix(fixer, left_span).with_message(
                                    format!(
                                        "Remove possibly unnecessary non-null assertion (!) in the left operand of the `{operator}` operator."
                                    ),
                                ),
                                wrap_left_fix(fixer, left_span, operator),
                            ],
                        );
                    } else {
                        ctx.diagnostic_with_suggestion(diagnostic, |fixer| {
                            wrap_left_fix(fixer, left_span, operator)
                        });
                    }
                } else if bang_depth == 0 {
                    ctx.diagnostic_with_suggestion(
                        not_need_no_confusing_non_null_assertion_diagnostic(
                            operator,
                            binary_expr.span,
                        ),
                        |fixer| {
                            remove_non_null_assertion_fix(fixer, left_span).with_message(
                                "Remove unnecessary non-null assertion (!) in equality test.",
                            )
                        },
                    );
                } else {
                    ctx.diagnostic_with_suggestion(
                        wrap_up_no_confusing_non_null_assertion_diagnostic(
                            operator,
                            binary_expr.span,
                        ),
                        |fixer| wrap_left_fix(fixer, left_span, operator),
                    );
                }
            }
            AstKind::AssignmentExpression(assignment_expr)
                if assignment_expr.operator == AssignmentOperator::Assign =>
            {
                let Some(simple_target) = assignment_expr.left.as_simple_assignment_target() else {
                    return;
                };
                let SimpleAssignmentTarget::TSNonNullExpression(non_null_expr) = simple_target
                else {
                    return;
                };
                ctx.diagnostic_with_suggestion(
                    confusing_non_null_assignment_assertion_diagnostic(
                        assignment_expr.operator.as_str(),
                        assignment_expr.span,
                    ),
                    |fixer| {
                        remove_non_null_assertion_fix(fixer, non_null_expr.span).with_message(
                            "Remove unnecessary non-null assertion (!) in assignment left-hand side.",
                        )
                    },
                );
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_typescript()
    }
}

fn remove_non_null_assertion_fix(fixer: RuleFixer<'_, '_>, span: Span) -> RuleFix {
    fixer.delete_range(Span::sized(span.end - 1, 1))
}

fn wrap_left_fix(fixer: RuleFixer<'_, '_>, span: Span, operator: &str) -> RuleFix {
    let fixer = fixer.for_multifix();
    let mut fix = fixer.new_fix_with_capacity(2);
    fix.push(fixer.insert_text_before_range(span, "("));
    fix.push(fixer.insert_text_after_range(span, ")"));
    fix.with_message(format!(
        r#"Wrap the left-hand side in parentheses to avoid confusion with "{operator}" operator."#
    ))
}

#[test]
fn test() {
    use crate::tester::{ExpectFixTestCase, Tester};

    let pass = vec![
        "a == b!;",
        "a = b!;",
        "a !== b;",
        "a != b;",
        "(a + b!) == c;",
        "(a + b!) in c;",
        "(a || b!) instanceof c;",
        "a! + b;",
        "a! += b;",
        "a! - b;",
        "a! -= b;",
        "a! / b;",
        "a! /= b;",
        "a! * b;",
        "a! *= b;",
        "a! ** b;",
        "a! **= b;",
        "a! != b;",
        "a! !== b;",
    ];

    let fail = vec![
        "a! == b;",
        "a! === b;",
        "a + b! == c;",
        "(obj = new new OuterObj().InnerObj).Name! == c;",
        "(a==b)! ==c;",
        "a! = b;",
        "(obj = new new OuterObj().InnerObj).Name! = c;",
        "a! in b;",
        "a !in b;",
        "a! instanceof b;",
        "foo?.bar! in obj;",
        "foo?.bar! instanceof C;",
    ];

    let fix: Vec<ExpectFixTestCase> = vec![
        ("a! == b;", "a == b;").into(),
        ("a! === b;", "a === b;").into(),
        ("a + b! == c;", "(a + b!) == c;").into(),
        (
            "(obj = new new OuterObj().InnerObj).Name! == c;",
            "(obj = new new OuterObj().InnerObj).Name == c;",
        )
            .into(),
        ("(a==b)! ==c;", "(a==b) ==c;").into(),
        ("a! = b;", "a = b;").into(),
        (
            "(obj = new new OuterObj().InnerObj).Name! = c;",
            "(obj = new new OuterObj().InnerObj).Name = c;",
        )
            .into(),
        ("a! in b;", ("a in b;", "(a!) in b;")).into(),
        ("a !in b;", ("a in b;", "(a !)in b;")).into(),
        ("a! instanceof b;", ("a instanceof b;", "(a!) instanceof b;")).into(),
        ("foo?.bar! in obj;", ("foo?.bar in obj;", "(foo?.bar!) in obj;")).into(),
        ("foo?.bar! instanceof C;", ("foo?.bar instanceof C;", "(foo?.bar!) instanceof C;")).into(),
    ];

    Tester::new(NoConfusingNonNullAssertion::NAME, NoConfusingNonNullAssertion::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

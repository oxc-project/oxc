use oxc_ast::{
    ast::{Expression, SimpleAssignmentTarget},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Default, Clone)]
pub struct NoConfusingNonNullAssertion;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow non-null assertion in locations that may be confusing.
    ///
    /// ### Why is this bad?
    /// Using a non-null assertion (!) next to an assign or equals check (= or == or ===) creates code that is confusing as it looks similar to a not equals check (!= !==).
    ///
    /// ### Example
    /// ```javascript
    ///    a! == b; // a non-null assertions(`!`) and an equals test(`==`)
    ///    a !== b; // not equals test(`!==`)
    ///    a! === b; // a non-null assertions(`!`) and an triple equals test(`===`)
    /// ```
    NoConfusingNonNullAssertion,
    suspicious,
    // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
);

fn not_need_no_confusing_non_null_assertion_diagnostic(op_str: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
            "Confusing combinations of non-null assertion and equal test like \"a! {op_str} b\", which looks very similar to not equal \"a !{op_str} b\"."
    ))
        .with_help(
            if op_str == "=" {
                "Unnecessary non-null assertion (!) in assignment left hand."
            }
            else {
                "Unnecessary non-null assertion (!) in equal test"
            })
    .with_label(span)
}

fn wrap_up_no_confusing_non_null_assertion_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Confusing combinations of non-null assertion and equal test like \"a! = b\", which looks very similar to not equal \"a != b\"."
    )
    .with_help("Wrap left-hand side in parentheses to avoid putting non-null assertion \"!\" and \"=\" together.")
    .with_label(span)
}

fn ends_in_bang(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::TSNonNullExpression(_) => true,
        Expression::BinaryExpression(binary_expr) => ends_in_bang(&binary_expr.right),
        Expression::UnaryExpression(unary_expr) => ends_in_bang(&unary_expr.argument),
        Expression::AssignmentExpression(assignment_expr) => ends_in_bang(&assignment_expr.right),
        _ => false,
    }
}


impl Rule for NoConfusingNonNullAssertion {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::BinaryExpression(binary_expr) => {
                if !ends_in_bang(&binary_expr.left) {
                    return;
                }
                ctx.diagnostic(not_need_no_confusing_non_null_assertion_diagnostic(
                    binary_expr.operator.as_str(),
                    binary_expr.span,
                ));
            }
            AstKind::AssignmentExpression(assignment_expr) => {
                let Some(simple_target) = assignment_expr.left.as_simple_assignment_target() else {return;};
                let SimpleAssignmentTarget::TSNonNullExpression(_) = simple_target else {return};
                ctx.diagnostic_with_fix(
                    wrap_up_no_confusing_non_null_assertion_diagnostic(assignment_expr.span),
                    |fixer| {
                        vec![
                            fixer.insert_text_before(&assignment_expr.left, "("),
                            fixer.insert_text_after(&assignment_expr.left, ")"),
                        ]
                    },
                );
            }
            _ => {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec!["a == b!;", "a = b!;", "a !== b;", "a != b;", "(a + b!) == c;"]; // "(a + b!) = c;"]; that's a parse error??
    let fail = vec![
        "a! == b;",
        "a! === b;",
        "a + b! == c;",
        "(obj = new new OuterObj().InnerObj).Name! == c;",
        "(a==b)! ==c;",
        "a! = b;",
        "(obj = new new OuterObj().InnerObj).Name! = c;",
        "(a=b)! =c;",
    ];
    let fix = vec![
    // source, expected, rule_config?
    ("f = 1 + d! == 2", "f = (1 + d!) == 2", None),
    ("f =  d! == 2", "f = d == 2", None)
    ];
    Tester::new(NoConfusingNonNullAssertion::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}

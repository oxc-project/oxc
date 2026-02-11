use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{AssignmentOperator, BinaryOperator, UnaryOperator};

use crate::{
    AstNode, context::LintContext, fixer::RuleFixer, rule::Rule, utils::pad_fix_with_token_boundary,
};

fn prefer_math_trunc_diagnostic(span: Span, bad_op: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `Math.trunc()` over instead of `{bad_op} 0`."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferMathTrunc;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers use of [`Math.trunc()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/trunc) instead of bitwise operations for clarity and more reliable results.
    ///
    /// It prevents the use of the following bitwise operations:
    /// - `x | 0` ([`bitwise OR`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_OR) with 0)
    /// - `~~x` (two [`bitwise NOT`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_NOT))
    /// - `x >> 0` ([`Signed Right Shift`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Right_shift) with 0)
    /// - `x << 0` ([`Left Shift`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Left_shift) with 0)
    /// - `x ^ 0` ([`bitwise XOR Shift`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Bitwise_XOR) with 0)
    ///
    /// ### Why is this bad?
    ///
    /// Using bitwise operations to truncate numbers is not clear and do not work in [some cases](https://stackoverflow.com/a/34706108/11687747).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const foo = 1.1 | 0;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const foo = Math.trunc(1.1);
    /// ```
    PreferMathTrunc,
    unicorn,
    pedantic,
    suggestion
);

impl Rule for PreferMathTrunc {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (operator, argument_span, is_assignment) = match node.kind() {
            AstKind::UnaryExpression(unary_expr) => {
                if !matches!(unary_expr.operator, UnaryOperator::BitwiseNot) {
                    return;
                }
                let Expression::UnaryExpression(inner_unary_expr) = &unary_expr.argument else {
                    return;
                };
                if !matches!(inner_unary_expr.operator, UnaryOperator::BitwiseNot) {
                    return;
                }

                if let Expression::UnaryExpression(inner_inner_unary_expr) =
                    &inner_unary_expr.argument
                    && matches!(inner_inner_unary_expr.operator, UnaryOperator::BitwiseNot)
                {
                    return;
                }

                (UnaryOperator::BitwiseNot.as_str(), inner_unary_expr.argument.span(), false)
            }
            AstKind::BinaryExpression(bin_expr) => {
                let Expression::NumericLiteral(right_num_lit) = &bin_expr.right else {
                    return;
                };
                if right_num_lit.value != 0.0 {
                    return;
                }
                if !matches!(
                    bin_expr.operator,
                    BinaryOperator::BitwiseOR
                        | BinaryOperator::ShiftRight
                        | BinaryOperator::ShiftRightZeroFill
                        | BinaryOperator::ShiftLeft
                        | BinaryOperator::BitwiseXOR
                ) {
                    return;
                }

                (bin_expr.operator.as_str(), bin_expr.left.span(), false)
            }
            AstKind::AssignmentExpression(assignment_expr) => {
                let Expression::NumericLiteral(right_num_lit) = &assignment_expr.right else {
                    return;
                };

                if right_num_lit.value != 0.0 {
                    return;
                }

                if !matches!(
                    assignment_expr.operator,
                    AssignmentOperator::BitwiseOR
                        | AssignmentOperator::ShiftRight
                        | AssignmentOperator::ShiftRightZeroFill
                        | AssignmentOperator::ShiftLeft
                        | AssignmentOperator::BitwiseXOR
                ) {
                    return;
                }

                (assignment_expr.operator.as_str(), assignment_expr.left.span(), true)
            }
            _ => return,
        };

        let span = node.kind().span();
        ctx.diagnostic_with_suggestion(
            prefer_math_trunc_diagnostic(span, operator),
            |fixer: RuleFixer<'_, 'a>| {
                let argument_text = ctx.source_range(argument_span);
                let mut replacement = if is_assignment {
                    // `x |= 0` -> `x = Math.trunc(x)`
                    format!("{argument_text} = Math.trunc({argument_text})")
                } else {
                    // `x | 0` or `~~x` -> `Math.trunc(x)`
                    format!("Math.trunc({argument_text})")
                };
                pad_fix_with_token_boundary(ctx.source_text(), span, &mut replacement);
                fixer.replace(span, replacement)
            },
        );
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"const foo = 1 | 1;",
        r"const foo = 0 | 1;",
        r"const foo = 1.4 | +0;",
        r"const foo = 1.4 | -0;",
        r"const foo = 1.4 | (.5 - 0.5);",
        r"const foo = 1.4 & 0xFFFFFFFF",
        r"const foo = 1.4 & 0xFF",
        r"const foo = 1.4 & 0x0",
        r"const foo = 1.4 & 0",
        r"const foo = ~3.9;",
        r"const foo = 1.1 >> 1",
        r"const foo = 0 << 1",
    ];

    let fail = vec![
        r"const foo = 1.1 | 0;",
        r"const foo = 111 | 0;",
        r"const foo = (1 + 2 / 3.4) | 0;",
        r"const foo = bar((1.4 | 0) + 2);",
        r"const foo = (0, 1.4) | 0;",
        r"function foo() {return.1 | 0;}",
        r"const foo = 1.4 | 0.;",
        r"const foo = 1.4 | .0;",
        r"const foo = 1.4 | 0.0000_0000_0000;",
        r"const foo = 1.4 | 0b0;",
        r"const foo = 1.4 | 0x0000_0000_0000;",
        r"const foo = 1.4 | 0o0;",
        r"const foo = 1.23 | 0 | 4;",
        r"const foo = ~~3.9;",
        r"const foo = ~~111;",
        r"const foo = ~~(1 + 2 / 3.4);",
        r"const foo = ~~1 + 2 / 3.4;",
        r"const foo = ~~(0, 1.4);",
        r"const foo = ~~~10.01;",
        r"const foo = ~~(~10.01);",
        r"const foo = ~(~~10.01);",
        r"const foo = ~~-10.01;",
        r"const foo = ~~~~10.01;",
        r"function foo() {return~~3.9;}",
        r"const foo = bar >> 0;",
        r"const foo = bar << 0;",
        r"const foo = bar ^ 0;",
        r"function foo() {return.1 ^0;}",
        r"function foo() {return[foo][0] ^= 0;};",
        r"const foo = /* first comment */ 3.4 | 0; // A B C",
        r"const foo = /* first comment */ ~~3.4; // A B C",
        r"const foo = /* will keep */ 3.4 /* will remove 1 */ | /* will remove 2 */ 0;",
        r"const foo = /* will keep */ ~ /* will remove 1 */ ~ /* will remove 2 */ 3.4;",
        r"const foo = ~~bar | 0;",
        r"const foo = ~~(bar| 0);",
        r"const foo = bar | 0 | 0;",
        r"const foo = ~~~~((bar | 0 | 0) >> 0 >> 0 << 0 << 0 ^ 0 ^0);",
    ];

    let fix = vec![
        (r"const foo = 1.1 | 0;", r"const foo = Math.trunc(1.1);"),
        (r"const foo = ~~3.9;", r"const foo = Math.trunc(3.9);"),
        (r"function foo() {return.1 | 0;}", r"function foo() {return Math.trunc(.1);}"),
        (r"function foo() {return~~3.9;}", r"function foo() {return Math.trunc(3.9);}"),
        (r"const foo = bar >> 0;", r"const foo = Math.trunc(bar);"),
        (r"const foo = bar << 0;", r"const foo = Math.trunc(bar);"),
        (r"const foo = bar ^ 0;", r"const foo = Math.trunc(bar);"),
        (
            r"function foo() {return[foo][0] ^= 0;};",
            r"function foo() {return[foo][0] = Math.trunc([foo][0]);};",
        ),
    ];

    Tester::new(PreferMathTrunc::NAME, PreferMathTrunc::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

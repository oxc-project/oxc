use oxc_ast::{
    ast::{Argument, BinaryExpression, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::BinaryOperator;

use crate::{
    ast_util::is_method_call, context::LintContext, rule::Rule, utils::is_same_expression, AstNode,
};

fn prefer_math_abs(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `Math.abs(x)` over alternatives").with_label(span)
}

fn prefer_math_hypot(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `Math.hypot(…)` over alternatives").with_label(span)
}

fn prefer_math_log_n(span: Span, good_method: &str, bad_method: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `Math.{good_method}(x)` over `{bad_method}`"))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferModernMathApis;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Checks for usage of legacy patterns for mathematical operations.
    ///
    /// ### Why is this bad?
    ///
    /// Modern JavaScript provides more concise and readable alternatives to legacy patterns.
    ///
    /// Currently, the following cases are checked:
    ///  - Prefer `Math.log10(x)` over alternatives
    ///  - Prefer `Math.hypot(…)` over alternatives
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// Math.log(x) * Math.LOG10E;
    /// Math.sqrt(a * a + b * b);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// Math.log10(x);
    /// Math.hypot(a, b);
    /// ```
    PreferModernMathApis,
    unicorn,
    restriction,
    pending
);

impl Rule for PreferModernMathApis {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // there are two main cases to check:
        // Bin expression:
        //     `Math.log(x) * Math.LOG10E`
        //     `Math.LOG10E * Math.log(x)`
        //     `Math.log(x) / Math.LN10`
        //     `Math.log(x) * Math.LOG2E`
        //     `Math.log(x) / Math.LN2`
        //     `Math.log(x) * Math.LOG10E`
        //     `Math.log(x) * Math.LOG10E`
        //
        // Call expressions:
        //     ONLY `Math.sqrt`
        //     where the contents are a bin expression (a * a + b * b) OR (a ** 2 + b ** 2)
        //
        //
        match node.kind() {
            AstKind::BinaryExpression(bin_expr) => {
                check_prefer_log(bin_expr, ctx);
            }

            AstKind::CallExpression(call_expr) => {
                if !is_method_call(call_expr, None, Some(&["sqrt"]), Some(1), Some(1)) {
                    return;
                };

                let Some(member_expr) = call_expr.callee.as_member_expression() else {
                    return;
                };

                if !member_expr.object().is_specific_id("Math") {
                    return;
                };

                let Some(arg) = call_expr.arguments[0].as_expression() else {
                    return;
                };

                let expressions = flat_plus_expression(arg);
                if expressions.iter().any(|expr| !is_pow_2_expression(expr, ctx)) {
                    return;
                }

                if expressions.len() == 1 {
                    ctx.diagnostic(prefer_math_abs(call_expr.span));
                } else {
                    ctx.diagnostic(prefer_math_hypot(call_expr.span));
                }
            }
            _ => {}
        }
    }
}

fn check_prefer_log<'a>(expr: &BinaryExpression<'a>, ctx: &LintContext<'a>) {
    match expr.operator {
        BinaryOperator::Multiplication => {
            check_multiplication(expr.span, &expr.left, &expr.right, ctx);
            check_multiplication(expr.span, &expr.right, &expr.left, ctx);
        }
        BinaryOperator::Division => {
            let Expression::CallExpression(call_expr) = &expr.left else {
                return;
            };

            if !is_method_call(call_expr, None, Some(&["log"]), Some(1), Some(1)) {
                return;
            };

            if matches!(call_expr.arguments[0], Argument::SpreadElement(_)) {
                return;
            }

            let Some(member_expr) = call_expr.callee.as_member_expression() else {
                return;
            };

            if !member_expr.object().is_specific_id("Math") {
                return;
            };

            let Some(member_expr) = expr.right.as_member_expression() else {
                return;
            };

            if !matches!(
                member_expr.static_property_name(),
                Some("LN10" | "LN2" | "LOG10E" | "LOG2E")
            ) {
                return;
            };

            if !member_expr.object().is_specific_id("Math") {
                return;
            };

            ctx.diagnostic(prefer_math_log_n(
                expr.span,
                get_math_log_replacement(member_expr.static_property_name()),
                &clean_string(expr.span.source_text(ctx.source_text())),
            ));
        }
        _ => {}
    }
}

fn get_math_log_replacement(name: Option<&str>) -> &'static str {
    match name {
        Some("LN2" | "LOG2E") => "log2",
        Some("LN10" | "LOG10E") => "log10",
        _ => unreachable!(),
    }
}

fn check_multiplication<'a, 'b>(
    expr_span: Span,
    left: &'b Expression<'a>,
    right: &'b Expression<'a>,
    ctx: &LintContext<'a>,
) {
    let Expression::CallExpression(call_expr) = left else {
        return;
    };

    if !is_method_call(call_expr, None, Some(&["log"]), Some(1), Some(1)) {
        return;
    };

    if matches!(call_expr.arguments[0], Argument::SpreadElement(_)) {
        return;
    }

    let Some(member_expr) = call_expr.callee.as_member_expression() else {
        return;
    };

    if !member_expr.object().is_specific_id("Math") {
        return;
    };

    let Some(member_expr) = right.as_member_expression() else {
        return;
    };

    if !matches!(member_expr.static_property_name(), Some("LN10" | "LN2" | "LOG10E" | "LOG2E")) {
        return;
    };

    if !member_expr.object().is_specific_id("Math") {
        return;
    };

    ctx.diagnostic(prefer_math_log_n(
        expr_span,
        get_math_log_replacement(member_expr.static_property_name()),
        &clean_string(expr_span.source_text(ctx.source_text())),
    ));
}

fn flat_plus_expression<'a>(expression: &'a Expression<'a>) -> Vec<&'a Expression<'a>> {
    let mut expressions = Vec::new();

    match expression.without_parentheses() {
        Expression::BinaryExpression(bin_expr) => {
            if matches!(bin_expr.operator, BinaryOperator::Addition) {
                expressions.append(&mut flat_plus_expression(&bin_expr.left));
                expressions.append(&mut flat_plus_expression(&bin_expr.right));
            } else {
                expressions.push(expression);
            }
        }
        _ => expressions.push(expression),
    }

    expressions
}

fn is_pow_2_expression(expression: &Expression, ctx: &LintContext<'_>) -> bool {
    if let Expression::BinaryExpression(bin_expr) = expression.without_parentheses() {
        match bin_expr.operator {
            BinaryOperator::Exponential => {
                if let Expression::NumericLiteral(number_lit) =
                    &bin_expr.right.without_parentheses()
                {
                    (number_lit.value - 2_f64).abs() < f64::EPSILON
                } else {
                    false
                }
            }
            BinaryOperator::Multiplication => {
                is_same_expression(&bin_expr.left, &bin_expr.right, ctx)
            }
            _ => false,
        }
    } else {
        false
    }
}

/// removes any newlines from the string
/// removes any duplicate spaces
fn clean_string(input: &str) -> String {
    let mut result = String::new();
    let mut prev_char = ' ';

    for c in input.chars() {
        if c != '\n' {
            let current_char = if c == '\t' { ' ' } else { c };
            if !(current_char == ' ' && prev_char == ' ') {
                result.push(current_char);
            }

            prev_char = current_char;
        }
    }

    result
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Prefer `Math.sqrt(x)`
        r"Math.notSqrt(a ** 2 + b ** 2)",
        r"NotMath.sqrt(a ** 2 + b ** 2)",
        r"Math.sqrt(a ** 2 - b ** 2)",
        r"Math.sqrt(a ** 2 + 2 ** b)",
        r"Math.sqrt(a * c + b * c)",
        r"Math.sqrt((++a) * (++a))",
        r"Math.sqrt(Math.pow(a, 2) + Math.pow(b, 2))",
        // Prefer `Math.log10(x)`
        r"Math.log(x) * Math.log(x)",
        r"Math.LOG10E * Math.LOG10E",
        r"Math.log(x) * Math[LOG10E]",
        r"Math.log(x) * LOG10E",
        r"Math[log](x) * Math.LOG10E",
        r"foo.Math.log(x) * Math.LOG10E",
        r"Math.log(x) * foo.Math.LOG10E",
        r"Math.log(x) * Math.NOT_LOG10E",
        r"Math.log(x) * Math?.LOG10E",
        r"Math?.log(x) * Math.LOG10E",
        r"log(x) * Math.LOG10E",
        r"new Math.log(x) * Math.LOG10E",
        r"Math.not_log(x) + Math.LOG10E",
        r"Math.log(x)[Math.LOG10E]",
        r"Math.log() * Math.LOG10E",
        r"Math.log(x, extraArgument) * Math.LOG10E",
        r"Math.log(...x) * Math.LOG10E",
        r"Math.LN10 / Math.LN10",
        r"Math.log(x) /Math[LN10]",
        r"Math.log(x) / LN10",
        r"Math[log](x) / Math.LN10",
        r"foo.Math.log(x) / Math.LN10",
        r"Math.log(x) / foo.Math.LN10",
        r"Math.log(x) / Math.log(x)",
        r"Math.log(x) / Math.NOT_LN10",
        r"Math.log(x) / Math?.LN10",
        r"Math?.log(x) / Math.LN10",
        r"log(x) / Math.LN10",
        r"new Math.log(x) / Math.LN10",
        r"Math.not_log(x) + Math.LN10",
        r"Math.log(x)[Math.LN10]",
        r"Math.log() / Math.LN10",
        r"Math.log(x, extraArgument) / Math.LN10",
        r"Math.log(...x) / Math.LN10",
        r"Math.log(x) * Math.log(x)",
        // Prefer `Math.log2(x)`
        r"Math.LOG2E * Math.LOG2E",
        r"Math.log(x) * Math[LOG2E]",
        r"Math.log(x) * LOG2E",
        r"Math[log](x) * Math.LOG2E",
        r"foo.Math.log(x) * Math.LOG2E",
        r"Math.log(x) * foo.Math.LOG2E",
        r"Math.log(x) * Math.NOT_LOG2E",
        r"Math.log(x) * Math?.LOG2E",
        r"Math?.log(x) * Math.LOG2E",
        r"log(x) * Math.LOG2E",
        r"new Math.log(x) * Math.LOG2E",
        r"Math.not_log(x) + Math.LOG2E",
        r"Math.log(x)[Math.LOG2E]",
        r"Math.log() * Math.LOG2E",
        r"Math.log(x, extraArgument) * Math.LOG2E",
        r"Math.log(...x) * Math.LOG2E",
        r"Math.LN2 / Math.LN2",
        r"Math.log(x) /Math[LN2]",
        r"Math.log(x) / LN2",
        r"Math[log](x) / Math.LN2",
        r"foo.Math.log(x) / Math.LN2",
        r"Math.log(x) / foo.Math.LN2",
        r"Math.log(x) / Math.log(x)",
        r"Math.log(x) / Math.NOT_LN2",
        r"Math.log(x) / Math?.LN2",
        r"Math?.log(x) / Math.LN2",
        r"log(x) / Math.LN2",
        r"new Math.log(x) / Math.LN2",
        r"Math.not_log(x) + Math.LN2",
        r"Math.log(x)[Math.LN2]",
        r"Math.log() / Math.LN2",
        r"Math.log(x, extraArgument) / Math.LN2",
        r"Math.log(...x) / Math.LN2",
    ];

    let fail = vec![
        // Prefer `Math.sqrt(x)`
        r"Math.sqrt(a * a + b * b)",
        r"Math.sqrt(a ** 2 + b ** 2)",
        r"Math.sqrt(a * a + b ** 2)",
        r"Math.sqrt(a * a + b * b + c * c)",
        r"Math.sqrt(a ** 2 + b ** 2 + c ** 2)",
        r"Math.sqrt(a * a)",
        r"Math.sqrt(a ** 2)",
        r"Math.sqrt(a * a,)",
        r"Math.sqrt(a ** 2,)",
        r"Math.sqrt((a, b) ** 2)",
        r"Math.sqrt((++a) ** 2)",
        r"Math.sqrt(a * a + b * b,)",
        r"Math.sqrt(a ** 2 + b ** 2,)",
        // Prefer `Math.log10(x)`
        r"Math.log(x) * Math.LOG10E",
        r"Math.LOG10E * Math.log(x)",
        r"Math.log(x) / Math.LN10",
        r"Math.log((( 0,x ))) * Math.LOG10E",
        r"Math.LOG10E * Math.log((( 0,x )))",
        r"Math.log((( 0,x ))) / Math.LN10",
        r"
			function foo(x) {
				return (
					Math.log(x)
						/ Math.LN10
				);
			}
		",
        // Prefer `Math.log2(x)`
        r"Math.log(x) * Math.LOG2E",
        r"Math.LOG2E * Math.log(x)",
        r"Math.log(x) / Math.LN2",
        r"Math.log((( 0,x ))) * Math.LOG2E",
        r"Math.LOG2E * Math.log((( 0,x )))",
        r"Math.log((( 0,x ))) / Math.LN2",
        r"
			function foo(x) {
				return (
					Math.log(x)
						/ Math.LN2
				);
			}
		",
    ];

    Tester::new(PreferModernMathApis::NAME, PreferModernMathApis::PLUGIN, pass, fail)
        .test_and_snapshot();
}

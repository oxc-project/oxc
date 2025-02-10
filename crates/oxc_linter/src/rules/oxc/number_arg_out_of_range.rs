use oxc_ast::{ast::Argument, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn number_arg_out_of_range_diagnostic(
    method_name: &str,
    min: usize,
    max: usize,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Radix or precision arguments of number-related functions should not exceed the limit",
    )
    .with_help(format!(
        "The first argument of 'Number.prototype.{method_name}' should be a number between {min} and {max}"
    ))
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NumberArgOutOfRange;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Checks whether the radix or precision arguments of number-related functions exceeds the limit.
    ///
    /// ### Why is this bad?
    ///
    /// The radix argument of `Number.prototype.toString` should be between 2 and 36.
    /// The precision argument of `Number.prototype.toFixed` and `Number.prototype.toExponential` should be between 0 and 20.
    /// The precision argument of `Number.prototype.toPrecision` should be between 1 and 21.
    ///
    /// ### Example
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// var x = 42;
    /// var s_radix_64 = x.toString(64);
    /// var s = x.toString(1);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// var x = 42;
    /// var s_radix_16 = x.toString(16);
    /// ```
    ///
    NumberArgOutOfRange,
    oxc,
    correctness
);

impl Rule for NumberArgOutOfRange {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(expr) = node.kind() else {
            return;
        };
        let Some(member) = expr.callee.get_member_expr() else {
            return;
        };

        if let Some(Argument::NumericLiteral(literal)) = expr.arguments.first() {
            let value = literal.value;
            match member.static_property_name() {
                Some(name @ "toString") => {
                    if !(2.0_f64..=36.0_f64).contains(&value) {
                        ctx.diagnostic(number_arg_out_of_range_diagnostic(name, 2, 36, expr.span));
                    }
                }
                Some(name @ ("toFixed" | "toExponential")) => {
                    if !(0.0_f64..=20.0_f64).contains(&value) {
                        ctx.diagnostic(number_arg_out_of_range_diagnostic(name, 0, 20, expr.span));
                    }
                }
                Some(name @ "toPrecision") => {
                    if !(1.0_f64..=21.0_f64).contains(&value) {
                        ctx.diagnostic(number_arg_out_of_range_diagnostic(name, 1, 21, expr.span));
                    }
                }
                _ => {}
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![("var x = 42;var s = x.toString(16);", None)];

    let fail = vec![
        ("var x = 42;var s = x.toString(1);", None),
        ("var x = 42;var s = x.toString(43);", None),
        ("var x = 42;var s = x.toFixed(22);", None),
        ("var x = 42;var s = x['toExponential'](22);", None),
        ("var x = 42;var s = x.toPrecision(0);", None),
        ("var x = 42;var s = x.toPrecision(100);", None),
    ];

    Tester::new(NumberArgOutOfRange::NAME, NumberArgOutOfRange::PLUGIN, pass, fail)
        .test_and_snapshot();
}

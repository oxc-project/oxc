use oxc_ast::{ast::Argument, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn number_arg_out_of_range_diagnostic(
    x0: &str,
    x1: usize,
    x2: usize,
    span3: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn("oxc(number-arg-out-of-range): Radix or precision arguments of number-related functions should not exceed the limit")
        .with_help(format!("The first argument of 'Number.prototype.{x0}' should be a number between {x1} and {x2}"))
        .with_label(span3)
}

#[derive(Debug, Default, Clone)]
pub struct NumberArgOutOfRange;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Checks whether the radix or precision arguments of number-related functions exceeds the limit.
    ///
    /// ### Example
    /// ```javascript
    /// var x = 42;
    /// var s_radix_64 = x.toString(64);
    /// var s = x.toString(1);
    /// ```
    NumberArgOutOfRange,
    correctness
);

impl Rule for NumberArgOutOfRange {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(expr) = node.kind() else {
            return;
        };

        if let Some(member) = expr.callee.get_member_expr() {
            if let Some(Argument::NumericLiteral(literal)) = expr.arguments.first() {
                let value = literal.value;
                match member.static_property_name() {
                    Some(name @ "toString") => {
                        if !(2.0_f64..=36.0_f64).contains(&value) {
                            ctx.diagnostic(number_arg_out_of_range_diagnostic(
                                name, 2, 36, expr.span,
                            ));
                        }
                    }
                    Some(name @ ("toFixed" | "toExponential")) => {
                        if !(0.0_f64..=20.0_f64).contains(&value) {
                            ctx.diagnostic(number_arg_out_of_range_diagnostic(
                                name, 0, 20, expr.span,
                            ));
                        }
                    }
                    Some(name @ "toPrecision") => {
                        if !(1.0_f64..=21.0_f64).contains(&value) {
                            ctx.diagnostic(number_arg_out_of_range_diagnostic(
                                name, 1, 21, expr.span,
                            ));
                        }
                    }
                    _ => {}
                }
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

    Tester::new(NumberArgOutOfRange::NAME, pass, fail).test_and_snapshot();
}

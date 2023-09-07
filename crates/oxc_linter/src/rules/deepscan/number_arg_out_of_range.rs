use oxc_ast::{
    ast::{Argument, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("deepscan(number-arg-out-of-range): Radix or precision arguments of number-related functions should not exceed the limit")]
#[diagnostic(
    severity(warning),
    help("The first argument of 'Number.prototype.{0}' should be a number between {1} and {2}")
)]
struct NumberArgOutOfRangeDiagnostic(Atom, usize, usize, #[label] pub Span);

/// `https://deepscan.io/docs/rules/missing-throw`
#[derive(Debug, Default, Clone)]
pub struct NumberArgOutOfRange;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Checks whether the radix or precision arguments of number-related functions exceeds the limit.
    ///
    /// ### Example
    /// ```javascript
    /// function foo() { throw Error() }
    /// const foo = () => { new Error() }
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
            if let Some(Argument::Expression(Expression::NumberLiteral(literal))) =
                expr.arguments.first()
            {
                let value = literal.value;
                match member.static_property_name() {
                    Some(name @ "toString") => {
                        if !(2.0_f64..=36.0_f64).contains(&value) {
                            let name = Atom::from(name);
                            ctx.diagnostic(NumberArgOutOfRangeDiagnostic(name, 2, 36, expr.span));
                        }
                    }
                    Some(name @ ("toFixed" | "toExponential")) => {
                        if !(0.0_f64..=20.0_f64).contains(&value) {
                            let name = Atom::from(name);
                            ctx.diagnostic(NumberArgOutOfRangeDiagnostic(name, 0, 20, expr.span));
                        }
                    }
                    Some(name @ "toPrecision") => {
                        if !(1.0_f64..=21.0_f64).contains(&value) {
                            let name = Atom::from(name);
                            ctx.diagnostic(NumberArgOutOfRangeDiagnostic(name, 1, 21, expr.span));
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

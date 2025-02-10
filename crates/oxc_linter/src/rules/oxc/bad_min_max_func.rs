use oxc_ast::{
    ast::{Argument, CallExpression, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn bad_min_max_func_diagnostic(constant_result: f64, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Math.min and Math.max combination leads to constant result")
        .with_help(format!(
            "This evaluates to {constant_result:?} because of the incorrect `Math.min`/`Math.max` combination"
        ))
        .with_label(span1)
}

#[derive(Debug, Default, Clone)]
pub struct BadMinMaxFunc;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Checks whether the clamp function `Math.min(Math.max(x, y), z)` always evaluate to a
    /// constant result because the arguments are in the wrong order.
    ///
    /// ### Why is this bad?
    ///
    /// The `Math.min(Math.max(x, y), z)` function is used to clamp a value between two other values.
    /// If the arguments are in the wrong order, the function will always evaluate to a constant result.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// Math.min(Math.max(100, x), 0);
    /// Math.max(1000, Math.min(0, z));
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    ///
    /// ```javascript
    /// Math.max(0, Math.min(100, x));
    /// Math.min(0, Math.max(1000, z));
    /// ```
    ///
    BadMinMaxFunc,
    oxc,
    correctness
);

impl Rule for BadMinMaxFunc {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some((out_min_max, inner_exprs)) = Self::min_max(call_expr) else {
            return;
        };

        for expr in inner_exprs {
            if let Some((inner_min_max, ..)) = Self::min_max(expr) {
                let constant_result = match (&out_min_max, &inner_min_max) {
                    (MinMax::Max(max), MinMax::Min(min)) => {
                        if max > min {
                            Some(max)
                        } else {
                            None
                        }
                    }
                    (MinMax::Min(min), MinMax::Max(max)) => {
                        if min < max {
                            Some(min)
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                if let Some(constant) = constant_result {
                    ctx.diagnostic(bad_min_max_func_diagnostic(*constant, call_expr.span));
                }
            }
        }
    }
}

enum MinMax {
    Min(f64),
    Max(f64),
}

impl BadMinMaxFunc {
    fn min_max<'a>(node: &'a CallExpression<'a>) -> Option<(MinMax, Vec<&'a CallExpression<'a>>)> {
        let CallExpression { callee, arguments, .. } = node;

        if let Some(member_expr) = callee.get_member_expr() {
            if let Expression::Identifier(ident) = member_expr.object() {
                if ident.name != "Math" {
                    return None;
                }

                let number_args = arguments.iter().filter_map(|arg| {
                    if let Argument::NumericLiteral(literal) = arg {
                        Some(literal.value)
                    } else {
                        None
                    }
                });

                let min_max = match member_expr.static_property_name() {
                    Some("max") => MinMax::Max(number_args.fold(f64::NEG_INFINITY, f64::max)),
                    Some("min") => MinMax::Min(number_args.fold(f64::INFINITY, f64::min)),
                    _ => return None,
                };

                let mut inner = vec![];

                for expr in arguments.iter().filter_map(|arg| {
                    if let Argument::CallExpression(expr) = arg {
                        Some(&**expr)
                    } else {
                        None
                    }
                }) {
                    inner.push(expr);
                }

                return Some((min_max, inner));
            }
        }

        None
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("Math.max(0, Math.min(100, x))", None),
        ("Math.max(Math.min(100, x), 0)", None),
        ("Math.min(100, Math.max(0.9, x))", None),
        ("Math.min(255.255, Math.max(0, x))", None),
        ("Math.min(Math.max(0, x), 255)", None),
        ("Math.min(0, Math.min(0, x))", None),
    ];

    let fail = vec![
        ("Math.min(Math.max(100, x), 0)", None),
        ("Math.max(255.255, Math.min(0, x))", None),
        ("Math.max(Math.min(0, x), 255)", None),
        ("Math.max(1000, Math.min(0, z))", None),
        ("Math[\"min\"](0, Math.max(100, x))", None),
        ("Math.min(Math.max(1000, x), 100, 3)", None),
        ("Math.min(0, 5, Math['max'](x, 100, 30))", None),
        ("Math.min(Math.max(1e3, x), 1.55e2)", None),
    ];

    Tester::new(BadMinMaxFunc::NAME, BadMinMaxFunc::PLUGIN, pass, fail).test_and_snapshot();
}

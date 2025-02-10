use std::f64::consts as f64;

// Based on https://github.com/rust-lang/rust-clippy//blob/c9a43b18f11219fa70fe632b29518581fcd589c8/clippy_lints/src/approx_const.rs
// https://rust-lang.github.io/rust-clippy/master/#approx_constant
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn approx_constant_diagnostic(span: Span, method_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Approximate value of `{method_name}` found."))
        .with_help(format!("Use `Math.{method_name}` instead"))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ApproxConstant;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows the use of approximate constants, instead preferring the use
    /// of the constants in the `Math` object.
    ///
    /// ### Why is this bad?
    ///
    /// Approximate constants are not as accurate as the constants in the `Math` object.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// let log10e = 0.434294
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// let log10e = Math.LOG10E
    /// ```
    ApproxConstant,
    oxc,
    suspicious
);

impl Rule for ApproxConstant {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NumericLiteral(number_literal) = node.kind() else {
            return;
        };

        let number_lit_str = number_literal.value.to_string();
        for (constant, name, min_digits) in &KNOWN_CONSTS {
            if is_approx_const(*constant, &number_lit_str, *min_digits) {
                ctx.diagnostic(approx_constant_diagnostic(number_literal.span, name));
            }
        }
    }
}

const KNOWN_CONSTS: [(f64, &str, usize); 8] = [
    (f64::E, "E", 4),
    (f64::LN_10, "LN10", 4),
    (f64::LN_2, "LN2", 4),
    (f64::LOG2_E, "LOG2E", 4),
    (f64::LOG10_E, "LOG10E", 4),
    (f64::PI, "PI", 4),
    (f64::FRAC_1_SQRT_2, "SQRT1_2", 4),
    (f64::SQRT_2, "SQRT2", 4),
];

#[must_use]
fn is_approx_const(constant: f64, value: &str, min_digits: usize) -> bool {
    if value.len() <= min_digits {
        false
    } else if constant.to_string().starts_with(value) {
        // The value is a truncated constant
        true
    } else {
        let round_const = format!("{constant:.*}", value.len() - 2);
        value == round_const
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec!["const x = 1234;"];

    let fail = vec![
        "const getArea = (radius) => 3.141 * radius * radius;",
        "let e = 2.718281",      // E
        "let ln10 = 2.302585",   // LN10
        "let ln2 = 0.693147",    // LN2
        "let log10e = 0.434294", // LOG10E
        "let log2e = 1.442695",  // LOG2E
        "let pi = 3.141592",     // PI
        "let sqrt12 = 0.707106", // SQRT1_2
        "let sqrt2 = 1.414213",  // SQRT2
    ];

    Tester::new(ApproxConstant::NAME, ApproxConstant::PLUGIN, pass, fail).test_and_snapshot();
}

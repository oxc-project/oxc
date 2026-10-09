use std::f64::consts as f64;

// Based on https://github.com/rust-lang/rust-clippy//blob/c9a43b18f11219fa70fe632b29518581fcd589c8/clippy_lints/src/approx_const.rs
// https://rust-lang.github.io/rust-clippy/master/#approx_constant
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_str::static_ident;

use crate::{AstNode, context::LintContext, fixer::RuleFixer, rule::Rule};

fn approx_constant_diagnostic(span: Span, method_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Approximate value of `{method_name}` found."))
        .with_help(format!("Use `Math.{method_name}` instead."))
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
    /// Using the `Math` constants improves code readability and accuracy.
    /// See https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math
    /// for more information.
    ///
    /// ### Examples
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
    suspicious,
    suggestion,
    version = "0.1.1",
    short_description = "Disallows the use of approximate constants, instead preferring the use of the constants in the `Math` object.",
);

impl Rule for ApproxConstant {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NumericLiteral(number_literal) = node.kind() else {
            return;
        };

        let Some(name) = find_approx_const(number_literal.value) else {
            return;
        };

        ctx.diagnostic_with_suggestion(
            approx_constant_diagnostic(number_literal.span, name),
            |fixer| {
                if ctx.scoping().find_binding(node.scope_id(), static_ident!("Math")).is_some() {
                    fixer.noop()
                } else {
                    Self::fix_with_math_constant(fixer, number_literal.span, name)
                }
            },
        );
    }
}

impl ApproxConstant {
    fn fix_with_math_constant(
        fixer: RuleFixer<'_, '_>,
        span: Span,
        name: &str,
    ) -> crate::fixer::RuleFix {
        fixer.replace(span, format!("Math.{name}"))
    }
}

struct KnownConst {
    value: f64,
    /// `value.to_string()`, precomputed. Checked against `value` in a test.
    text: &'static str,
    name: &'static str,
    min_digits: usize,
}

const KNOWN_CONSTS: [KnownConst; 8] = [
    KnownConst { value: f64::E, text: "2.718281828459045", name: "E", min_digits: 4 },
    KnownConst { value: f64::LN_10, text: "2.302585092994046", name: "LN10", min_digits: 4 },
    KnownConst { value: f64::LN_2, text: "0.6931471805599453", name: "LN2", min_digits: 4 },
    KnownConst { value: f64::LOG2_E, text: "1.4426950408889634", name: "LOG2E", min_digits: 4 },
    KnownConst { value: f64::LOG10_E, text: "0.4342944819032518", name: "LOG10E", min_digits: 4 },
    KnownConst { value: f64::PI, text: "3.141592653589793", name: "PI", min_digits: 4 },
    KnownConst {
        value: f64::FRAC_1_SQRT_2,
        text: "0.7071067811865476",
        name: "SQRT1_2",
        min_digits: 4,
    },
    KnownConst { value: f64::SQRT_2, text: "1.4142135623730951", name: "SQRT2", min_digits: 4 },
];

/// Every known constant, and every approximation of one, lies in this range.
/// The smallest candidate is `0.434` (a 5-character truncation of `LOG10_E`) and the largest
/// is `3.142` (`PI` rounded to 3 decimals), so a literal outside it can't match.
const APPROX_RANGE: std::ops::Range<f64> = 0.4..3.2;

/// The name of the `Math` constant that `value` approximates, if any.
fn find_approx_const(value: f64) -> Option<&'static str> {
    // Cheap rejection for nearly all numeric literals (integers, large numbers, ...),
    // which avoids formatting them into a string.
    if !APPROX_RANGE.contains(&value) {
        return None;
    }

    let value = value.to_string();
    KNOWN_CONSTS
        .iter()
        .find(|known| is_approx_const(known.value, known.text, &value, known.min_digits))
        .map(|known| known.name)
}

#[must_use]
fn is_approx_const(constant: f64, constant_text: &str, value: &str, min_digits: usize) -> bool {
    if value.len() <= min_digits {
        false
    } else if constant_text.starts_with(value) {
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

    let pass = vec![
        "const x = 1234;",
        "const x = /* 3.141592 */ 3.14;",
        "const x = 3.14 // 3.141592",
        "let pi = Math.PI;",
        "let e = Math.E;",
        "let ln10 = Math.LN10;",
        "let ln2 = Math.LN2;",
        "let log10e = Math.LOG10E;",
        "let log2e = Math.LOG2E;",
        "let sqrt12 = Math.SQRT1_2;",
        "let sqrt2 = Math.SQRT2;",
        "let pi = 3.14;",
        "let pi = '3.141592';",
        "let pi = \"3.141592\";",
        "let e = 2.71;",
    ];

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
        "const text = `the value of pi is ${3.141592}`;",
    ];

    let fix = vec![
        (
            "const getArea = (radius) => 3.141 * radius * radius;",
            "const getArea = (radius) => Math.PI * radius * radius;",
            None,
        ),
        ("let e = 2.718281", "let e = Math.E", None),
        ("let ln10 = 2.302585", "let ln10 = Math.LN10", None),
        ("let ln2 = 0.693147", "let ln2 = Math.LN2", None),
        ("let log10e = 0.434294", "let log10e = Math.LOG10E", None),
        ("let log2e = 1.442695", "let log2e = Math.LOG2E", None),
        ("let pi = 3.141592", "let pi = Math.PI", None),
        ("let sqrt12 = 0.707106", "let sqrt12 = Math.SQRT1_2", None),
        ("let sqrt2 = 1.414213", "let sqrt2 = Math.SQRT2", None),
        (
            "const t = `the value of pi is ${3.141592}`;",
            "const t = `the value of pi is ${Math.PI}`;",
            None,
        ),
        (
            "const Math = {}; const t = `pi = ${3.141592}`;",
            "const Math = {}; const t = `pi = ${3.141592}`;",
            None,
        ),
        (
            "if (x) { const Math = {}; } const t = `pi = ${3.141592}`;",
            "if (x) { const Math = {}; } const t = `pi = ${Math.PI}`;",
            None,
        ),
    ];

    Tester::new(ApproxConstant::NAME, ApproxConstant::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

#[cfg(test)]
mod equivalence_tests {
    use super::{APPROX_RANGE, KNOWN_CONSTS, find_approx_const};

    /// The implementation before the range check and precomputed strings were added.
    fn reference(value: f64) -> Option<&'static str> {
        fn is_approx_const(constant: f64, value: &str, min_digits: usize) -> bool {
            if value.len() <= min_digits {
                false
            } else if constant.to_string().starts_with(value) {
                true
            } else {
                let round_const = format!("{constant:.*}", value.len() - 2);
                value == round_const
            }
        }

        let value = value.to_string();
        KNOWN_CONSTS
            .iter()
            .find(|known| is_approx_const(known.value, &value, known.min_digits))
            .map(|known| known.name)
    }

    fn assert_same(value: f64) {
        assert_eq!(find_approx_const(value), reference(value), "value: {value:?}");
    }

    #[test]
    fn precomputed_text_matches_value() {
        for known in &KNOWN_CONSTS {
            assert_eq!(known.value.to_string(), known.text, "{}", known.name);
        }
    }

    #[test]
    fn range_contains_every_constant() {
        for known in &KNOWN_CONSTS {
            assert!(APPROX_RANGE.contains(&known.value), "{}", known.name);
        }
    }

    /// Every truncation and every rounding of every constant, and their neighbors.
    #[test]
    fn truncations_and_roundings_of_constants() {
        for known in &KNOWN_CONSTS {
            for decimals in 0..=20 {
                let rounded: f64 = format!("{:.*}", decimals, known.value).parse().unwrap();
                let truncated_text = &known.text[..known.text.len().min(decimals + 2)];
                let truncated: f64 = truncated_text.parse().unwrap();

                for value in [rounded, truncated] {
                    assert_same(value);
                    assert_same(value.next_up());
                    assert_same(value.next_down());
                    let step = 10f64.powi(-i32::try_from(decimals).unwrap());
                    assert_same(value + step);
                    assert_same(value - step);
                }
            }
        }
    }

    /// Values at and around the edges of the range, and non-finite values.
    #[test]
    fn range_boundaries_and_special_values() {
        let mut values = vec![
            0.0,
            -0.0,
            f64::MIN_POSITIVE,
            f64::EPSILON,
            f64::MAX,
            f64::INFINITY,
            f64::NAN,
            1e21,
            1e-7,
        ];
        for edge in [APPROX_RANGE.start, APPROX_RANGE.end] {
            values.extend([edge, edge.next_up(), edge.next_down()]);
        }
        for value in values {
            if value.is_nan() {
                assert_eq!(find_approx_const(value), None);
                assert_eq!(reference(value), None);
            } else {
                assert_same(value);
            }
        }
    }

    /// Integers and decimals with a fixed number of digits, across and beyond the range.
    #[test]
    fn many_generated_values() {
        for n in 0..=10_000_u32 {
            assert_same(f64::from(n));
        }

        // Deterministic xorshift, so failures are reproducible.
        let mut state = 0x2545_F491_4F6C_DD1D_u64;
        let mut next_u64 = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };

        for _ in 0..20_000 {
            // 32 random bits mapped to [0, 1)
            let unit = f64::from(u32::try_from(next_u64() >> 32).unwrap()) / 4_294_967_296.0;
            let decimals = i32::try_from(next_u64() % 17).unwrap();
            let scale = 10f64.powi(decimals);

            // Mostly inside and just outside the range, some far outside.
            for base in [unit * 4.0, unit.mul_add(3.0, 0.3), unit * 100.0, unit * 1e9] {
                assert_same(base);
                assert_same((base * scale).round() / scale);
                assert_same((base * scale).trunc() / scale);
            }
        }
    }
}

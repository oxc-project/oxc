use std::borrow::Cow;

use cow_utils::CowUtils;
use oxc_ast::{AstKind, ast::NumericLiteral};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_loss_of_precision_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("This number literal will lose precision at runtime.")
        .with_help(
            "Use a number literal representable by a 64-bit floating-point number, or use a `BigInt` literal (for example, `9007199254740993n`) for exact large integers.",
        )
        .with_note(
            "In JavaScript, `Number` values exactly represent integers only in the range -9007199254740991 to 9007199254740991 (`Number.MIN_SAFE_INTEGER` to `Number.MAX_SAFE_INTEGER`). `BigInt` supports arbitrarily large integers.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoLossOfPrecision;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow precision loss of number literal.
    ///
    /// ### Why is this bad?
    ///
    /// It can lead to unexpected results in certain situations.
    /// For example, when performing mathematical operations.
    ///
    /// In JavaScript, Numbers are stored as double-precision floating-point numbers
    /// according to the IEEE 754 standard. Because of this, numbers can only
    /// retain accuracy up to a certain amount of digits. If the programmer
    /// enters additional digits, those digits will be lost in the conversion
    /// to the Number type and will result in unexpected/incorrect behavior.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// var x = 2e999;
    /// ```
    ///
    /// ```javascript
    /// var x = 9007199254740993;
    /// ```
    ///
    /// ```javascript
    /// var x = 5123000000000000000000000000001;
    /// ```
    ///
    /// ```javascript
    /// var x = 1230000000000000000000000.0;
    /// ```
    ///
    /// ```javascript
    /// var x = 0X200000_0000000_1;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// var x = 12345;
    /// ```
    ///
    /// ```javascript
    /// var x = 123.456;
    /// ```
    ///
    /// ```javascript
    /// var x = 123.0000000000000000000000;
    /// ```
    ///
    /// ```javascript
    /// var x = 123e34;
    /// ```
    ///
    /// ```javascript
    /// var x = 0x1FFF_FFFF_FFF_FFF;
    /// ```
    NoLossOfPrecision,
    eslint,
    correctness
);

impl Rule for NoLossOfPrecision {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::NumericLiteral(node) if Self::lose_precision(node) => {
                ctx.diagnostic(no_loss_of_precision_diagnostic(node.span));
            }
            _ => {}
        }
    }
}

pub struct RawNum<'a> {
    int: &'a str,
    frac: &'a str,
    exp: isize,
}

#[derive(Debug)]
pub struct ScientificNotation<'a> {
    int: &'a str,
    frac: Cow<'a, str>,
    exp: isize,
}

impl PartialEq for ScientificNotation<'_> {
    fn eq(&self, other: &Self) -> bool {
        if self.int == other.int && self.frac == other.frac {
            if self.int == "0" && self.frac.is_empty() {
                return true;
            }
            return self.exp == other.exp;
        }
        false
    }
}

impl<'a> RawNum<'a> {
    fn new(num: &str) -> Option<RawNum<'_>> {
        // remove sign
        let num = num.trim_start_matches(['+', '-']);

        let (int, num_without_int) = {
            // skip leading zeros
            let num_without_zeros = num.trim_start_matches('0');

            // read the integer part and store the end index
            let int_end = num_without_zeros
                .chars()
                .position(|ch| !ch.is_ascii_digit())
                .unwrap_or(num_without_zeros.len());

            // if no integer part was found, default to 0
            let int = if int_end == 0 { "0" } else { &num_without_zeros[..int_end] };

            // make a slice of the rest of the string
            let num_without_int = &num_without_zeros[int_end..];

            (int, num_without_int)
        };

        // if next char is a dot, parse the fractional part
        let (frac, num_without_frac) =
            num_without_int.strip_prefix('.').map_or(("", num_without_int), |num_without_dot| {
                let frac_end = num_without_dot
                    .chars()
                    .position(|ch| !ch.is_ascii_digit())
                    .unwrap_or(num_without_dot.len());

                // slice the fractional part and the rest of the string
                let frac = &num_without_dot[..frac_end];
                let num_without_frac = &num_without_dot[frac_end..];

                (frac, num_without_frac)
            });

        // if next char is an e, treat the rest as the exponent
        let exp =
            num_without_frac.strip_prefix(['e', 'E']).map_or("0", |num_without_e| num_without_e);

        let Ok(exp) = exp.parse::<isize>() else {
            return None;
        };

        Some(RawNum { int, frac, exp })
    }

    fn normalize(&mut self) -> ScientificNotation<'a> {
        if self.int == "0" && !self.frac.is_empty() {
            let frac_zeros = self.frac.chars().take_while(|&ch| ch == '0').count();
            #[expect(clippy::cast_possible_wrap)]
            let exp = self.exp - 1 - frac_zeros as isize;
            self.frac = &self.frac[frac_zeros..];

            match self.frac.len() {
                0 => ScientificNotation { int: "0", frac: Cow::Borrowed(""), exp },
                1 => ScientificNotation { int: &self.frac[..1], frac: Cow::Borrowed(""), exp },
                _ => ScientificNotation {
                    int: &self.frac[..1],
                    frac: Cow::Borrowed(&self.frac[1..]),
                    exp,
                },
            }
        } else {
            #[expect(clippy::cast_possible_wrap)]
            let exp = self.exp + self.int.len() as isize - 1;
            if self.int.len() == 1 {
                ScientificNotation { int: self.int, frac: Cow::Borrowed(self.frac), exp }
            } else {
                let frac = if self.frac.is_empty() {
                    let int_trimmed = self.int.trim_end_matches('0');
                    if int_trimmed.len() == 1 {
                        Cow::Borrowed("")
                    } else {
                        Cow::Borrowed(&int_trimmed[1..])
                    }
                } else {
                    Cow::Owned(format!("{}{}", &self.int[1..], self.frac))
                };

                ScientificNotation { int: &self.int[..1], frac, exp }
            }
        }
    }
}

impl NoLossOfPrecision {
    fn not_base_ten_loses_precision(node: &'_ NumericLiteral) -> bool {
        let raw = node.raw.as_ref().unwrap().as_str().cow_replace('_', "");
        let raw = raw.cow_to_ascii_uppercase();
        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        // AST always store number as f64, need a cast to format in bin/oct/hex
        let value = node.value as u64;
        let suffix = if raw.starts_with("0B") {
            format!("{value:b}")
        } else if raw.starts_with("0X") {
            format!("{value:x}")
        } else {
            format!("{value:o}")
        };
        !raw.ends_with(&suffix.cow_to_ascii_uppercase().as_ref())
    }

    fn base_ten_loses_precision(node: &'_ NumericLiteral) -> bool {
        let raw = node.raw.as_ref().unwrap().as_str().cow_replace('_', "");
        let Some(raw) = Self::normalize(&raw) else {
            return true;
        };

        let total_significant_digits = raw.int.len() + raw.frac.len();

        if total_significant_digits > 100 {
            return true;
        }

        let stored = to_precision(node.value, total_significant_digits);

        let Some(stored) = Self::normalize(&stored) else {
            return true;
        };
        raw != stored
    }

    fn normalize(num: &str) -> Option<ScientificNotation<'_>> {
        Some(RawNum::new(num)?.normalize())
    }

    pub fn lose_precision(node: &'_ NumericLiteral) -> bool {
        if node.base.is_base_10() {
            Self::base_ten_loses_precision(node)
        } else {
            Self::not_base_ten_loses_precision(node)
        }
    }
}

/// Mimics JavaScript's `Number.prototype.toPrecision()` method
///
/// The `toPrecision()` method returns a string representing the Number object to the specified precision.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-number.prototype.toprecision
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/toPrecision
pub fn to_precision(num: f64, precision: usize) -> String {
    // Validate precision range (1-100)
    debug_assert!((1..=100).contains(&precision), "Precision must be between 1 and 100");

    // Handle non-finite numbers
    if !num.is_finite() {
        if num.is_nan() {
            return "NaN".to_string();
        } else if num.is_infinite() {
            return if num.is_sign_positive() { "Infinity" } else { "-Infinity" }.to_string();
        }
    }

    if num == 0.0 {
        return if precision == 1 {
            "0e0".to_string()
        } else {
            format!("0.{}e0", "0".repeat(precision - 1))
        };
    }

    // Scientific formatting gives the same significant-digit rounding as JS `toPrecision`
    // and avoids fixed-decimal truncation for very small exponents (for example `3e-308`).
    format!("{num:.precision$e}", precision = precision - 1)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var x = 12345",
        "var x = 123.456",
        "var x = -123.456",
        "var x = -123456",
        "var x = 123e34",
        "var x = 123.0e34",
        "var x = 123e-34",
        "var x = -123e34",
        "var x = -123e-34",
        "var x = 12.3e34",
        "var x = 12.3e-34",
        "var x = -12.3e34",
        "var x = -12.3e-34",
        "var x = 12300000000000000000000000",
        "var x = -12300000000000000000000000",
        "var x = 0.00000000000000000000000123",
        "var x = -0.00000000000000000000000123",
        "var x = 9007199254740991",
        "var x = 0",
        "var x = 0.0",
        "var x = 0.000000000000000000000000000000000000000000000000000000000000000000000000000000",
        "var x = -0",
        "var x = 123.0000000000000000000000",
        "var x = 9.00e2",
        "var x = 9.000e3",
        "var x = 9.0000000000e10",
        "var x = 9.00E2",
        "var x = 9.000E3",
        "var x = 9.100E3",
        "var x = 9.0000000000E10",
        "var x = 019.5",
        "var x = 0195",
        "var x = 00195",
        "var x = 0008",
        "var x = 0e5",
        "var x = .42",
        "var x = 42.",
        "var x = 12_34_56",
        "var x = 12_3.4_56",
        "var x = -12_3.4_56",
        "var x = -12_34_56",
        "var x = 12_3e3_4",
        "var x = 123.0e3_4",
        "var x = 12_3e-3_4",
        "var x = 12_3.0e-3_4",
        "var x = -1_23e-3_4",
        "var x = -1_23.8e-3_4",
        "var x = 1_230000000_00000000_00000_000",
        "var x = -1_230000000_00000000_00000_000",
        "var x = 0.0_00_000000000_000000000_00123",
        "var x = -0.0_00_000000000_000000000_00123",
        "var x = 0e5_3",
        "var x = 0b11111111111111111111111111111111111111111111111111111",
        "var x = 0b111_111_111_111_1111_11111_111_11111_1111111111_11111111_111_111",
        "var x = 0B11111111111111111111111111111111111111111111111111111",
        "var x = 0B111_111_111_111_1111_11111_111_11111_1111111111_11111111_111_111",
        "var x = 0o377777777777777777",
        "var x = 0o3_77_777_777_777_777_777",
        "var x = 0O377777777777777777",
        "var x = 0377777777777777777",
        "var x = 0x1FFFFFFFFFFFFF",
        "var x = 0X1FFFFFFFFFFFFF",
        "var x = true",
        "var x = 'abc'",
        "var x = ''",
        "var x = null",
        "var x = undefined",
        "var x = {}",
        "var x = ['a', 'b']",
        "var x = new Date()",
        "var x = '9007199254740993'",
        "var x = 0x1FFF_FFFF_FFF_FFF",
        "var x = 0X1_FFF_FFFF_FFF_FFF",
        "var a = Infinity",
        "var a = 480.00",
        "var a = -30.00",
        "let a = Infinity",
        "let a = 480.00",
        "let a = -30.00",
        "const a = Infinity",
        "const a = 480.00",
        "const a = -30.00",
        "(1000000000000000128).toFixed(0)",
        "const x = 3e-308",
        "const x = 5e-324",
        "const x = 12345;",
        "const x = 123.456;",
        "const x = -123.456;",
        "const x = 123_456;",
        "const x = 123_00_000_000_000_000_000_000_000;",
        "const x = 123.000_000_000_000_000_000_000_0;",
    ];

    let fail = vec![
        "var x = 9007199254740993",
        "var x = 9007199254740.993e3",
        "var x = 9.007199254740993e15",
        "var x = 90071992547409930e-1",
        "var x = .9007199254740993e16",
        "var x = 900719925474099.30e1",
        "var x = -9007199254740993",
        "var x = 900719.9254740994",
        "var x = -900719.9254740994",
        "var x = 900719925474099_3",
        "var x = 90_0719925_4740.9_93e3",
        "var x = 9.0_0719925_474099_3e15",
        "var x = -9_00719_9254_740993",
        "var x = 900_719.92_54740_994",
        "var x = -900_719.92_5474_0994",
        "let x = 9.0_0719925_474099_3e15",
        "let x = -9_00719_9254_740993",
        "let x = 900_719.92_54740_994",
        "let x = -900_719.92_5474_0994",
        "const x = 9.0_0719925_474099_3e15",
        "const x = -9_00719_9254_740993",
        "const x = 900_719.92_54740_994",
        "const x = -900_719.92_5474_0994",
        "var x = 5123000000000000000000000000001",
        "var x = -5123000000000000000000000000001",
        "var x = 1230000000000000000000000.0",
        "var x = 1.0000000000000000000000123",
        "var x = 17498005798264095394980017816940970922825355447145699491406164851279623993595007385788105416184430592",
        "var x = 2e999",
        "var x = .1230000000000000000000000",
        "var x = 0b100000000000000000000000000000000000000000000000000001",
        "var x = 0B100000000000000000000000000000000000000000000000000001",
        "var x = 0o400000000000000001",
        "var x = 0O400000000000000001",
        "var x = 0400000000000000001",
        "var x = 0x20000000000001",
        "var x = 0X20000000000001",
        "var x = 5123_00000000000000000000000000_1",
        "var x = -5_12300000000000000000000_0000001",
        "var x = 123_00000000000000000000_00.0_0",
        "var x = 1.0_00000000000000000_0000123",
        "var x = 174_980057982_640953949800178169_409709228253554471456994_914061648512796239935950073857881054_1618443059_2",
        "var x = 2e9_99",
        "var x = .1_23000000000000_00000_0000_0",
        "var x = 0b1_0000000000000000000000000000000000000000000000000000_1",
        "var x = 0B10000000000_0000000000000000000000000000_000000000000001",
        "var x = 0o4_00000000000000_001",
        "var x = 0O4_0000000000000000_1",
        "var x = 0x2_0000000000001",
        "var x = 0X200000_0000000_1",
        "var x = 1e18_446_744_073_709_551_615",
        "const x = 4e-324",
        "const x = 1e-324",
        "const x = 9007199254740993;",
        "const x = 9_007_199_254_740_993;",
        "const x = 9_007_199_254_740.993e3;",
        "const x = 0b100_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_001;",
    ];

    Tester::new(NoLossOfPrecision::NAME, NoLossOfPrecision::PLUGIN, pass, fail).test_and_snapshot();
}

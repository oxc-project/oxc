use oxc_ast::ast::NumberLiteral;
use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use std::borrow::Cow;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-loss-of-precision): This number literal will lose precision at runtime.")]
#[diagnostic(severity(warning))]
struct NoLossOfPrecisionDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoLossOfPrecision;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow precision loss of number literal
    ///
    /// ### Why is this bad?
    ///
    /// It can lead to unexpected results in certain situations
    /// For example, when performing mathematical operations
    ///
    /// ### Example
    ///
    /// ```javascript
    /// var x = 2e999;
    /// ```
    NoLossOfPrecision,
    nursery // There are false positives, see https://github.com/web-infra-dev/oxc/issues/656
);

impl Rule for NoLossOfPrecision {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::NumberLiteral(node) if Self::lose_precision(node) => {
                ctx.diagnostic(NoLossOfPrecisionDiagnostic(node.span));
            }
            _ => {}
        }
    }
}

#[derive(Debug, Eq)]
pub struct NormalizedNum<'a> {
    magnitude: isize,
    coefficient: Cow<'a, str>,
}

impl<'a> PartialEq for NormalizedNum<'a> {
    fn eq(&self, other: &Self) -> bool {
        if self.coefficient == "0" {
            true
        } else {
            self.magnitude == other.magnitude && self.coefficient == other.coefficient
        }
    }
}

impl NoLossOfPrecision {
    fn get_raw<'a>(node: &'a NumberLiteral) -> Cow<'a, str> {
        if node.raw.contains('_') {
            Cow::Owned(node.raw.replace('_', ""))
        } else {
            Cow::Borrowed(node.raw)
        }
    }

    fn not_base_ten_loses_precision(node: &'_ NumberLiteral) -> bool {
        let raw = Self::get_raw(node).to_uppercase();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        // AST always store number as f64, need a cast to format in bin/oct/hex
        let value = node.value as u64;
        let suffix = if raw.starts_with("0B") {
            format!("{value:b}")
        } else if raw.starts_with("0X") {
            format!("{value:x}")
        } else {
            format!("{value:o}")
        };
        !raw.ends_with(&suffix.to_uppercase())
    }

    fn base_ten_loses_precision(node: &'_ NumberLiteral) -> bool {
        let normalized_raw_num =
            if let Some(s) = Self::normalize(Self::get_raw(node)) { s } else { return true };
        let precision = normalized_raw_num.coefficient.len();

        if precision > 100 {
            return true;
        }

        let stored_num = Cow::Owned(format!("{:1$}", node.value, precision));
        let normalized_stored_num =
            if let Some(s) = Self::normalize(stored_num) { s } else { return true };
        normalized_raw_num != normalized_stored_num
    }

    fn remove_leading_zeros(num: Cow<'_, str>) -> Cow<'_, str> {
        if num.starts_with('0') {
            Cow::Owned(
                match num.trim_start_matches('0') {
                    "" => "0",
                    s => s,
                }
                .to_string(),
            )
        } else {
            num
        }
    }

    fn remove_trailing_zeros(num: Cow<'_, str>) -> Cow<'_, str> {
        if num.ends_with('0') {
            Cow::Owned(
                match num.trim_end_matches('0') {
                    "" => "0",
                    s => s,
                }
                .to_string(),
            )
        } else {
            num
        }
    }

    fn normalize_int(num: Cow<'_, str>) -> NormalizedNum<'_> {
        // specially deal with 0
        if num == "0" {
            return NormalizedNum { magnitude: 0, coefficient: Cow::Borrowed("0") };
        }

        #[allow(clippy::cast_possible_wrap)]
        // the length of number is larger then isize is almost impossible in real-world codebase
        let magnitude =
            if num.starts_with('0') { num.len() as isize - 2 } else { num.len() as isize - 1 };
        let significant_digits = Self::remove_leading_zeros(Self::remove_trailing_zeros(num));
        NormalizedNum { magnitude, coefficient: significant_digits }
    }

    fn normalize_float(num: Cow<'_, str>) -> NormalizedNum<'_> {
        let trimmed_float = Self::remove_leading_zeros(num);

        return trimmed_float.strip_prefix('.').map_or_else(
            || {
                let trimmed_float = trimmed_float.trim_end_matches('0');
                // unwrap here will never panic, we guarantee the input contains a `.`
                #[allow(clippy::cast_possible_wrap)]
                let magnitude = (trimmed_float.find('.').unwrap() - 1) as isize;
                NormalizedNum { coefficient: Cow::Owned(trimmed_float.replace('.', "")), magnitude }
            },
            |stripped| {
                let decimal_digits = stripped.len();
                let significant_digits =
                    Self::remove_leading_zeros(Cow::Owned(stripped.to_string()));
                #[allow(clippy::cast_possible_wrap)]
                // the length of number is larger then isize is almost impossible in real-world codebase
                let magnitude = significant_digits.len() as isize - decimal_digits as isize - 1;
                NormalizedNum { coefficient: significant_digits, magnitude }
            },
        );
    }

    fn normalize<'a, S: AsRef<str>>(num: S) -> Option<NormalizedNum<'a>> {
        let split_num = num.as_ref().split(|c| c == 'e' || c == 'E').collect::<Vec<_>>();
        let original_coefficient = Cow::Owned(split_num[0].to_owned());
        let normalize_num = if num.as_ref().contains('.') {
            Self::normalize_float(original_coefficient)
        } else {
            Self::normalize_int(original_coefficient)
        };

        let coefficient = normalize_num.coefficient;

        let magnitude = if split_num.len() > 1 {
            if let Ok(n) = split_num[1].parse::<isize>() {
                n + normalize_num.magnitude
            } else {
                return None;
            }
        } else {
            normalize_num.magnitude
        };

        Some(NormalizedNum { magnitude, coefficient })
    }

    pub fn lose_precision(node: &'_ NumberLiteral) -> bool {
        if node.base.is_base_10() {
            Self::base_ten_loses_precision(node)
        } else {
            Self::not_base_ten_loses_precision(node)
        }
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var x = 12345", None),
        ("var x = 123.456", None),
        ("var x = -123.456", None),
        ("var x = -123456", None),
        ("var x = 123e34", None),
        ("var x = 123.0e34", None),
        ("var x = 123e-34", None),
        ("var x = -123e34", None),
        ("var x = -123e-34", None),
        ("var x = 12.3e34", None),
        ("var x = 12.3e-34", None),
        ("var x = -12.3e34", None),
        ("var x = -12.3e-34", None),
        ("var x = 12300000000000000000000000", None),
        ("var x = -12300000000000000000000000", None),
        ("var x = 0.00000000000000000000000123", None),
        ("var x = -0.00000000000000000000000123", None),
        ("var x = 9007199254740991", None),
        ("var x = 0", None),
        ("var x = 0.0", None),
        ("var x = 0.000000000000000000000000000000000000000000000000000000000000000000000000000000", None),
        ("var x = -0", None),
        ("var x = 123.0000000000000000000000", None),
        ("var x = 019.5", None),
        ("var x = 0195", None),
        ("var x = 0e5", None),
        ("var x = 12_34_56", None),
        ("var x = 12_3.4_56", None),
        ("var x = -12_3.4_56", None),
        ("var x = -12_34_56", None),
        ("var x = 12_3e3_4", None),
        ("var x = 123.0e3_4", None),
        ("var x = 12_3e-3_4", None),
        ("var x = 12_3.0e-3_4", None),
        ("var x = -1_23e-3_4", None),
        ("var x = -1_23.8e-3_4", None),
        ("var x = 1_230000000_00000000_00000_000", None),
        ("var x = -1_230000000_00000000_00000_000", None),
        ("var x = 0.0_00_000000000_000000000_00123", None),
        ("var x = -0.0_00_000000000_000000000_00123", None),
        ("var x = 0e5_3", None),
        ("var x = 0b11111111111111111111111111111111111111111111111111111", None),
        ("var x = 0b111_111_111_111_1111_11111_111_11111_1111111111_11111111_111_111", None),
        ("var x = 0B11111111111111111111111111111111111111111111111111111", None),
        ("var x = 0B111_111_111_111_1111_11111_111_11111_1111111111_11111111_111_111", None),
        ("var x = 0o377777777777777777", None),
        ("var x = 0o3_77_777_777_777_777_777", None),
        ("var x = 0O377777777777777777", None),
        // ("var x = 0377777777777777777", None), /* '0'-prefixed octal literals and octal escape sequences are deprecated */
        ("var x = 0x1FFFFFFFFFFFFF", None),
        ("var x = 0X1FFFFFFFFFFFFF", None),
        ("var x = true", None),
        ("var x = 'abc'", None),
        ("var x = ''", None),
        ("var x = null", None),
        ("var x = undefined", None),
        ("var x = {}", None),
        ("var x = ['a', 'b']", None),
        ("var x = new Date()", None),
        ("var x = '9007199254740993'", None),
        ("var x = 0x1FFF_FFFF_FFF_FFF", None),
        ("var x = 0X1_FFF_FFFF_FFF_FFF", None),
        ("var a = Infinity", None),
    ];

    let fail = vec![
        ("var x = 9007199254740993", None),
        ("var x = 9007199254740.993e3", None),
        ("var x = 9.007199254740993e15", None),
        ("var x = -9007199254740993", None),
        ("var x = 900719.9254740994", None),
        ("var x = -900719.9254740994", None),
        ("var x = 900719925474099_3", None),
        ("var x = 90_0719925_4740.9_93e3", None),
        ("var x = 9.0_0719925_474099_3e15", None),
        ("var x = -9_00719_9254_740993", None),
        ("var x = 900_719.92_54740_994", None),
        ("var x = -900_719.92_5474_0994", None),
        ("var x = 5123000000000000000000000000001", None),
        ("var x = -5123000000000000000000000000001", None),
        ("var x = 1230000000000000000000000.0", None),
        ("var x = 1.0000000000000000000000123", None),
        ("var x = 17498005798264095394980017816940970922825355447145699491406164851279623993595007385788105416184430592", None),
        ("var x = 2e999", None),
        ("var x = .1230000000000000000000000", None),
        ("var x = 0b100000000000000000000000000000000000000000000000000001", None),
        ("var x = 0B100000000000000000000000000000000000000000000000000001", None),
        ("var x = 0o400000000000000001", None),
        ("var x = 0O400000000000000001", None),
        ("var x = 0400000000000000001", None),
        ("var x = 0x20000000000001", None),
        ("var x = 0X20000000000001", None),
        ("var x = 5123_00000000000000000000000000_1", None),
        ("var x = -5_12300000000000000000000_0000001", None),
        ("var x = 123_00000000000000000000_00.0_0", None),
        ("var x = 1.0_00000000000000000_0000123", None),
        ("var x = 174_980057982_640953949800178169_409709228253554471456994_914061648512796239935950073857881054_1618443059_2", None),
        ("var x = 2e9_99", None),
        ("var x = .1_23000000000000_00000_0000_0", None),
        ("var x = 0b1_0000000000000000000000000000000000000000000000000000_1", None),
        ("var x = 0B10000000000_0000000000000000000000000000_000000000000001", None),
        ("var x = 0o4_00000000000000_001", None),
        ("var x = 0O4_0000000000000000_1", None),
        ("var x = 0x2_0000000000001", None),
        ("var x = 0X200000_0000000_1", None),
        ("var x = 1e18_446_744_073_709_551_615", None),
    ];

    Tester::new(NoLossOfPrecision::NAME, pass, fail).test_and_snapshot();
}

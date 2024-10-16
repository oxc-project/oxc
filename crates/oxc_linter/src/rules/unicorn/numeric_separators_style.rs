use cow_utils::CowUtils;
use oxc_ast::{
    ast::{BigIntLiteral, NumericLiteral},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn numeric_separators_style_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Invalid group length in numeric value.")
        .with_help("Group digits with numeric separators (_) so longer numbers are easier to read.")
        .with_label(span)
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NumericSeparatorsStyle(Box<NumericSeparatorsStyleConfig>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumericSeparatorsStyleConfig {
    only_if_contains_separator: bool,
    hexadecimal: NumericBaseConfig,
    binary: NumericBaseConfig,
    octal: NumericBaseConfig,
    number: NumericBaseConfig,
}

impl std::ops::Deref for NumericSeparatorsStyle {
    type Target = NumericSeparatorsStyleConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for NumericSeparatorsStyleConfig {
    fn default() -> Self {
        Self {
            only_if_contains_separator: false,
            binary: NumericBaseConfig { group_length: 4, minimum_digits: 0 },
            hexadecimal: NumericBaseConfig { group_length: 2, minimum_digits: 0 },
            number: NumericBaseConfig { group_length: 3, minimum_digits: 5 },
            octal: NumericBaseConfig { group_length: 4, minimum_digits: 0 },
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    /// Enforces a convention of grouping digits using numeric separators.
    ///
    /// ### Why is this bad?
    /// Long numbers can become really hard to read, so cutting it into groups of digits,
    /// separated with a _, is important to keep your code clear. This rule also enforces
    /// a proper usage of the numeric separator, by checking if the groups of digits are
    /// of the correct size.
    ///
    ///
    /// ### Example
    /// ```javascript
    /// const invalid = [
    ///   1_23_4444,
    ///   1_234.56789,
    ///   0xAB_C_D_EF,
    ///   0b10_00_1111,
    ///   0o1_0_44_21,
    ///   1_294_28771_2n,
    /// ];
    /// const valid = [
    ///   1_234_567,
    ///   1_234.567_89,
    ///   0xAB_CD_EF,
    ///   0b1000_1111,
    ///   0o10_4421,
    ///   1_294_287_712n,
    /// ];
    /// ```
    NumericSeparatorsStyle,
    style,
    fix
);

impl Rule for NumericSeparatorsStyle {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::NumericLiteral(number) => {
                if self.only_if_contains_separator && !number.raw.contains('_') {
                    return;
                }

                let formatted = self.format_number(number);

                if formatted != number.raw {
                    ctx.diagnostic_with_fix(
                        numeric_separators_style_diagnostic(number.span),
                        |fixer| fixer.replace(number.span, formatted),
                    );
                }
            }
            AstKind::BigIntLiteral(number) => {
                let raw = number.span.source_text(ctx.source_text());

                if self.only_if_contains_separator && !raw.contains('_') {
                    return;
                }

                let formatted = self.format_bigint(number, raw);

                if formatted.len() != number.span.size() as usize {
                    ctx.diagnostic_with_fix(
                        numeric_separators_style_diagnostic(number.span),
                        |fixer| fixer.replace(number.span, formatted),
                    );
                }
            }
            _ => {}
        };
    }

    fn from_configuration(value: serde_json::Value) -> Self {
        let mut cfg = NumericSeparatorsStyleConfig::default();

        if let Some(config) = value.get(0) {
            if let Some(config) = config.get("binary") {
                cfg.binary.set_numeric_base_from_config(config);
            }
            if let Some(config) = config.get("hexadecimal") {
                cfg.hexadecimal.set_numeric_base_from_config(config);
            }
            if let Some(config) = config.get("number") {
                cfg.number.set_numeric_base_from_config(config);
            }
            if let Some(config) = config.get("octal") {
                cfg.octal.set_numeric_base_from_config(config);
            }

            if let Some(val) =
                config.get("onlyIfContainsSeparator").and_then(serde_json::Value::as_bool)
            {
                cfg.only_if_contains_separator = val;
            }
        }

        Self(Box::new(cfg))
    }
}

impl NumericSeparatorsStyle {
    fn format_number(&self, number: &NumericLiteral) -> String {
        use oxc_syntax::number::NumberBase;

        match number.base {
            NumberBase::Binary => self.format_binary(number.raw),
            NumberBase::Decimal | NumberBase::Float => self.format_decimal(number.raw),
            NumberBase::Hex => self.format_hex(number.raw),
            NumberBase::Octal => self.format_octal(number.raw),
        }
    }

    fn format_bigint(&self, number: &BigIntLiteral, raw: &str) -> String {
        use oxc_syntax::number::BigintBase;

        let raw_without_bigint_n_suffix = &raw[..raw.len() - 1];
        let mut formatted = match number.base {
            BigintBase::Binary => self.format_binary(raw_without_bigint_n_suffix),
            BigintBase::Decimal => self.format_decimal(raw_without_bigint_n_suffix),
            BigintBase::Hex => self.format_hex(raw_without_bigint_n_suffix),
            BigintBase::Octal => self.format_octal(raw_without_bigint_n_suffix),
        };
        formatted.push('n');
        formatted
    }

    fn format_binary(&self, raw_number: &str) -> String {
        let prefix = &raw_number[0..2];

        let mut to_format = raw_number[2..].cow_replace('_', "").into_owned();

        add_separators(&mut to_format, &SeparatorDir::Right, &self.binary);
        to_format.insert_str(0, prefix);
        to_format
    }

    fn format_hex(&self, number_raw: &str) -> String {
        let prefix = &number_raw[0..2];

        let mut to_format = number_raw[2..].cow_replace('_', "").into_owned();

        add_separators(&mut to_format, &SeparatorDir::Right, &self.hexadecimal);
        to_format.insert_str(0, prefix);
        to_format
    }

    fn format_octal(&self, number_raw: &str) -> String {
        // Legacy octal numbers are 0-prefixed, e.g. `010 === 8`.
        // Legacy octal notation does not support `_` prefixes.
        let is_legacy = number_raw.as_bytes()[1] != b'o' && number_raw.as_bytes()[1] != b'O';
        if is_legacy {
            return number_raw.to_string();
        }

        let prefix = &number_raw[0..2];

        let mut to_format = number_raw[2..].cow_replace('_', "").into_owned();

        add_separators(&mut to_format, &SeparatorDir::Right, &self.octal);
        to_format.insert_str(0, prefix);
        to_format
    }

    fn format_decimal(&self, number_raw: &str) -> String {
        let parsed = parse_number_literal(number_raw);

        // Temporary string used for formatting the number
        let mut tmp = String::with_capacity(number_raw.len());
        // Final formatted output string
        let mut out = String::with_capacity(number_raw.len());

        let mut push_formatted_part = |part: &str, dir: &SeparatorDir, out: &mut String| {
            tmp.push_str(part);
            add_separators(&mut tmp, dir, &self.number);
            out.push_str(&tmp);
            tmp.clear();
        };

        if let Some(integer_part) = parsed.integer_part {
            push_formatted_part(
                integer_part.cow_replace('_', "").as_ref(),
                &SeparatorDir::Right,
                &mut out,
            );
        }

        if let Some(decimal_part) = parsed.decimal_part {
            out.push('.');
            push_formatted_part(
                decimal_part.cow_replace('_', "").as_ref(),
                &SeparatorDir::Left,
                &mut out,
            );
        }

        if let Some(exponent_mark) = parsed.exponent_mark {
            out.push(exponent_mark);

            if let Some(exponent_sign) = parsed.exponent_sign {
                out.push_str(exponent_sign);
            }

            if let Some(exponent_part) = parsed.exponent_part {
                push_formatted_part(
                    exponent_part.cow_replace('_', "").as_ref(),
                    &SeparatorDir::Right,
                    &mut out,
                );
            }
        }

        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NumericBaseConfig {
    group_length: usize,
    minimum_digits: usize,
}
impl NumericBaseConfig {
    pub(self) fn set_numeric_base_from_config(&mut self, val: &serde_json::Value) {
        if let Some(group_length) = val.get("groupLength").and_then(serde_json::Value::as_u64) {
            self.group_length = usize::try_from(group_length).unwrap();
        }
        if let Some(minimum_digits) = val.get("minimumDigits").and_then(serde_json::Value::as_u64) {
            self.minimum_digits = usize::try_from(minimum_digits).unwrap();
        }
    }
}

enum SeparatorDir {
    Left,
    Right,
}

fn add_separators(s: &mut String, dir: &SeparatorDir, config: &NumericBaseConfig) {
    if s.len() < config.minimum_digits || s.len() < config.group_length + 1 {
        return;
    }

    match dir {
        SeparatorDir::Right => {
            let mut pos = s.len();
            while pos > config.group_length {
                pos -= config.group_length;
                s.insert(pos, '_');
            }
        }
        SeparatorDir::Left => {
            let mut pos = config.group_length;
            while pos < s.len() {
                s.insert(pos, '_');
                pos += config.group_length + 1;
            }
        }
    }
}

/// Represents a number literal, broken down into its integer, fractional, and exponent parts.
#[derive(Debug)]
struct ParsedNumberLiteral<'n> {
    integer_part: Option<&'n str>,
    decimal_part: Option<&'n str>,
    exponent_mark: Option<char>,
    exponent_sign: Option<&'n str>,
    exponent_part: Option<&'n str>,
}

fn parse_number_literal(num: &str) -> ParsedNumberLiteral {
    let mut parsed = ParsedNumberLiteral {
        integer_part: None,
        decimal_part: None,
        exponent_mark: None,
        exponent_sign: None,
        exponent_part: None,
    };

    let mut offset = 0;

    // Integer part is everything until the decimal separator or the exponent (exclusive).
    let integer_part = num.split_once(['.', 'e', 'E']).map_or(num, |(integer, _)| integer);
    offset += integer_part.len();

    if !integer_part.is_empty() {
        parsed.integer_part = Some(integer_part);
    }

    // Decimal separator is just a dot '.'.
    if let Some(ch) = num[offset..].chars().next() {
        if ch == '.' {
            offset += 1;

            // Decimal part is everything after the decimal separator until the exponent.
            let decimal_part =
                num[offset..].split_once(['e', 'E']).map_or(&num[offset..], |(decimal, _)| decimal);
            offset += decimal_part.len();
            if !decimal_part.is_empty() {
                parsed.decimal_part = Some(decimal_part);
            }
        }
    }

    // Exponent marker is either 'e' or 'E", following the integer part.
    if let Some(ch) = num[offset..].chars().next() {
        if ch == 'e' || ch == 'E' {
            parsed.exponent_mark = Some(ch);
            offset += 1; // note: assuming that 'e' or 'E' is always one byte long

            // Exponent sign is either '+' or '-', following the exponent marker.
            if let Some(ch) = num[offset..].chars().next() {
                if ch == '+' || ch == '-' {
                    parsed.exponent_sign = Some(&num[offset..=offset]);
                    offset += 1; // note: assuming that '+' or '-' is always one byte long
                }
            }

            // Exponent part is everything after the exponent sign.
            let exponent_part = &num[offset..];
            if !exponent_part.is_empty() {
                parsed.exponent_part = Some(exponent_part);
            }
        }
    }

    parsed
}

#[test]
fn test_with_snapshot() {
    use crate::tester::Tester;

    let fail = vec![
        "const foo = 0b10_10_0001",
        "const foo = 0b0_00_0",
        "const foo = 0b10101010101010",
        "const foo = 0B10101010101010",
        "const foo = 0xA_B_CDE_F0",
        "const foo = 0xABCDEF",
        "const foo = 0xA_B",
        "const foo = 0XAB_C_D",
        "const foo = 0o12_34_5670",
        "const foo = 0o7_7_77",
        "const foo = 0o010101010101",
        "const foo = 0O010101010101",
        "const foo = 0b10_10_0001n",
        "const foo = 0b0_00_0n",
        "const foo = 0b10101010101010n",
        "const foo = 0B10101010101010n",
        "const foo = 1_9_223n",
        "const foo = 80_7n",
        "const foo = 123456789_100n",
        "const foo = 1e10000",
        "const foo = 39804e10000",
        "const foo = -123456e100",
        "const foo = -100000e-10000",
        "const foo = -1000e+10000",
        "const foo = -1000e+00010000",
        "const foo = 3.6e12000",
        "const foo = -1200000e5",
        "const foo = 3.65432E12000",
        "const foo = 9807.1234567",
        "const foo = 3819.123_4325",
        "const foo = 138789.12343_2_42",
        "const foo = .000000_1",
        "const foo = 12345678..toString()",
        "const foo = 12345678 .toString()",
        "const foo = .00000",
        "const foo = 0.00000",
        // Numbers
        "const foo = 1_2_345_678",
        "const foo = 12_3",
        "const foo = 1234567890",
        // Negative numbers
        "const foo = -100000_1",
    ];

    let fix = vec![("const foo = 0b10_10_0001", "const foo = 0b1010_0001")];

    Tester::new(NumericSeparatorsStyle::NAME, vec![], fail).expect_fix(fix).test_and_snapshot();
}

#[test]
fn test_number_binary() {
    use crate::tester::Tester;

    let pass = vec![
        "const foo = 0b1010_0001_1000_0101",
        "const foo = 0b0000",
        "const foo = 0b10",
        "const foo = 0b1_0111_0101_0101",
        "const foo = 0B1010",
    ];

    let fail = vec![
        "const foo = 0b10_10_0001",
        "const foo = 0b0_00_0",
        "const foo = 0b10101010101010",
        "const foo = 0B10101010101010",
    ];

    let fix = vec![
        ("const foo = 0b10_10_0001", "const foo = 0b1010_0001", None),
        ("const foo = 0b0_00_0", "const foo = 0b0000", None),
        ("const foo = 0b10101010101010", "const foo = 0b10_1010_1010_1010", None),
        ("const foo = 0B10101010101010", "const foo = 0B10_1010_1010_1010", None),
    ];

    Tester::new(NumericSeparatorsStyle::NAME, pass, fail).expect_fix(fix).test();
}

#[test]
fn test_number_hexadecimal() {
    use crate::tester::Tester;

    let pass = vec![
        "const foo = 0xAB_CD",
        "const foo = 0xAB",
        "const foo = 0xA",
        "const foo = 0xA_BC_DE_F0",
        "const foo = 0xab_e8_12",
        "const foo = 0xe",
        "const foo = 0Xab_e3_cd",
    ];

    let fail = vec![
        "const foo = 0xA_B_CDE_F0",
        "const foo = 0xABCDEF",
        "const foo = 0xA_B",
        "const foo = 0XAB_C_D",
    ];

    let fix = vec![("const foo = 0xA_B_CDE_F0", "const foo = 0xA_BC_DE_F0", None)];

    Tester::new(NumericSeparatorsStyle::NAME, pass, fail).expect_fix(fix).test();
}

#[test]
fn test_number_octal() {
    use crate::tester::Tester;

    let pass = vec![
        "const foo = 0o1234_5670",
        "const foo = 0o7777",
        "const foo = 0o01",
        "const foo = 0o12_7000_0000",
        "const foo = 0O1111_1111",
        // Legacy
        "const foo = 0777777",
        "let foo = 0111222",
    ];

    let fail = vec![
        "const foo = 0o12_34_5670",
        "const foo = 0o7_7_77",
        "const foo = 0o010101010101",
        "const foo = 0O010101010101",
    ];

    let fix = vec![("const foo = 0o12_34_5670", "const foo = 0o1234_5670", None)];

    Tester::new(NumericSeparatorsStyle::NAME, pass, fail).expect_fix(fix).test();
}

#[test]
fn test_bigint_binary() {
    use crate::tester::Tester;

    let pass = vec![
        "const foo = 0b1010_0001_1000_0101n",
        "const foo = 0b0000n",
        "const foo = 0b10n",
        "const foo = 0b1_0111_0101_0101n",
        "const foo = 0B1010n",
    ];

    let fail = vec![
        "const foo = 0b10_10_0001n",
        "const foo = 0b0_00_0n",
        "const foo = 0b10101010101010n",
        "const foo = 0B10101010101010n",
    ];

    let fix = vec![
        ("const foo = 0b10_10_0001n", "const foo = 0b1010_0001n", None),
        ("const foo = 0b0_00_0n", "const foo = 0b0000n", None),
        ("const foo = 0b10101010101010n", "const foo = 0b10_1010_1010_1010n", None),
        ("const foo = 0B10101010101010n", "const foo = 0B10_1010_1010_1010n", None),
    ];

    Tester::new(NumericSeparatorsStyle::NAME, pass, fail).expect_fix(fix).test();
}

#[test]
fn test_bigint() {
    use crate::tester::Tester;

    let pass = vec![
        "const foo = 9_223_372_036_854_775_807n",
        "const foo = 807n",
        "const foo = 1n",
        "const foo = 9_372_854_807n",
        "const foo = 9807n",
        "const foo = 0n",
    ];

    let fail = vec![
        // BigInt
        "const foo = 1_9_223n",
        "const foo = 80_7n",
        "const foo = 123456789_100n",
    ];

    let fix = vec![("const foo = 1_9_223n", "const foo = 19_223n", None)];

    Tester::new(NumericSeparatorsStyle::NAME, pass, fail).expect_fix(fix).test();
}

#[test]
fn test_number_decimal_exponential() {
    use crate::tester::Tester;

    let pass = vec![
        "const foo = 1e10_000",
        "const foo = 39_804e1000",
        "const foo = -123_456e-100",
        "const foo = -100_000e-100_000",
        "const foo = -100_000e+100_000",
        "const foo = 3.6e12_000",
        "const foo = 3.6E12_000",
        "const foo = -1_200_000e5",
    ];

    let fail = vec![
        "const foo = 1e10000",
        "const foo = 39804e10000",
        "const foo = -123456e100",
        "const foo = -100000e-10000",
        "const foo = -1000e+10000",
        "const foo = -1000e+00010000",
        "const foo = 3.6e12000",
        "const foo = -1200000e5",
        "const foo = 3.65432E12000",
    ];

    let fix = vec![
        ("const foo = 1e10000", "const foo = 1e10_000", None),
        ("const foo = 39804e10000", "const foo = 39_804e10_000", None),
        ("const foo = -123456e100", "const foo = -123_456e100", None),
        ("const foo = -100000e-10000", "const foo = -100_000e-10_000", None),
        ("const foo = -1000e+10000", "const foo = -1000e+10_000", None),
        ("const foo = -1000e+00010000", "const foo = -1000e+00_010_000", None),
        ("const foo = 3.6e12000", "const foo = 3.6e12_000", None),
        ("const foo = -1200000e5", "const foo = -1_200_000e5", None),
        ("const foo = 3.65432E12000", "const foo = 3.654_32E12_000", None),
    ];

    Tester::new(NumericSeparatorsStyle::NAME, pass, fail).expect_fix(fix).test();
}

#[test]
fn test_number_decimal_float() {
    use crate::tester::Tester;

    let pass = vec![
        "const foo = 9807.123",
        "const foo = 3819.123_432",
        "const foo = 138_789.123_432_42",
        "const foo = .000_000_1",
    ];

    let fail = vec![
        "const foo = 9807.1234567",
        "const foo = 3819.123_4325",
        "const foo = 138789.12343_2_42",
        "const foo = .000000_1",
        "const foo = 12345678..toString()",
        "const foo = 12345678 .toString()",
        "const foo = .00000",
        "const foo = 0.00000",
    ];

    let fix = vec![("const foo = 9807.1234567", "const foo = 9807.123_456_7", None)];

    Tester::new(NumericSeparatorsStyle::NAME, pass, fail).expect_fix(fix).test();
}

#[test]
fn test_number_decimal_integer() {
    use crate::tester::Tester;

    let pass = vec![
        // Numbers
        "const foo = 123_456",
        "const foo = 12_345_678",
        "const foo = 123",
        "const foo = 1",
        "const foo = 1234",
        // Negative numbers
        "const foo = -4000",
        "const foo = -50_000",
        "const foo = -600_000",
        "const foo = -7_000_000",
        "const foo = -80_000_000",
    ];

    let fail = vec![
        // Numbers
        "const foo = 1_2_345_678",
        "const foo = 12_3",
        "const foo = 1234567890",
        // Negative numbers
        "const foo = -100000_1",
    ];

    let fix = vec![
        ("const foo = 1234567890", "const foo = 1_234_567_890", None),
        ("const foo = -100000_1", "const foo = -1_000_001", None),
    ];

    Tester::new(NumericSeparatorsStyle::NAME, pass, fail).expect_fix(fix).test();
}

#[test]
fn test_with_config() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        ("1234567890", Some(json!([{ "onlyIfContainsSeparator": true }]))),
        ("0b11111111", Some(json!([{ "onlyIfContainsSeparator": true }]))),
        ("0o77777777", Some(json!([{ "onlyIfContainsSeparator": true }]))),
        ("0xffffffff", Some(json!([{ "onlyIfContainsSeparator": true }]))),
        ("1234567890n", Some(json!([{ "onlyIfContainsSeparator": true }]))),
        ("0b11111111n", Some(json!([{ "onlyIfContainsSeparator": true }]))),
        ("0o77777777n", Some(json!([{ "onlyIfContainsSeparator": true }]))),
        ("0xffffffffn", Some(json!([{ "onlyIfContainsSeparator": true }]))),
        ("12_34_56", Some(json!([{ "number": { "groupLength": 2 } }]))),
        ("12345", Some(json!([{ "number": { "minimumDigits": 10 } }]))),
        ("0b1_1_1_1_1", Some(json!([{ "binary": { "groupLength": 1 } }]))),
        ("0o7_7_7_7_7", Some(json!([{ "octal": { "groupLength": 1 } }]))),
        ("0xf_f_f_f_f", Some(json!([{ "hexadecimal": { "groupLength": 1 } }]))),
        ("12_34_56n", Some(json!([{ "number": { "groupLength": 2 } }]))),
        ("12345n", Some(json!([{ "number": { "minimumDigits": 10 } }]))),
        ("0b1_1_1_1_1n", Some(json!([{ "binary": { "groupLength": 1 } }]))),
        ("0o7_7_7_7_7n", Some(json!([{ "octal": { "groupLength": 1 } }]))),
        ("0xf_f_f_f_fn", Some(json!([{ "hexadecimal": { "groupLength": 1 } }]))),
    ];

    let fail = vec![];

    Tester::new(NumericSeparatorsStyle::NAME, pass, fail).intentionally_allow_no_fix_tests().test();
}

#[test]
fn test_misc() {
    use crate::tester::Tester;

    let pass = vec![
        "const foo = -282_932 - (1938 / 10_000) * .1 + 18.100_000_2",
        "const foo = NaN",
        "const foo = Infinity",
        "const foo = -Infinity",
        "const foo = '1234567n'",
    ];

    let fail = vec!["1_23_4444"];
    let fix = vec![("1_23_4444", "1_234_444")];

    Tester::new(NumericSeparatorsStyle::NAME, pass, fail).expect_fix(fix).test();
}

#[cfg(test)]
mod internal_tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_from_configuration() {
        let config = json!([{
                "binary": {"groupLength": 2, "minimumDigits": 4},
                "hexadecimal": {"groupLength": 8, "minimumDigits": 16},
                "number": {"groupLength": 32, "minimumDigits": 64},
                "octal": {"groupLength": 128, "minimumDigits": 256},
                "onlyIfContainsSeparator": true
        }]);
        let rule = NumericSeparatorsStyle::from_configuration(config);

        assert_eq!(rule.binary.group_length, 2);
        assert_eq!(rule.binary.minimum_digits, 4);
        assert_eq!(rule.hexadecimal.group_length, 8);
        assert_eq!(rule.hexadecimal.minimum_digits, 16);
        assert_eq!(rule.number.group_length, 32);
        assert_eq!(rule.number.minimum_digits, 64);
        assert_eq!(rule.octal.group_length, 128);
        assert_eq!(rule.octal.minimum_digits, 256);
        assert!(rule.only_if_contains_separator);
    }

    #[test]
    fn test_from_empty_configuration() {
        let rule = NumericSeparatorsStyle::from_configuration(json!([]));
        assert_eq!(rule, NumericSeparatorsStyle::default());
    }
}

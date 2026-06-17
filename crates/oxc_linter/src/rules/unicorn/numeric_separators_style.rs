use cow_utils::CowUtils;
use oxc_ast::{
    AstKind,
    ast::{BigIntLiteral, BigintBase, NumberBase, NumericLiteral},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{AstNode, context::LintContext, rule::Rule};

fn numeric_separators_style_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Invalid group length in numeric value.")
        .with_help("Group digits with numeric separators (_) so longer numbers are easier to read.")
        .with_label(span)
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct NumericSeparatorsStyle(Box<NumericSeparatorsStyleConfig>);

#[derive(Debug, Clone, PartialEq, Eq, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NumericSeparatorsStyleConfig {
    /// Only enforce the rule when the numeric literal already contains a separator (`_`).
    ///
    /// When `true`, numbers without separators are left as-is; when `false` (default),
    /// grouping will be enforced for eligible numbers even if they don't include separators yet.
    only_if_contains_separator: bool,
    /// Configuration for hexadecimal literals (e.g. `0xAB_CD`, `0Xab_cd`, and bigint variants).
    /// Controls how digits are grouped and when separators are applied.
    hexadecimal: NumericBaseConfig,
    /// Configuration for binary literals (e.g. `0b1010_0001` and bigint variants).
    /// Controls how digits are grouped and when separators are applied.
    binary: NumericBaseConfig,
    /// Configuration for octal literals (e.g. `0o1234_5670` and bigint variants).
    /// Controls how digits are grouped and when separators are applied.
    octal: NumericBaseConfig,
    /// Configuration for decimal numbers (integers, fraction parts, and exponents).
    /// Controls how digits are grouped and when separators are applied.
    number: NumericNumberConfig,
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
            binary: NumericBaseConfig {
                group_length: 4,
                minimum_digits: 0,
                only_if_contains_separator: None,
            },
            hexadecimal: NumericBaseConfig {
                group_length: 2,
                minimum_digits: 0,
                only_if_contains_separator: None,
            },
            number: NumericNumberConfig::default(),
            octal: NumericBaseConfig {
                group_length: 4,
                minimum_digits: 0,
                only_if_contains_separator: None,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct NumericBaseConfig {
    /// Only enforce the rule when the numeric literal already contains a separator (`_`).
    ///
    /// When `true`, numbers without separators are left as-is; when `false` (default),
    /// grouping will be enforced for eligible numbers even if they don't include separators yet.
    #[serde(skip_serializing_if = "Option::is_none")]
    only_if_contains_separator: Option<bool>,
    /// The number of digits per group when inserting numeric separators.
    /// For example, a `groupLength` of 3 formats `1234567` as `1_234_567`.
    group_length: usize,
    /// The minimum number of digits required before grouping is applied.
    /// Values with fewer digits than this threshold will not be grouped.
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

        if let Some(val) = val.get("onlyIfContainsSeparator").map(serde_json::Value::as_bool) {
            self.only_if_contains_separator = val;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct NumericNumberConfig {
    #[serde(flatten)]
    base: NumericBaseConfig,
    /// The size a group of digits in the fractional part (after the decimal point) should be.
    fraction_group_length: usize,
}

impl NumericNumberConfig {
    pub(self) fn set_numeric_number_from_config(&mut self, val: &serde_json::Value) {
        self.base.set_numeric_base_from_config(val);
        if let Some(fraction_group_length) =
            val.get("fractionGroupLength").and_then(serde_json::Value::as_u64)
        {
            self.fraction_group_length = usize::try_from(fraction_group_length).unwrap();
        }
    }
}

impl Default for NumericNumberConfig {
    fn default() -> Self {
        Self {
            base: NumericBaseConfig {
                group_length: 3,
                minimum_digits: 5,
                only_if_contains_separator: None,
            },
            fraction_group_length: usize::MAX,
        }
    }
}
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces a convention of grouping digits using numeric separators.
    ///
    /// ### Why is this bad?
    ///
    /// A long series of digits can be difficult to read, and
    /// it can be difficult to determine the value of the number at a glance.
    /// Breaking up the digits with numeric separators (`_`) can greatly
    /// improve readability.
    ///
    /// Compare the following two numbers and how easy it is to understand their magnitude:
    ///
    /// ```js
    /// 1000000000;
    /// 1_000_000_000;
    /// ```
    ///
    /// This rule also enforces proper group size, for example
    /// enforcing that the `_` separator is used every 3 digits.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const invalid = [
    ///   1_23_4444,
    ///   1_234.56789,
    ///   0xAB_C_D_EF,
    ///   0b10_00_1111,
    ///   0o1_0_44_21,
    ///   1_294_28771_2n,
    /// ];
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
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
    unicorn,
    style,
    fix,
    config = NumericSeparatorsStyleConfig,
    version = "0.0.19",
    short_description = "Enforces a convention of grouping digits using numeric separators.",
);

impl Rule for NumericSeparatorsStyle {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let mut cfg = NumericSeparatorsStyleConfig::default();

        if let Some(config) = value.get(0) {
            if let Some(config) = config.get("binary") {
                cfg.binary.set_numeric_base_from_config(config);
            }
            if let Some(config) = config.get("hexadecimal") {
                cfg.hexadecimal.set_numeric_base_from_config(config);
            }
            if let Some(config) = config.get("number") {
                cfg.number.set_numeric_number_from_config(config);
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

        Ok(Self(Box::new(cfg)))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::NumericLiteral(number) => {
                let raw = number.raw.as_ref().unwrap().as_str();
                if !raw.contains('_') && self.skip_number_separator(number.base) {
                    return;
                }

                let formatted = self.format_number(number);

                if formatted != raw {
                    ctx.diagnostic_with_fix(
                        numeric_separators_style_diagnostic(number.span),
                        |fixer| fixer.replace(number.span, formatted),
                    );
                }
            }
            AstKind::BigIntLiteral(number) => {
                let raw = number.raw.unwrap().as_str();
                if !raw.contains('_') && self.skip_bigint_separator(number.base) {
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
        }
    }
}

impl NumericSeparatorsStyle {
    fn skip_number_separator(&self, base: NumberBase) -> bool {
        let base_config = match base {
            NumberBase::Binary => self.binary.only_if_contains_separator,
            NumberBase::Decimal | NumberBase::Float => self.number.base.only_if_contains_separator,
            NumberBase::Hex => self.hexadecimal.only_if_contains_separator,
            NumberBase::Octal => self.octal.only_if_contains_separator,
        };
        base_config.unwrap_or(self.only_if_contains_separator)
    }

    fn skip_bigint_separator(&self, base: BigintBase) -> bool {
        let base_config = match base {
            BigintBase::Binary => self.binary.only_if_contains_separator,
            BigintBase::Decimal => self.number.base.only_if_contains_separator,
            BigintBase::Hex => self.hexadecimal.only_if_contains_separator,
            BigintBase::Octal => self.octal.only_if_contains_separator,
        };
        base_config.unwrap_or(self.only_if_contains_separator)
    }

    fn format_number(&self, number: &NumericLiteral) -> String {
        use oxc_syntax::number::NumberBase;

        let raw = number.raw.as_ref().unwrap();
        match number.base {
            NumberBase::Binary => self.format_binary(raw),
            NumberBase::Decimal | NumberBase::Float => self.format_decimal(raw),
            NumberBase::Hex => self.format_hex(raw),
            NumberBase::Octal => self.format_octal(raw),
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

        add_separators(&mut to_format, &SeparatorDir::Right, &self.binary, None);
        to_format.insert_str(0, prefix);
        to_format
    }

    fn format_hex(&self, number_raw: &str) -> String {
        let prefix = &number_raw[0..2];

        let mut to_format = number_raw[2..].cow_replace('_', "").into_owned();

        add_separators(&mut to_format, &SeparatorDir::Right, &self.hexadecimal, None);
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

        add_separators(&mut to_format, &SeparatorDir::Right, &self.octal, None);
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
            add_separators(
                &mut tmp,
                dir,
                &self.number.base,
                Some(self.number.fraction_group_length),
            );
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

#[derive(Debug, PartialEq, Eq)]
enum SeparatorDir {
    Left,
    Right,
}

fn add_separators(
    s: &mut String,
    dir: &SeparatorDir,
    config: &NumericBaseConfig,
    fraction_group_length: Option<usize>,
) {
    let length = if *dir == SeparatorDir::Left && fraction_group_length.is_some() {
        fraction_group_length.unwrap_or(usize::MAX)
    } else {
        config.group_length
    };

    if s.len() < config.minimum_digits || s.len() < length.saturating_add(1) {
        return;
    }

    match dir {
        SeparatorDir::Right => {
            let mut pos = s.len();
            while pos > length {
                pos -= length;
                s.insert(pos, '_');
            }
        }
        SeparatorDir::Left => {
            let mut pos = length;
            while pos < s.len() {
                s.insert(pos, '_');
                pos += length + 1;
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

fn parse_number_literal(num: &str) -> ParsedNumberLiteral<'_> {
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
    if let Some(ch) = num[offset..].chars().next()
        && ch == '.'
    {
        offset += 1;

        // Decimal part is everything after the decimal separator until the exponent.
        let decimal_part =
            num[offset..].split_once(['e', 'E']).map_or(&num[offset..], |(decimal, _)| decimal);
        offset += decimal_part.len();
        if !decimal_part.is_empty() {
            parsed.decimal_part = Some(decimal_part);
        }
    }

    // Exponent marker is either 'e' or 'E", following the integer part.
    if let Some(ch) = num[offset..].chars().next()
        && (ch == 'e' || ch == 'E')
    {
        parsed.exponent_mark = Some(ch);
        offset += 1; // note: assuming that 'e' or 'E' is always one byte long

        // Exponent sign is either '+' or '-', following the exponent marker.
        if let Some(ch) = num[offset..].chars().next()
            && (ch == '+' || ch == '-')
        {
            parsed.exponent_sign = Some(&num[offset..=offset]);
            offset += 1; // note: assuming that '+' or '-' is always one byte long
        }

        // Exponent part is everything after the exponent sign.
        let exponent_part = &num[offset..];
        if !exponent_part.is_empty() {
            parsed.exponent_part = Some(exponent_part);
        }
    }

    parsed
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("const foo = 0xAB_CD", None),
        ("const foo = 0xAB", None),
        ("const foo = 0xA", None),
        ("const foo = 0xA_BC_DE_F0", None),
        ("const foo = 0xab_e8_12", None),
        ("const foo = 0xe", None),
        ("const foo = 0Xab_e3_cd", None),
        ("const foo = 0o1234_5670", None),
        ("const foo = 0o7777", None),
        ("const foo = 0o01", None),
        ("const foo = 0o12_7000_0000", None),
        ("const foo = 0O1111_1111", None),
        ("const foo = 0b1010_0001_1000_0101", None),
        ("const foo = 0b0000", None),
        ("const foo = 0b10", None),
        ("const foo = 0b1_0111_0101_0101", None),
        ("const foo = 0B1010", None),
        ("const foo = 0b1010n", None),
        ("const foo = 0b1010_1010n", None),
        ("const foo = 9_223_372_036_854_775_807n", None),
        ("const foo = 807n", None),
        ("const foo = 1n", None),
        ("const foo = 9_372_854_807n", None),
        ("const foo = 9807n", None),
        ("const foo = 0n", None),
        ("const foo = 12_345_678", None),
        ("const foo = 123", None),
        ("const foo = 1", None),
        ("const foo = 1234", None),
        (
            "const foo = 1_234",
            Some(serde_json::json!([{"number": {"minimumDigits": 0, "groupLength": 3}}])),
        ),
        ("const foo = 9807.123", None),
        ("const foo = 3819.123432", None),
        ("const foo = 138_789.12343242", None),
        ("const foo = .0000001", None),
        ("const foo = .00000", None),
        ("const foo = 0.00000", None),
        ("const foo = 0.55228474983", None),
        (
            "const foo = 1.234_567",
            Some(serde_json::json!([{"number": {"fractionGroupLength": 3}}])),
        ),
        (
            "const foo = 149_597_870_700.31415_92653_58979",
            Some(
                serde_json::json!([{"number": {"minimumDigits": 0, "groupLength": 3, "fractionGroupLength": 5}}]),
            ),
        ),
        ("const foo = 1.2345", Some(serde_json::json!([{"number": {"fractionGroupLength": 2}}]))),
        ("const foo = -3000", None),
        ("const foo = -10_000_000", None),
        ("const foo = 1e10_000", None),
        ("const foo = 39_804e1000", None),
        ("const foo = -123_456e-100", None),
        ("const foo = -100_000e-100_000", None),
        ("const foo = -100_000e+100_000", None),
        ("const foo = 3.6e12_000", None),
        ("const foo = 3.6E12_000", None),
        ("const foo = -1_200_000e5", None),
        ("const foo = -282_932 - (1938 / 10_000) * .1 + 18.1000002", None),
        ("const foo = NaN", None),
        ("const foo = Infinity", None),
        (r#"const foo = "1234567n""#, None),
        ("const foo = 10000", Some(serde_json::json!([{"number": {"minimumDigits": 6}}]))),
        ("const foo = 100_0000_0000", Some(serde_json::json!([{"number": {"groupLength": 4}}]))),
        (
            "const foo = 0xA_B_C_D_E_1_2_3_4",
            Some(serde_json::json!([{"hexadecimal": {"groupLength": 1}}])),
        ),
        (
            "const foo = 0b111",
            Some(serde_json::json!([{"number": {"minimumDigits": 3, "groupLength": 1}}])),
        ),
        (
            "const binary = 0b10101010;
            const octal = 0o76543210;
            const hexadecimal = 0xfedcba97;
            const number = 12345678.12345678e12345678;",
            Some(serde_json::json!([{ "onlyIfContainsSeparator": true, }])),
        ),
        (
            "const binary = 0b1010_1010;
            const octal = 0o76543210;
            const hexadecimal = 0xfedcba97;
            const number = 12345678.12345678e12345678;",
            Some(
                serde_json::json!([{ "onlyIfContainsSeparator": true, "binary": { "onlyIfContainsSeparator": false, }, }]),
            ),
        ),
        (
            "const binary = 0b10_10_10_10;
            const octal = 0o76543210;
            const hexadecimal = 0xfedcba97;
            const number = 12345678.12345678e12345678;",
            Some(
                serde_json::json!([{ "onlyIfContainsSeparator": true, "binary": { "onlyIfContainsSeparator": false, "groupLength": 2, }, }]),
            ),
        ),
        (
            "const binary = 0b10101010;
            const octal = 0o7654_3210;
            const hexadecimal = 0xfe_dc_ba_97;
            const number = 12_345_678.12345678e12_345_678;",
            Some(serde_json::json!([{ "binary": { "onlyIfContainsSeparator": true, }, }])),
        ),
        (
            "const foo = 12345",
            Some(serde_json::json!([{"number": {"onlyIfContainsSeparator": true}}])),
        ),
        (
            "const foo = 12345678",
            Some(serde_json::json!([{"number": {"onlyIfContainsSeparator": true}}])),
        ),
        (
            "const foo = 12_345",
            Some(serde_json::json!([{"number": {"onlyIfContainsSeparator": true}}])),
        ),
        (
            "const foo = -100_000e+100_000",
            Some(serde_json::json!([{"number": {"onlyIfContainsSeparator": true}}])),
        ),
        (
            "const foo = -100000e+100000",
            Some(serde_json::json!([{"number": {"onlyIfContainsSeparator": true}}])),
        ),
        (
            "const foo = 0xA_B_C_D_E",
            Some(
                serde_json::json!([{"hexadecimal": {"onlyIfContainsSeparator": true, "groupLength": 1}}]),
            ),
        ),
        (
            "const foo = 0o7777",
            Some(
                serde_json::json!([{"octal": {"onlyIfContainsSeparator": true, "minimumDigits": 4}}]),
            ),
        ),
        (
            "const foo = 0xABCDEF012",
            Some(serde_json::json!([{"hexadecimal": {"onlyIfContainsSeparator": true}}])),
        ),
        (
            "const foo = 0o777777",
            Some(
                serde_json::json!([{"octal": {"onlyIfContainsSeparator": true, "minimumDigits": 3}}]),
            ),
        ),
        (
            "const foo = 0o777777",
            Some(
                serde_json::json!([{"octal": {"onlyIfContainsSeparator": true, "minimumDigits": 3, "groupLength": 2}}]),
            ),
        ),
        (
            "const foo = 0o777_777",
            Some(
                serde_json::json!([{"octal": {"onlyIfContainsSeparator": true, "minimumDigits": 2, "groupLength": 3}}]),
            ),
        ),
        (
            "const foo = 0b01010101",
            Some(
                serde_json::json!([{"onlyIfContainsSeparator": true, "binary": {"onlyIfContainsSeparator": true}}]),
            ),
        ),
        (
            "const foo = 0b0101_0101",
            Some(
                serde_json::json!([{"onlyIfContainsSeparator": false, "binary": {"onlyIfContainsSeparator": true}}]),
            ),
        ),
        (
            "const foo = 0b0101_0101",
            Some(
                serde_json::json!([{"onlyIfContainsSeparator": false, "binary": {"onlyIfContainsSeparator": false}}]),
            ),
        ),
    ];

    let fail = vec![
        ("const foo = 0xA_B_CDE_F0", None),
        ("const foo = 0xABCDEF", None),
        ("const foo = 0xA_B", None),
        ("const foo = 0XAB_C_D", None),
        ("const foo = 0o12_34_5670", None),
        ("const foo = 0o7_7_77", None),
        ("const foo = 0o010101010101", None),
        ("const foo = 0O010101010101", None),
        ("const foo = 0b10_10_0001", None),
        ("const foo = 0b0_00_0", None),
        ("const foo = 0b10101010101010", None),
        ("const foo = 0B10101010101010", None),
        ("const foo = 1_9_223n", None),
        ("const foo = 80_7n", None),
        ("const foo = 123456789_100n", None),
        ("const foo = 1_2_345_678", None),
        ("const foo = 12_3", None),
        ("const foo = 1234567890", None),
        ("const foo = 0.552_284_749_83", None),
        ("const foo = 3819.123_4325", None),
        ("const foo = 138789.12343_2_42", None),
        ("const foo = .000000_1", None),
        ("const foo = 12345678..toString()", None),
        ("const foo = 12345678 .toString()", None),
        ("const foo = 1.234567", Some(serde_json::json!([{"number": {"fractionGroupLength": 3}}]))),
        (
            "const foo = 1789.123_432_42",
            Some(serde_json::json!([{"number": {"onlyIfContainsSeparator": true}}])),
        ),
        (
            "const foo = -282_932 - (1938 / 10_000) * .1 + 18.100_000_2",
            Some(serde_json::json!([{"number": {"onlyIfContainsSeparator": true}}])),
        ),
        ("const foo = -100000_1", None),
        ("const foo = 1e10000", None),
        ("const foo = 39804e10000", None),
        ("const foo = -123456e100", None),
        ("const foo = -100000e-10000", None),
        ("const foo = -1000e+10000", None),
        ("const foo = -1000e+00010000", None),
        ("const foo = 3.6e12000", None),
        ("const foo = -1200000e5", None),
        ("const foo = 3.65432E12000", None),
        ("const foo = 1000000", Some(serde_json::json!([{"number": {"minimumDigits": 6}}]))),
        ("const foo = 10_000_000_000", Some(serde_json::json!([{"number": {"groupLength": 4}}]))),
        ("const foo = 0xA_B_CD", Some(serde_json::json!([{"hexadecimal": {"groupLength": 1}}]))),
        (
            "const foo = 0b1_11",
            Some(serde_json::json!([{"number": {"minimumDigits": 3, "groupLength": 2}}])),
        ),
        (
            "const foo = -100000e+100000",
            Some(serde_json::json!([{"number": {"onlyIfContainsSeparator": false}}])),
        ),
        (
            "const binary = 0b10_101010;
            const octal = 0o76_543210;
            const hexadecimal = 0xfe_dcba97;
            const number = 12_345678.12345678e12345678;",
            Some(serde_json::json!([{ "onlyIfContainsSeparator": true, }])),
        ),
        (
            "const binary = 0b10101010;
            const octal = 0o76_543210;
            const hexadecimal = 0xfe_dcba97;
            const number = 12_345678.12345678e12345678;",
            Some(
                serde_json::json!([{ "onlyIfContainsSeparator": true, "binary": { "onlyIfContainsSeparator": false, }, }]),
            ),
        ),
        (
            "const binary = 0b10101010;
            const octal = 0o76_543210;
            const hexadecimal = 0xfe_dcba97;
            const number = 12_345678.12345678e12345678;",
            Some(
                serde_json::json!([{ "onlyIfContainsSeparator": true, "binary": { "onlyIfContainsSeparator": false, "groupLength": 2, }, }]),
            ),
        ),
        (
            "const binary = 0b10_101010;
            const octal = 0o76543210;
            const hexadecimal = 0xfedcba97;
            const number = 12345678.12345678e12345678;",
            Some(serde_json::json!([{ "binary": { "onlyIfContainsSeparator": true, }, }])),
        ),
        ("console.log(0XdeEdBeeFn)", None),
        ("const foo = 12345678..toString()", None),
    ];

    let fix = vec![
        ("const foo = 0xA_B_CDE_F0", "const foo = 0xA_BC_DE_F0", None),
        ("const foo = 0xABCDEF", "const foo = 0xAB_CD_EF", None),
        ("const foo = 0xA_B", "const foo = 0xAB", None),
        ("const foo = 0XAB_C_D", "const foo = 0XAB_CD", None),
        ("const foo = 0o12_34_5670", "const foo = 0o1234_5670", None),
        ("const foo = 0o7_7_77", "const foo = 0o7777", None),
        ("const foo = 0o010101010101", "const foo = 0o0101_0101_0101", None),
        ("const foo = 0O010101010101", "const foo = 0O0101_0101_0101", None),
        ("const foo = 0b10_10_0001", "const foo = 0b1010_0001", None),
        ("const foo = 0b0_00_0", "const foo = 0b0000", None),
        ("const foo = 0b10101010101010", "const foo = 0b10_1010_1010_1010", None),
        ("const foo = 0B10101010101010", "const foo = 0B10_1010_1010_1010", None),
        ("const foo = 1_9_223n", "const foo = 19_223n", None),
        ("const foo = 80_7n", "const foo = 807n", None),
        ("const foo = 123456789_100n", "const foo = 123_456_789_100n", None),
        ("const foo = 1_2_345_678", "const foo = 12_345_678", None),
        ("const foo = 12_3", "const foo = 123", None),
        ("const foo = 1234567890", "const foo = 1_234_567_890", None),
        ("const foo = 0.552_284_749_83", "const foo = 0.55228474983", None),
        ("const foo = 3819.123_4325", "const foo = 3819.1234325", None),
        ("const foo = 138789.12343_2_42", "const foo = 138_789.12343242", None),
        ("const foo = .000000_1", "const foo = .0000001", None),
        // ("const foo = 12345678..toString()", "const foo = 12_345_678..toString()", None), // TODO: the second dot is needed
        ("const foo = 12345678 .toString()", "const foo = 12_345_678 .toString()", None),
        (
            "const foo = 1.234567",
            "const foo = 1.234_567",
            Some(serde_json::json!([{"number": {"fractionGroupLength": 3}}])),
        ),
        (
            "const foo = 1789.123_432_42",
            "const foo = 1789.12343242",
            Some(serde_json::json!([{"number": {"onlyIfContainsSeparator": true}}])),
        ),
        (
            "const foo = -282_932 - (1938 / 10_000) * .1 + 18.100_000_2",
            "const foo = -282_932 - (1938 / 10_000) * .1 + 18.1000002",
            Some(serde_json::json!([{"number": {"onlyIfContainsSeparator": true}}])),
        ),
        ("const foo = -100000_1", "const foo = -1_000_001", None),
        ("const foo = 1e10000", "const foo = 1e10_000", None),
        ("const foo = 39804e10000", "const foo = 39_804e10_000", None),
        ("const foo = -123456e100", "const foo = -123_456e100", None),
        ("const foo = -100000e-10000", "const foo = -100_000e-10_000", None),
        ("const foo = -1000e+10000", "const foo = -1000e+10_000", None),
        ("const foo = -1000e+00010000", "const foo = -1000e+00_010_000", None),
        ("const foo = 3.6e12000", "const foo = 3.6e12_000", None),
        ("const foo = -1200000e5", "const foo = -1_200_000e5", None),
        ("const foo = 3.65432E12000", "const foo = 3.65432E12_000", None),
        (
            "const foo = 1000000",
            "const foo = 1_000_000",
            Some(serde_json::json!([{"number": {"minimumDigits": 6}}])),
        ),
        (
            "const foo = 10_000_000_000",
            "const foo = 100_0000_0000",
            Some(serde_json::json!([{"number": {"groupLength": 4}}])),
        ),
        (
            "const foo = 0xA_B_CD",
            "const foo = 0xA_B_C_D",
            Some(serde_json::json!([{"hexadecimal": {"groupLength": 1}}])),
        ),
        (
            "const foo = 0b1_11",
            "const foo = 0b111",
            Some(serde_json::json!([{"number": {"minimumDigits": 3, "groupLength": 2}}])),
        ),
        (
            "const foo = -100000e+100000",
            "const foo = -100_000e+100_000",
            Some(serde_json::json!([{"number": {"onlyIfContainsSeparator": false}}])),
        ),
        (
            "const binary = 0b10_101010;
            const octal = 0o76_543210;
            const hexadecimal = 0xfe_dcba97;
            const number = 12_345678.12345678e12345678;",
            "const binary = 0b1010_1010;
            const octal = 0o7654_3210;
            const hexadecimal = 0xfe_dc_ba_97;
            const number = 12_345_678.12345678e12_345_678;",
            Some(serde_json::json!([{ "onlyIfContainsSeparator": true, }])),
        ),
        (
            "const binary = 0b10101010;
            const octal = 0o76_543210;
            const hexadecimal = 0xfe_dcba97;
            const number = 12_345678.12345678e12345678;",
            "const binary = 0b1010_1010;
            const octal = 0o7654_3210;
            const hexadecimal = 0xfe_dc_ba_97;
            const number = 12_345_678.12345678e12_345_678;",
            Some(
                serde_json::json!([{ "onlyIfContainsSeparator": true, "binary": { "onlyIfContainsSeparator": false, }, }]),
            ),
        ),
        (
            "const binary = 0b10101010;
            const octal = 0o76_543210;
            const hexadecimal = 0xfe_dcba97;
            const number = 12_345678.12345678e12345678;",
            "const binary = 0b10_10_10_10;
            const octal = 0o7654_3210;
            const hexadecimal = 0xfe_dc_ba_97;
            const number = 12_345_678.12345678e12_345_678;",
            Some(
                serde_json::json!([{ "onlyIfContainsSeparator": true, "binary": { "onlyIfContainsSeparator": false, "groupLength": 2, }, }]),
            ),
        ),
        (
            "const binary = 0b10_101010;
            const octal = 0o76543210;
            const hexadecimal = 0xfedcba97;
            const number = 12345678.12345678e12345678;",
            "const binary = 0b1010_1010;
            const octal = 0o7654_3210;
            const hexadecimal = 0xfe_dc_ba_97;
            const number = 12_345_678.12345678e12_345_678;",
            Some(serde_json::json!([{ "binary": { "onlyIfContainsSeparator": true, }, }])),
        ),
    ];

    Tester::new(NumericSeparatorsStyle::NAME, NumericSeparatorsStyle::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
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
                "number": {"groupLength": 32, "minimumDigits": 64, "fractionGroupLength": 128},
                "octal": {"groupLength": 128, "minimumDigits": 256},
                "onlyIfContainsSeparator": true
        }]);
        let rule = NumericSeparatorsStyle::from_configuration(config).unwrap();

        assert_eq!(rule.binary.group_length, 2);
        assert_eq!(rule.binary.minimum_digits, 4);
        assert_eq!(rule.hexadecimal.group_length, 8);
        assert_eq!(rule.hexadecimal.minimum_digits, 16);
        assert_eq!(rule.number.base.group_length, 32);
        assert_eq!(rule.number.base.minimum_digits, 64);
        assert_eq!(rule.number.fraction_group_length, 128);
        assert_eq!(rule.octal.group_length, 128);
        assert_eq!(rule.octal.minimum_digits, 256);
        assert!(rule.only_if_contains_separator);
    }

    #[test]
    fn test_from_empty_configuration() {
        let rule = NumericSeparatorsStyle::from_configuration(json!([])).unwrap();
        assert_eq!(rule, NumericSeparatorsStyle::default());
    }
}

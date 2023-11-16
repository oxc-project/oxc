use itertools::{intersperse, Itertools};
use oxc_ast::{
    ast::{BigintLiteral, NumberLiteral},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_formatter::Gen;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use regex::Regex;

use crate::{context::LintContext, fixer::Fix, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(numeric-separators-style): Enforce the style of numeric separators by correctly grouping digits")]
#[diagnostic(
    severity(error),
    help("Use the number separator `_` to break up longer numbers for easier reading.")
)]
struct NumericSeparatorsStyleDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NumericSeparatorsStyle;
impl NumericSeparatorsStyle {
    fn format_number(&self, number: &NumberLiteral) -> String {
        FormatNumber(number).format()
    }

    fn format_bigint(&self, number: &BigintLiteral) -> String {
        todo!()
    }
}

struct FormatNumber<'a>(&'a NumberLiteral<'a>);
impl<'a> FormatNumber<'a> {
    fn format(&self) -> String {
        match self.0.base {
            oxc_syntax::NumberBase::Binary => {
                format!("{:b}", self)
            }
            oxc_syntax::NumberBase::Decimal => {
                format!("{}", self)
            }
            oxc_syntax::NumberBase::Float => {
                format!("{}", self)
            }
            oxc_syntax::NumberBase::Hex => todo!(),
            oxc_syntax::NumberBase::Octal => todo!(),
        }
    }
}
impl<'a> std::fmt::Display for FormatNumber<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = format!("{}", self.0.value);
        if let Some((whole, decimal)) = string.split_once('.') {
            let starts_with_decimal = self.0.raw.starts_with('.');

            if (starts_with_decimal) {
                let mut decimal = decimal.to_string();
                add_separators(
                    &mut decimal,
                    SeparatorDir::Left,
                    DECIMAL_MINIMUM_DIGITS,
                    DECIMAL_GROUP_LENGTH,
                );
                f.write_str(".")?;
                f.write_str(&decimal)
            } else {
                let mut whole = whole.to_string();
                let mut decimal = decimal.to_string();
                add_separators(
                    &mut whole,
                    SeparatorDir::Right,
                    DECIMAL_MINIMUM_DIGITS,
                    DECIMAL_GROUP_LENGTH,
                );
                add_separators(
                    &mut decimal,
                    SeparatorDir::Left,
                    DECIMAL_MINIMUM_DIGITS,
                    DECIMAL_GROUP_LENGTH,
                );
                f.write_str(&whole)?;
                f.write_str(".")?;
                f.write_str(&decimal)
            }
        } else {
            add_separators(
                &mut string,
                SeparatorDir::Right,
                DECIMAL_MINIMUM_DIGITS,
                DECIMAL_GROUP_LENGTH,
            );
            f.write_str(&string)
        }
    }
}
impl<'a> std::fmt::Binary for FormatNumber<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let binary_prefix = &self.0.raw[0..2];
        let padding = self.0.raw.replace('_', "").len() - 2;
        let mut binary_string = format!("{:0width$b}", self.0.value as i64, width = padding);
        // dbg!(padding);
        // dbg!(binary_string.to_string());
        add_separators(
            &mut binary_string,
            SeparatorDir::Right,
            BINARY_MINIMUM_DIGITS,
            BINARY_GROUP_LENGTH,
        );
        f.write_str(binary_prefix)?;
        f.write_str(&binary_string)
    }
}

// struct FormatBigint<'a>(&'a BigintLiteral);
// impl<'a> FormatBigint<'a> {
//     fn format(&self) -> String {
//         let mut string = format!("{}", self.0.value);
//         add_separators(&mut string, SeparatorDir::Right, DECIMAL_GROUP_LENGTH);
//         string.push('n');
//         string
//     }
// }

const BINARY_GROUP_LENGTH: usize = 4;
const BINARY_MINIMUM_DIGITS: usize = 0;

const DECIMAL_GROUP_LENGTH: usize = 3;
const DECIMAL_MINIMUM_DIGITS: usize = 6;

enum SeparatorDir {
    Left,
    Right,
}

fn add_separators(s: &mut String, dir: SeparatorDir, minimum_digits: usize, group_length: usize) {
    if s.len() < minimum_digits || s.len() < group_length {
        println!("bailing");
        return;
    }

    match dir {
        SeparatorDir::Right => {
            let mut pos = s.len();
            while pos > group_length {
                dbg!(group_length, minimum_digits, pos,);
                pos -= group_length;
                s.insert(pos, '_');
            }
        }
        SeparatorDir::Left => {
            let mut pos = group_length;
            while pos < s.len() {
                dbg!(group_length, minimum_digits, pos);
                s.insert(pos, '_');
                pos += group_length + 1;
            }
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    /// Enforces a convention of grouping digits using numeric separators.
    ///
    /// ### Why is this bad?
    /// Long numbers can become really hard to read, so cutting it into groups of digits, separated with a _, is important to keep your code clear. This rule also enforces a proper usage of the numeric separator, by checking if the groups of digits are of the correct size.
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
    ///   1_234_444,
    ///   1_234.567_89,
    ///   0xAB_CD_EF,
    ///   0b1000_1111,
    ///   0o10_4421,
    ///   1_294_287_712n,
    /// ];
    /// ```
    NumericSeparatorsStyle,
    style
);

impl Rule for NumericSeparatorsStyle {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::NumberLiteral(number) => {
                let formatted = self.format_number(number);
                println!("{} {:?}", formatted, number.raw);
                if formatted != number.raw {
                    ctx.diagnostic_with_fix(NumericSeparatorsStyleDiagnostic(number.span), || {
                        Fix::new(formatted, number.span)
                    });
                }
            }
            AstKind::BigintLiteral(number) => {
                todo!();
                let formatted = self.format_bigint(number);
                if formatted.len() as u32 != number.span.size() {
                    ctx.diagnostic_with_fix(NumericSeparatorsStyleDiagnostic(number.span), || {
                        Fix::new(formatted, number.span)
                    });
                }
            }
            _ => {}
        };
    }
}

#[test]
fn test_binary() {
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

    Tester::new_without_config(NumericSeparatorsStyle::NAME, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

#[test]
fn test_binary_bigint() {
    todo!("Bigint binary");

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

    Tester::new_without_config(NumericSeparatorsStyle::NAME, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

#[test]
fn test_bigint() {
    todo!();
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

    Tester::new_without_config(NumericSeparatorsStyle::NAME, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

#[test]
fn test_number() {
    use crate::tester::Tester;

    let pass = vec![
        // Numbers
        "const foo = 123_456",
        "const foo = 12_345_678",
        "const foo = 123",
        "const foo = 1",
        "const foo = 1234",
        // Decimal numbers
        "const foo = 9807.123",
        "const foo = 3819.123_432",
        "const foo = 138_789.123_432_42",
        "const foo = .000_000_1",
        // Negative numbers
        "const foo = -3000",
        "const foo = -10_000_000",
        // Exponential notation
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
        // Numbers
        "const foo = 1_2_345_678",
        "const foo = 12_3",
        "const foo = 1234567890",
        // Decimal numbers
        "const foo = 9807.1234567",
        "const foo = 3819.123_4325",
        "const foo = 138789.12343_2_42",
        "const foo = .000000_1",
        "const foo = 12345678..toString()",
        "const foo = 12345678 .toString()",
        "const foo = .00000",
        "const foo = 0.00000",
        // Negative numbers
        "const foo = -100000_1",
        // Exponential notation
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
        ("const foo = 1234567890", "const foo = 1_234_567_890", None),
        ("const foo = 9807.1234567", "const foo = 9807.123_456_7", None),
        ("const foo = -100000_1", "const foo = -1_000_001", None),
        ("const foo = -100000_1", "const foo = -1_000_001", None),
        // Exponential notation
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

    Tester::new_without_config(NumericSeparatorsStyle::NAME, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Hexadecimal
        "const foo = 0xAB_CD",
        "const foo = 0xAB",
        "const foo = 0xA",
        "const foo = 0xA_BC_DE_F0",
        "const foo = 0xab_e8_12",
        "const foo = 0xe",
        "const foo = 0Xab_e3_cd",
        // Octal
        "const foo = 0o1234_5670",
        "const foo = 0o7777",
        "const foo = 0o01",
        "const foo = 0o12_7000_0000",
        "const foo = 0O1111_1111",
        // Legacy octal
        "const foo = 0777777",
        "var foo = 0999999",
        "let foo = 0111222",
        // Miscellaneous values
        "const foo = -282_932 - (1938 / 10_000) * .1 + 18.100_000_2",
        "const foo = NaN",
        "const foo = Infinity",
        "const foo = -Infinity",
        "const foo = '1234567n'",
    ];

    let fail = vec![
        // Hexadecimal
        "const foo = 0xA_B_CDE_F0",
        "const foo = 0xABCDEF",
        "const foo = 0xA_B",
        "const foo = 0XAB_C_D",
        // Octal
        "const foo = 0o12_34_5670",
        "const foo = 0o7_7_77",
        "const foo = 0o010101010101",
        "const foo = 0O010101010101",
    ];

    let fix = vec![
        ("const foo = 0xA_B_CDE_F0", "const foo = 0xA_BC_DE_F0", None),
        ("const foo = 0o12_34_5670", "const foo = 0o1234_5670", None),
    ];

    Tester::new_without_config(NumericSeparatorsStyle::NAME, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

use oxc_ast::ast::NumberLiteral;
use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use regex::Regex;
use std::borrow::Cow;
use once_cell::sync::Lazy;

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
    scientific: bool,
    precision: usize,
}

impl PartialEq for ScientificNotation<'_> {
    fn eq(&self, other: &Self) -> bool {
        if self.int == other.int && self.frac == other.frac {
            if self.int == "0" && self.frac == "" {
                return true;
            }
            return self.exp == other.exp;
        }
        false
    }
}

impl<'a> RawNum<'a> {
    fn new(num: &str) -> Option<RawNum<'_>> {
        static RE: Lazy<Regex> = Lazy::new(|| {Regex::new(r"-?0*(?P<int>0|[1-9]\d*)?(?:\.(?P<frac>\d+))?(?:[eE](?P<exp>[+-]?\d+))?").unwrap()});

        if let Some(captures) = RE.captures(num) {
            let int = captures.name("int").map_or("0", |m| m.as_str());
            let frac = captures.name("frac").map_or("", |m| m.as_str());
            let exp = captures.name("exp").map_or("0", |m| m.as_str());

            let exp = match exp.parse::<isize>() {
                Ok(x) => x,
                Err(_) => return None,
            };

            Some(RawNum { int, frac, exp })
        } else {
            None
        }
    }

    fn normalize(&mut self) -> ScientificNotation<'a> {
        let scientific = self.exp != 0;
        let precision = self.frac.len();
        if self.int.starts_with('0') {
            #[allow(clippy::cast_possible_wrap)]
            let exp = self.exp - 1 - self.frac.chars().take_while(|&ch| ch == '0').count() as isize;
            self.frac = self.frac.trim_start_matches('0');

            match self.frac.len() {
                0 => ScientificNotation { int: "0", frac: Cow::Borrowed(""), exp, scientific, precision },
                1 => ScientificNotation { int: &self.frac[..1], frac: Cow::Borrowed(""), exp, scientific, precision },
                _ => ScientificNotation {
                    int: &self.frac[..1],
                    frac: Cow::Borrowed(&self.frac[1..]),
                    exp,
                    scientific, precision
                },
            }
        } else {
            #[allow(clippy::cast_possible_wrap)]
            let exp = self.exp + self.int.len() as isize - 1;
            if self.int.len() == 1 {
                ScientificNotation { int: self.int, frac: Cow::Borrowed(self.frac), exp, scientific, precision }
            } else {
                ScientificNotation {
                    int: &self.int[..1],
                    frac: Cow::Owned(
                        format!("{}{}", &self.int[1..], self.frac).trim_end_matches('0').to_string(),
                    ),
                    exp,
                    scientific, precision
                }
            }
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
        let raw = Self::get_raw(node);
        let raw = if let Some(s) = Self::normalize(&raw) { s } else { return true };

        if raw.frac.len() >= 100 {
            return true;
        }
        let stored = match (raw.scientific, raw.precision) {
            (true, _) => format!("{:.1$e}", node.value, raw.frac.len()),
            (false, _precision @ 0) => format!("{}", node.value),
            (false, precision ) => format!("{:.1$}", node.value, precision),
        };
        let stored = if let Some(s) = Self::normalize(&stored) { s } else { return true };
        raw != stored
    }

    fn normalize(num: &str) -> Option<ScientificNotation<'_>> {
        Some(RawNum::new(num)?.normalize())
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
        ("var a = 480.00", None),
        ("var a = -30.00", None),
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

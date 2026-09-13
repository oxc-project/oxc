use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule, utils::loses_precision};

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
    /// Disallow precision loss in numeric literals.
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
    correctness,
    version = "0.0.7",
    short_description = "Disallow precision loss in numeric literals.",
);

impl Rule for NoLossOfPrecision {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::NumericLiteral(node) if loses_precision(node) => {
                ctx.diagnostic(no_loss_of_precision_diagnostic(node.span));
            }
            _ => {}
        }
    }
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
        "var x = 12_34_56",                          // { "ecmaVersion": 2021 },
        "var x = 12_3.4_56",                         // { "ecmaVersion": 2021 },
        "var x = -12_3.4_56",                        // { "ecmaVersion": 2021 },
        "var x = -12_34_56",                         // { "ecmaVersion": 2021 },
        "var x = 12_3e3_4",                          // { "ecmaVersion": 2021 },
        "var x = 123.0e3_4",                         // { "ecmaVersion": 2021 },
        "var x = 12_3e-3_4",                         // { "ecmaVersion": 2021 },
        "var x = 12_3.0e-3_4",                       // { "ecmaVersion": 2021 },
        "var x = -1_23e-3_4",                        // { "ecmaVersion": 2021 },
        "var x = -1_23.8e-3_4",                      // { "ecmaVersion": 2021 },
        "var x = 1_230000000_00000000_00000_000",    // { "ecmaVersion": 2021 },
        "var x = -1_230000000_00000000_00000_000",   // { "ecmaVersion": 2021 },
        "var x = 0.0_00_000000000_000000000_00123",  // { "ecmaVersion": 2021 },
        "var x = -0.0_00_000000000_000000000_00123", // { "ecmaVersion": 2021 },
        "var x = 0e5_3",                             // { "ecmaVersion": 2021 },
        "var x = 0b11111111111111111111111111111111111111111111111111111", // { "ecmaVersion": 6 },
        "var x = 0b111_111_111_111_1111_11111_111_11111_1111111111_11111111_111_111", // { "ecmaVersion": 2021 },
        "var x = 0B11111111111111111111111111111111111111111111111111111", // { "ecmaVersion": 6 },
        "var x = 0B111_111_111_111_1111_11111_111_11111_1111111111_11111111_111_111", // { "ecmaVersion": 2021 },
        "var x = 0o377777777777777777",       // { "ecmaVersion": 6 },
        "var x = 0o3_77_777_777_777_777_777", // { "ecmaVersion": 2021 },
        "var x = 0O377777777777777777",       // { "ecmaVersion": 6 },
        // "var x = 0377777777777777777", // '0'-prefixed octal literals and octal escape sequences are deprecated
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
        "var x = 0x1FFF_FFFF_FFF_FFF",  // { "ecmaVersion": 2021 },
        "var x = 0X1_FFF_FFFF_FFF_FFF", // { "ecmaVersion": 2021 },
        "const x = 12345;",
        "const x = 123.456;",
        "const x = -123.456;",
        "const x = 123_456;",
        "const x = 123_00_000_000_000_000_000_000_000;",
        "const x = 123.000_000_000_000_000_000_000_0;",
        "var a = Infinity",
        "var a = 480.00",
        "var a = -30.00",
        "let a = Infinity",
        "let a = 480.00",
        "let a = -30.00",
        "const a = Infinity",
        "const a = 480.00",
        "const a = -30.00",
        "const x = 3e-308",
        "(1000000000000000128).toFixed(0)",
    ];

    let fail = vec![
        "var x = 9007199254740993",
        "var x = 9007199254740.993e3",
        "var x = 9.007199254740993e15",
        "var x = -9007199254740993",
        "var x = 900719.9254740994",
        "var x = -900719.9254740994",
        "const x = -9_00719_9254_740993",
        "const x = -900_719.92_5474_0994",
        "const x = 9.0_0719925_474099_3e15",
        "const x = 900_719.92_54740_994",
        "let x = -9_00719_9254_740993",
        "let x = -900_719.92_5474_0994",
        "let x = 9.0_0719925_474099_3e15",
        "let x = 900_719.92_54740_994",
        "var x = -9_00719_9254_740993",  // { "ecmaVersion": 2021 },
        "var x = -900_719.92_5474_0994", // { "ecmaVersion": 2021 },
        "var x = .9007199254740993e16",
        "var x = 9.0_0719925_474099_3e15", // { "ecmaVersion": 2021 },
        "var x = 90_0719925_4740.9_93e3",  // { "ecmaVersion": 2021 },
        "var x = 900_719.92_54740_994",    // { "ecmaVersion": 2021 },
        "var x = 900719925474099_3",       // { "ecmaVersion": 2021 },
        "var x = 900719925474099.30e1",
        "var x = 90071992547409930e-1",
        "var x = 5123000000000000000000000000001",
        "var x = -5123000000000000000000000000001",
        "var x = 1230000000000000000000000.0",
        "var x = 1.0000000000000000000000123",
        "var x = 17498005798264095394980017816940970922825355447145699491406164851279623993595007385788105416184430592",
        "var x = 2e999",
        "var x = .1230000000000000000000000",
        "var x = 0b100000000000000000000000000000000000000000000000000001", // { "ecmaVersion": 6 },
        "var x = 0B100000000000000000000000000000000000000000000000000001", // { "ecmaVersion": 6 },
        "var x = 0o400000000000000001",                                     // { "ecmaVersion": 6 },
        "var x = 0O400000000000000001",                                     // { "ecmaVersion": 6 },
        "var x = 0400000000000000001",
        "var x = 0x20000000000001",
        "var x = 0X20000000000001",
        "var x = 5123_00000000000000000000000000_1", // { "ecmaVersion": 2021 },
        "var x = -5_12300000000000000000000_0000001", // { "ecmaVersion": 2021 },
        "var x = 123_00000000000000000000_00.0_0",   // { "ecmaVersion": 2021 },
        "var x = 1.0_00000000000000000_0000123",     // { "ecmaVersion": 2021 },
        "var x = 174_980057982_640953949800178169_409709228253554471456994_914061648512796239935950073857881054_1618443059_2", // { "ecmaVersion": 2021 },
        "var x = 2e9_99",                         // { "ecmaVersion": 2021 },
        "var x = .1_23000000000000_00000_0000_0", // { "ecmaVersion": 2021 },
        "var x = 0b1_0000000000000000000000000000000000000000000000000000_1", // { "ecmaVersion": 2021 },
        "var x = 0B10000000000_0000000000000000000000000000_000000000000001", // { "ecmaVersion": 2021 },
        "var x = 0o4_00000000000000_001", // { "ecmaVersion": 2021 },
        "var x = 0O4_0000000000000000_1", // { "ecmaVersion": 2021 },
        "var x = 0x2_0000000000001",      // { "ecmaVersion": 2021 },
        "var x = 0X200000_0000000_1",     // { "ecmaVersion": 2021 },
        "const x = 9007199254740993;",
        "const x = 9_007_199_254_740_993;",
        "const x = 9_007_199_254_740.993e3;",
        "const x = 0b100_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_001;",
        "var x = 1e18_446_744_073_709_551_615",
        "var x = 96215808661.52751e-84",
    ];

    Tester::new(NoLossOfPrecision::NAME, NoLossOfPrecision::PLUGIN, pass, fail).test_and_snapshot();
}

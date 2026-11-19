use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::StringToBigInt;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::IsGlobalReference;
use oxc_span::{GetSpan, Span};
use oxc_str::JSStr;
use oxc_syntax::{
    identifier::is_white_space,
    line_terminator::is_line_terminator,
    number::NumberBase,
    operator::{BinaryOperator, UnaryOperator},
};

use crate::{
    AstNode,
    ast_util::could_be_asi_hazard,
    context::LintContext,
    rule::Rule,
    utils::{integer_literal_loses_precision, pad_fix_with_token_boundary},
};

fn prefer_bigint_literals_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer bigint literals over `BigInt(...)`.")
        .with_help("Use a bigint literal (e.g. `123n`) instead of calling `BigInt` with a literal argument.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferBigintLiterals;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Requires using BigInt literals (e.g. `123n`) instead of calling the `BigInt()` constructor
    /// with literal arguments such as numbers or numeric strings
    ///
    /// ### Why is this bad?
    ///
    /// Using `BigInt(…)` with literal values is unnecessarily verbose and less idiomatic than using
    /// a BigInt literal.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// BigInt(0);
    /// BigInt(123);
    /// BigInt(0xFF);
    /// BigInt(1e3);
    /// BigInt("42");
    /// BigInt("0x10");
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// 0n;
    /// 123n;
    /// 0xFFn;
    /// 1000n;
    /// // Non-integer, dynamic, or non-literal input:
    /// BigInt(x);
    /// BigInt("not-a-number");
    /// BigInt("1.23");
    /// ```
    PreferBigintLiterals,
    unicorn,
    style,
    recommended,
    fix,
    version = "1.30.0",
    short_description = "Requires using BigInt literals (e.g. `123n`) instead of calling the `BigInt()` constructor with literal arguments such as numbers or numeric strings.",
);

impl Rule for PreferBigintLiterals {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call) = node.kind() else { return };
        let Some(reference) = call.callee.get_identifier_reference() else {
            return;
        };

        if reference.name != "BigInt"
            || !reference.is_global_reference(ctx.scoping())
            || call.optional
            || call.arguments.len() != 1
        {
            return;
        }

        let arg = &call.arguments[0];

        let Some(argument_expression) = arg.as_expression() else {
            return;
        };

        if argument_expression.is_big_int_literal() {
            ctx.diagnostic_with_fix(prefer_bigint_literals_diagnostic(arg.span()), |fixer| {
                let mut replacement = ctx.source_range(argument_expression.span()).to_string();
                pad_fix_with_token_boundary(ctx.source_text(), call.span, &mut replacement);
                fixer.replace(call.span, replacement)
            });
            return;
        }

        let mut inner_expression = argument_expression.get_inner_expression();
        let mut has_sign = false;
        let mut negated = false;
        while let Expression::UnaryExpression(unary) = inner_expression {
            match unary.operator {
                UnaryOperator::UnaryNegation => negated = !negated,
                UnaryOperator::UnaryPlus => {}
                _ => return,
            }
            has_sign = true;
            inner_expression = unary.argument.get_inner_expression();
        }

        match inner_expression {
            Expression::StringLiteral(string_literal) if !has_sign => {
                report_string_argument(string_literal.value, node, arg.span(), ctx);
            }
            Expression::TemplateLiteral(template) if !has_sign => {
                if let Some(value) = template.single_quasi() {
                    report_string_argument(value, node, arg.span(), ctx);
                }
            }
            Expression::NumericLiteral(numeric_literal) => {
                if numeric_literal.value.fract() != 0.0 {
                    return;
                }

                let raw_text = numeric_literal.raw.as_ref().map_or_else(
                    || {
                        debug_assert!(false, "ASTs from the linter should always have raw values");
                        ctx.source_range(numeric_literal.span)
                    },
                    |raw| raw.as_str(),
                );

                if integer_literal_loses_precision(numeric_literal) {
                    ctx.diagnostic(
                        prefer_bigint_literals_diagnostic(arg.span())
                            .with_note("Integer literal loses precision"),
                    );
                } else if let Some(mut replacement) =
                    bigint_literal_from_numeric(raw_text, numeric_literal.base)
                {
                    ctx.diagnostic_with_fix(
                        prefer_bigint_literals_diagnostic(arg.span()),
                        |fixer| {
                            if negated && numeric_literal.value != 0.0 {
                                replacement.insert(0, '-');
                                guard_unary_negation(node, ctx, &mut replacement);
                            }
                            pad_fix_with_token_boundary(
                                ctx.source_text(),
                                call.span,
                                &mut replacement,
                            );
                            fixer.replace(call.span, replacement)
                        },
                    );
                } else {
                    ctx.diagnostic(prefer_bigint_literals_diagnostic(arg.span()));
                }
            }

            _ => {}
        }
    }
}

/// Add whitespace or parentheses if needed to prevent a negative value from changing how the code parses
fn guard_unary_negation(node: &AstNode, ctx: &LintContext, replacement: &mut String) {
    let span = node.span();
    if unary_negation_needs_parens(node, ctx) {
        *replacement = format!("({replacement})");
    } else if span.start > 0 && ctx.source_text().as_bytes()[span.start as usize - 1] == b'-' {
        // `x-BigInt("-1")` must not become `x--1n`.
        replacement.insert(0, ' ');
    }
    if could_be_asi_hazard(node, ctx) {
        replacement.insert(0, ';');
    }
}

fn unary_negation_needs_parens(node: &AstNode, ctx: &LintContext) -> bool {
    let parent = ctx.nodes().parent_node(node.id());
    let span = node.span();
    match parent.kind() {
        AstKind::BinaryExpression(binary) => {
            binary.operator == BinaryOperator::Exponential && binary.left.span() == span
        }
        AstKind::StaticMemberExpression(_)
        | AstKind::ComputedMemberExpression(_)
        | AstKind::PrivateFieldExpression(_)
        | AstKind::CallExpression(_)
        | AstKind::TaggedTemplateExpression(_)
        | AstKind::TSNonNullExpression(_)
        | AstKind::TSInstantiationExpression(_) => parent.span().start == span.start,
        AstKind::Class(class) => {
            class.heritage.as_ref().is_some_and(|heritage| heritage.expression.span() == span)
        }
        _ => false,
    }
}

fn report_string_argument<'a>(
    value: JSStr<'a>,
    node: &AstNode<'a>,
    arg_span: Span,
    ctx: &LintContext<'a>,
) {
    let Some(mut replacement) = value.as_str().and_then(bigint_literal_from_string) else {
        return;
    };
    ctx.diagnostic_with_fix(prefer_bigint_literals_diagnostic(arg_span), |fixer| {
        if replacement.starts_with('-') {
            guard_unary_negation(node, ctx, &mut replacement);
        }
        pad_fix_with_token_boundary(ctx.source_text(), node.span(), &mut replacement);
        fixer.replace(node.span(), replacement)
    });
}

fn bigint_literal_from_string(value: &str) -> Option<String> {
    let bigint = value.string_to_big_int()?;
    let trimmed = value.trim_matches(|c| is_white_space(c) || is_line_terminator(c));
    let is_non_decimal = trimmed.len() > 2
        && trimmed.starts_with('0')
        && matches!(trimmed.as_bytes()[1], b'b' | b'B' | b'o' | b'O' | b'x' | b'X');
    Some(if is_non_decimal { format!("{trimmed}n") } else { format!("{bigint}n") })
}

fn trim_leading_zeros(raw: &str) -> &str {
    let trimmed = raw.trim_start_matches('0');
    if trimmed.is_empty() { "0" } else { trimmed }
}

fn bigint_literal_from_numeric(raw: &str, base: NumberBase) -> Option<String> {
    if raw.contains('e') || raw.contains('E') {
        return None;
    }

    let literal = match base {
        NumberBase::Binary | NumberBase::Hex => format!("{raw}n"),
        NumberBase::Octal => {
            if raw.starts_with("0o") || raw.starts_with("0O") {
                format!("{raw}n")
            } else {
                // Legacy octal like `0777` is invalid as a BigInt `0777n`, so normalize to `0o`.
                format!("0o{}n", trim_leading_zeros(raw))
            }
        }
        NumberBase::Decimal => format!("{}n", trim_leading_zeros(raw)),
        NumberBase::Float => return None,
    };
    Some(literal)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"1n",
        r"BigInt()",
        r"BigInt(1, 1)",
        r"BigInt(...[1])",
        r"BigInt(true)",
        r"BigInt(null)",
        r"new BigInt(1)",
        r"Not_BigInt(1)",
        r#"BigInt("1.0")"#,
        r#"BigInt("1.1")"#,
        r#"BigInt("1e3")"#,
        r"BigInt(`1${x}`)",
        r#"BigInt("1" + "2")"#,
        r"BigInt?.(1)",
        r"BigInt(1.1)",
        r"typeof BigInt",
        r"((BigInt) => BigInt(1n))(() => 2n)",
        r"function f(BigInt) { return BigInt(1); }",
        r"const BigInt = () => 2n; BigInt(1n);",
        r"import BigInt from 'custom-bigint'; BigInt('1');",
        r"function BigInt() { return 2n; } BigInt(1n);",
        r#"BigInt("not-number")"#,
        r#"BigInt("1_2")"#,
        r#"BigInt("-0x10")"#,
        r#"BigInt("-")"#,
        "BigInt(\"\u{0085}1\")",
        r"BigInt(!1)",
        r"BigInt(-Infinity)",
        r#"BigInt(-"1")"#,
    ];

    let fail = vec![
        r#"BigInt("0")"#,
        r#"BigInt("  0  ")"#,
        r#"BigInt("9007199254740993")"#,
        r#"BigInt("0B11")"#,
        r#"BigInt("0O777")"#,
        r#"BigInt("0XFe")"#,
        r#"BigInt("-9223372036854775808")"#,
        r"BigInt(-1)",
        r"BigInt(+1)",
        r"BigInt(-9007199254740993)",
        r"BigInt(0)",
        r"BigInt(0B11_11)",
        r"BigInt(0O777_777)",
        r"BigInt(0XFe_fE)",
        r"BigInt(0777)",
        r"BigInt(0888)",
        r"BigInt(1.0)",
        r"BigInt(1e2)",
        r"BigInt(1e6)",
        r"BigInt(1E6)",
        r"BigInt(/* comment */1)",
        r"BigInt(9007199254740993)",
        r"BigInt(0x20000000000001)",
        r"BigInt(9_007_199_254_740_993)",
        r"BigInt(0x20_00_00_00_00_00_01)",
        r"BigInt(0777777777777777777)",
        r"BigInt(1000000000000000100)",
        r"BigInt(100000000000000000000000)",
        r"BigInt(100_000_000_000_000_010_000)",
        r"BigInt(0xffffffffffffffff)",
        r"BigInt(0xffffffffffffffffffff)",
        r"BigInt(0b1111111111111111111111111111111111111111111111111111111111111111)",
        r"BigInt(0o1777777777777777777777)",
        r"BigInt(9999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999)",
    ];

    let fix = vec![
        (r"BigInt('42')", "42n"),
        (r"BigInt('  0xFF  ')", "0xFFn"),
        (r"BigInt(0)", "0n"),
        (r"BigInt(1n)", r"1n"),
        (r"BigInt(1n)in {'1':0}", r"1n in {'1':0}"),
        (r"BigInt(1n)instanceof Object", r"1n instanceof Object"),
        (r"BigInt(1n)as bigint", r"1n as bigint"),
        (r"BigInt(1n)satisfies bigint", r"1n satisfies bigint"),
        (r"BigInt(1)in {'1':0}", r"1n in {'1':0}"),
        (r"BigInt('1')instanceof Object", r"1n instanceof Object"),
        (r"BigInt(1n).toString()", r"1n.toString()"),
        (r"BigInt(0B11_11)", "0B11_11n"),
        (r"BigInt(0O777_777)", "0O777_777n"),
        (r"BigInt(0777)", "0o777n"),
        (r"BigInt(0888)", "888n"),
        (r"BigInt(1000000000000000100)", r"BigInt(1000000000000000100)"),
        (r"BigInt(100000000000000000000000)", r"BigInt(100000000000000000000000)"),
        (r"BigInt(100_000_000_000_000_010_000)", r"BigInt(100_000_000_000_000_010_000)"),
        (r"BigInt(1000000000000000000)", "1000000000000000000n"),
        (r"BigInt(9007199254740992)", "9007199254740992n"),
        (r"BigInt(9_007_199_254_740_992)", "9_007_199_254_740_992n"),
        (r"BigInt(0xffffffffffffffff)", r"BigInt(0xffffffffffffffff)"),
        (r"BigInt(0xffffffffffffffffffff)", r"BigInt(0xffffffffffffffffffff)"),
        (
            r"BigInt(0b1111111111111111111111111111111111111111111111111111111111111111)",
            r"BigInt(0b1111111111111111111111111111111111111111111111111111111111111111)",
        ),
        (r"BigInt(0o1777777777777777777777)", r"BigInt(0o1777777777777777777777)"),
        (r"BigInt(0x10000000000000000)", "0x10000000000000000n"),
        (r"BigInt(0X1_0000_0000_0000_0000)", "0X1_0000_0000_0000_0000n"),
        (r"BigInt(0o2000000000000000000000)", "0o2000000000000000000000n"),
        (
            r"BigInt(0b10000000000000000000000000000000000000000000000000000000000000000)",
            "0b10000000000000000000000000000000000000000000000000000000000000000n",
        ),
        (r#"BigInt("0777")"#, "777n"),
        (r#"BigInt("0888")"#, "888n"),
        (r#"BigInt("0b1010")"#, "0b1010n"),
        (r#"BigInt("0B0011")"#, "0B0011n"),
        (r#"BigInt("0O123")"#, "0O123n"),
        (r#"BigInt(" 0001 ")"#, "1n"),
        (r#"BigInt("\u{31}")"#, "1n"),
        ("BigInt(\"1\\\n2\")", "12n"),
        (r#"BigInt("")"#, "0n"),
        (r#"BigInt("   ")"#, "0n"),
        ("BigInt(\"\u{FEFF}1\")", "1n"),
        (r"BigInt(`1`)", "1n"),
        (r"BigInt(`\u{31}`)", "1n"),
        (r"BigInt(`-1`).toString()", "(-1n).toString()"),
        (r#"BigInt("-9223372036854775808")"#, "-9223372036854775808n"),
        (r#"BigInt(" -00001 ")"#, "-1n"),
        (r#"BigInt("+42")"#, "42n"),
        (r#"BigInt("-0")"#, "0n"),
        (r#"const x = BigInt("-1")"#, "const x = -1n"),
        (r#"obj[BigInt("-1")]"#, "obj[-1n]"),
        (r#"2n ** BigInt("-1")"#, "2n ** -1n"),
        (r#"BigInt("-1") ** 2n"#, "(-1n) ** 2n"),
        (r#"BigInt("-1").toString()"#, "(-1n).toString()"),
        (r#"(BigInt("-1")).toString()"#, "(-1n).toString()"),
        (r#"class A extends BigInt("-1") {}"#, "class A extends (-1n) {}"),
        (r#"-BigInt("-1")"#, "- -1n"),
        ("foo\nBigInt(\"-1\").toString()", "foo\n;(-1n).toString()"),
        (r"BigInt(-1)", "-1n"),
        (r"BigInt(+1)", "1n"),
        (r"BigInt(-(+(-10)))", "10n"),
        (r"BigInt(- - -3)", "-3n"),
        (r"BigInt(-+-3)", "3n"),
        (r"BigInt(+-+3)", "-3n"),
        (r"BigInt(-0)", "0n"),
        (r"BigInt(-9007199254740993)", r"BigInt(-9007199254740993)"),
        (r"BigInt(-1).toString()", "(-1n).toString()"),
        (
            r"BigInt('9999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999')",
            "9999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999n",
        ),
        (r"BigInt(1e2)", r"BigInt(1e2)"),
        (r"BigInt(1e6)", r"BigInt(1e6)"),
        (r"BigInt(1E6)", r"BigInt(1E6)"),
    ];

    Tester::new(PreferBigintLiterals::NAME, PreferBigintLiterals::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

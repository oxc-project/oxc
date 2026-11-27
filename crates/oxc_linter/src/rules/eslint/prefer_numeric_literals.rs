use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, MemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode, ast_util::get_symbol_id_of_variable, context::LintContext, rule::Rule,
    utils::pad_fix_with_token_boundary,
};

fn prefer_numeric_literals_diagnostic(span: Span, prefix_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Use {prefix_name} literals instead of parseInt()."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferNumericLiterals;

fn radix_map(base: &str) -> Option<(&'static str, &'static str)> {
    match base {
        "2" => Some(("binary", "0b")),
        "8" => Some(("octal", "0o")),
        "16" => Some(("hexadecimal", "0x")),
        _ => None,
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow `parseInt()` and `Number.parseInt()` in favor of binary, octal, and hexadecimal
    /// literals.
    ///
    /// ### Why is this bad?
    ///
    /// The `parseInt()` and `Number.parseInt()` functions can be used to turn binary, octal, and
    /// hexadecimal strings into integers. As binary, octal, and hexadecimal literals are supported
    /// in ES2015, this rule encourages use of those numeric literals instead of `parseInt()` or
    /// `Number.parseInt()`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// parseInt("111110111", 2) === 503;
    /// parseInt(`111110111`, 2) === 503;
    /// parseInt("767", 8) === 503;
    /// parseInt("1F7", 16) === 503;
    /// Number.parseInt("111110111", 2) === 503;
    /// Number.parseInt("767", 8) === 503;
    /// Number.parseInt("1F7", 16) === 503;
    /// ```
    PreferNumericLiterals,
    eslint,
    style,
    conditional_fix,
    version = "0.7.0",
    short_description = "Disallow `parseInt()` and `Number.parseInt()` in favor of binary, octal, and hexadecimal literals.",
);

impl Rule for PreferNumericLiterals {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let is_parse_int = match call_expr.callee.without_parentheses() {
            Expression::Identifier(ident) => {
                ident.name == "parseInt" && get_symbol_id_of_variable(ident, ctx).is_none()
            }
            Expression::ChainExpression(chain_expr) => chain_expr
                .expression
                .as_member_expression()
                .is_some_and(|member_expr| is_number_parse_int(member_expr, ctx)),
            callee => callee
                .as_member_expression()
                .is_some_and(|member_expr| is_number_parse_int(member_expr, ctx)),
        };
        if is_parse_int {
            check_arguments(call_expr, ctx);
        }
    }
}

fn is_number_parse_int(member_expr: &MemberExpression, ctx: &LintContext) -> bool {
    member_expr.static_property_name().is_some_and(|name| name == "parseInt")
        && matches!(
            member_expr.object().without_parentheses(),
            Expression::Identifier(ident)
                if ident.name == "Number" && get_symbol_id_of_variable(ident, ctx).is_none()
        )
}

fn check_arguments<'a>(call_expr: &CallExpression<'a>, ctx: &LintContext<'a>) {
    if call_expr.arguments.len() != 2 {
        return;
    }

    let Some(string_arg) = call_expr.arguments[0]
        .as_expression()
        .map(Expression::without_parentheses)
        .filter(|arg| is_string_type(arg))
    else {
        return;
    };

    let Some(Expression::NumericLiteral(numeric_lit)) =
        call_expr.arguments[1].as_expression().map(Expression::without_parentheses)
    else {
        return;
    };

    let raw = numeric_lit.raw.as_ref().unwrap().as_str();
    if let Some((name, prefix)) = radix_map(raw) {
        match is_fixable(string_arg, raw) {
            Ok(argument) if !ctx.has_comments_between(call_expr.span) => {
                ctx.diagnostic_with_fix(
                    prefer_numeric_literals_diagnostic(call_expr.span, name),
                    |fixer| {
                        let span = call_expr.span;
                        let mut replacement =
                            String::with_capacity(prefix.len() + argument.len() + 2);
                        replacement.push_str(prefix);
                        replacement.push_str(argument);
                        pad_fix_with_token_boundary(ctx.source_text(), span, &mut replacement);
                        fixer.replace(span, replacement)
                    },
                );
            }
            _ => ctx.diagnostic(prefer_numeric_literals_diagnostic(call_expr.span, name)),
        }
    }
}

fn is_string_type(expr: &Expression) -> bool {
    match expr {
        Expression::StringLiteral(_) => true,
        Expression::TemplateLiteral(t) => t.is_no_substitution_template(),
        _ => false,
    }
}

fn is_fixable<'a>(
    string_arg: &'a Expression,
    radix: &str,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    let Some(string_argument) = get_string_value(string_arg) else {
        return Err("".into());
    };

    if string_argument.is_empty() || string_argument.starts_with(['+', '-']) {
        return Err("".into());
    }

    let radix_num = radix.parse::<u32>()?;
    i64::from_str_radix(string_argument, radix_num)?;

    Ok(string_argument)
}

fn get_string_value<'a>(expr: &'a Expression) -> Option<&'a str> {
    match expr {
        Expression::StringLiteral(lit) => lit.value.as_str(),
        Expression::TemplateLiteral(lit) => {
            lit.quasis.first().map(|quasi| quasi.value.raw.as_str())
        }
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "parseInt(1);",
        "parseInt(1, 3);",
        "Number.parseInt(1);",
        "Number.parseInt(1, 3);",
        "0b111110111 === 503;",
        "0o767 === 503;",
        "0x1F7 === 503;",
        "a[parseInt](1,2);",
        "parseInt(foo);",
        "parseInt(foo, 2);",
        "Number.parseInt(foo);",
        "Number.parseInt(foo, 2);",
        "parseInt(11, 2);",
        "Number.parseInt(1, 8);",
        "parseInt(1e5, 16);",
        "parseInt('11', '2');",
        "Number.parseInt('11', '8');",
        "parseInt(/foo/, 2);",
        "parseInt(`11${foo}`, 2);",
        "parseInt('11', 2n);",        // { "ecmaVersion": 2020 },
        "Number.parseInt('11', 8n);", // { "ecmaVersion": 2020 },
        "parseInt('11', 16n);",       // { "ecmaVersion": 2020 },
        "parseInt(`11`, 16n);",       // { "ecmaVersion": 2020 },
        "parseInt(1n, 2);",           // { "ecmaVersion": 2020 },
        r#"class C { #parseInt; foo() { Number.#parseInt("111110111", 2); } }"#, // { "ecmaVersion": 2022 }
        "parseInt('ff', ...x);",
        "Number.parseInt('ff', ...x);",
        "Number[parseInt]('ff', 16);",
    ];

    let fail = vec![
        r#"parseInt("111110111", 2) === 503;"#,
        r#"parseInt("767", 8) === 503;"#,
        r#"parseInt("1F7", 16) === 255;"#,
        r#"Number.parseInt("111110111", 2) === 503;"#,
        r#"Number.parseInt("767", 8) === 503;"#,
        r#"Number.parseInt("1F7", 16) === 255;"#,
        "parseInt('7999', 8);",
        "parseInt('1234', 2);",
        "parseInt('1234.5', 8);",
        "parseInt('1️⃣3️⃣3️⃣7️⃣', 16);",
        "Number.parseInt('7999', 8);",
        "Number.parseInt('1234', 2);",
        "Number.parseInt('1234.5', 8);",
        "Number.parseInt('1️⃣3️⃣3️⃣7️⃣', 16);",
        "parseInt(`111110111`, 2) === 503;",
        "parseInt(`767`, 8) === 503;",
        "parseInt(`1F7`, 16) === 255;",
        "parseInt('', 8);",
        "parseInt(``, 8);",
        "parseInt(`7999`, 8);",
        "parseInt(`1234`, 2);",
        "parseInt(`1234.5`, 8);",
        "parseInt('11', 2)",
        "Number.parseInt('67', 8)",
        "5+parseInt('A', 16)",
        "function *f(){ yield(Number).parseInt('11', 2) }", // { "ecmaVersion": 6 },
        "function *f(){ yield(Number.parseInt)('67', 8) }", // { "ecmaVersion": 6 },
        "function *f(){ yield(parseInt)('A', 16) }",        // { "ecmaVersion": 6 },
        "function *f(){ yield Number.parseInt('11', 2) }",  // { "ecmaVersion": 6 },
        "function *f(){ yield/**/Number.parseInt('67', 8) }", // { "ecmaVersion": 6 },
        "function *f(){ yield(parseInt('A', 16)) }",        // { "ecmaVersion": 6 },
        "parseInt('11', 2)+5",
        "Number.parseInt('17', 8)+5",
        "parseInt('A', 16)+5",
        "parseInt('11', 2)in foo",
        "Number.parseInt('17', 8)in foo",
        "parseInt('A', 16)in foo",
        "parseInt('11', 2) in foo",
        "Number.parseInt('17', 8)/**/in foo",
        "(parseInt('A', 16))in foo",
        "/* comment */Number.parseInt('11', 2);",
        "Number/**/.parseInt('11', 2);",
        "Number//
			.parseInt('11', 2);",
        "Number./**/parseInt('11', 2);",
        "Number.parseInt(/**/'11', 2);",
        "Number.parseInt('11', /**/2);",
        "Number.parseInt('11', 2)/* comment */;",
        "parseInt/**/('11', 2);",
        "parseInt(//
			'11', 2);",
        "parseInt('11'/**/, 2);",
        "parseInt(`11`/**/, 2);",
        "parseInt('11', 2 /**/);",
        "parseInt('11', 2)//comment
			;",
        r#"parseInt?.("1F7", 16) === 255;"#,
        r#"Number?.parseInt("1F7", 16) === 255;"#,
        r#"Number?.parseInt?.("1F7", 16) === 255;"#,
        r#"(Number?.parseInt)("1F7", 16) === 255;"#,
        r#"(Number?.parseInt)?.("1F7", 16) === 255;"#,
        "Number['parseInt']('ff', 16);",
        "parseInt('ff', (16));",
        "parseInt('1_0', 2);",
        "Number.parseInt('5_000', 8);",
        "parseInt('0_1', 16);",
        "Number.parseInt('0_0', 16);",
        r#"
            parseInt("767", 8) === 503;
            function foo() {
                function parseInt() {
                    throw new Error()
                }
            }
        "#,
    ];

    let fix = vec![
        (r#"parseInt("111110111", 2) === 503;"#, "0b111110111 === 503;", None),
        (r#"parseInt("767", 8) === 503;"#, "0o767 === 503;", None),
        (r#"parseInt("1F7", 16) === 255;"#, "0x1F7 === 255;", None),
        (r#"Number.parseInt("111110111", 2) === 503;"#, "0b111110111 === 503;", None),
        (r#"Number.parseInt("767", 8) === 503;"#, "0o767 === 503;", None),
        (r#"Number.parseInt("1F7", 16) === 255;"#, "0x1F7 === 255;", None),
        ("parseInt(`111110111`, 2) === 503;", "0b111110111 === 503;", None),
        ("parseInt(`767`, 8) === 503;", "0o767 === 503;", None),
        ("parseInt(`1F7`, 16) === 255;", "0x1F7 === 255;", None),
        ("parseInt('11', 2)", "0b11", None),
        ("Number.parseInt('67', 8)", "0o67", None),
        ("5+parseInt('A', 16)", "5+0xA", None),
        ("function *f(){ yield(Number).parseInt('11', 2) }", "function *f(){ yield 0b11 }", None),
        ("function *f(){ yield(Number.parseInt)('67', 8) }", "function *f(){ yield 0o67 }", None),
        ("function *f(){ yield(parseInt)('A', 16) }", "function *f(){ yield 0xA }", None),
        ("function *f(){ yield Number.parseInt('11', 2) }", "function *f(){ yield 0b11 }", None),
        (
            "function *f(){ yield/**/Number.parseInt('67', 8) }",
            "function *f(){ yield/**/0o67 }",
            None,
        ),
        ("function *f(){ yield(parseInt('A', 16)) }", "function *f(){ yield(0xA) }", None),
        ("parseInt('11', 2)+5", "0b11+5", None),
        ("Number.parseInt('17', 8)+5", "0o17+5", None),
        ("parseInt('A', 16)+5", "0xA+5", None),
        ("parseInt('11', 2)in foo", "0b11 in foo", None),
        ("Number.parseInt('17', 8)in foo", "0o17 in foo", None),
        ("parseInt('A', 16)in foo", "0xA in foo", None),
        ("parseInt('11', 2) in foo", "0b11 in foo", None),
        ("Number.parseInt('17', 8)/**/in foo", "0o17/**/in foo", None),
        ("(parseInt('A', 16))in foo", "(0xA)in foo", None),
        ("/* comment */Number.parseInt('11', 2);", "/* comment */0b11;", None),
        ("Number.parseInt('11', 2)/* comment */;", "0b11/* comment */;", None),
        (
            "parseInt('11', 2)//comment
    ;",
            "0b11//comment
    ;",
            None,
        ),
        (r#"parseInt?.("1F7", 16) === 255;"#, "0x1F7 === 255;", None),
        (r#"Number?.parseInt("1F7", 16) === 255;"#, "0x1F7 === 255;", None),
        (r#"Number?.parseInt?.("1F7", 16) === 255;"#, "0x1F7 === 255;", None),
        (r#"(Number?.parseInt)("1F7", 16) === 255;"#, "0x1F7 === 255;", None),
        (r#"(Number?.parseInt)?.("1F7", 16) === 255;"#, "0x1F7 === 255;", None),
        ("Number['parseInt']('ff', 16);", "0xff;", None),
        ("Number[`parseInt`]('ff', 16);", "0xff;", None),
        ("Number?.['parseInt']('ff', 16);", "0xff;", None),
        ("parseInt('ff', (16));", "0xff;", None),
        ("parseInt(('ff'), 16);", "0xff;", None),
        ("parseInt(/* binary mask */ '11', 2);", "parseInt(/* binary mask */ '11', 2);", None),
        ("Number/**/.parseInt('11', 2);", "Number/**/.parseInt('11', 2);", None),
        ("parseInt('11', 2 /**/);", "parseInt('11', 2 /**/);", None),
    ];

    Tester::new(PreferNumericLiterals::NAME, PreferNumericLiterals::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

#[test]
fn signed_arguments_have_no_fix() {
    use crate::tester::Tester;

    let fail = vec![
        r"parseInt('+11', 2);",
        r"parseInt('-11', 2);",
        r"parseInt(`+11`, 2);",
        r"parseInt(`-11`, 2);",
        r"parseInt('+17', 8);",
        r"parseInt('-17', 8);",
        r"parseInt(`+17`, 8);",
        r"parseInt(`-17`, 8);",
        r"parseInt('+ff', 16);",
        r"parseInt('-ff', 16);",
        r"parseInt(`+ff`, 16);",
        r"parseInt(`-ff`, 16);",
        r"Number.parseInt('+11', 2);",
        r"Number.parseInt('-11', 2);",
        r"Number.parseInt(`+11`, 2);",
        r"Number.parseInt(`-11`, 2);",
        r"Number.parseInt('+17', 8);",
        r"Number.parseInt('-17', 8);",
        r"Number.parseInt(`+17`, 8);",
        r"Number.parseInt(`-17`, 8);",
        r"Number.parseInt('+ff', 16);",
        r"Number.parseInt('-ff', 16);",
        r"Number.parseInt(`+ff`, 16);",
        r"Number.parseInt(`-ff`, 16);",
        r#"parseInt("-0", 2);"#,
        r#"parseInt("+0", 2);"#,
        r#"parseInt(" -ff", 16);"#,
        r#"parseInt("+ff ", 16);"#,
        r#"parseInt("\t-ff", 16);"#,
        r#"parseInt("\x2dff", 16);"#,
    ];
    let fix = fail.iter().map(|source| (*source, *source, None)).collect();
    Tester::new(PreferNumericLiterals::NAME, PreferNumericLiterals::PLUGIN, vec![], fail)
        .expect_fix(fix)
        .test();
}

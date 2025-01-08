use oxc_ast::{
    ast::{
        Argument, CallExpression, Expression, IdentifierReference, MemberExpression,
        StaticMemberExpression,
    },
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use phf::{phf_map, phf_ordered_set, Map};

use crate::{
    ast_util::get_symbol_id_of_variable, context::LintContext, fixer::RuleFixer, rule::Rule,
    AstNode,
};

fn prefer_numeric_literals_diagnostic(span: Span, prefix_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Use {prefix_name} literals instead of parseInt()."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferNumericLiterals;

const RADIX_MAP: Map<&'static str, phf::OrderedSet<&'static str>> = phf_map! {
    "2" => phf_ordered_set!{"binary", "0b"},
    "8" => phf_ordered_set!{"octal", "0o"},
    "16" => phf_ordered_set!{"hexadecimal", "0x"},
};

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow parseInt() and Number.parseInt() in favor of binary, octal, and hexadecimal
    /// literals.
    ///
    /// ### Why is this bad?
    ///
    /// The parseInt() and Number.parseInt() functions can be used to turn binary, octal, and
    /// hexadecimal strings into integers. As binary, octal, and hexadecimal literals are supported
    /// in ES6, this rule encourages use of those numeric literals instead of parseInt() or
    /// Number.parseInt().
    ///
    /// ### Example
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
    conditional_fix
);

impl Rule for PreferNumericLiterals {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        match &call_expr.callee.without_parentheses() {
            Expression::Identifier(ident) if ident.name == "parseInt" => {
                if is_parse_int_call(ctx, ident, None) {
                    check_arguments(call_expr, ctx);
                }
            }
            Expression::StaticMemberExpression(member_expr) => {
                if let Expression::Identifier(ident) = &member_expr.object {
                    if is_parse_int_call(ctx, ident, Some(member_expr)) {
                        check_arguments(call_expr, ctx);
                    }
                } else if let Expression::ParenthesizedExpression(paren_expr) = &member_expr.object
                {
                    if let Expression::Identifier(ident) = &paren_expr.expression {
                        if is_parse_int_call(ctx, ident, Some(member_expr)) {
                            check_arguments(call_expr, ctx);
                        }
                    }
                }
            }
            Expression::ChainExpression(chain_expr) => {
                if let Some(MemberExpression::StaticMemberExpression(member_expr)) =
                    chain_expr.expression.as_member_expression()
                {
                    if let Expression::Identifier(ident) = &member_expr.object {
                        if is_parse_int_call(ctx, ident, Some(member_expr)) {
                            check_arguments(call_expr, ctx);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn is_string_type(arg: &Argument) -> bool {
    match arg {
        Argument::StringLiteral(_) => true,
        Argument::TemplateLiteral(t) => t.is_no_substitution_template(),
        _ => false,
    }
}

fn is_parse_int_call(
    ctx: &LintContext,
    ident: &IdentifierReference,
    static_member_expr: Option<&StaticMemberExpression>,
) -> bool {
    if let Some(member_expr) = static_member_expr {
        return ident.name == "Number"
            && member_expr.property.name == "parseInt"
            && get_symbol_id_of_variable(ident, ctx).is_none();
    }

    get_symbol_id_of_variable(ident, ctx).is_none()
}

fn check_arguments<'a>(call_expr: &CallExpression<'a>, ctx: &LintContext<'a>) {
    if call_expr.arguments.len() != 2 {
        return;
    }

    let string_arg = &call_expr.arguments[0];
    if !is_string_type(string_arg) {
        return;
    }

    let radix_arg = &call_expr.arguments[1];
    let Expression::NumericLiteral(numeric_lit) = &radix_arg.to_expression() else {
        return;
    };

    let raw = numeric_lit.raw.as_ref().unwrap().as_str();
    if let Some(name_prefix_set) = RADIX_MAP.get(raw) {
        let name = name_prefix_set.index(0).unwrap();
        let prefix = name_prefix_set.index(1).unwrap();

        match is_fixable(call_expr, raw) {
            Ok(argument) => {
                ctx.diagnostic_with_fix(
                    prefer_numeric_literals_diagnostic(call_expr.span, name),
                    |fixer| {
                        fixer.replace(
                            call_expr.span,
                            generate_fix(fixer, ctx, call_expr, &argument, prefix),
                        )
                    },
                );
            }
            Err(_) => ctx.diagnostic(prefer_numeric_literals_diagnostic(call_expr.span, name)),
        }
    }
}

fn is_fixable(
    call_expr: &CallExpression,
    radix: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let Some(string_argument) = get_string_argument(call_expr) else {
        return Err("".into());
    };

    if string_argument.is_empty() {
        return Err("".into());
    }

    let radix_num = radix.parse::<u32>()?;
    i64::from_str_radix(&string_argument, radix_num)?;

    Ok(string_argument)
}

fn get_string_argument(call_expr: &CallExpression) -> Option<String> {
    if let Argument::StringLiteral(ident) = &call_expr.arguments[0] {
        return Some(ident.value.to_string());
    } else if let Argument::TemplateLiteral(temp) = &call_expr.arguments[0] {
        if temp.quasis.is_empty() {
            return None;
        }
        return Some(temp.quasis[0].value.raw.to_string());
    }

    None
}

fn generate_fix<'a>(
    fixer: RuleFixer<'_, 'a>,
    ctx: &LintContext<'a>,
    call_expr: &CallExpression<'a>,
    argument: &str,
    prefix: &str,
) -> String {
    let mut formatter = fixer.codegen();
    let span = call_expr.span;

    if span.start > 1 {
        let start = ctx.source_text().as_bytes()[span.start as usize - 1];
        if start.is_ascii_alphabetic() || !start.is_ascii() {
            formatter.print_str(" ");
        }
    }

    formatter.print_str(prefix);
    formatter.print_str(argument);

    if (span.end as usize) < ctx.source_text().len() {
        let end = ctx.source_text().as_bytes()[span.end as usize];
        if end.is_ascii_alphabetic() || !end.is_ascii() {
            formatter.print_str(" ");
        }
    }

    formatter.into_source_text()
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
    ];

    Tester::new(PreferNumericLiterals::NAME, PreferNumericLiterals::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

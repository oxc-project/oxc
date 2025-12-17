use oxc_allocator::Allocator;
use oxc_ast::{
    AstBuilder, AstKind,
    ast::{Argument, MemberExpression, RegExpFlags},
};
use oxc_codegen::CodegenOptions;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_regular_expression::ast::Term;
use oxc_span::{CompactStr, GetSpan, SPAN, Span};

use crate::{AstNode, ast_util::extract_regex_flags, context::LintContext, rule::Rule};

fn string_literal(span: Span, replacement: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("This pattern can be replaced with `{replacement}`."))
        .with_label(span)
}

fn use_replace_all(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `String#replaceAll()` over `String#replace()` when using a regex with the global flag.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferStringReplaceAll;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers [`String#replaceAll()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/replaceAll) over [`String#replace()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/replace) when using a regex with the global flag.
    ///
    /// ### Why is this bad?
    ///
    /// The [`String#replaceAll()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/replaceAll) method is both faster and safer as you don't have to use a regex and remember to escape it if the string is not a literal. And when used with a regex, it makes the intent clearer.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// foo.replace(/a/g, bar)
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// foo.replace(/a/, bar)
    /// foo.replaceAll(/a/, bar)
    ///
    /// const pattern = "not-a-regexp"
    /// foo.replace(pattern, bar)
    /// ```
    PreferStringReplaceAll,
    unicorn,
    pedantic,
    fix
);

impl Rule for PreferStringReplaceAll {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };

        let MemberExpression::StaticMemberExpression(static_member_expr) = member_expr else {
            return;
        };

        let method_name_str = static_member_expr.property.name.as_str();

        if !matches!(method_name_str, "replace" | "replaceAll") {
            return;
        }

        if call_expr.arguments.len() != 2 {
            return;
        }

        let pattern = &call_expr.arguments[0];
        match method_name_str {
            "replaceAll" => {
                if let Some(k) = get_pattern_replacement(pattern) {
                    ctx.diagnostic_with_fix(string_literal(pattern.span(), &k), |fixer| {
                        // foo.replaceAll(/hello world/g, bar) => foo.replaceAll('hello world', bar)
                        let mut codegen = fixer.codegen().with_options(CodegenOptions {
                            single_quote: true,
                            ..Default::default()
                        });
                        let alloc = Allocator::default();
                        let ast = AstBuilder::new(&alloc);
                        codegen.print_expression(&ast.expression_string_literal(
                            SPAN,
                            ast.atom(&k),
                            None,
                        ));
                        fixer.replace(pattern.span(), codegen.into_source_text())
                    });
                }
            }
            "replace" if is_reg_exp_with_global_flag(pattern) => {
                ctx.diagnostic_with_fix(
                    use_replace_all(static_member_expr.property.span),
                    |fixer| fixer.replace(static_member_expr.property.span, "replaceAll"),
                );
            }
            _ => {}
        }
    }
}

fn is_reg_exp_with_global_flag<'a>(expr: &'a Argument<'a>) -> bool {
    if let Argument::RegExpLiteral(reg_exp_literal) = expr {
        return reg_exp_literal.regex.flags.contains(RegExpFlags::G);
    }

    if let Argument::NewExpression(new_expr) = expr {
        if !new_expr.callee.is_specific_id("RegExp") {
            return false;
        }

        if let Some(flags) = extract_regex_flags(&new_expr.arguments) {
            return flags.contains(RegExpFlags::G);
        }
    }

    false
}

fn get_pattern_replacement<'a>(expr: &'a Argument<'a>) -> Option<CompactStr> {
    let Argument::RegExpLiteral(reg_exp_literal) = expr else {
        return None;
    };

    if !reg_exp_literal.regex.flags.contains(RegExpFlags::G) {
        return None;
    }

    let pattern_terms = reg_exp_literal
        .regex
        .pattern
        .pattern
        .as_deref()
        .filter(|pattern| pattern.body.body.len() == 1)
        .and_then(|pattern| pattern.body.body.first().map(|it| &it.body))?;

    // Convert the regex pattern to a string by extracting character values
    // from the parsed AST instead of using the raw source text.
    // This ensures escape sequences are properly handled.
    let mut result = String::new();
    for term in pattern_terms {
        let Term::Character(ch) = term else {
            return None;
        };

        match char::from_u32(ch.value) {
            Some(c) => result.push(c),
            // Invalid unicode character, fall back to source text
            None => return Some(CompactStr::new(reg_exp_literal.regex.pattern.text.as_str())),
        }
    }

    Some(CompactStr::new(&result))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"foo.replace(/a/, bar)",
        r"foo.replaceAll(/a/, bar)",
        r"foo.replaceAll(/a|b/g, bar)",
        r#"foo.replace("string", bar)"#,
        r#"foo.replaceAll("string", bar)"#,
        r"foo.replace(/a/g)",
        r"foo.replaceAll(/a/g)",
        r"foo.replace(/\\./g)",
        r"foo.replaceAll(/\\./g)",
        r"new foo.replace(/a/g, bar)",
        r"new foo.replaceAll(/a/g, bar)",
        r"replace(/a/g, bar)",
        r"replaceAll(/a/g, bar)",
        r"foo[replace](/a/g, bar);",
        r"foo[replaceAll](/a/g, bar);",
        r"foo.methodNotReplace(/a/g, bar);",
        r"foo['replace'](/a/g, bar)",
        r"foo['replaceAll'](/a/g, bar)",
        r"foo.replace(/a/g, bar, extra);",
        r"foo.replaceAll(/a/g, bar, extra);",
        r"foo.replace();",
        r"foo.replaceAll();",
        r"foo.replace(...argumentsArray, ...argumentsArray2)",
        r"foo.replaceAll(...argumentsArray, ...argumentsArray2)",
        r"foo.replace(unknown, bar)",
        r#"const pattern = new RegExp("foo", unknown); foo.replace(pattern, bar)"#,
        r#"const pattern = new RegExp("foo"); foo.replace(pattern, bar)"#,
        r"const pattern = new RegExp(); foo.replace(pattern, bar)",
        r#"const pattern = "string"; foo.replace(pattern, bar)"#,
        r#"const pattern = new RegExp("foo", "g"); foo.replace(...[pattern], bar)"#,
        r#"const pattern = "not-a-regexp"; foo.replace(pattern, bar)"#,
        r#"const pattern = new RegExp("foo", "i"); foo.replace(pattern, bar)"#,
        r#"foo.replace(new NotRegExp("foo", "g"), bar)"#,
    ];

    let fail = vec![
        r"foo.replace(/a/g, bar)",
        r#"foo.replace(/"'/g, '\'')"#,
        r"foo.replace(/\./g, bar)",
        r"foo.replace(/\\\./g, bar)",
        r"foo.replace(/\|/g, bar)",
        r"foo.replace(/a/gu, bar)",
        r"foo.replace(/a/ug, bar)",
        r"foo.replace(/[a]/g, bar)",
        r"foo.replace(/a?/g, bar)",
        r"foo.replace(/.*/g, bar)",
        r"foo.replace(/a|b/g, bar)",
        r"foo.replace(/\W/g, bar)",
        r"foo.replace(/\u{61}/g, bar)",
        r"foo.replace(/\u{61}/gu, bar)",
        r"foo.replace(/\u{61}/gv, bar)",
        r#"foo.replace(/]/g, "bar")"#,
        r"foo.replace(/a/gi, bar)",
        r"foo.replace(/a/gui, bar)",
        r"foo.replace(/a/uig, bar)",
        r"foo.replace(/a/vig, bar)",
        // r#"const pattern = new RegExp("foo", "g"); foo.replace(pattern, bar)"#,
        r#"foo.replace(new RegExp("foo", "g"), bar)"#,
        r"foo.replace(/a]/g, _)",
        r"foo.replace(/[a]/g, _)",
        r"foo.replace(/a{1/g, _)",
        r"foo.replace(/a{1}/g, _)",
        r"foo.replace(/\u0022/g, _)",
        r"foo.replace(/\u0027/g, _)",
        r"foo.replace(/\cM\cj/g, _)",
        r"foo.replace(/\x22/g, _)",
        r"foo.replace(/\x27/g, _)",
        r"foo.replace(/\uD83D\ude00/g, _)",
        r"foo.replace(/\u{1f600}/gu, _)",
        r"foo.replace(/\n/g, _)",
        r"foo.replace(/\u{20}/gu, _)",
        r"foo.replace(/\u{20}/gv, _)",
        r"foo.replaceAll(/a]/g, _)",
        // we need a regex parser to handle this
        // r"foo.replaceAll(/\r\n\u{1f600}/gu, _)",
        // r"foo.replaceAll(/\r\n\u{1f600}/gv, _)",
        r"foo.replaceAll(/a very very very very very very very very very very very very very very very very very very very very very very very very very very very very very long string/g, _)",
        r#"foo.replace(/(?!a)+/g, "")"#,
        // https://github.com/oxc-project/oxc/issues/1790
        // report error as `/world/g` can be replaced with string literal
        r#""Hello world".replaceAll(/world/g, 'world!');"#,
    ];

    let fix = vec![
        ("foo.replace(/a/g, bar)", "foo.replaceAll(/a/g, bar)"),
        ("foo.replaceAll(/a/g, bar)", "foo.replaceAll('a', bar)"),
        (r"text.replaceAll(/\\`/g, '`')", r"text.replaceAll('\\`', '`')"),
    ];

    Tester::new(PreferStringReplaceAll::NAME, PreferStringReplaceAll::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

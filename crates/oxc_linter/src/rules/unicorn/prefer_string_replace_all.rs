use oxc_ast::{
    ast::{Argument, Expression, MemberExpression, RegExpFlags},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{ast_util::extract_regex_flags, context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(prefer-string-replace-all): Prefer `String#replaceAll()` over `String#replace()`.")]
#[diagnostic(severity(warning))]
struct PreferStringReplaceAllDiagnostic(#[label] pub Span);

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
    /// ### Example
    /// ```javascript
    /// ```
    PreferStringReplaceAll,
    pedantic
);

impl Rule for PreferStringReplaceAll {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };

        let Some(member_expr) = call_expr.callee.get_member_expr() else { return };

        let MemberExpression::StaticMemberExpression(static_member_expr) = member_expr else {
            return;
        };

        if !matches!(static_member_expr.property.name.as_str(), "replace" | "replaceAll") {
            return;
        }

        if call_expr.arguments.len() != 2 {
            return;
        }

        let Argument::Expression(pattern) = &call_expr.arguments[0] else { return };

        if !is_reg_exp_with_global_flag(pattern) {
            return;
        }

        ctx.diagnostic(PreferStringReplaceAllDiagnostic(static_member_expr.property.span));
    }
}

fn is_reg_exp_with_global_flag<'a>(expr: &'a Expression<'a>) -> bool {
    if let Expression::RegExpLiteral(reg_exp_literal) = expr {
        return reg_exp_literal.regex.flags.contains(RegExpFlags::G);
    }

    if let Expression::NewExpression(new_expr) = expr {
        if !new_expr.callee.is_specific_id("RegExp") {
            return false;
        }

        if let Some(flags) = extract_regex_flags(&new_expr.arguments) {
            return flags.contains(RegExpFlags::G);
        }
    }

    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"foo.replace(/a/, bar)",
        r"foo.replaceAll(/a/, bar)",
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
        r"foo.replaceAll(/\r\n\u{1f600}/gu, _)",
        r"foo.replaceAll(/\r\n\u{1f600}/gv, _)",
        r"foo.replaceAll(/a",
        r#"foo.replace(/(?!a)+/g, "")"#,
    ];

    Tester::new_without_config(PreferStringReplaceAll::NAME, pass, fail).test_and_snapshot();
}

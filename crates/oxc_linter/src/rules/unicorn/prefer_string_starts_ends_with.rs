use oxc_ast::{
    ast::{Expression, MemberExpression, RegExpFlags, RegExpLiteral},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
enum PreferStringStartsEndsWithDiagnostic {
    #[error("eslint-plugin-unicorn(prefer-string-starts-ends-with): Prefer String#startsWith over a regex with a caret.")]
    #[diagnostic(severity(warning))]
    StartsWith(#[label] Span),
    #[error("eslint-plugin-unicorn(prefer-string-starts-ends-with): Prefer String#endsWith over a regex with a dollar sign.")]
    #[diagnostic(severity(warning))]
    EndsWith(#[label] Span),
}

#[derive(Debug, Default, Clone)]
pub struct PreferStringStartsEndsWith;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer [`String#startsWith()`](https://developer.mozilla.org/en/docs/Web/JavaScript/Reference/Global_Objects/String/startsWith) and [`String#endsWith()`](https://developer.mozilla.org/en/docs/Web/JavaScript/Reference/Global_Objects/String/endsWith) over using a regex with `/^foo/` or `/foo$/`.
    ///
    /// ### Why is this bad?
    ///
    /// Using `String#startsWith()` and `String#endsWith()` is more readable and performant as it does not need to parse a regex.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// const foo = "hello";
    /// /^abc/.test(foo);
    ///
    /// // Good
    /// const foo = "hello";
    /// foo.startsWith("abc");
    /// ```
    PreferStringStartsEndsWith,
    correctness
);

impl Rule for PreferStringStartsEndsWith {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if call_expr.optional {
            return;
        }

        let Some(member_expr) = call_expr.callee.get_member_expr() else { return };

        let MemberExpression::StaticMemberExpression(static_member_expr) = &member_expr else {
            return;
        };

        if !matches!(static_member_expr.property.name.as_str(), "test") {
            return;
        }

        let Expression::RegExpLiteral(regex) = &member_expr.object().without_parenthesized() else {
            return;
        };

        let Some(err_kind) = check_regex(regex) else { return };

        match err_kind {
            ErrorKind::StartsWith => {
                ctx.diagnostic(PreferStringStartsEndsWithDiagnostic::StartsWith(call_expr.span));
            }
            ErrorKind::EndsWith => {
                ctx.diagnostic(PreferStringStartsEndsWithDiagnostic::EndsWith(call_expr.span));
            }
        }
    }
}

enum ErrorKind {
    StartsWith,
    EndsWith,
}

fn check_regex(regexp_lit: &RegExpLiteral) -> Option<ErrorKind> {
    if regexp_lit.regex.flags.contains(RegExpFlags::I | RegExpFlags::M) {
        return None;
    }

    if regexp_lit.regex.pattern.starts_with('^')
        && is_simple_string(&regexp_lit.regex.pattern.as_str()[1..regexp_lit.regex.pattern.len()])
    {
        return Some(ErrorKind::StartsWith);
    }

    if regexp_lit.regex.pattern.ends_with('$')
        && is_simple_string(
            &regexp_lit.regex.pattern.as_str()[0..regexp_lit.regex.pattern.len() - 2],
        )
    {
        return Some(ErrorKind::EndsWith);
    }

    None
}

fn is_simple_string(str: &str) -> bool {
    str.chars()
        .all(|c| !matches!(c, '^' | '$' | '+' | '[' | '{' | '(' | '\\' | '.' | '?' | '*' | '|'))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"foo.startsWith("bar")"#,
        r#"foo.endsWith("bar")"#,
        r#"reject(new Error("foo"))"#,
        r#""".test()"#,
        r"test()",
        r"test.test()",
        r#"startWith("bar")"#,
        r"foo()()",
        r"if (foo.match(/^foo/)) {}",
        r"if (/^foo/.exec(foo)) {}",
    ];

    let fail = vec![
        r"const foo = {}; /^abc/.test(foo);",
        r"const foo = 123; /^abc/.test(foo);",
        r#"const foo = "hello"; /^abc/.test(foo);"#,
        r"/^b/.test((a))",
        r"(/^b/).test((a))",
        r"const fn = async () => /^b/.test(await foo)",
        r"const fn = async () => (/^b/).test(await foo)",
        r#"/^a/.test("string")"#,
        r#"/^a/.test((0, "string"))"#,
        r"async function a() {return /^a/.test(await foo())}",
        r"/^a/.test(foo + bar)",
        r"/^a/.test(foo || bar)",
        r"/^a/.test(new SomeString)",
        r"/^a/.test(new (SomeString))",
        r"/^a/.test(new SomeString())",
        r"/^a/.test(new new SomeClassReturnsAStringSubClass())",
        r"/^a/.test(new SomeString(/* comment */))",
        r#"/^a/.test(new SomeString("string"))"#,
        r"/^a/.test(foo.bar)",
        r"/^a/.test(foo.bar())",
        r"/^a/.test(foo?.bar)",
        r"/^a/.test(foo?.bar())",
        r"/^a/.test(`string`)",
        r"/^a/.test(tagged`string`)",
        r#"(/^a/).test((0, "string"))"#,
        r"/^a/.test(true ? a : b)",
        r"/a$/.test(a ??= b)",
        r"/^a/.test(a || b)",
        r"/^a/.test(a && b)",
        r#"/^a/u.test("string")"#,
        r#"/^a/v.test("string")"#,
        r"/a$/.test(`${unknown}`)",
        r"/a$/.test(String(unknown))",
    ];

    Tester::new_without_config(PreferStringStartsEndsWith::NAME, pass, fail).test_and_snapshot();
}

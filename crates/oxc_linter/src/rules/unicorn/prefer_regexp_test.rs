use oxc_ast::{
    ast::{Expression, MemberExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{ast_util::outermost_paren_parent, context::LintContext, rule::Rule, AstNode};

fn prefer_regexp_test_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer RegExp#test() over String#match() and RegExp#exec()")
        .with_help("RegExp#test() exclusively returns a boolean and therefore is more efficient")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferRegexpTest;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers `RegExp#test()` over `String#match()` and `String#exec()`.
    ///
    /// ### Why is this bad?
    ///
    /// When you want to know whether a pattern is found in a string, use [`RegExp#test()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/test) instead of [`String#match()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/match) and [`RegExp#exec()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/exec), as it exclusively returns a boolean and therefore is more efficient.
    ///
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// if (string.match(/unicorn/)) { }
    /// if (/unicorn/.exec(string)) {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// if (/unicorn/.test(string)) {}
    /// Boolean(string.match(/unicorn/))
    ///
    /// ```
    PreferRegexpTest,
    unicorn,
    pedantic,
    fix
);

impl Rule for PreferRegexpTest {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };

        if call_expr.optional || call_expr.arguments.len() != 1 {
            return;
        }

        if call_expr.arguments[0].is_spread() {
            return;
        }

        let (span, name) = match member_expr {
            MemberExpression::StaticMemberExpression(v) => {
                if !matches!(v.property.name.as_str(), "match" | "exec") {
                    return;
                }
                (v.property.span, &v.property.name)
            }
            _ => return,
        };

        let Some(parent) = outermost_paren_parent(node, ctx) else {
            return;
        };

        match parent.kind() {
            AstKind::ForStatement(for_stmt) => {
                let Some(test) = &for_stmt.test else { return };

                let Expression::CallExpression(call_expr2) = test else {
                    return;
                };

                // Check if the `test` of the for statement is the same node as the call expression.
                if !std::ptr::eq(call_expr2.as_ref(), call_expr) {
                    return;
                }
            }
            AstKind::ConditionalExpression(conditional_expr) => {
                let Expression::CallExpression(call_expr2) = &conditional_expr.test else {
                    return;
                };

                // Check if the `test` of the conditional expression is the same node as the call expression.
                if !std::ptr::eq(call_expr2.as_ref(), call_expr) {
                    return;
                }
            }

            AstKind::Argument(_) => {
                let Some(parent) = outermost_paren_parent(parent, ctx) else {
                    return;
                };

                let AstKind::CallExpression(call_expr) = parent.kind() else {
                    return;
                };

                let Expression::Identifier(ident) = &call_expr.callee else {
                    return;
                };

                if ident.name.as_str() != "Boolean" {
                    return;
                }
            }
            AstKind::WhileStatement(_)
            | AstKind::DoWhileStatement(_)
            | AstKind::IfStatement(_)
            | AstKind::UnaryExpression(_) => {}
            _ => return,
        }

        match name.as_str() {
            "match" => {
                if member_expr.object().is_literal()
                    && !matches!(member_expr.object(), Expression::RegExpLiteral(_))
                {
                    return;
                }

                if let Some(expr) = call_expr.arguments[0].as_expression() {
                    if expr.is_literal() && !matches!(expr, Expression::RegExpLiteral(_)) {
                        return;
                    }
                }
            }
            "exec" => {
                if member_expr.object().is_literal()
                    && !matches!(member_expr.object(), Expression::RegExpLiteral(_))
                {
                    return;
                }
            }
            _ => unreachable!("expected match or test, got: {:?}", name),
        }

        ctx.diagnostic_with_fix(prefer_regexp_test_diagnostic(span), |fixer| {
            if name.as_str() == "exec" {
                return fixer.replace(span, "test");
            }
            let mut fix = fixer.new_fix_with_capacity(3);

            fix.push(fixer.replace(span, "test"));

            fix.push(fixer.replace(
                call_expr.arguments[0].span(),
                fixer.source_range(member_expr.object().span()),
            ));

            fix.push(fixer.replace(
                member_expr.object().span(),
                fixer.source_range(call_expr.arguments[0].span()),
            ));

            fix
        });
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"const bar = !re.test(foo)",
        r"const matches = foo.match(re) || []",
        r"const matches = foo.match(re)",
        r"const matches = re.exec(foo)",
        r"while (foo = re.exec(bar)) {}",
        r"while ((foo = re.exec(bar))) {}",
        r"if (foo.notMatch(re)) {}",
        r"if (re.notExec(foo)) {}",
        r"if (foo.match) {}",
        r"if (re.exec) {}",
        r"if (foo[match](re)) {}",
        r"if (re[exec](foo)) {}",
        r#"if (foo["match"](re)) {}"#,
        r#"if (re["exec"](foo)) {}"#,
        r"if (match(re)) {}",
        r"if (exec(foo)) {}",
        r"if (foo.match()) {}",
        r"if (re.exec()) {}",
        r"if (foo.match(re, another)) {}",
        r"if (re.exec(foo, another)) {}",
        r"if (foo.match(...[regexp])) {}",
        r"if (re.exec(...[string])) {}",
        r"if (foo.match(1)) {}",
        r#"if (foo.match("1")) {}"#,
        r"if (foo.match(null)) {}",
        r"if (foo.match(1n)) {}",
        r"if (foo.match(true)) {}",
    ];

    let fail = vec![
        r"const re = /a/; const bar = !foo.match(re)",
        r"const re = /a/; const bar = Boolean(foo.match(re))",
        r"const re = /a/; if (foo.match(re)) {}",
        r"const re = /a/; const bar = foo.match(re) ? 1 : 2",
        r"const re = /a/; while (foo.match(re)) foo = foo.slice(1);",
        r"const re = /a/; do {foo = foo.slice(1)} while (foo.match(re));",
        r"const re = /a/; for (; foo.match(re); ) foo = foo.slice(1);",
        r"const re = /a/; const bar = !re.exec(foo)",
        r"const re = /a/; const bar = Boolean(re.exec(foo))",
        r"const re = /a/; if (re.exec(foo)) {}",
        r"const re = /a/; const bar = re.exec(foo) ? 1 : 2",
        r"const re = /a/; while (re.exec(foo)) foo = foo.slice(1);",
        r"const re = /a/; do {foo = foo.slice(1)} while (re.exec(foo));",
        r"const re = /a/; for (; re.exec(foo); ) foo = foo.slice(1);",
        r"const re = /a/; if ((0, foo).match(re)) {}",
        r"const re = /a/; if ((0, foo).match((re))) {}",
        r"const re = /a/; if ((foo).match(re)) {}",
        r"const re = /a/; if ((foo).match((re))) {}",
        r"if (foo.match(/re/)) {}",
        r"const re = /a/; if (foo.match(re)) {}",
        r"const bar = {bar: /a/}; if (foo.match(bar.baz)) {}",
        r"if (foo.match(bar.baz())) {}",
        r#"if (foo.match(new RegExp("re", "g"))) {}"#,
        r"if (foo.match(new SomeRegExp())) {}",
        r"if (foo.match(new SomeRegExp)) {}",
        r"if (foo.match(bar?.baz)) {}",
        r"if (foo.match(bar?.baz())) {}",
        r"if (foo.match(bar || baz)) {}",
        r"if ((foo).match(/re/)) {}",
        r"if ((foo).match(new SomeRegExp)) {}",
        r"if ((foo).match(bar?.baz)) {}",
        r"if ((foo).match(bar?.baz())) {}",
        r"const bar = false; const baz = /a/; if ((foo).match(bar || baz)) {}",
        r"const re = [/a/]; if (foo.match([re][0])) {}",
        r"if (foo.match(unknown)) {}",
        r"if (foo.match(/a/g));",
        r"if (foo.match(/a/y));",
        r"if (foo.match(/a/gy));",
        r"if (foo.match(/a/ig));",
        r#"if (foo.match(new RegExp("a", "g")));"#,
        r"if (/a/g.exec(foo));",
        r"if (/a/y.exec(foo));",
        r"if (/a/gy.exec(foo));",
        r"if (/a/yi.exec(foo));",
        r#"if (new RegExp("a", "g").exec(foo));"#,
        r#"if (new RegExp("a", "y").exec(foo));"#,
        r"!/a/u.exec(foo)",
        r"!/a/v.exec(foo)",
    ];

    let fix = vec![
        ("const re = /a/; const bar = !foo.match(re)", "const re = /a/; const bar = !re.test(foo)"),
        (
            "const re = /a/; const bar = Boolean(foo.match(re))",
            "const re = /a/; const bar = Boolean(re.test(foo))",
        ),
        ("const re = /a/; if (foo.match(re)) {}", "const re = /a/; if (re.test(foo)) {}"),
        (
            "const re = /a/; const bar = foo.match(re) ? 1 : 2",
            "const re = /a/; const bar = re.test(foo) ? 1 : 2",
        ),
        (
            "const re = /a/; while (foo.match(re)) foo = foo.slice(1);",
            "const re = /a/; while (re.test(foo)) foo = foo.slice(1);",
        ),
        (
            "const re = /a/; do {foo = foo.slice(1)} while (foo.match(re));",
            "const re = /a/; do {foo = foo.slice(1)} while (re.test(foo));",
        ),
        (
            "const re = /a/; for (; foo.match(re); ) foo = foo.slice(1);",
            "const re = /a/; for (; re.test(foo); ) foo = foo.slice(1);",
        ),
        ("const re = /a/; const bar = !re.exec(foo)", "const re = /a/; const bar = !re.test(foo)"),
        (
            "const re = /a/; const bar = Boolean(re.exec(foo))",
            "const re = /a/; const bar = Boolean(re.test(foo))",
        ),
        ("const re = /a/; if (re.exec(foo)) {}", "const re = /a/; if (re.test(foo)) {}"),
        ("const re = /a/; if (someStr.match(re)) {}", "const re = /a/; if (re.test(someStr)) {}"),
    ];

    Tester::new(PreferRegexpTest::NAME, PreferRegexpTest::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

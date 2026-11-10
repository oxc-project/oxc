use oxc_ast::{
    AstKind,
    ast::{Expression, MemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::UnaryOperator;

use crate::{
    AstNode,
    ast_util::{get_symbol_id_of_variable, outermost_paren_parent},
    context::LintContext,
    rule::Rule,
    utils::{is_logical_expression, is_regexp_callee},
};

fn prefer_regexp_test_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `RegExp#test()` over `String#match()` and `RegExp#exec()`.")
        .with_help("`RegExp#test()` exclusively returns a boolean and therefore is more efficient.")
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
    /// When you want to know whether a pattern is found in a string, use
    /// [`RegExp#test()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/test)
    /// instead of [`String#match()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/match)
    /// or [`RegExp#exec()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/exec),
    /// as it exclusively returns a boolean and therefore is more efficient.
    ///
    /// ### Examples
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
    /// ```
    PreferRegexpTest,
    unicorn,
    pedantic,
    recommended,
    fix_suggestion,
    version = "0.0.16",
    short_description = "Prefers `RegExp#test()` over `String#match()` and `String#exec()`.",
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

        let (span, is_match) = match member_expr {
            MemberExpression::StaticMemberExpression(v) => {
                let is_match = match v.property.name.as_str() {
                    "match" => true,
                    "exec" => false,
                    _ => return,
                };
                (v.property.span, is_match)
            }
            _ => return,
        };

        let mut boolean_node = node;
        let parent = loop {
            let Some(parent) = outermost_paren_parent(boolean_node, ctx) else {
                return;
            };
            if !is_logical_expression(parent) {
                break parent;
            }
            boolean_node = parent;
        };

        match parent.kind() {
            AstKind::ForStatement(for_stmt) => {
                let Some(test) = &for_stmt.test else { return };

                if test.get_inner_expression().span() != boolean_node.kind().span() {
                    return;
                }
            }
            AstKind::ConditionalExpression(conditional_expr) => {
                if conditional_expr.test.get_inner_expression().span() != boolean_node.kind().span()
                {
                    return;
                }
            }
            AstKind::CallExpression(call_expr) => {
                if call_expr.optional || call_expr.arguments.len() != 1 {
                    return;
                }

                let Expression::Identifier(ident) = &call_expr.callee else {
                    return;
                };

                if ident.name.as_str() != "Boolean" || !ctx.is_reference_to_global_variable(ident) {
                    return;
                }
            }
            AstKind::UnaryExpression(unary_expr)
                if unary_expr.operator == UnaryOperator::LogicalNot => {}
            AstKind::WhileStatement(_) | AstKind::DoWhileStatement(_) | AstKind::IfStatement(_) => {
            }
            _ => return,
        }

        if member_expr.object().is_literal()
            && !matches!(member_expr.object(), Expression::RegExpLiteral(_))
        {
            return;
        }

        if is_match
            && let Some(expr) = call_expr.arguments[0].as_expression()
            && expr.is_literal()
            && !matches!(expr, Expression::RegExpLiteral(_))
        {
            return;
        }

        let fix_kind = if !is_match || is_known_regexp(&call_expr.arguments[0], ctx) {
            FixKind::Fix
        } else {
            FixKind::Suggestion
        };
        ctx.diagnostic_with_fix_of_kind(prefer_regexp_test_diagnostic(span), fix_kind, |fixer| {
            if !is_match {
                return fixer.replace(span, "test");
            }
            let mut fix = fixer.new_fix_with_capacity(3);

            fix.push(fixer.replace(span, "test"));
            fix.push(fixer.replace_with(&call_expr.arguments[0], member_expr.object()));
            fix.push(fixer.replace_with(member_expr.object(), &call_expr.arguments[0]));

            fix.with_message("Replace with `RegExp.test()`")
        });
    }
}

fn is_known_regexp<'a>(argument: &oxc_ast::ast::Argument<'a>, ctx: &LintContext<'a>) -> bool {
    let Some(mut expr) = argument.as_expression().map(Expression::get_inner_expression) else {
        return false;
    };
    if let Expression::Identifier(ident) = expr {
        let Some(symbol) = get_symbol_id_of_variable(ident, ctx) else { return false };
        if ctx.scoping().symbol_is_mutated(symbol)
            || !ctx.scoping().symbol_redeclarations(symbol).is_empty()
        {
            return false;
        }
        let AstKind::VariableDeclarator(declaration) =
            ctx.nodes().kind(ctx.scoping().symbol_declaration(symbol))
        else {
            return false;
        };
        if !declaration.id.is_binding_identifier()
            || matches!(ctx.nodes().parent_kind(ctx.scoping().symbol_declaration(symbol)), AstKind::VariableDeclaration(declaration) if declaration.kind.is_var())
        {
            return false;
        }
        let Some(init) = &declaration.init else { return false };
        expr = init.get_inner_expression();
    }
    match expr {
        Expression::RegExpLiteral(_) => true,
        Expression::NewExpression(constructor) => is_regexp_callee(&constructor.callee, ctx),
        Expression::CallExpression(call) => !call.optional && is_regexp_callee(&call.callee, ctx),
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "const bar = !re.test(foo)",
        "const matches = foo.match(re) || []",
        "const matches = foo.match(re)",
        "const matches = re.exec(foo)",
        "while (foo = re.exec(bar)) {}",
        "while ((foo = re.exec(bar))) {}",
        "if (foo.notMatch(re)) {}",
        "if (re.notExec(foo)) {}",
        "if (foo.match) {}",
        "if (re.exec) {}",
        "if (foo[match](re)) {}",
        "if (re[exec](foo)) {}",
        r#"if (foo["match"](re)) {}"#,
        r#"if (re["exec"](foo)) {}"#,
        "if (match(re)) {}",
        "if (exec(foo)) {}",
        "if (foo.match()) {}",
        "if (re.exec()) {}",
        "if (foo.match(re, another)) {}",
        "if (re.exec(foo, another)) {}",
        "if (foo.match(...[regexp])) {}",
        "if (re.exec(...[string])) {}",
        "if (foo.match(1)) {}",
        r#"if (foo.match("1")) {}"#,
        "if (foo.match(null)) {}",
        "if (foo.match(1n)) {}",
        "if (foo.match(true)) {}",
        "const value = a && s.match(/x/);",
        "const value = s.match(/x/) || a;",
        "if (s.match(/x/) ?? a) {}",
        "if (a ?? s.match(/x/)) {}",
        "for (a && s.match(/x/); ; ) {}",
        "for (; ; a || /x/.exec(s)) {}",
        "const value = a ? s.match(/x/) || b : b && /x/.exec(s);",
        "if (consume(a && s.match(/x/))) {}",
        "const value = typeof (a && s.match(/x/));",
        "const value = +(a || /x/.exec(s));",
        "Boolean(a && s.match(/x/), b);",
        "Boolean(...(a && s.match(/x/)));",
        "Boolean?.(a && s.match(/x/));",
        "function f(Boolean) { return Boolean(a && s.match(/x/)); }",
        "if ((a && s.match(/x/)) ?? b) {}",
        "if (a ?? (b || /x/.exec(s))) {}",
        "if (a && (b ?? s.match(/x/))) {}",
    ];

    let fail = vec![
        "const re = /a/; const bar = !foo.match(re)",
        "const re = /a/; const bar = Boolean(foo.match(re))",
        "const re = /a/; if (foo.match(re)) {}",
        "const re = /a/; const bar = foo.match(re) ? 1 : 2",
        "const re = /a/; while (foo.match(re)) foo = foo.slice(1);",
        "const re = /a/; do {foo = foo.slice(1)} while (foo.match(re));",
        "const re = /a/; for (; foo.match(re); ) foo = foo.slice(1);",
        "const re = /a/; const bar = !re.exec(foo)",
        "const re = /a/; const bar = Boolean(re.exec(foo))",
        "const re = /a/; if (re.exec(foo)) {}",
        "const re = /a/; const bar = re.exec(foo) ? 1 : 2",
        "const re = /a/; while (re.exec(foo)) foo = foo.slice(1);",
        "const re = /a/; do {foo = foo.slice(1)} while (re.exec(foo));",
        "const re = /a/; for (; re.exec(foo); ) foo = foo.slice(1);",
        "const re = /a/; if ((0, foo).match(re)) {}",
        "const re = /a/; if ((0, foo).match((re))) {}",
        "const re = /a/; if ((foo).match(re)) {}",
        "const re = /a/; if ((foo).match((re))) {}",
        "if (foo.match(/re/)) {}",
        "const re = /a/; if (foo.match(re)) {}",
        "const bar = {bar: /a/}; if (foo.match(bar.baz)) {}",
        "if (foo.match(bar.baz())) {}",
        r#"if (foo.match(new RegExp("re", "g"))) {}"#,
        "if (foo.match(new SomeRegExp())) {}",
        "if (foo.match(new SomeRegExp)) {}",
        "if (foo.match(bar?.baz)) {}",
        "if (foo.match(bar?.baz())) {}",
        "if (foo.match(bar || baz)) {}",
        "async function a() {
                if (foo.match(await bar())) {}
            }",
        "if ((foo).match(/re/)) {}",
        "if ((foo).match(new SomeRegExp)) {}",
        "if ((foo).match(bar?.baz)) {}",
        "if ((foo).match(bar?.baz())) {}",
        "const bar = false; const baz = /a/; if ((foo).match(bar || baz)) {}",
        "async function a() {
                if ((foo).match(await bar())) {}
            }",
        "const re = [/a/]; if (foo.match([re][0])) {}",
        "async function a() {
                if (
                    /* 1 */ foo() /* 2 */
                        ./* 3 */ match /* 4 */ (
                            /* 5 */ await /* 6 */ bar() /* 7 */
                            ,
                            /* 8 */
                        )
                ) {}
            }",
        r"const string = '[.!?]\\\\s*$';
            if (foo.match(string)) {
            }",
        r"const regex = new RegExp('[.!?]\\\\s*$');
            if (foo.match(regex)) {}",
        "if (foo.match(unknown)) {}",
        "if (foo.match(/a/g));",
        "if (foo.match(/a/y));",
        "if (foo.match(/a/gy));",
        "if (foo.match(/a/ig));",
        r#"if (foo.match(new RegExp("a", "g")));"#,
        "if (/a/g.exec(foo));",
        "if (/a/y.exec(foo));",
        "if (/a/gy.exec(foo));",
        "if (/a/yi.exec(foo));",
        r#"if (new RegExp("a", "g").exec(foo));"#,
        r#"if (new RegExp("a", "y").exec(foo));"#,
        "const regex = /weird/g;
            if (foo.match(regex));",
        "const regex = /weird/g;
            if (regex.exec(foo));",
        "const regex = /weird/y;
            if (regex.exec(foo));",
        "const regex = /weird/gyi;
            if (regex.exec(foo));",
        "let re = new RegExp('foo', 'g');
            if(str.match(re));",
        "!/a/u.exec(foo)",
        "!/a/v.exec(foo)",
        "export function f(a, s) {\n  if (a && s.match(/x/)) {}\n  if (s.match(/x/) || a) {}\n  if (s.match(/x/)) {}\n}",
        "if (s.match(/x/) && a) {}",
        "if (a || s.match(/x/)) {}",
        "if ((a && (b || (s.match(/x/))))) {}",
        "while (a && /x/.exec(s)) {}",
        "do {} while (/x/.exec(s) || a);",
        "for (; (a && s.match(/x/)); ) {}",
        "const value = (s.match(/x/) || a) ? 1 : 2;",
        "const value = Boolean(a && s.match(/x/));",
        "const value = !(a || /x/.exec(s));",
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
        (
            "export function f(a, s) {\n  if (a && s.match(/x/)) {}\n  if (s.match(/x/) || a) {}\n  if (s.match(/x/)) {}\n}",
            "export function f(a, s) {\n  if (a && /x/.test(s)) {}\n  if (/x/.test(s) || a) {}\n  if (/x/.test(s)) {}\n}",
        ),
        ("if (s.match(/x/) && a) {}", "if (/x/.test(s) && a) {}"),
        ("if (a || s.match(/x/)) {}", "if (a || /x/.test(s)) {}"),
        ("if ((a && (b || (s.match(/x/))))) {}", "if ((a && (b || (/x/.test(s))))) {}"),
        ("while (a && /x/.exec(s)) {}", "while (a && /x/.test(s)) {}"),
        ("do {} while (/x/.exec(s) || a);", "do {} while (/x/.test(s) || a);"),
        ("for (; (a && s.match(/x/)); ) {}", "for (; (a && /x/.test(s)); ) {}"),
        ("const value = (s.match(/x/) || a) ? 1 : 2;", "const value = (/x/.test(s) || a) ? 1 : 2;"),
        ("const value = Boolean(a && s.match(/x/));", "const value = Boolean(a && /x/.test(s));"),
        ("const value = !(a || /x/.exec(s));", "const value = !(a || /x/.test(s));"),
    ];

    Tester::new(PreferRegexpTest::NAME, PreferRegexpTest::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

#[test]
fn test_fix_requires_regexp() {
    use crate::{fixer::FixKind, tester::Tester};

    let unchanged = [
        r#"const query = "a"; if (title.match(query)) {}"#,
        "if (glob.match(file)) {}",
        r#"let pattern = /a/; pattern = "a"; if (title.match(pattern)) {}"#,
        "function f(RegExp) { if (title.match(new RegExp('a'))) {} }",
        "if (title.match(getPattern())) {}",
        "if (title.match(RegExp?.('a'))) {}",
        "const title = 'a'; if (title.match(re)) {} var re = /a/;",
        "if (false) { var re = /a/; } if (title.match(re)) {}",
        "var re = /a/; var re = 'a'; if (title.match(re)) {}",
    ];
    let mut fixes: Vec<_> =
        unchanged.into_iter().map(|source| (source, source, None, FixKind::Fix)).collect();
    fixes.extend([
        ("if (glob.match(file)) {}", "if (file.test(glob)) {}", None, FixKind::Suggestion),
        ("if (title.match((/a/))) {}", "if ((/a/).test(title)) {}", None, FixKind::Fix),
        ("if (title.match(RegExp('a'))) {}", "if (RegExp('a').test(title)) {}", None, FixKind::Fix),
        (
            "const re = new RegExp('a'); if (title.match(re)) {}",
            "const re = new RegExp('a'); if (re.test(title)) {}",
            None,
            FixKind::Fix,
        ),
        (
            "const re = /a/; if (title.match((re))) {}",
            "const re = /a/; if ((re).test(title)) {}",
            None,
            FixKind::Fix,
        ),
        ("if (/a/.exec(title)) {}", "if (/a/.test(title)) {}", None, FixKind::Fix),
    ]);
    Tester::new(
        PreferRegexpTest::NAME,
        PreferRegexpTest::PLUGIN,
        Vec::<&str>::new(),
        unchanged.to_vec(),
    )
    .expect_fix(fixes)
    .test();
}

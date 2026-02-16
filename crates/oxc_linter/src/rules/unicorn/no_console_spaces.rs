use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    ast_util::{call_expr_method_callee_info, is_method_call},
    context::LintContext,
    rule::Rule,
};

fn no_console_spaces_diagnostic(
    leading_or_trailing: &str,
    method_name: &str,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Do not use {leading_or_trailing} spaces with `console.{method_name}` parameters"))
        .with_help("The `console.log()` method and similar methods join the parameters with a space so adding a leading/trailing space to a parameter, results in two spaces being added.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoConsoleSpaces;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows leading/trailing space inside `console.log()` and similar methods.
    ///
    /// ### Why is this bad?
    ///
    /// The `console.log()` method and similar methods join the parameters with a space so adding a leading/trailing space to a parameter, results in two spaces being added.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// console.log("abc ", "def");
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// console.log("abc", "def");
    /// ```
    NoConsoleSpaces,
    unicorn,
    style,
    fix
);

impl Rule for NoConsoleSpaces {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_method_call(
            call_expr,
            Some(&["console"]),
            Some(&["log", "debug", "info", "warn", "error"]),
            None,
            None,
        ) {
            return;
        }

        let call_expr_arg_len = call_expr.arguments.len();

        for (i, arg) in call_expr.arguments.iter().enumerate() {
            if let Some(expression_arg) = arg.as_expression() {
                let (literal_raw, is_template_lit) = match expression_arg {
                    Expression::StringLiteral(string_lit) => {
                        let literal_raw = string_lit.value.as_str();

                        (literal_raw, false)
                    }
                    Expression::TemplateLiteral(string_lit) => {
                        let literal_raw = string_lit
                            .span
                            .source_text(ctx.source_text().as_ref())
                            .trim_start_matches('`')
                            .trim_end_matches('`');

                        (literal_raw, true)
                    }

                    _ => continue,
                };

                if check_literal_leading(i, literal_raw) {
                    report_diagnostic(
                        "leading",
                        // SAFETY: `is_method_call` ensures that `call_expr`'s `callee` is a `MemberExpression` with a `MemberExpression` as its `object`.
                        call_expr_method_callee_info(call_expr).unwrap().1,
                        expression_arg.span(),
                        is_template_lit,
                        ctx,
                    );
                }

                if check_literal_trailing(i, literal_raw, call_expr_arg_len) {
                    report_diagnostic(
                        "trailing",
                        // SAFETY: `is_method_call` ensures that `call_expr`'s `callee` is a `MemberExpression` with a `MemberExpression` as its `object`.
                        call_expr_method_callee_info(call_expr).unwrap().1,
                        expression_arg.span(),
                        is_template_lit,
                        ctx,
                    );
                }
            }
        }
    }
}

fn check_literal_leading(i: usize, literal: &str) -> bool {
    i != 0 && literal.starts_with(' ')
}
fn check_literal_trailing(i: usize, literal: &str, call_expr_arg_len: usize) -> bool {
    i != call_expr_arg_len - 1 && literal.ends_with(' ')
}
fn report_diagnostic<'a>(
    direction: &'static str,
    ident: &'a str,
    span: Span,
    is_template_lit: bool,
    ctx: &LintContext<'a>,
) {
    let span = if is_template_lit { span } else { Span::new(span.start + 1, span.end - 1) };

    ctx.diagnostic_with_fix(no_console_spaces_diagnostic(direction, ident, span), |fixer| {
        // Use raw source text to preserve escape sequences (e.g. `\n`, `\'`)
        let raw_text = fixer.source_range(span);
        let content = if is_template_lit {
            format!("`{}`", raw_text.trim_matches('`').trim())
        } else {
            raw_text.trim().to_string()
        };
        fixer.replace(span, content)
    });
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "console.log(\"abc\");",
        "console.log(\"abc\", \"def\");",
        "console.log('abc', \"def\");",
        "console.log(`abc`, \"def\");",
        "console.log(\"abc\", \"def\");",
        "console.log(\"abc\\t\", \"def\");",
        "console.log(\"abc\\n\", \"def\");",
        "console.log(\"  abc\", \"def\");",
        "console.log(\" abc\", \"def\");",
        "console.log(\"abc\", \"def \");",
        "console.log();",
        "console.log(\"\");",
        "console.log(123);",
        "console.log(null);",
        "console.log(undefined);",
        "console.dir(\"abc \");",
        "new console.log(\" a \", \" b \");",
        "new console.debug(\" a \", \" b \");",
        "new console.info(\" a \", \" b \");",
        "new console.warn(\" a \", \" b \");",
        "new console.error(\" a \", \" b \");",
        "log(\" a \", \" b \");",
        "debug(\" a \", \" b \");",
        "info(\" a \", \" b \");",
        "warn(\" a \", \" b \");",
        "error(\" a \", \" b \");",
        "console[log](\" a \", \" b \");",
        "console[debug](\" a \", \" b \");",
        "console[info](\" a \", \" b \");",
        "console[warn](\" a \", \" b \");",
        "console[error](\" a \", \" b \");",
        "console.foo(\" a \", \" b \");",
        "foo.log(\" a \", \" b \");",
        "foo.debug(\" a \", \" b \");",
        "foo.info(\" a \", \" b \");",
        "foo.warn(\" a \", \" b \");",
        "foo.error(\" a \", \" b \");",
        "lib.console.log(\" a \", \" b \");",
        "lib.console.debug(\" a \", \" b \");",
        "lib.console.info(\" a \", \" b \");",
        "lib.console.warn(\" a \", \" b \");",
        "lib.console.error(\" a \", \" b \");",
    ];

    let fail = vec![
        "console.log(\"abc \", \"def\");",
        "console.log(\"abc\", \" def\");",
        "console.log(\" abc \", \"def\");",
        "console.debug(\"abc \", \"def\");",
        "console.debug(`abc `, \"def\");",
        "console.info(\"abc \", \"def\");",
        "console.warn(\"abc \", \"def\");",
        "console.error(\"abc \", \"def\");",
        "console.log(\"abc\", \" def \", \"ghi\");",
        "console.log(\"abc \", \"def \", \"ghi\");",
        "console.log('abc ', \"def\");",
        "console.log(`abc `, \"def\");",
        "console.error('abc ', \"def\");",
        "console.error(`abc `, \"def\");",
        "console.log(`abc ${1 + 2} `, \"def\");",
        "console.log(\"abc\", \" def \", \"ghi\");",
        "console.log(\"_\", \" leading\", \"_\")",
        "console.log(\"_\", \"trailing \", \"_\")",
        "console.log(\"_\", \" leading and trailing \", \"_\")",
        "console.error(\"abc\", \" def \", \"ghi\");",
        "console.error(\"_\", \" leading\", \"_\")",
        "console.error(\"_\", \"trailing \", \"_\")",
        "console.error(\"_\", \" leading and trailing \", \"_\")",
        "console.log(\"_\", \" log \", \"_\")",
        "console.debug(\"_\", \" debug \", \"_\")",
        "console.info(\"_\", \" info \", \"_\")",
        "console.warn(\"_\", \" warn \", \"_\")",
        "console.error(\"_\", \" error \", \"_\")",
        // Note: This behavior differs to `eslint-plugin-unicorn(no-console-spaces)` as it "passes" there.
        "console[\"log\"](\" a \", \" b \");",
        "console[\"debug\"](\" a \", \" b \");",
        "console[\"info\"](\" a \", \" b \");",
        "console[\"warn\"](\" a \", \" b \");",
        "console[\"error\"](\" a \", \" b \");",
    ];

    let fix = vec![
        ("console.log(\"foo \", bar)", "console.log(\"foo\", bar)"),
        ("console.debug(\"foo \", bar)", "console.debug(\"foo\", bar)"),
        ("console.info(\"foo \", bar)", "console.info(\"foo\", bar)"),
        ("console.warn(\"foo \", bar)", "console.warn(\"foo\", bar)"),
        ("console.error(\"foo \", bar)", "console.error(\"foo\", bar)"),
        ("console.log(foo, \" bar\")", "console.log(foo, \"bar\")"),
        ("console.debug(foo, \" bar\")", "console.debug(foo, \"bar\")"),
        ("console.info(foo, \" bar\")", "console.info(foo, \"bar\")"),
        ("console.warn(foo, \" bar\")", "console.warn(foo, \"bar\")"),
        ("console.error(foo, \" bar\")", "console.error(foo, \"bar\")"),
        ("console.log(`foo `, bar)", "console.log(`foo`, bar)"),
        ("console.debug(`foo `, bar)", "console.debug(`foo`, bar)"),
        ("console.info(`foo `, bar)", "console.info(`foo`, bar)"),
        ("console.warn(`foo `, bar)", "console.warn(`foo`, bar)"),
        ("console.error(`foo `, bar)", "console.error(`foo`, bar)"),
        ("console.log(foo, ` bar`)", "console.log(foo, `bar`)"),
        ("console.debug(foo, ` bar`)", "console.debug(foo, `bar`)"),
        ("console.info(foo, ` bar`)", "console.info(foo, `bar`)"),
        ("console.warn(foo, ` bar`)", "console.warn(foo, `bar`)"),
        ("console.error(foo, ` bar`)", "console.error(foo, `bar`)"),
        (r#"console.log("foo\\n ", bar)"#, r#"console.log("foo\\n", bar)"#),
        (r#"console.log(foo, " \\nbar")"#, r#"console.log(foo, "\\nbar")"#),
    ];

    Tester::new(NoConsoleSpaces::NAME, NoConsoleSpaces::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

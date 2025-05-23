use std::borrow::Cow;

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
                        literal_raw,
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
                        literal_raw,
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
    literal_raw: &'a str,
    is_template_lit: bool,
    ctx: &LintContext<'a>,
) {
    let span = if is_template_lit { span } else { Span::new(span.start + 1, span.end - 1) };

    ctx.diagnostic_with_fix(no_console_spaces_diagnostic(direction, ident, span), |fixer| {
        let content = if is_template_lit {
            Cow::Owned(format!("`{}`", literal_raw.trim()))
        } else {
            Cow::Borrowed(literal_raw.trim())
        };
        fixer.replace(span, content)
    });
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("console.log(\"abc\");", None),
        ("console.log(\"abc\", \"def\");", None),
        ("console.log('abc', \"def\");", None),
        ("console.log(`abc`, \"def\");", None),
        ("console.log(\"abc\", \"def\");", None),
        ("console.log(\"abc\\t\", \"def\");", None),
        ("console.log(\"abc\\n\", \"def\");", None),
        ("console.log(\"  abc\", \"def\");", None),
        ("console.log(\" abc\", \"def\");", None),
        ("console.log(\"abc\", \"def \");", None),
        ("console.log();", None),
        ("console.log(\"\");", None),
        ("console.log(123);", None),
        ("console.log(null);", None),
        ("console.log(undefined);", None),
        ("console.dir(\"abc \");", None),
        ("new console.log(\" a \", \" b \");", None),
        ("new console.debug(\" a \", \" b \");", None),
        ("new console.info(\" a \", \" b \");", None),
        ("new console.warn(\" a \", \" b \");", None),
        ("new console.error(\" a \", \" b \");", None),
        ("log(\" a \", \" b \");", None),
        ("debug(\" a \", \" b \");", None),
        ("info(\" a \", \" b \");", None),
        ("warn(\" a \", \" b \");", None),
        ("error(\" a \", \" b \");", None),
        ("console[log](\" a \", \" b \");", None),
        ("console[debug](\" a \", \" b \");", None),
        ("console[info](\" a \", \" b \");", None),
        ("console[warn](\" a \", \" b \");", None),
        ("console[error](\" a \", \" b \");", None),
        ("console.foo(\" a \", \" b \");", None),
        ("foo.log(\" a \", \" b \");", None),
        ("foo.debug(\" a \", \" b \");", None),
        ("foo.info(\" a \", \" b \");", None),
        ("foo.warn(\" a \", \" b \");", None),
        ("foo.error(\" a \", \" b \");", None),
        ("lib.console.log(\" a \", \" b \");", None),
        ("lib.console.debug(\" a \", \" b \");", None),
        ("lib.console.info(\" a \", \" b \");", None),
        ("lib.console.warn(\" a \", \" b \");", None),
        ("lib.console.error(\" a \", \" b \");", None),
    ];

    let fail = vec![
        ("console.log(\"abc \", \"def\");", None),
        ("console.log(\"abc\", \" def\");", None),
        ("console.log(\" abc \", \"def\");", None),
        ("console.debug(\"abc \", \"def\");", None),
        ("console.debug(`abc `, \"def\");", None),
        ("console.info(\"abc \", \"def\");", None),
        ("console.warn(\"abc \", \"def\");", None),
        ("console.error(\"abc \", \"def\");", None),
        ("console.log(\"abc\", \" def \", \"ghi\");", None),
        ("console.log(\"abc \", \"def \", \"ghi\");", None),
        ("console.log('abc ', \"def\");", None),
        ("console.log(`abc `, \"def\");", None),
        ("console.error('abc ', \"def\");", None),
        ("console.error(`abc `, \"def\");", None),
        ("console.log(`abc ${1 + 2} `, \"def\");", None),
        ("console.log(\"abc\", \" def \", \"ghi\");", None),
        ("console.log(\"_\", \" leading\", \"_\")", None),
        ("console.log(\"_\", \"trailing \", \"_\")", None),
        ("console.log(\"_\", \" leading and trailing \", \"_\")", None),
        ("console.error(\"abc\", \" def \", \"ghi\");", None),
        ("console.error(\"_\", \" leading\", \"_\")", None),
        ("console.error(\"_\", \"trailing \", \"_\")", None),
        ("console.error(\"_\", \" leading and trailing \", \"_\")", None),
        ("console.log(\"_\", \" log \", \"_\")", None),
        ("console.debug(\"_\", \" debug \", \"_\")", None),
        ("console.info(\"_\", \" info \", \"_\")", None),
        ("console.warn(\"_\", \" warn \", \"_\")", None),
        ("console.error(\"_\", \" error \", \"_\")", None),
        // Note: This behavior differs to `eslint-plugin-unicorn(no-console-spaces)` as it "passes" there.
        ("console[\"log\"](\" a \", \" b \");", None),
        ("console[\"debug\"](\" a \", \" b \");", None),
        ("console[\"info\"](\" a \", \" b \");", None),
        ("console[\"warn\"](\" a \", \" b \");", None),
        ("console[\"error\"](\" a \", \" b \");", None),
    ];

    let fix = vec![
        ("console.log(\"foo \", bar)", "console.log(\"foo\", bar)", None),
        ("console.debug(\"foo \", bar)", "console.debug(\"foo\", bar)", None),
        ("console.info(\"foo \", bar)", "console.info(\"foo\", bar)", None),
        ("console.warn(\"foo \", bar)", "console.warn(\"foo\", bar)", None),
        ("console.error(\"foo \", bar)", "console.error(\"foo\", bar)", None),
        ("console.log(foo, \" bar\")", "console.log(foo, \"bar\")", None),
        ("console.debug(foo, \" bar\")", "console.debug(foo, \"bar\")", None),
        ("console.info(foo, \" bar\")", "console.info(foo, \"bar\")", None),
        ("console.warn(foo, \" bar\")", "console.warn(foo, \"bar\")", None),
        ("console.error(foo, \" bar\")", "console.error(foo, \"bar\")", None),
        ("console.log(`foo `, bar)", "console.log(`foo`, bar)", None),
        ("console.debug(`foo `, bar)", "console.debug(`foo`, bar)", None),
        ("console.info(`foo `, bar)", "console.info(`foo`, bar)", None),
        ("console.warn(`foo `, bar)", "console.warn(`foo`, bar)", None),
        ("console.error(`foo `, bar)", "console.error(`foo`, bar)", None),
        ("console.log(foo, ` bar`)", "console.log(foo, `bar`)", None),
        ("console.debug(foo, ` bar`)", "console.debug(foo, `bar`)", None),
        ("console.info(foo, ` bar`)", "console.info(foo, `bar`)", None),
        ("console.warn(foo, ` bar`)", "console.warn(foo, `bar`)", None),
        ("console.error(foo, ` bar`)", "console.error(foo, `bar`)", None),
    ];

    Tester::new(NoConsoleSpaces::NAME, NoConsoleSpaces::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

use oxc_ast::{
    ast::{Argument, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode, Fix};

#[derive(Debug, Error, Diagnostic)]
#[error(
    "eslint-plugin-unicorn(no-console-spaces): Do not use {0} spaces with `console.{1}` parameters"
)]
#[diagnostic(severity(warning), help("The `console.log()` method and similar methods join the parameters with a space so adding a leading/trailing space to a parameter, results in two spaces being added."))]
struct NoConsoleSpacesDiagnostic(&'static str, String, #[label] pub Span);

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
    /// ### Example
    /// ```javascript
    ///
    /// // Bad
    /// console.log("abc ", "def");
    ///
    /// // Good
    /// console.log("abc", "def");
    ///
    /// ```
    NoConsoleSpaces,
    style
);

impl Rule for NoConsoleSpaces {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let call_expr = match node.kind() {
            AstKind::CallExpression(call_expr) => call_expr,
            _ => return,
        };

        let member_expr = match &call_expr.callee {
            Expression::MemberExpression(member_expr) => member_expr,
            _ => return,
        };

        match member_expr.object() {
            Expression::Identifier(ident) if ident.name == "console" => ident,
            _ => return,
        };

        if let Some(ident) = member_expr.static_property_name() {
            if matches!(ident, "log" | "debug" | "info" | "warn" | "error") {
                let call_expr_arg_len = call_expr.arguments.len();

                for (i, arg) in call_expr.arguments.iter().enumerate() {
                    if let Argument::Expression(expression_arg) = &arg {
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
                                ident,
                                expression_arg.span(),
                                literal_raw,
                                is_template_lit,
                                ctx,
                            );
                        }

                        if check_literal_trailing(i, literal_raw, call_expr_arg_len) {
                            report_diagnostic(
                                "trailing",
                                ident,
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
    }
}

fn check_literal_leading(i: usize, literal: &str) -> bool {
    i != 0 && literal.starts_with(' ')
}
fn check_literal_trailing(i: usize, literal: &str, call_expr_arg_len: usize) -> bool {
    i != call_expr_arg_len - 1 && literal.ends_with(' ')
}
fn report_diagnostic(
    direction: &'static str,
    ident: &str,
    span: Span,
    literal_raw: &str,
    is_template_lit: bool,
    ctx: &LintContext,
) {
    let (start, end) =
        if is_template_lit { (span.start, span.end) } else { (span.start + 1, span.end - 1) };

    let fix = if is_template_lit {
        format!("`{}`", literal_raw.trim())
    } else {
        literal_raw.trim().to_string()
    };

    ctx.diagnostic_with_fix(NoConsoleSpacesDiagnostic(direction, ident.to_string(), span), || {
        Fix::new(fix, Span { start, end })
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

    Tester::new(NoConsoleSpaces::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}

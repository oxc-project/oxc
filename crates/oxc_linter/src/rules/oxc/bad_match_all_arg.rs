use oxc_ast::{AstKind, ast::RegExpFlags};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    ast_util::{is_method_call, resolve_regex_flags},
    context::LintContext,
    rule::Rule,
};

fn bad_match_all_arg_diagnostic(match_all_span: Span, regex_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Global flag (g) is missing in the regular expression supplied to the `matchAll` method.",
    )
    .with_help("Add the global flag (g) to the regular expression.")
    .with_note("`matchAll` throws a `TypeError` when passed a non-global regular expression.")
    .with_labels([
        match_all_span.label("`matchAll` called here"),
        regex_span.label("RegExp supplied here"),
    ])
}

#[derive(Debug, Default, Clone)]
pub struct BadMatchAllArg;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule warns when the `matchAll` method is called with a regular expression that does
    /// not have the global flag (g).
    ///
    /// ### Why is this bad?
    ///
    /// When called with a regular expression, the `matchAll` method requires the global flag (g).
    /// Otherwise, it throws a `TypeError` at runtime instead of returning an iterator.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// text.matchAll(/pattern/);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// text.matchAll(/pattern/g);
    /// ```
    BadMatchAllArg,
    oxc,
    correctness,
    version = "next",
    short_description = "Warn when `matchAll` is called with a non-global regular expression.",
);

impl Rule for BadMatchAllArg {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::CallExpression(call_expr) = node.kind()
            && is_method_call(call_expr, None, Some(&["matchAll"]), Some(1), None)
            && let Some(regexp_argument) = call_expr.arguments[0].as_expression()
            && let Some((flags, regex_span)) = resolve_regex_flags(regexp_argument, ctx)
            && !flags.contains(RegExpFlags::G)
            && let Some(call_expr_callee) = call_expr.callee.as_member_expression()
            && let Some((match_all_span, _)) = call_expr_callee.static_property_info()
        {
            ctx.diagnostic(bad_match_all_arg_diagnostic(match_all_span, regex_span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Global regular expressions.
        r#""".matchAll(/a/g);"#,
        r#""".matchAll(/a/giu);"#,
        r#""".matchAll(new RegExp("a", "g"));"#,
        r#""".matchAll(RegExp("a", "g"));"#,
        r#""".matchAll(globalThis.RegExp("a", "g"));"#,
        // Calls that cannot throw for a known non-global regular expression.
        r#""".matchAll("a");"#,
        r#""".matchAll();"#,
        r"matchAll(/a/);",
        r"new matchAll(/a/);",
        r#"function RegExp() {} "".matchAll(new RegExp("a"));"#,
        // Resolved variables.
        r#"const pattern = "a"; "".matchAll(pattern);"#,
        r#"const pattern = /a/g; "".matchAll(pattern);"#,
        r#"const pattern = new RegExp("a", "g"); "".matchAll(pattern);"#,
        r#"const pattern = new RegExp("a", condition ? "g" : "gi"); "".matchAll(pattern);"#,
        r#"const pattern = pattern; "".matchAll(pattern);"#,
        r#"const a = b; const b = a; "".matchAll(a);"#,
    ];

    let fail = vec![
        r#""".matchAll(/a/);"#,
        r#""".matchAll(/a/u);"#,
        r#""".matchAll(new RegExp("a"));"#,
        r#""".matchAll(new RegExp("a", "i"));"#,
        r#""".matchAll(RegExp("a"));"#,
        r#""".matchAll(globalThis.RegExp("a", "i"));"#,
        r#"""["matchAll"](/a/);"#,
        // Resolved variables.
        r#"const pattern = /a/; "".matchAll(pattern);"#,
        r#"const pattern = /a/i; "".matchAll(pattern);"#,
        r#"const pattern = new RegExp("a"); "".matchAll(pattern);"#,
    ];

    Tester::new(BadMatchAllArg::NAME, BadMatchAllArg::PLUGIN, pass, fail).test_and_snapshot();
}

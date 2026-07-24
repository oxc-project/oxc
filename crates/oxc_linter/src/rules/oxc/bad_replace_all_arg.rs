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

fn bad_replace_all_arg_diagnostic(replace_all_span: Span, regex_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Global flag (g) is missing in the regular expression supplied to the `replaceAll` method.")
        .with_help("To replace all occurrences of a string, use the `replaceAll` method with the global flag (g) in the regular expression.")
        .with_note("Unlike `replace`, `replaceAll` throws a `TypeError` when passed a non-global regular expression instead of replacing only the first match.")
        .with_labels([
            replace_all_span.label("`replaceAll` called here"),
            regex_span.label("RegExp supplied here"),
        ])
}

#[derive(Debug, Default, Clone)]
pub struct BadReplaceAllArg;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule warns when the `replaceAll` method is called with a regular expression that does not have the global flag (g).
    ///
    /// ### Why is this bad?
    ///
    /// When called with a regular expression, the `replaceAll` method requires the global flag (g).
    /// Otherwise, it throws a `TypeError` at runtime instead of performing a replacement.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// withSpaces.replaceAll(/\s+/, ',');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// withSpaces.replaceAll(/\s+/g, ',');
    /// ```
    BadReplaceAllArg,
    oxc,
    correctness,
    version = "0.0.22",
    short_description = "This rule warns when the `replaceAll` method is called with a regular expression that does not have the global flag (g).",
);

impl Rule for BadReplaceAllArg {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_method_call(call_expr, None, Some(&["replaceAll"]), Some(1), None) {
            return;
        }

        let Some(regexp_argument) = call_expr.arguments[0].as_expression() else {
            return;
        };

        let Some((flags, regex_span)) = resolve_regex_flags(regexp_argument, ctx) else {
            return;
        };

        if !flags.contains(RegExpFlags::G) {
            let Some(call_expr_callee) = call_expr.callee.as_member_expression() else {
                return;
            };
            let Some((replace_all_span, _)) = call_expr_callee.static_property_info() else {
                return;
            };

            ctx.diagnostic(bad_replace_all_arg_diagnostic(replace_all_span, regex_span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // valid call
        r"withSpaces.replaceAll(/\s+/g, ',');",
        // incorrect number of arguments
        r"withSpaces.replaceAll();",
        // not a method call
        r"replaceAll(/\s+/, ',');",
        // not a method call
        r"withSpaces();",
        // new RegExp
        r"withSpaces.replaceAll(new RegExp('\s+', 'g'), ',');",
        // new replaceAll
        r"new replaceAll(/\s+/, ',');",
        // resolved vars
        r#"const foo = "string"; withSpaces.replaceAll(foo, ',');"#,
        // resolved vars
        r"const foo = /\s+/g; withSpaces.replaceAll(foo, ',');",
        // resolved vars
        r"const foo = new RegExp('\s+', 'g'); withSpaces.replaceAll(foo, ',');",
        r"const foo = new RegExp('\s+', isWindows ? 'g' : 'gi'); withSpaces.replaceAll(foo, ',');",
    ];

    let fail = vec![
        r"withSpaces.replaceAll(/\s+/, ',');",
        r"withSpaces.replaceAll(/\s+/i, ',');",
        r"withSpaces.replaceAll(new RegExp('\s+'), ',');",
        r"withSpaces.replaceAll(new RegExp('\s+','i'), ',');",
        // resolved vars
        r"
            const foo = /\s+/;

            withSpaces.replaceAll(foo, ',');
        ",
        r"
            const foo = /\s+/i;

            withSpaces.replaceAll(foo, ',');
        ",
        r"
            const foo = new RegExp('\s+');

            withSpaces.replaceAll(foo, ',');
        ",
    ];

    Tester::new(BadReplaceAllArg::NAME, BadReplaceAllArg::PLUGIN, pass, fail).test_and_snapshot();
}

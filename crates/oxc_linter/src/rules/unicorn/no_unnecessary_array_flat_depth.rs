use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode, ast_util::is_method_call, context::LintContext, fixer::RuleFixer, rule::Rule,
};

fn no_unnecessary_array_flat_depth_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Passing `1` as the `depth` argument is unnecessary.")
        .with_help("Remove the argument")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnnecessaryArrayFlatDepth;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows passing `1` to `Array.prototype.flat`.
    ///
    /// ### Why is this bad?
    ///
    /// Passing `1` is unnecessary.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// foo.flat(1)
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// foo.flat()
    /// ```
    NoUnnecessaryArrayFlatDepth,
    unicorn,
    pedantic,
    suggestion
);

impl Rule for NoUnnecessaryArrayFlatDepth {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };
        if call_expr.optional || !is_method_call(call_expr, None, Some(&["flat"]), Some(1), Some(1))
        {
            return;
        }

        let Some(arg) = call_expr.arguments[0].as_expression() else { return };
        let arg = arg.get_inner_expression();
        if arg.is_number_value(1.0) {
            let diagnostic_span = call_expr
                .callee
                .as_member_expression()
                .and_then(oxc_ast::ast::MemberExpression::static_property_info)
                .map_or(call_expr.span, |(span, _)| span);

            ctx.diagnostic_with_suggestion(
                no_unnecessary_array_flat_depth_diagnostic(diagnostic_span),
                |fixer: RuleFixer<'_, 'a>| {
                    fixer.delete_range(Span::new(arg.span().start, call_expr.span.end - 1))
                },
            );
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "foo.flat()",
        "foo.flat?.(1)",
        "foo?.flat()",
        "foo.flat(1, extra)",
        "flat(1)",
        "new foo.flat(1)",
        "const ONE = 1; foo.flat(ONE)",
        "foo.notFlat(1)",
    ];

    let fail = vec!["foo.flat(1)", "foo.flat(1.0)", "foo.flat(0b01)", "foo?.flat(1)"];

    let fix = vec![
        ("foo.flat(1)", "foo.flat()"),
        ("foo.flat(1.0)", "foo.flat()"),
        ("foo.flat(0b01)", "foo.flat()"),
        ("foo?.flat(1)", "foo?.flat()"),
        ("foo?.flat(1,)", "foo?.flat()"),
    ];

    Tester::new(NoUnnecessaryArrayFlatDepth::NAME, NoUnnecessaryArrayFlatDepth::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

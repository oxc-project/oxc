use oxc_ast::{
    ast::{CallExpression, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{ast_util::is_method_call, context::LintContext, rule::Rule, AstNode};

fn no_magic_array_flat_map_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Magic number for `Array.prototype.flat` depth is not allowed.")
        .with_help("Add a comment explaining the depth.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoMagicArrayFlatDepth;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow magic numbers for `Array.prototype.flat` depth.
    ///
    /// ### Why is this bad?
    ///
    /// Magic numbers are hard to understand and maintain. When calling `Array.prototype.flat`, it is usually called with `1` or infinity. If you are using a different number, it is better to add a comment explaining the depth.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// array.flat(2);
    /// array.flat(20);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// array.flat(2 /* explanation */);
    /// array.flat(1);
    /// array.flat();
    /// array.flat(Infinity);
    /// ```
    NoMagicArrayFlatDepth,
    restriction,
);

impl Rule for NoMagicArrayFlatDepth {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expression) = node.kind() else {
            return;
        };

        if !is_method_call(call_expression, None, Some(&["flat"]), Some(1), Some(1))
            || call_expression.optional
        {
            return;
        }

        let first_arg = call_expression.arguments.first().expect("missing argument");
        let Some(Expression::NumericLiteral(arg)) =
            first_arg.as_expression().map(oxc_ast::ast::Expression::without_parentheses)
        else {
            return;
        };

        if (arg.value - 1.0).abs() < f64::EPSILON {
            return;
        }

        let Some(arguments_span) = get_call_expression_parentheses_pos(call_expression, ctx) else {
            return;
        };

        let has_explaining_comment =
            ctx.semantic().comments_range(arguments_span.start..arguments_span.end).count() != 0;

        if has_explaining_comment {
            return;
        }

        ctx.diagnostic(no_magic_array_flat_map_diagnostic(arg.span));
    }
}

// gets the opening `(` and closing `)` of the argument
#[allow(clippy::cast_possible_truncation)]
fn get_call_expression_parentheses_pos<'a>(
    call_expr: &CallExpression<'a>,
    ctx: &LintContext<'a>,
) -> Option<Span> {
    call_expr.callee.get_member_expr().map(|member_expr| {
        let callee_span = member_expr.object().span();

        // walk forward from the end of callee_span to find the opening `(` of the argument
        let source = ctx.semantic().source_text().char_indices();

        let start = source
            .skip(callee_span.end as usize)
            .find(|(_, c)| c == &'(')
            .map(|(i, _)| i as u32)
            .expect("missing opening `(` for call expression argument");

        let end = call_expr.span.end;

        Span::new(start, end)
    })
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "array.flat(1)",
        "array.flat(1.0)",
        "array.flat(0x01)",
        "array.flat(unknown)",
        "array.flat(Number.POSITIVE_INFINITY)",
        "array.flat(Infinity)",
        "array.flat(/* explanation */2)",
        "array.flat(2/* explanation */)",
        "array.flat()",
        "array.flat(2, extraArgument)",
        "new array.flat(2)",
        "array.flat?.(2)",
        "array.notFlat(2)",
        "flat(2)",
    ];

    let fail = vec!["array.flat(2)", "array?.flat(2)", "array.flat(99,)", "array.flat(0b10,)"];

    Tester::new(NoMagicArrayFlatDepth::NAME, pass, fail).test_and_snapshot();
}

use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, ast_util::is_method_call, context::LintContext, rule::Rule};

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
    /// Disallow magic numbers for [`Array.prototype.flat`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/flat)
    /// depth.
    ///
    /// ### Why is this bad?
    ///
    /// Magic numbers are hard to understand and maintain.
    /// When calling `Array.prototype.flat`, it is usually called with
    /// `1` or `Infinity`. If you are using a different number, it is
    /// better to add a comment explaining the reason for the depth provided.
    ///
    /// ### Examples
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
    unicorn,
    restriction,
    version = "0.4.2",
    short_description = "Disallow magic numbers for `Array.prototype.flat` depth.",
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
            first_arg.as_expression().map(Expression::without_parentheses)
        else {
            return;
        };

        if (arg.value - 1.0).abs() < f64::EPSILON {
            return;
        }

        // the arguments start at the `(` following the callee
        let callee_end = call_expression.callee.span().end;
        let call_end = call_expression.span.end;
        let Some(offset) = ctx.find_next_token_within(callee_end, call_end, "(") else {
            return;
        };

        let has_explaining_comment = ctx.comments_range(callee_end + offset..call_end).count() != 0;

        if has_explaining_comment {
            return;
        }

        ctx.diagnostic(no_magic_array_flat_map_diagnostic(arg.span));
    }
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
        // multi-byte characters before the call must not shift the argument span
        "const s = \"😀😀\";\narray.flat(2 /* explanation */)",
        "array[key].flat(2 /* explanation */)",
    ];

    let fail = vec![
        "array.flat(2)",
        "array?.flat(2)",
        "array.flat(99,)",
        "array.flat(0b10,)",
        "const s = \"😀😀😀😀😀😀😀😀😀😀\";\narray.flat(2)",
        // a comment in the computed key is not an explanation of the depth
        "array[key(/* not an explanation */)].flat(2)",
        // a `(` inside a comment is not the start of the arguments
        "array.flat/* ( */(2)",
    ];

    Tester::new(NoMagicArrayFlatDepth::NAME, NoMagicArrayFlatDepth::PLUGIN, pass, fail)
        .test_and_snapshot();
}

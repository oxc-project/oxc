use oxc_ast::{
    ast::{Argument, CallExpression, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;

use super::{
    is_equality_matcher, parse_jest_fn_call, ParsedExpectFnCall, ParsedJestFnCallNew,
    PossibleJestNode,
};
use crate::LintContext;

mod valid_vitest_fn;
pub use crate::utils::vitest::valid_vitest_fn::VALID_VITEST_FN_CALL_CHAINS;

pub fn parse_expect_and_typeof_vitest_fn_call<'a>(
    call_expr: &'a CallExpression<'a>,
    possible_jest_node: &PossibleJestNode<'a, '_>,
    ctx: &LintContext<'a>,
) -> Option<ParsedExpectFnCall<'a>> {
    let jest_fn_call = parse_jest_fn_call(call_expr, possible_jest_node, ctx)?;

    match jest_fn_call {
        ParsedJestFnCallNew::Expect(jest_fn_call)
        | ParsedJestFnCallNew::ExpectTypeOf(jest_fn_call) => Some(jest_fn_call),
        ParsedJestFnCallNew::GeneralJest(_) => None,
    }
}

pub fn prefer_to_be_simply_bool<'a>(
    possible_vitest_node: &PossibleJestNode<'a, '_>,
    ctx: &LintContext<'a>,
    value: bool,
) {
    let node = possible_vitest_node.node;
    let AstKind::CallExpression(call_expr) = node.kind() else {
        return;
    };
    let Some(vitest_expect_fn_call) =
        parse_expect_and_typeof_vitest_fn_call(call_expr, possible_vitest_node, ctx)
    else {
        return;
    };
    let Some(matcher) = vitest_expect_fn_call.matcher() else {
        return;
    };
    if !is_equality_matcher(matcher) || vitest_expect_fn_call.args.len() == 0 {
        return;
    }
    let Some(arg_expr) = vitest_expect_fn_call.args.first().and_then(Argument::as_expression)
    else {
        return;
    };

    if let Expression::BooleanLiteral(arg) = arg_expr.get_inner_expression() {
        if arg.value == value {
            let span = Span::new(matcher.span.start, call_expr.span.end);

            let is_cmp_mem_expr = match matcher.parent {
                Some(Expression::ComputedMemberExpression(_)) => true,
                Some(
                    Expression::StaticMemberExpression(_) | Expression::PrivateFieldExpression(_),
                ) => false,
                _ => return,
            };

            let call_name = if value { "toBeTruthy" } else { "toBeFalsy" };

            ctx.diagnostic_with_fix(
                OxcDiagnostic::warn(format!("Use `{call_name}` instead.")).with_label(span),
                |fixer| {
                    let new_matcher = if is_cmp_mem_expr {
                        format!("[\"{call_name}\"]()")
                    } else {
                        format!("{call_name}()")
                    };
                    fixer.replace(span, new_matcher)
                },
            );
        }
    }
}

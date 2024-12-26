use oxc_ast::ast::CallExpression;

use super::{
    parse_jest_fn_call, JestGeneralFnKind, ParsedExpectFnCall, ParsedJestFnCallNew,
    PossibleJestNode,
};
use crate::{utils::JestFnKind, LintContext};

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

pub fn parse_vitest_fn_call<'a>(
    call_expr: &'a CallExpression<'a>,
    possible_jest_node: &PossibleJestNode<'a, '_>,
    ctx: &LintContext<'a>,
) -> Option<JestGeneralFnKind> {
    let jest_fn_call = parse_jest_fn_call(call_expr, possible_jest_node, ctx)?;

    match jest_fn_call {
        ParsedJestFnCallNew::GeneralJest(jest_fn_call) => match jest_fn_call.kind {
            JestFnKind::General(kind) => Some(kind),
            _ => None,
        },
        _ => None,
    }
}

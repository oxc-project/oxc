use num_bigint::ToBigInt;

use oxc_ast::ast::Expression;
use oxc_ecmascript::constant_evaluation::ConstantValue;
use oxc_ecmascript::is_global_reference::IsGlobalReference;
use oxc_span::GetSpan;

use super::{PeepholeOptimizations, State};
use crate::ctx::Ctx;

impl<'a> PeepholeOptimizations {
    pub fn inline_const(
        &self,
        expr: &mut Expression<'a>,
        state: &mut State,
        ctx: &mut Ctx<'a, '_>,
    ) {
        if !ctx.state.options.inline_const {
            return;
        }
        let Expression::Identifier(ident) = expr else { return };
        let Some(reference_id) = ident.reference_id.get() else { return };
        let Some(cv) = ctx.get_constant_value_for_reference_id(reference_id) else { return };
        // Inline character lengths <= 3
        let accept = match &cv {
            ConstantValue::Number(n) => n.fract() == 0.0 && *n <= 100.0 && *n >= -10.0,
            ConstantValue::BigInt(n) => *n <= 10.to_bigint().unwrap(),
            ConstantValue::String(s) => s.len() <= 3,
            ConstantValue::Boolean(_) => true,
            // TODO
            ConstantValue::Undefined => false,
            ConstantValue::Null => true,
        };
        if accept {
            state.changed = true;
            *expr = ctx.value_to_expr(expr.span(), cv);
        }
    }
}

use crate::generated::ancestor::Ancestor;
use oxc_ast::ast::*;
use oxc_ecmascript::constant_evaluation::{ConstantEvaluation, ConstantValue};
use oxc_span::GetSpan;

use crate::TraverseCtx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    pub fn init_symbol_value(decl: &VariableDeclarator<'a>, ctx: &mut TraverseCtx<'a>) {
        let BindingPattern::BindingIdentifier(ident) = &decl.id else { return };
        let Some(symbol_id) = ident.symbol_id.get() else { return };
        let value = if decl.kind.is_var() || Self::is_for_statement_init(ctx) {
            // - Skip constant value inlining for `var` declarations, due to TDZ problems.
            // - Set None for for statement initializers as the value of these are set by the for statement.
            None
        } else {
            decl.init.as_ref().map_or(Some(ConstantValue::Undefined), |e| e.evaluate_value(ctx))
        };
        ctx.init_value(symbol_id, value);
    }

    fn is_for_statement_init(ctx: &TraverseCtx<'a>) -> bool {
        ctx.ancestors().nth(1).is_some_and(Ancestor::is_parent_of_for_statement_left)
    }

    pub fn inline_identifier_reference(expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::Identifier(ident) = expr else { return };
        let reference_id = ident.reference_id();
        let Some(symbol_id) = ctx.scoping().get_reference(reference_id).symbol_id() else { return };
        let Some(symbol_value) = ctx.state.symbol_values.get_symbol_value(symbol_id) else {
            return;
        };
        // Skip if there are write references.
        if symbol_value.write_references_count > 0 {
            return;
        }
        let Some(cv) = &symbol_value.initialized_constant else { return };
        if symbol_value.read_references_count == 1
            || match cv {
                ConstantValue::Number(n) => n.fract() == 0.0 && *n >= -99.0 && *n <= 999.0,
                ConstantValue::BigInt(_) => false,
                ConstantValue::String(s) => s.len() <= 3,
                ConstantValue::Boolean(_) | ConstantValue::Undefined | ConstantValue::Null => true,
            }
        {
            *expr = ctx.value_to_expr(expr.span(), cv.clone());
            ctx.state.changed = true;
        }
    }
}

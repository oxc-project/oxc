use oxc_ast::ast::*;
use oxc_ast::VisitMut;
use oxc_ast::VisitResult;

use crate::context::TransformerContext;
use crate::visit_utils::TransformResult;

pub struct NumericSeparators<'a> {
    ctx: TransformerContext<'a>,
}

impl<'a> VisitMut<'a> for NumericSeparators<'a> {
    type Result = TransformResult<'a>;

    fn visit_bigint_literal(&mut self, lit: &mut BigIntLiteral<'a>) -> Self::Result {
        if lit.raw.contains('_') {
            lit.raw = self.ctx.ast.new_atom(lit.raw.replace('_', "").as_str());
        }

        TransformResult::keep()
    }

    fn visit_number_literal(&mut self, lit: &mut NumericLiteral<'a>) -> Self::Result {
        if lit.raw.contains('_') {
            lit.raw = self.ctx.ast.new_str(lit.raw.replace('_', "").as_str());
        }

        TransformResult::keep()
    }
}

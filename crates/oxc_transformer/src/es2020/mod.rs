use oxc_ast::ast::*;
use oxc_diagnostics::OxcDiagnostic;
use oxc_traverse::Traverse;

use crate::{
    state::TransformState, context::TraverseCtx,
};

mod nullish_coalescing_operator;
mod optional_chaining;
mod options;
use nullish_coalescing_operator::NullishCoalescingOperator;
pub use optional_chaining::OptionalChaining;
pub use options::ES2020Options;

pub struct ES2020<'a> {
    options: ES2020Options,

    // Plugins
    nullish_coalescing_operator: NullishCoalescingOperator<'a>,
    optional_chaining: OptionalChaining<'a>,
}

impl<'a> ES2020<'a> {
    pub fn new(options: ES2020Options) -> Self {
        Self {
            options,
            nullish_coalescing_operator: NullishCoalescingOperator::new(),
            optional_chaining: OptionalChaining::new(),
        }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for ES2020<'a, '_> {
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.nullish_coalescing_operator {
            self.nullish_coalescing_operator.enter_expression(expr, ctx);
        }

        if self.options.optional_chaining {
            self.optional_chaining.enter_expression(expr, ctx);
        }
    }

    fn enter_formal_parameters(
        &mut self,
        node: &mut FormalParameters<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.optional_chaining {
            self.optional_chaining.enter_formal_parameters(node, ctx);
        }
    }

    fn exit_formal_parameters(
        &mut self,
        node: &mut FormalParameters<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.optional_chaining {
            self.optional_chaining.exit_formal_parameters(node, ctx);
        }
    }

    fn enter_big_int_literal(&mut self, node: &mut BigIntLiteral<'a>, _ctx: &mut TraverseCtx<'a>) {
        if self.options.big_int {
            let warning = OxcDiagnostic::warn(
                "Big integer literals are not available in the configured target environment.",
            )
            .with_label(node.span);
            self.ctx.error(warning);
        }
    }
}

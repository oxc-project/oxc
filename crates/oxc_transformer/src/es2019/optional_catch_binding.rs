use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::SPAN;

use crate::{context::TransformerCtx, options::TransformTarget, utils::CreateVars};

/// ES2019: Optional Catch Binding
///
/// References:
/// * <https://babel.dev/docs/babel-plugin-transform-optional-catch-binding>
/// * <https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-optional-catch-binding>
pub struct OptionalCatchBinding<'a> {
    ctx: TransformerCtx<'a>,
    vars: Vec<'a, VariableDeclarator<'a>>,
}

impl<'a> CreateVars<'a> for OptionalCatchBinding<'a> {
    fn ctx(&self) -> &TransformerCtx<'a> {
        &self.ctx
    }

    fn vars_mut(&mut self) -> &mut Vec<'a, VariableDeclarator<'a>> {
        &mut self.vars
    }
}

impl<'a> OptionalCatchBinding<'a> {
    pub fn new(ctx: TransformerCtx<'a>) -> Option<Self> {
        (ctx.options.target < TransformTarget::ES2019 || ctx.options.optional_catch_binding)
            .then_some(Self { vars: ctx.ast.new_vec(), ctx })
    }

    pub fn transform_catch_clause<'b>(&mut self, clause: &'b mut CatchClause<'a>) {
        if clause.param.is_some() {
            return;
        }
        let unused = self.create_new_named_var("unused");
        let binding_identifier = BindingIdentifier::new(SPAN, unused.name);
        let binding_pattern_kind = self.ctx.ast.binding_pattern_identifier(binding_identifier);
        let binding_pattern = self.ctx.ast.binding_pattern(binding_pattern_kind, None, false);
        clause.param = Some(binding_pattern);
    }
}

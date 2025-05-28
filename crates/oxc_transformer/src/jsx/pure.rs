/// React Pure Annotations
///
/// TODO: port [@babel/plugin-transform-react-pure-annotations](https://github.com/babel/babel/tree/main/packages/babel-plugin-transform-react-pure-annotations)
use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::context::TransformCtx;

pub struct ReactPureAnnotations<'a, 'ctx> {
    #[expect(dead_code)]
    ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> ReactPureAnnotations<'a, 'ctx> {
    pub fn new(ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { ctx }
    }
}

impl<'a> Traverse<'a> for ReactPureAnnotations<'a, '_> {
    fn enter_call_expression(
        &mut self,
        _call: &mut CallExpression<'a>,
        _ctx: &mut TraverseCtx<'a>,
    ) {
        // Checks if the callee is a specific react function
        // call.pure = true
    }
}

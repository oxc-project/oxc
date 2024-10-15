//! Async Generator Function
//!

use oxc_traverse::Traverse;

use crate::context::TransformCtx;

pub struct AsyncGeneratorFunctions<'a, 'ctx> {
    ctx: &'ctx TransformCtx<'a>,
}

impl<'a, 'ctx> AsyncGeneratorFunctions<'a, 'ctx> {
    pub fn new(ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { ctx }
    }
}

impl<'a, 'ctx> Traverse<'a> for AsyncGeneratorFunctions<'a, 'ctx> {}

impl<'a, 'ctx> AsyncGeneratorFunctions<'a, 'ctx> {}

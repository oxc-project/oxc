use oxc_ast::ast::{ArrowFunctionExpression, CallExpression, Function, NewExpression};

use crate::annotation_comment::gen_comment;
use crate::{Codegen, Context};

/// the [GenComment] trait only generate annotate comments like `/* @__PURE__ */` and `/* @__NO_SIDE_EFFECTS__ */`.
pub trait GenComment<const MINIFY: bool> {
    fn gen_comment(&self, _p: &mut Codegen<{ MINIFY }>, _ctx: Context) {}
}

impl<const MINIFY: bool> GenComment<MINIFY> for ArrowFunctionExpression<'_> {
    fn gen_comment(&self, codegen: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        gen_comment(self.span.start, codegen);
    }
}

impl<const MINIFY: bool> GenComment<MINIFY> for Function<'_> {
    fn gen_comment(&self, codegen: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        gen_comment(self.span.start, codegen);
    }
}

impl<const MINIFY: bool> GenComment<MINIFY> for CallExpression<'_> {
    fn gen_comment(&self, codegen: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        gen_comment(self.span.start, codegen);
    }
}

impl<const MINIFY: bool> GenComment<MINIFY> for NewExpression<'_> {
    fn gen_comment(&self, codegen: &mut Codegen<{ MINIFY }>, _ctx: Context) {
        gen_comment(self.span.start, codegen);
    }
}

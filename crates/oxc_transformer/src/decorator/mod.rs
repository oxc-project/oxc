mod legacy;

use legacy::LegacyDecorators;
use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::TransformCtx;

mod options;

pub use options::DecoratorOptions;

pub struct Decorator<'a, 'ctx> {
    options: DecoratorOptions,

    // Plugins
    legacy_decorator: LegacyDecorators<'a, 'ctx>,
}

impl<'a, 'ctx> Decorator<'a, 'ctx> {
    pub fn new(options: DecoratorOptions, ctx: &'ctx TransformCtx<'a>) -> Self {
        Self { legacy_decorator: LegacyDecorators::new(ctx), options }
    }
}

impl<'a> Traverse<'a> for Decorator<'a, '_> {
    fn enter_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.legacy {
            self.legacy_decorator.enter_statement(stmt, ctx);
        }
    }
}

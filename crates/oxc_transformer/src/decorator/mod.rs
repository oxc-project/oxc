mod legacy;
mod options;

use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::TransformCtx;

use legacy::LegacyDecorator;
pub use options::DecoratorOptions;

pub struct Decorator<'a, 'ctx> {
    options: DecoratorOptions,

    // Plugins
    legacy_decorator: LegacyDecorator<'a, 'ctx>,
}

impl<'a, 'ctx> Decorator<'a, 'ctx> {
    pub fn new(options: DecoratorOptions, ctx: &'ctx TransformCtx<'a>) -> Self {
        Self {
            legacy_decorator: LegacyDecorator::new(options.emit_decorator_metadata, ctx),
            options,
        }
    }
}

impl<'a> Traverse<'a> for Decorator<'a, '_> {
    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.legacy {
            self.legacy_decorator.exit_statement(stmt, ctx);
        }
    }

    #[inline]
    fn enter_class(&mut self, node: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.legacy {
            self.legacy_decorator.enter_class(node, ctx);
        }
    }

    #[inline]
    fn exit_class(&mut self, node: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.legacy {
            self.legacy_decorator.exit_class(node, ctx);
        }
    }

    #[inline]
    fn enter_method_definition(
        &mut self,
        node: &mut MethodDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.legacy {
            self.legacy_decorator.enter_method_definition(node, ctx);
        }
    }

    #[inline]
    fn enter_accessor_property(
        &mut self,
        node: &mut AccessorProperty<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.legacy {
            self.legacy_decorator.enter_accessor_property(node, ctx);
        }
    }

    #[inline]
    fn enter_property_definition(
        &mut self,
        node: &mut PropertyDefinition<'a>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.options.legacy {
            self.legacy_decorator.enter_property_definition(node, ctx);
        }
    }
}

mod legacy;
mod options;

use oxc_ast::ast::*;
use oxc_traverse::Traverse;

use crate::{
    state::TransformState, context::TraverseCtx,
};

use legacy::LegacyDecorator;
pub use options::DecoratorOptions;

pub struct Decorator<'a> {
    options: DecoratorOptions,

    // Plugins
    legacy_decorator: LegacyDecorator<'a>,
}

impl<'a> Decorator<'a> {
    pub fn new(options: DecoratorOptions, ) -> Self {
        Self {
            legacy_decorator: LegacyDecorator::new(options.emit_decorator_metadata),
            options,
        }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for Decorator<'a> {
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

impl<'a> Decorator<'a> {
    pub fn exit_class_at_end(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.legacy {
            self.legacy_decorator.exit_class_at_end(class, ctx);
        }
    }
}

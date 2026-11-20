mod legacy;
mod options;

use oxc_allocator::ArenaVec;
use oxc_ast::ast::*;
use oxc_traverse::Traverse;

use crate::{context::TraverseCtx, state::TransformState};

use legacy::LegacyDecorator;
pub use options::{DecoratorMode, DecoratorOptions};

pub struct Decorator<'a> {
    mode: DecoratorMode,

    // Plugins
    legacy_decorator: LegacyDecorator<'a>,
}

impl Decorator<'_> {
    pub fn new(options: DecoratorOptions, mode: DecoratorMode) -> Self {
        Self { legacy_decorator: LegacyDecorator::new(options), mode }
    }
}

impl<'a> Traverse<'a, TransformState<'a>> for Decorator<'a> {
    #[inline]
    fn exit_program(
        &mut self,
        node: &mut Program<'a>,
        ctx: &mut oxc_traverse::TraverseCtx<'a, TransformState<'a>>,
    ) {
        if self.mode.is_legacy() {
            self.legacy_decorator.exit_program(node, ctx);
        }
    }

    #[inline]
    fn enter_statements(
        &mut self,
        stmts: &mut ArenaVec<'a, Statement<'a>>,
        ctx: &mut TraverseCtx<'a>,
    ) {
        if self.mode.is_legacy() {
            self.legacy_decorator.enter_statements(stmts, ctx);
        }
    }

    #[inline]
    fn exit_statement(&mut self, stmt: &mut Statement<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.mode.is_legacy() {
            self.legacy_decorator.exit_statement(stmt, ctx);
        }
    }

    #[inline]
    fn enter_class(&mut self, node: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.mode.is_legacy() {
            self.legacy_decorator.enter_class(node, ctx);
        }
    }

    #[inline]
    fn exit_class(&mut self, node: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.mode.is_legacy() {
            self.legacy_decorator.exit_class(node, ctx);
        }
    }
}

impl<'a> Decorator<'a> {
    #[inline]
    pub fn exit_class_at_end(&mut self, class: &mut Class<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.mode.is_legacy() {
            self.legacy_decorator.exit_class_at_end(class, ctx);
        }
    }
}

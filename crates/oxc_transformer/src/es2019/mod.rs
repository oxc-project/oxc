mod optional_catch_binding;
mod options;

use std::rc::Rc;

pub use optional_catch_binding::OptionalCatchBinding;
pub use options::ES2019Options;
use oxc_ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::context::Ctx;

#[allow(dead_code)]
pub struct ES2019<'a> {
    ctx: Ctx<'a>,
    options: ES2019Options,

    // Plugins
    optional_catch_binding: OptionalCatchBinding<'a>,
}

impl<'a> ES2019<'a> {
    pub fn new(options: ES2019Options, ctx: Ctx<'a>) -> Self {
        Self { optional_catch_binding: OptionalCatchBinding::new(Rc::clone(&ctx)), ctx, options }
    }
}

impl<'a> Traverse<'a> for ES2019<'a> {
    fn enter_catch_clause(&mut self, clause: &mut CatchClause<'a>, ctx: &mut TraverseCtx<'a>) {
        if self.options.optional_catch_binding {
            self.optional_catch_binding.enter_catch_clause(clause, ctx);
        }
    }
}

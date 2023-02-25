use std::fmt::Debug;

use oxc_ast::AstKind;

use crate::context::LintContext;

pub trait Rule: Sized + Default + Debug {
    fn run<'a>(&self, kind: AstKind<'a>, _ctx: &LintContext<'a>);
}

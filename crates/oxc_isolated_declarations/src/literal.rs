use oxc_allocator::ArenaBox;
use oxc_ast::ast::{StringLiteral, TemplateLiteral};

use crate::IsolatedDeclarations;

impl<'a> IsolatedDeclarations<'a> {
    pub(crate) fn transform_template_to_string(
        &self,
        lit: &TemplateLiteral<'a>,
    ) -> Option<ArenaBox<'a, StringLiteral<'a>>> {
        lit.single_quasi().map(|value| StringLiteral::boxed(lit.span, value, None, self))
    }
}

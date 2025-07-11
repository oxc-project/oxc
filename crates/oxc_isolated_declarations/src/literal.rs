use oxc_allocator::Box as ArenaBox;
use oxc_ast::ast::{StringLiteral, TemplateLiteral};

use crate::IsolatedDeclarations;

impl<'a> IsolatedDeclarations<'a> {
    pub(crate) fn transform_template_to_string(
        &self,
        lit: &TemplateLiteral<'a>,
    ) -> Option<ArenaBox<'a, StringLiteral<'a>>> {
        if lit.lead.is_empty() {
            Some(self.ast.alloc_string_literal(
                lit.span,
                lit.tail.value.cooked.unwrap_or(lit.tail.value.raw),
                None,
            ))
        } else {
            None
        }
    }
}

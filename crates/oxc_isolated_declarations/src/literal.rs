use oxc_allocator::ArenaBox;
use oxc_ast::ast::{StringLiteral, TemplateLiteral};

use crate::IsolatedDeclarations;

impl<'a> IsolatedDeclarations<'a> {
    pub(crate) fn transform_template_to_string(
        &self,
        lit: &TemplateLiteral<'a>,
    ) -> Option<ArenaBox<'a, StringLiteral<'a>>> {
        if lit.expressions.is_empty() {
            lit.quasis.first().map(|item| {
                StringLiteral::boxed(
                    lit.span,
                    item.value.cooked.unwrap_or(item.value.raw),
                    None,
                    self,
                )
            })
        } else {
            None
        }
    }
}

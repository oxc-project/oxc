use oxc_allocator::Box;
use oxc_ast::ast::{StringLiteral, TemplateLiteral};

use crate::IsolatedDeclarations;

impl<'a> IsolatedDeclarations<'a> {
    pub(crate) fn transform_template_to_string(
        &self,
        lit: &TemplateLiteral<'a>,
    ) -> Option<Box<'a, StringLiteral<'a>>> {
        if lit.expressions.is_empty() {
            lit.quasis.first().map(|item| {
                self.ast.alloc(self.ast.string_literal(
                    lit.span,
                    if let Some(cooked) = &item.value.cooked { cooked } else { &item.value.raw },
                ))
            })
        } else {
            None
        }
    }
}

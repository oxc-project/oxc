use oxc_allocator::Vec;
use oxc_ast::ast::*;

use crate::{array, format::Format, ir::Doc, text, Prettier};

#[allow(clippy::enum_variant_names)]
pub(super) enum TemplateLiteralPrinter<'a, 'b> {
    TemplateLiteral(&'b TemplateLiteral<'a>),
    TSTemplateLiteralType(&'b TSTemplateLiteralType<'a>),
}

impl<'a> TemplateLiteralPrinter<'a, '_> {
    fn quasis(&self) -> &[TemplateElement<'a>] {
        match self {
            Self::TemplateLiteral(template_literal) => &template_literal.quasis,
            Self::TSTemplateLiteralType(template_literal) => &template_literal.quasis,
        }
    }

    fn get_nth_expr_doc(&self, p: &mut Prettier<'a>, index: usize) -> Option<Doc<'a>> {
        match self {
            Self::TemplateLiteral(template_literal) => {
                template_literal.expressions.get(index).map(|expression| expression.format(p))
            }
            Self::TSTemplateLiteralType(template_literal) => {
                template_literal.types.get(index).map(|type_| type_.format(p))
            }
        }
    }
}

pub(super) fn print_template_literal<'a, 'b>(
    p: &mut Prettier<'a>,
    template_literal: &'b TemplateLiteralPrinter<'a, 'b>,
) -> Doc<'a> {
    let mut parts = Vec::new_in(p.allocator);
    parts.push(text!("`"));

    for (index, quais) in template_literal.quasis().iter().enumerate() {
        parts.push(quais.format(p));
        let Some(expr_doc) = template_literal.get_nth_expr_doc(p, index) else {
            break;
        };

        parts.push(text!("${"));
        parts.push(expr_doc);
        parts.push(text!("}"));
    }

    parts.push(text!("`"));

    array!(p, parts)
}

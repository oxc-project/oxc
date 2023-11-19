use oxc_ast::ast::{TemplateLiteral, *};

use crate::{doc::Doc, format::Format, ss, Prettier};

#[allow(clippy::enum_variant_names)]
pub enum TemplateLiteralPrinter<'a, 'b> {
    TemplateLiteral(&'b TemplateLiteral<'a>),
    TSTemplateLiteralType(&'b TSTemplateLiteralType<'a>),
}

impl<'a, 'b> TemplateLiteralPrinter<'a, 'b> {
    fn quasis(&self) -> &[TemplateElement] {
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

pub(super) fn print_template_literal<'a>(
    p: &mut Prettier<'a>,
    template_literal: &TemplateLiteralPrinter<'a, '_>,
) -> Doc<'a> {
    let mut parts = p.vec();
    parts.push(ss!("`"));

    for (index, quais) in template_literal.quasis().iter().enumerate() {
        parts.push(quais.format(p));
        let Some(expr_doc) = template_literal.get_nth_expr_doc(p, index) else {
            break;
        };

        parts.push(ss!("${"));
        parts.push(expr_doc);
        parts.push(ss!("}"));
    }

    parts.push(ss!("`"));

    Doc::Array(parts)
}

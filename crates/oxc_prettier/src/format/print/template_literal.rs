use oxc_allocator::Vec;
use oxc_ast::ast::*;

use crate::{
    array, format::Format, group, ir::Doc, line_suffix_boundary, softline, text, Prettier,
};

#[allow(clippy::enum_variant_names)]
pub enum TemplateLiteralLike<'a, 'b> {
    TemplateLiteral(&'b TemplateLiteral<'a>),
    TSTemplateLiteralType(&'b TSTemplateLiteralType<'a>),
}

impl<'a> TemplateLiteralLike<'a, '_> {
    fn quasis(&self) -> &[TemplateElement<'a>] {
        match self {
            Self::TemplateLiteral(template_literal) => &template_literal.quasis,
            Self::TSTemplateLiteralType(template_literal) => &template_literal.quasis,
        }
    }

    fn get_nth_expr_doc(&self, p: &mut Prettier<'a>, idx: usize) -> Option<Doc<'a>> {
        match self {
            Self::TemplateLiteral(template_literal) => {
                template_literal.expressions.get(idx).map(|expression| expression.format(p))
            }
            Self::TSTemplateLiteralType(template_literal) => {
                template_literal.types.get(idx).map(|type_| type_.format(p))
            }
        }
    }
}

pub fn print_template_literal<'a>(
    p: &mut Prettier<'a>,
    template_literal: &TemplateLiteralLike<'a, '_>,
) -> Doc<'a> {
    // TODO: Special support for Jest `.each`

    let mut parts = Vec::new_in(p.allocator);
    // parts.push(line_suffix_boundary!());
    parts.push(text!("`"));

    for (idx, quasi) in template_literal.quasis().iter().enumerate() {
        parts.push(quasi.format(p));

        let Some(expr_doc) = template_literal.get_nth_expr_doc(p, idx) else {
            break;
        };

        // TODO: Handle indent size, align!(), add_alignment_to_doc!() etc...

        parts.push(group!(
            p,
            [
                text!("${"),
                expr_doc,
                // line_suffix_boundary!(),
                text!("}"),
            ]
        ));
    }

    parts.push(text!("`"));

    array!(p, parts)
}

pub fn print_tagged_template_literal<'a>(
    p: &mut Prettier<'a>,
    tagged_template_literal: &TaggedTemplateExpression<'a>,
) -> Doc<'a> {
    let mut parts = Vec::new_in(p.allocator);

    parts.push(tagged_template_literal.tag.format(p));
    if let Some(type_parameters) = &tagged_template_literal.type_parameters {
        parts.push(type_parameters.format(p));
    }
    parts.push(line_suffix_boundary!());
    parts.push(tagged_template_literal.quasi.format(p));

    // TODO: Wrap with `label!()`
    array!(p, parts)
}

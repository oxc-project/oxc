use oxc_allocator::Vec;
use oxc_ast::ast::{ObjectAssignmentTarget, ObjectExpression, ObjectPattern};
use oxc_span::{GetSpan, Span};

use crate::{
    doc::{Doc, Group},
    group, if_break, line, softline, ss, Prettier,
};

use super::{misc, Format};

#[allow(clippy::enum_variant_names)]
pub enum ObjectLike<'a, 'b> {
    ObjectExpression(&'b ObjectExpression<'a>),
    ObjectAssignmentTarget(&'b ObjectAssignmentTarget<'a>),
    ObjectPattern(&'b ObjectPattern<'a>),
}

impl ObjectLike<'_, '_> {
    pub fn span(&self) -> Span {
        match self {
            ObjectLike::ObjectExpression(object) => object.span,
            ObjectLike::ObjectAssignmentTarget(object) => object.span,
            ObjectLike::ObjectPattern(object) => object.span,
        }
    }
}

pub(super) fn print_object_properties<'a, F: Format<'a> + GetSpan>(
    p: &mut Prettier<'a>,
    object: &ObjectLike<'a, '_>,
    properties: &Vec<'a, F>,
) -> Doc<'a> {
    let left_brace = ss!("{");
    let right_brace = ss!("}");

    let content = if properties.is_empty() {
        group![p, left_brace, softline!(), right_brace]
    } else {
        let mut parts = p.vec();
        parts.push(ss!("{"));

        let mut indent_parts = p.vec();
        indent_parts.push(if p.options.bracket_spacing { line!() } else { softline!() });
        for (i, prop) in properties.iter().enumerate() {
            indent_parts.push(prop.format(p));
            if i < properties.len() - 1 {
                indent_parts.push(Doc::Str(","));
                indent_parts.push(Doc::Line);
            }
        }

        parts.push(Doc::Indent(indent_parts));
        parts.push(if_break!(p, ","));

        if p.options.bracket_spacing {
            parts.push(Doc::Line);
        } else {
            parts.push(Doc::Softline);
        }

        parts.push(ss!("}"));
        let should_break =
            misc::has_new_line_in_range(p.source_text, object.span().start, object.span().end);
        Doc::Group(Group::new(parts, should_break))
    };

    content
}

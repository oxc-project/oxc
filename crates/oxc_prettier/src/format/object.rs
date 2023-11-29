use oxc_ast::ast::{ObjectAssignmentTarget, ObjectExpression, ObjectPattern};
use oxc_span::Span;

use crate::{
    doc::{Doc, DocBuilder, Group},
    group, if_break, line, softline, ss, Prettier,
};

use super::{misc, Format};

#[derive(Debug, Clone, Copy)]
pub enum ObjectLike<'a, 'b> {
    Expression(&'b ObjectExpression<'a>),
    AssignmentTarget(&'b ObjectAssignmentTarget<'a>),
    Pattern(&'b ObjectPattern<'a>),
}

impl<'a, 'b> ObjectLike<'a, 'b> {
    fn len(&self) -> usize {
        match self {
            ObjectLike::Expression(object) => object.properties.len(),
            ObjectLike::AssignmentTarget(object) => object.properties.len(),
            ObjectLike::Pattern(object) => object.properties.len(),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            ObjectLike::Expression(object) => object.properties.is_empty(),
            ObjectLike::AssignmentTarget(object) => object.is_empty(),
            ObjectLike::Pattern(object) => object.is_empty(),
        }
    }

    fn is_object_pattern(&self) -> bool {
        matches!(self, ObjectLike::Pattern(_))
    }

    fn span(&self) -> Span {
        match self {
            ObjectLike::Expression(object) => object.span,
            ObjectLike::AssignmentTarget(object) => object.span,
            ObjectLike::Pattern(object) => object.span,
        }
    }

    fn iter(&'b self, p: &'b mut Prettier<'a>) -> Box<dyn Iterator<Item = Doc<'a>> + 'b> {
        match self {
            ObjectLike::Expression(object) => {
                Box::new(object.properties.iter().map(|prop| prop.format(p)))
            }
            ObjectLike::AssignmentTarget(object) => {
                Box::new(object.properties.iter().map(|prop| prop.format(p)))
            }
            ObjectLike::Pattern(object) => {
                Box::new(object.properties.iter().map(|prop| prop.format(p)))
            }
        }
    }
}

pub(super) fn print_object_properties<'a>(
    p: &mut Prettier<'a>,
    object: ObjectLike<'a, '_>,
) -> Doc<'a> {
    let left_brace = ss!("{");
    let right_brace = ss!("}");

    let content = if object.is_empty() {
        group![p, left_brace, softline!(), right_brace]
    } else {
        let mut parts = p.vec();
        parts.push(ss!("{"));
        parts.push(Doc::Indent({
            let len = object.len();
            let mut indent_parts = p.vec();
            indent_parts.push(if p.options.bracket_spacing { line!() } else { softline!() });
            for (i, doc) in object.iter(p).enumerate() {
                indent_parts.push(doc);
                if i < len - 1 {
                    indent_parts.push(ss!(","));
                    indent_parts.push(line!());
                }
            }
            match object {
                ObjectLike::Expression(object) => {}
                ObjectLike::AssignmentTarget(object) => {
                    if let Some(rest) = &object.rest {
                        indent_parts.push(ss!("..."));
                        indent_parts.push(rest.format(p));
                    }
                }
                ObjectLike::Pattern(object) => {
                    if let Some(rest) = &object.rest {
                        indent_parts.push(ss!(","));
                        indent_parts.push(line!());
                        indent_parts.push(rest.format(p));
                    }
                }
            }
            indent_parts
        }));
        if p.should_print_es5_comma() {
            parts.push(if_break!(p, ",", "", None));
        }
        parts.push(if p.options.bracket_spacing { line!() } else { softline!() });
        parts.push(ss!("}"));

        if object.is_object_pattern() {
            Doc::Array(parts)
        } else {
            let should_break =
                misc::has_new_line_in_range(p.source_text, object.span().start, object.span().end);
            Doc::Group(Group::new(parts, should_break))
        }
    };

    content
}

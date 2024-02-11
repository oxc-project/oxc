use oxc_ast::{
    ast::{ObjectAssignmentTarget, ObjectExpression, ObjectPattern},
    AstKind,
};
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
            Self::Expression(expr) => expr.properties.len(),
            Self::AssignmentTarget(target) => target.properties.len(),
            Self::Pattern(object) => object.properties.len(),
        }
    }

    fn has_rest(&self) -> bool {
        match self {
            Self::Expression(expr) => false,
            Self::AssignmentTarget(target) => target.rest.is_some(),
            Self::Pattern(object) => object.rest.is_some(),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Self::Expression(object) => object.properties.is_empty(),
            Self::AssignmentTarget(object) => object.is_empty(),
            Self::Pattern(object) => object.is_empty(),
        }
    }

    fn is_object_pattern(&self) -> bool {
        matches!(self, Self::Pattern(_))
    }

    fn span(&self) -> Span {
        match self {
            Self::Expression(object) => object.span,
            Self::AssignmentTarget(object) => object.span,
            Self::Pattern(object) => object.span,
        }
    }

    fn iter(&'b self, p: &'b mut Prettier<'a>) -> Box<dyn Iterator<Item = Doc<'a>> + 'b> {
        match self {
            Self::Expression(object) => {
                Box::new(object.properties.iter().map(|prop| prop.format(p)))
            }
            Self::AssignmentTarget(object) => {
                Box::new(object.properties.iter().map(|prop| prop.format(p)))
            }
            Self::Pattern(object) => Box::new(object.properties.iter().map(|prop| prop.format(p))),
        }
    }
}

pub(super) fn print_object_properties<'a>(
    p: &mut Prettier<'a>,
    object: ObjectLike<'a, '_>,
) -> Doc<'a> {
    let left_brace = ss!("{");
    let right_brace = ss!("}");

    let should_break = false;

    let content = if object.is_empty() {
        group![p, left_brace, softline!(), right_brace]
    } else {
        let mut parts = p.vec();
        parts.push(ss!("{"));
        parts.push(Doc::Indent({
            let len = object.len();
            let has_rest = object.has_rest();
            let mut indent_parts = p.vec();
            indent_parts.push(if p.options.bracket_spacing { line!() } else { softline!() });
            for (i, doc) in object.iter(p).enumerate() {
                indent_parts.push(doc);
                if i == len - 1 && !has_rest {
                    break;
                }
                indent_parts.push(ss!(","));
                indent_parts.push(line!());
            }
            match object {
                ObjectLike::Expression(_) => {}
                ObjectLike::AssignmentTarget(target) => {
                    if let Some(rest) = &target.rest {
                        indent_parts.push(ss!("..."));
                        indent_parts.push(rest.format(p));
                    }
                }
                ObjectLike::Pattern(object) => {
                    if let Some(rest) = &object.rest {
                        indent_parts.push(rest.format(p));
                    }
                }
            }
            indent_parts
        }));
        if p.should_print_es5_comma()
            && match object {
                ObjectLike::Expression(expr) => true,
                ObjectLike::AssignmentTarget(target) => true,
                ObjectLike::Pattern(pattern) => pattern.rest.is_none(),
            }
        {
            parts.push(if_break!(p, ",", "", None));
        }
        parts.push(if p.options.bracket_spacing { line!() } else { softline!() });
        parts.push(ss!("}"));

        let parent_kind = p.parent_kind();
        if (object.is_object_pattern() && should_hug_the_only_parameter(p, parent_kind))
            || (!should_break
                && object.is_object_pattern()
                && matches!(
                    parent_kind,
                    AstKind::AssignmentExpression(_) | AstKind::VariableDeclarator(_)
                ))
        {
            Doc::Array(parts)
        } else {
            let should_break =
                misc::has_new_line_in_range(p.source_text, object.span().start, object.span().end);
            Doc::Group(Group::new(parts).with_break(should_break))
        }
    };

    content
}

fn should_hug_the_only_parameter(p: &mut Prettier<'_>, kind: AstKind<'_>) -> bool {
    match kind {
        AstKind::FormalParameters(params) => {
            super::function_parameters::should_hug_the_only_function_parameter(p, params)
        }
        _ => false,
    }
}

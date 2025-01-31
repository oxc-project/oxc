use oxc_allocator::Vec;
use oxc_ast::{ast::*, AstKind};
use oxc_span::Span;

use crate::{
    array,
    format::print::{function_parameters, misc},
    group, if_break, indent,
    ir::Doc,
    line, softline, text, Format, Prettier,
};

pub enum ObjectLike<'a, 'b> {
    ObjectExpression(&'b ObjectExpression<'a>),
    ObjectAssignmentTarget(&'b ObjectAssignmentTarget<'a>),
    ObjectPattern(&'b ObjectPattern<'a>),
    TSTypeLiteral(&'b TSTypeLiteral<'a>),
    // TODO: TSInterfaceBody
}

impl<'a, 'b> ObjectLike<'a, 'b> {
    fn len(&self) -> usize {
        match self {
            Self::ObjectExpression(expr) => expr.properties.len(),
            Self::ObjectAssignmentTarget(target) => target.properties.len(),
            Self::ObjectPattern(object) => object.properties.len(),
            Self::TSTypeLiteral(literal) => literal.members.len(),
        }
    }

    fn has_rest(&self) -> bool {
        match self {
            Self::ObjectExpression(expr) => false,
            Self::ObjectAssignmentTarget(target) => target.rest.is_some(),
            Self::ObjectPattern(object) => object.rest.is_some(),
            Self::TSTypeLiteral(_) => false,
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Self::ObjectExpression(object) => object.properties.is_empty(),
            Self::ObjectAssignmentTarget(object) => object.is_empty(),
            Self::ObjectPattern(object) => object.is_empty(),
            Self::TSTypeLiteral(literal) => literal.members.is_empty(),
        }
    }

    fn is_object_pattern(&self) -> bool {
        matches!(self, Self::ObjectPattern(_))
    }

    fn span(&self) -> Span {
        match self {
            Self::ObjectExpression(object) => object.span,
            Self::ObjectAssignmentTarget(object) => object.span,
            Self::ObjectPattern(object) => object.span,
            Self::TSTypeLiteral(literal) => literal.span,
        }
    }

    fn iter(&'b self, p: &'b mut Prettier<'a>) -> Box<dyn Iterator<Item = Doc<'a>> + 'b> {
        match self {
            Self::ObjectExpression(object) => {
                Box::new(object.properties.iter().map(|prop| prop.format(p)))
            }
            Self::ObjectAssignmentTarget(object) => {
                Box::new(object.properties.iter().map(|prop| prop.format(p)))
            }
            Self::ObjectPattern(object) => {
                Box::new(object.properties.iter().map(|prop| prop.format(p)))
            }
            Self::TSTypeLiteral(literal) => {
                Box::new(literal.members.iter().map(|member| member.format(p)))
            }
        }
    }

    fn member_separator(&self, p: &'b Prettier<'a>) -> &'static str {
        match self {
            Self::TSTypeLiteral(_) => {
                if p.semi().is_some() {
                    ";"
                } else {
                    ""
                }
            }
            _ => ",",
        }
    }
}

pub fn print_object<'a>(p: &mut Prettier<'a>, object: &ObjectLike<'a, '_>) -> Doc<'a> {
    let should_break = false;
    let member_separator = object.member_separator(p);

    let content = if object.is_empty() {
        group!(p, [text!("{"), softline!(), text!("}")])
    } else {
        let mut parts = Vec::new_in(p.allocator);
        parts.push(text!("{"));

        let indent_parts = {
            let len = object.len();
            let has_rest = object.has_rest();
            let mut indent_parts = Vec::new_in(p.allocator);

            indent_parts.push(if p.options.bracket_spacing { line!() } else { softline!() });

            let object_docs = object.iter(p).collect::<std::vec::Vec<_>>();
            for (i, doc) in object_docs.into_iter().enumerate() {
                indent_parts.push(doc);
                if i == len - 1 && !has_rest {
                    break;
                }

                indent_parts.push(text!(member_separator));
                indent_parts.push(line!());
            }

            match object {
                ObjectLike::ObjectExpression(_) | ObjectLike::TSTypeLiteral(_) => {}
                ObjectLike::ObjectAssignmentTarget(target) => {
                    if let Some(rest) = &target.rest {
                        indent_parts.push(rest.format(p));
                    }
                }
                ObjectLike::ObjectPattern(object) => {
                    if let Some(rest) = &object.rest {
                        indent_parts.push(rest.format(p));
                    }
                }
            }
            indent_parts
        };
        parts.push(indent!(p, indent_parts));
        if p.should_print_es5_comma()
            && match object {
                ObjectLike::ObjectPattern(pattern) => pattern.rest.is_none(),
                _ => true,
            }
        {
            parts.push(if_break!(p, text!(member_separator)));
        }
        parts.push(if p.options.bracket_spacing { line!() } else { softline!() });
        parts.push(text!("}"));

        if matches!(p.current_kind(), AstKind::Program(_)) {
            let should_break =
                misc::has_new_line_in_range(p.source_text, object.span().start, object.span().end);
            return group!(p, parts, should_break, None);
        }

        let parent_kind = p.parent_kind();
        if (object.is_object_pattern() && should_hug_the_only_parameter(p, parent_kind))
            || (!should_break
                && object.is_object_pattern()
                && matches!(
                    parent_kind,
                    AstKind::AssignmentExpression(_) | AstKind::VariableDeclarator(_)
                ))
        {
            array!(p, parts)
        } else {
            let should_break =
                misc::has_new_line_in_range(p.source_text, object.span().start, object.span().end);
            group!(p, parts, should_break, None)
        }
    };

    content
}

fn should_hug_the_only_parameter(p: &mut Prettier<'_>, kind: AstKind<'_>) -> bool {
    match kind {
        AstKind::FormalParameters(params) => {
            function_parameters::should_hug_the_only_function_parameter(p, params)
        }
        _ => false,
    }
}

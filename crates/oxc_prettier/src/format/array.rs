use oxc_ast::ast::*;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::UnaryOperator;

use super::Format;
use crate::{
    comments::{CommentFlags, DanglingCommentsPrintOptions},
    group,
    ir::{Doc, DocBuilder, Group},
    p_vec, Prettier,
};

#[allow(clippy::enum_variant_names)]
pub enum Array<'a, 'b> {
    ArrayExpression(&'b ArrayExpression<'a>),
    TSTupleType(&'b TSTupleType<'a>),
    ArrayPattern(&'b ArrayPattern<'a>),
    ArrayAssignmentTarget(&'b ArrayAssignmentTarget<'a>),
}

impl Array<'_, '_> {
    fn len(&self) -> usize {
        match self {
            Self::ArrayExpression(array) => array.elements.len(),
            Self::TSTupleType(tuple) => tuple.element_types.len(),
            Self::ArrayPattern(array) => array.elements.len(),
            Self::ArrayAssignmentTarget(array) => array.elements.len(),
        }
    }

    fn span(&self) -> Span {
        match self {
            Self::ArrayExpression(array) => array.span,
            Self::TSTupleType(tuple) => tuple.span,
            Self::ArrayPattern(array) => array.span,
            Self::ArrayAssignmentTarget(array) => array.span,
        }
    }

    fn is_concisely_printed(&self) -> bool {
        match self {
            Self::ArrayExpression(array) => {
                if array.elements.len() <= 1 {
                    return false;
                }

                array.elements.iter().all(|element| match element {
                    ArrayExpressionElement::NumericLiteral(_) => true,
                    ArrayExpressionElement::UnaryExpression(unary_expr) => {
                        matches!(
                            unary_expr.operator,
                            UnaryOperator::UnaryPlus | UnaryOperator::UnaryNegation
                        ) && matches!(unary_expr.argument, Expression::NumericLiteral(_))
                    }
                    _ => false,
                })
            }
            Self::ArrayPattern(_) | Self::ArrayAssignmentTarget(_) | Self::TSTupleType(_) => false,
        }
    }
}

pub fn print_array<'a>(p: &mut Prettier<'a>, arr: &Array<'a, '_>) -> Doc<'a> {
    if arr.len() == 0 {
        return print_empty_array_elements(p, arr);
    }

    let (needs_forced_trailing_comma, can_have_trailing_comma) =
        if let Array::ArrayExpression(arr) = arr {
            arr.elements.last().map_or((false, false), |last| {
                (
                    matches!(last, ArrayExpressionElement::Elision(_)),
                    !matches!(last, ArrayExpressionElement::SpreadElement(_)),
                )
            })
        } else {
            (false, false)
        };

    let id = p.next_id();
    let should_use_concise_formatting = arr.is_concisely_printed();

    let trailing_comma_fn = |p: &Prettier<'a>| {
        if !can_have_trailing_comma {
            p.text("")
        } else if needs_forced_trailing_comma {
            p.text(",")
        } else if should_use_concise_formatting {
            p.if_break(p.text(","), p.text(""), Some(id))
        } else {
            p.if_break(p.text(","), p.text(""), None)
        }
    };

    let mut parts = p.vec();

    parts.push(Doc::Group(
        Group::new({
            let mut group = p.vec();
            group.push(p.text("["));

            let indent_parts = {
                let mut indent_parts = p.vec();
                indent_parts.push(p.softline());

                indent_parts.push(if should_use_concise_formatting {
                    print_array_elements_concisely(p, arr, trailing_comma_fn)
                } else {
                    let trailing_comma = trailing_comma_fn(p);
                    let elements = print_array_elements(p, arr);
                    p.array(p_vec!(p, elements, trailing_comma))
                });
                if let Some(dangling_comments) = p.print_dangling_comments(arr.span(), None) {
                    indent_parts.push(dangling_comments);
                };
                indent_parts
            };

            group.push(p.indent(indent_parts));
            group.push(p.softline());
            group.push(p.text("]"));
            group
        })
        .with_break(should_break(arr))
        .with_id(id),
    ));

    p.array(parts)
}

fn print_empty_array_elements<'a>(p: &mut Prettier<'a>, array: &Array<'a, '_>) -> Doc<'a> {
    let dangling_options = DanglingCommentsPrintOptions::default().with_ident(true);
    p.print_dangling_comments(array.span(), Some(&dangling_options)).map_or_else(
        || p.text("[]"),
        |dangling_comments| group![p, p.text("["), dangling_comments, p.softline(), p.text("]")],
    )
}

fn print_array_elements<'a>(p: &mut Prettier<'a>, arr: &Array<'a, '_>) -> Doc<'a> {
    let mut parts = p.vec();
    match arr {
        Array::ArrayExpression(array) => {
            for (i, element) in array.elements.iter().enumerate() {
                parts.push(element.format(p));
                let is_last = i == array.elements.len() - 1;
                if !is_last {
                    parts.push(p.text(","));
                    parts.push(p.line());
                    if !element.is_elision() && is_line_after_element_empty(p, element.span().end) {
                        parts.push(p.softline());
                    }
                }
            }
        }
        Array::TSTupleType(tuple) => {
            for (i, element) in tuple.element_types.iter().enumerate() {
                if i > 0 && i < tuple.element_types.len() {
                    parts.push(p.text(","));
                    parts.push(p.line());
                }

                parts.push(element.format(p));
            }
        }
        Array::ArrayPattern(array_pat) => {
            let len = array_pat.elements.len();
            let has_rest = array_pat.rest.is_some();
            for (i, element) in array_pat.elements.iter().enumerate() {
                if let Some(binding_pat) = element {
                    parts.push(group!(p, binding_pat.format(p)));
                }
                if i == len - 1 && !has_rest {
                    break;
                }
                parts.push(p.text(","));
                parts.push(p.line());
            }
            if let Some(rest) = &array_pat.rest {
                parts.push(group!(p, rest.format(p)));
            }
        }
        Array::ArrayAssignmentTarget(array_pat) => {
            for (i, element) in array_pat.elements.iter().enumerate() {
                if i > 0 && i < array_pat.elements.len() {
                    parts.push(p.text(","));
                    parts.push(p.line());
                }

                if let Some(binding_pat) = element {
                    parts.push(binding_pat.format(p));
                }
            }

            if let Some(rest) = &array_pat.rest {
                parts.push(p.text(","));
                parts.push(p.line());
                parts.push(rest.format(p));
            }
        }
    }

    p.array(parts)
}

fn print_array_elements_concisely<'a, F>(
    p: &mut Prettier<'a>,
    arr: &Array<'a, '_>,
    trailing_comma_fn: F,
) -> Doc<'a>
where
    F: Fn(&Prettier<'a>) -> Doc<'a>,
{
    let mut parts = p.vec();
    match arr {
        Array::ArrayExpression(arr) => {
            for (i, element) in arr.elements.iter().enumerate() {
                let is_last = i == arr.elements.len() - 1;
                let element_doc = element.format(p);
                let part = if is_last {
                    p.array(p_vec!(p, element_doc, trailing_comma_fn(p)))
                } else {
                    p.array(p_vec!(p, element_doc, p.text(",")))
                };
                parts.push(part);

                if !is_last {
                    if is_line_after_element_empty(p, element.span().end) {
                        let mut space_parts = p.vec();
                        space_parts.extend(p.hardline());
                        space_parts.extend(p.hardline());
                        parts.push(p.array(space_parts));
                    } else if arr.elements.get(i + 1).is_some_and(|next| {
                        p.has_comment(next.span(), CommentFlags::Leading | CommentFlags::Line)
                    }) {
                        let mut space_parts = p.vec();
                        space_parts.extend(p.hardline());
                        parts.push(p.array(space_parts));
                    } else {
                        parts.push(p.line());
                    }
                }
            }
        }
        _ => {
            // TODO: implement
            let elements = print_array_elements(p, arr);
            p.array(p_vec!(p, elements, trailing_comma_fn(p)));
        }
    }

    p.fill(parts)
}

fn should_break(array: &Array) -> bool {
    if array.len() <= 1 {
        return false;
    }

    match array {
        Array::ArrayExpression(array) => {
            array.elements.iter().enumerate().all(|(index, element)| {
                if let Some(next_element) = array.elements.get(index + 1) {
                    let all_array_or_object = matches!(
                        (element, next_element),
                        (
                            ArrayExpressionElement::ArrayExpression(_),
                            ArrayExpressionElement::ArrayExpression(_)
                        ) | (
                            ArrayExpressionElement::ObjectExpression(_),
                            ArrayExpressionElement::ObjectExpression(_)
                        )
                    );
                    if !all_array_or_object {
                        return false;
                    }
                }

                match element {
                    ArrayExpressionElement::ArrayExpression(array) => array.elements.len() > 1,
                    ArrayExpressionElement::ObjectExpression(object) => object.properties.len() > 1,
                    _ => false,
                }
            })
        }
        Array::TSTupleType(tuple) => {
            tuple.element_types.iter().enumerate().all(|(index, element)| {
                let TSTupleElement::TSTupleType(array) = element else {
                    return false;
                };

                if let Some(next_element @ match_ts_type!(TSTupleElement)) =
                    tuple.element_types.get(index + 1)
                {
                    if !matches!(next_element, TSTupleElement::TSTupleType(_)) {
                        return false;
                    }
                }

                array.element_types.len() > 1
            })
        }
        Array::ArrayPattern(array) => false,
        Array::ArrayAssignmentTarget(array) => false,
    }
}

fn skip_comment(p: &Prettier<'_>, idx: u32) -> Option<u32> {
    p.skip_inline_comment(p.skip_trailing_comment(Some(idx)))
}

#[allow(clippy::cast_possible_truncation)]
fn skip_to_comma(p: &Prettier<'_>, current_idx: Option<u32>) -> Option<u32> {
    let current_idx = current_idx?;
    match p.source_text[current_idx as usize..].chars().next() {
        Some(',') => Some(current_idx),
        Some(c) => skip_to_comma(p, skip_comment(p, current_idx + c.len_utf8() as u32)),
        None => None,
    }
}

fn is_line_after_element_empty(p: &Prettier<'_>, index: u32) -> bool {
    let Some(start_index) = skip_to_comma(p, Some(index)) else { return false };
    p.is_next_line_empty_after_index(start_index)
}

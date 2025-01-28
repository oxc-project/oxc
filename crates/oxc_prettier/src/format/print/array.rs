use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::UnaryOperator;

use crate::{
    array, fill, group, hardline, if_break, indent, ir::Doc, line, softline, text, Format, Prettier,
};

pub enum ArrayLike<'a, 'b> {
    ArrayExpression(&'b ArrayExpression<'a>),
    TSTupleType(&'b TSTupleType<'a>),
    ArrayPattern(&'b ArrayPattern<'a>),
    ArrayAssignmentTarget(&'b ArrayAssignmentTarget<'a>),
}

impl ArrayLike<'_, '_> {
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

pub fn print_array<'a>(p: &mut Prettier<'a>, arr: &ArrayLike<'a, '_>) -> Doc<'a> {
    if arr.len() == 0 {
        return text!("[]");
    }

    let (needs_forced_trailing_comma, can_have_trailing_comma) =
        if let ArrayLike::ArrayExpression(arr) = arr {
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
            text!("")
        } else if needs_forced_trailing_comma {
            text!(",")
        } else if should_use_concise_formatting {
            if_break!(p, text!(","), text!(""), Some(id))
        } else {
            if_break!(p, text!(","))
        }
    };

    let mut parts = Vec::new_in(p.allocator);

    let group = {
        let mut group = Vec::new_in(p.allocator);
        group.push(text!("["));

        let indent_parts = {
            let mut indent_parts = Vec::new_in(p.allocator);
            indent_parts.push(softline!());

            indent_parts.push(if should_use_concise_formatting {
                print_array_elements_concisely(p, arr, trailing_comma_fn)
            } else {
                let trailing_comma = trailing_comma_fn(p);
                let elements = print_array_elements(p, arr);
                array!(p, [elements, trailing_comma])
            });
            indent_parts
        };

        group.push(indent!(p, indent_parts));
        group.push(softline!());
        group.push(text!("]"));

        group
    };
    parts.push(group!(p, group, should_break(arr), Some(id)));

    array!(p, parts)
}

pub fn is_concisely_printed_array(arr: &Expression) -> bool {
    match arr {
        Expression::ArrayExpression(array) => {
            ArrayLike::ArrayExpression(array).is_concisely_printed()
        }
        _ => false,
    }
}

fn print_array_elements<'a>(p: &mut Prettier<'a>, arr: &ArrayLike<'a, '_>) -> Doc<'a> {
    let mut parts = Vec::new_in(p.allocator);
    match arr {
        ArrayLike::ArrayExpression(array) => {
            for (i, element) in array.elements.iter().enumerate() {
                parts.push(element.format(p));
                let is_last = i == array.elements.len() - 1;
                if !is_last {
                    parts.push(text!(","));
                    parts.push(line!());
                    if !element.is_elision() && is_line_after_element_empty(p, element.span().end) {
                        parts.push(softline!());
                    }
                }
            }
        }
        ArrayLike::TSTupleType(tuple) => {
            for (i, element) in tuple.element_types.iter().enumerate() {
                if i > 0 && i < tuple.element_types.len() {
                    parts.push(text!(","));
                    parts.push(line!());
                }

                parts.push(element.format(p));
            }
        }
        ArrayLike::ArrayPattern(array_pat) => {
            let len = array_pat.elements.len();
            let has_rest = array_pat.rest.is_some();
            for (i, element) in array_pat.elements.iter().enumerate() {
                if let Some(binding_pat) = element {
                    let binding_pat_doc = binding_pat.format(p);
                    parts.push(group!(p, [binding_pat_doc]));
                }
                if i == len - 1 && !has_rest {
                    break;
                }
                parts.push(text!(","));
                parts.push(line!());
            }
            if let Some(rest) = &array_pat.rest {
                let rest_doc = rest.format(p);
                parts.push(group!(p, [rest_doc]));
            }
        }
        ArrayLike::ArrayAssignmentTarget(array_pat) => {
            for (i, element) in array_pat.elements.iter().enumerate() {
                if i > 0 && i < array_pat.elements.len() {
                    parts.push(text!(","));
                    parts.push(line!());
                }

                if let Some(binding_pat) = element {
                    parts.push(binding_pat.format(p));
                }
            }

            if let Some(rest) = &array_pat.rest {
                parts.push(text!(","));
                parts.push(line!());
                parts.push(rest.format(p));
            }
        }
    }

    array!(p, parts)
}

fn print_array_elements_concisely<'a, F>(
    p: &mut Prettier<'a>,
    arr: &ArrayLike<'a, '_>,
    trailing_comma_fn: F,
) -> Doc<'a>
where
    F: Fn(&Prettier<'a>) -> Doc<'a>,
{
    let mut parts = Vec::new_in(p.allocator);
    if let ArrayLike::ArrayExpression(arr) = arr {
        for (i, element) in arr.elements.iter().enumerate() {
            let is_last = i == arr.elements.len() - 1;
            let element_doc = element.format(p);
            let part = if is_last {
                array!(p, [element_doc, trailing_comma_fn(p)])
            } else {
                array!(p, [element_doc, text!(",")])
            };
            parts.push(part);

            if !is_last {
                if is_line_after_element_empty(p, element.span().end) {
                    parts.push(array!(p, [hardline!(p), hardline!(p)]));
                } else {
                    parts.push(line!());
                }
            }
        }
    } else {
        // TODO: implement
        let elements = print_array_elements(p, arr);
        array!(p, [elements, trailing_comma_fn(p)]);
    }

    fill!(p, parts)
}

fn should_break(array: &ArrayLike) -> bool {
    if array.len() <= 1 {
        return false;
    }

    match array {
        ArrayLike::ArrayExpression(array) => {
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
        ArrayLike::TSTupleType(tuple) => {
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
        ArrayLike::ArrayPattern(array) => false,
        ArrayLike::ArrayAssignmentTarget(array) => false,
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

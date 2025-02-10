use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::UnaryOperator;

use crate::{
    array, fill, group, hardline, if_break, indent, ir::Doc, line, softline, text, Format, Prettier,
};

pub enum ArrayLike<'a, 'b> {
    ArrayExpression(&'b ArrayExpression<'a>),
    ArrayPattern(&'b ArrayPattern<'a>),
    ArrayAssignmentTarget(&'b ArrayAssignmentTarget<'a>),
    TSTupleType(&'b TSTupleType<'a>),
}

impl<'a> ArrayLike<'a, '_> {
    /// This includes rest element for `ArrayPattern` and `ArrayAssignmentTarget`
    fn total_len(&self) -> usize {
        match self {
            Self::ArrayExpression(array) => array.elements.len(),
            Self::ArrayPattern(array) => array.elements.len() + usize::from(array.rest.is_some()),
            Self::ArrayAssignmentTarget(array) => {
                array.elements.len() + usize::from(array.rest.is_some())
            }
            Self::TSTupleType(tuple) => tuple.element_types.len(),
        }
    }

    fn is_concisely_printed(&self) -> bool {
        let Self::ArrayExpression(array_expr) = self else {
            return false;
        };
        if array_expr.elements.len() <= 1 {
            return false;
        }
        array_expr.elements.iter().all(
            |el| match el {
                ArrayExpressionElement::NumericLiteral(_) => true,
                ArrayExpressionElement::UnaryExpression(unary_expr) => {
                    matches!(
                        unary_expr.operator,
                        UnaryOperator::UnaryPlus | UnaryOperator::UnaryNegation
                    ) && matches!(unary_expr.argument, Expression::NumericLiteral(_))
                    // TODO: && !hasComment(el.argument)
                }
                _ => false,
            }, // TODO: && !hasComment(...),
        )
    }

    fn should_break(&self) -> bool {
        // TODO: if (options.__inJestEach) return false;
        if self.total_len() < 2 {
            return false;
        }
        match self {
            ArrayLike::ArrayExpression(arr_expr) => {
                // Check all elements are the same type and have more than 1 element
                arr_expr.elements.iter().enumerate().all(|(idx, el)| {
                    if let Some(next_el) = arr_expr.elements.get(idx + 1) {
                        let all_array_or_object = matches!(
                            (el, next_el),
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

                    match el {
                        ArrayExpressionElement::ArrayExpression(arr) => arr.elements.len() > 1,
                        ArrayExpressionElement::ObjectExpression(obj) => obj.properties.len() > 1,
                        _ => false,
                    }
                })
            }
            // In Prettier, other branches are also performed the same check.
            // But it seems that they always return `false`.
            _ => false,
        }
    }

    // There are 4 branches and may look like similar, differences are:
    // - `ArrayExpression`: Handles elision and `is_line_after_element_empty()` check
    // - `ArrayPattern`: Handles rest element and each element is optional
    // - `ArrayAssignmentTarget`: The same as `ArrayPattern`
    // - `TSTupleType`: The same as `ArrayExpression` but without elision
    //
    // In Prettier, these are handled in a single function.
    // `is_line_after_element_empty()` check is performed for all branches,
    // but only `ArrayExpression` seems to return `true`.
    fn print_array_elements(&self, p: &mut Prettier<'a>) -> Doc<'a> {
        match self {
            ArrayLike::ArrayExpression(arr_expr) => {
                let mut parts = Vec::new_in(p.allocator);

                let len = arr_expr.elements.len();
                for (idx, el) in arr_expr.elements.iter().enumerate() {
                    parts.push(group!(p, [el.format(p)]));

                    if idx != len - 1 {
                        parts.push(text!(","));
                        parts.push(line!());

                        if !el.is_elision() && is_line_after_element_empty(p, el.span().end) {
                            parts.push(softline!());
                        }
                    }
                }

                array!(p, parts)
            }
            ArrayLike::ArrayPattern(arr_pat) => {
                let mut parts = Vec::new_in(p.allocator);

                let len = arr_pat.elements.len();
                let has_rest = arr_pat.rest.is_some();
                for (idx, el) in arr_pat.elements.iter().enumerate() {
                    if let Some(binding_pat) = el {
                        parts.push(group!(p, [binding_pat.format(p)]));
                    }

                    if idx != len - 1 || has_rest {
                        parts.push(text!(","));
                        parts.push(line!());
                    }
                }
                if let Some(rest) = &arr_pat.rest {
                    parts.push(group!(p, [rest.format(p)]));
                }

                array!(p, parts)
            }
            ArrayLike::ArrayAssignmentTarget(arr_pat) => {
                let mut parts = Vec::new_in(p.allocator);

                let len = arr_pat.elements.len();
                let has_rest = arr_pat.rest.is_some();
                for (idx, el) in arr_pat.elements.iter().enumerate() {
                    if let Some(binding_pat) = el {
                        parts.push(group!(p, [binding_pat.format(p)]));
                    }

                    if idx != len - 1 || has_rest {
                        parts.push(text!(","));
                        parts.push(line!());
                    }
                }
                if let Some(rest) = &arr_pat.rest {
                    parts.push(group!(p, [rest.format(p)]));
                }

                array!(p, parts)
            }
            ArrayLike::TSTupleType(tuple) => {
                let mut parts = Vec::new_in(p.allocator);

                let len = tuple.element_types.len();
                for (idx, el) in tuple.element_types.iter().enumerate() {
                    parts.push(group!(p, [el.format(p)]));

                    if idx != len - 1 {
                        parts.push(text!(","));
                        parts.push(line!());
                    }
                }

                array!(p, parts)
            }
        }
    }

    fn print_array_elements_concisely<F>(
        &self,
        p: &mut Prettier<'a>,
        trailing_comma_fn: F,
    ) -> Doc<'a>
    where
        F: Fn(&Prettier<'a>) -> Doc<'a>,
    {
        let ArrayLike::ArrayExpression(arr_expr) = self else { unreachable!() };

        let mut parts = Vec::new_in(p.allocator);

        let len = arr_expr.elements.len();
        for (idx, el) in arr_expr.elements.iter().enumerate() {
            let is_last = idx == len - 1;

            parts.push(array!(
                p,
                [el.format(p), if is_last { trailing_comma_fn(p) } else { text!(",") }]
            ));

            if !is_last {
                if is_line_after_element_empty(p, el.span().end) {
                    parts.push(array!(p, [hardline!(p), hardline!(p)]));
                } else {
                    // TODO: hasComment(next_el) ? hardline : line
                    parts.push(line!());
                }
            }
        }

        fill!(p, parts)
    }
}

// ---

pub fn print_array<'a>(p: &mut Prettier<'a>, arr: &ArrayLike<'a, '_>) -> Doc<'a> {
    if arr.total_len() == 0 {
        // TODO: Comments
        // group!(p, [text!("["), dangling_comment, softline!(), text!("]")])
        return text!("[]");
    }

    let (needs_forced_trailing_comma, can_have_trailing_comma) = match arr {
        ArrayLike::ArrayExpression(arr) => arr.elements.last().map_or((false, false), |last| {
            (
                matches!(last, ArrayExpressionElement::Elision(_)),
                !matches!(last, ArrayExpressionElement::SpreadElement(_)),
            )
        }),
        _ => (false, false),
    };
    let group_id = p.next_id();
    let should_use_concise_formatting = arr.is_concisely_printed();

    let trailing_comma_fn = |p: &Prettier<'a>| {
        if !can_have_trailing_comma {
            return text!("");
        }
        if needs_forced_trailing_comma {
            return text!(",");
        }
        if !p.should_print_es5_comma() {
            return text!("");
        }
        if should_use_concise_formatting {
            return if_break!(p, text!(","), text!(""), Some(group_id));
        }
        if_break!(p, text!(","))
    };

    let elements_doc = indent!(
        p,
        [
            softline!(),
            if should_use_concise_formatting {
                arr.print_array_elements_concisely(p, trailing_comma_fn)
            } else {
                array!(p, [arr.print_array_elements(p), trailing_comma_fn(p)])
            },
            // TODO: Dangling comments
        ]
    );

    group!(
        p,
        [text!("["), elements_doc, softline!(), text!("]")],
        arr.should_break(),
        Some(group_id)
    )
}

pub fn is_concisely_printed_array(expr: &Expression) -> bool {
    match expr {
        Expression::ArrayExpression(arr_expr) => {
            ArrayLike::ArrayExpression(arr_expr).is_concisely_printed()
        }
        _ => false,
    }
}

// ---

fn is_line_after_element_empty(p: &Prettier<'_>, index: u32) -> bool {
    fn skip_comment(p: &Prettier<'_>, idx: u32) -> Option<u32> {
        p.skip_inline_comment(p.skip_trailing_comment(Some(idx)))
    }

    fn skip_to_comma(p: &Prettier<'_>, current_idx: Option<u32>) -> Option<u32> {
        let current_idx = current_idx?;
        match p.source_text[current_idx as usize..].chars().next() {
            Some(',') => Some(current_idx),
            #[expect(clippy::cast_possible_truncation)]
            Some(c) => skip_to_comma(p, skip_comment(p, current_idx + c.len_utf8() as u32)),
            None => None,
        }
    }

    let Some(start_index) = skip_to_comma(p, Some(index)) else { return false };
    p.is_next_line_empty_after_index(start_index)
}

use oxc_ast::ast::{ArrayExpression, ArrayExpressionElement, Expression};
use oxc_formatter_core::{
    Buffer, Format,
    builders::{
        block_indent, empty_line, group, soft_block_indent, soft_line_break_or_space, text,
    },
    write,
};
use oxc_span::GetSpan;

use crate::{
    comments::{FormatTrailingInsideComments, write_dangling_comments},
    context::JsonFormatContext,
    separated::{TrailingSeparator, blank_line_after_comma, write_separated},
};

use super::{FmtJsonValue, FormatInvalidJson, JsonFormatter, format_with};

pub struct FmtJsonArray<'a, 'b> {
    pub array: &'b ArrayExpression<'a>,
}

impl<'a> Format<'a, JsonFormatContext<'a>> for FmtJsonArray<'a, '_> {
    fn fmt(&self, f: &mut JsonFormatter<'_, 'a>) {
        write!(f, "[");

        if self.array.elements.is_empty() {
            let dangling = f.context().comments().take_before(self.array.span.end);
            if dangling.is_empty() {
                write!(f, "]");
                return;
            }
            let inner = format_with(move |f: &mut JsonFormatter<'_, 'a>| {
                write_dangling_comments(dangling, f);
            });
            write!(f, [block_indent(&inner), "]"]);
            return;
        }

        // Sparse-only arrays like `[,]`, `[, , ,]`
        // would produce empty fill/group entries (Elision emits nothing).
        // Group builders assert on empty content, so emit the source slice verbatim
        // which already captures the user's holes.
        if self.array.elements.iter().all(|el| matches!(el, ArrayExpressionElement::Elision(_))) {
            let inner_start = self.array.span.start + 1;
            let inner_end = self.array.span.end - 1;
            let source = f.context().source_text();
            if inner_end > inner_start && (inner_end as usize) <= source.len() {
                write!(f, text(source.slice_range(inner_start, inner_end)));
            }
            write!(f, "]");
            return;
        }

        // When the last element is an Elision (e.g. `[1, , 2,,,,]`),
        // the trailing `,` is load-bearing for the hole count.
        // Otherwise (`[1, 2, 3,]`) the trailing comma is just JSON5 syntax that Prettier's `json` parser strips.
        let trailing_comma =
            matches!(self.array.elements.last(), Some(ArrayExpressionElement::Elision(_)));

        // Fill packs multiple numeric entries on one line.
        // The trailing `,` lives on the item (mirrors Prettier's `printArrayElementsConcisely`)
        // so the fill measurer counts it toward "does this item fit on the current line?".
        // Without that, the comma trailing the last on-line item gets pushed past `line_width`.
        // The separator carries only the break (or `empty_line` to preserve a user blank).
        if can_concisely_print(&self.array.elements) {
            let elements = &self.array.elements;
            let source = f.context().source_text();
            let last_idx = elements.len() - 1;
            let body = format_with(|f: &mut JsonFormatter<'_, 'a>| {
                let mut filler = f.fill();
                let mut prev_end: Option<u32> = None;
                for (i, element) in elements.iter().enumerate() {
                    let curr_start = element.span().start;
                    let pe = prev_end;
                    let sep = format_with(move |f: &mut JsonFormatter<'_, 'a>| {
                        if let Some(pe_val) = pe {
                            let between = source.bytes_range(pe_val, curr_start);
                            if blank_line_after_comma(between) {
                                write!(f, empty_line());
                            } else {
                                write!(f, soft_line_break_or_space());
                            }
                        }
                    });
                    let item = format_with(move |f: &mut JsonFormatter<'_, 'a>| {
                        write_array_element(element, f);
                        if i < last_idx {
                            write!(f, ",");
                        }
                    });
                    filler.entry(&sep, &item);
                    prev_end = Some(element.span().end);
                }
                filler.finish();

                // `elements` is non-empty in this branch (the empty-array early return is above),
                // so `last()` cannot be `None`.
                let last_end = elements.last().expect("non-empty elements").span().end;
                write!(
                    f,
                    FormatTrailingInsideComments {
                        lower_bound: last_end,
                        upper_bound: self.array.span.end,
                    }
                );
            });
            write!(f, [group(&soft_block_indent(&body)), "]"]);
            return;
        }

        let spans: Vec<_> = self.array.elements.iter().map(oxc_span::GetSpan::span).collect();
        let elements = format_with(|f: &mut JsonFormatter<'_, 'a>| {
            write_separated(f, &spans, TrailingSeparator::Disallowed, |i, f| {
                write_array_element(&self.array.elements[i], f);
            });

            if trailing_comma {
                write!(f, ",");
            }
            let last_end = spans.last().expect("non-empty elements").end;
            write!(
                f,
                FormatTrailingInsideComments {
                    lower_bound: last_end,
                    upper_bound: self.array.span.end,
                }
            );
        });

        // Mirrors Prettier's `shouldBreak` heuristic for arrays of arrays/objects
        // (`language-js/print/array.js`): force-expand only when there's > 1 element,
        // every element is the same kind of composite (all arrays or all objects),
        // and every element has at least 2 inner items.
        // Matrices and arrays-of-records qualify;
        // `[[1, 2], [3]]` or a single-element wrapper does not.
        let expand = should_force_expand(&self.array.elements);
        write!(f, [group(&soft_block_indent(&elements)).should_expand(expand), "]"]);
    }
}

fn should_force_expand(elements: &[ArrayExpressionElement<'_>]) -> bool {
    if elements.len() < 2 {
        return false;
    }
    let mut prev_is_array: Option<bool> = None;
    for el in elements {
        let (is_array, inner_len) = match el.as_expression() {
            Some(Expression::ArrayExpression(a)) => (true, a.elements.len()),
            Some(Expression::ObjectExpression(o)) => (false, o.properties.len()),
            _ => return false,
        };
        if inner_len < 2 {
            return false;
        }
        if let Some(prev) = prev_is_array
            && prev != is_array
        {
            return false;
        }
        prev_is_array = Some(is_array);
    }
    true
}

/// Writes one array element.
/// Elision (`[1, , 2]` holes) emits nothing,
/// the surrounding separator alone represents the hole.
/// `SpreadElement` isn't valid JSON;
/// we record a diagnostic and emit its source slice verbatim.
fn write_array_element<'a>(element: &ArrayExpressionElement<'a>, f: &mut JsonFormatter<'_, 'a>) {
    if matches!(element, ArrayExpressionElement::Elision(_)) {
        return;
    }
    if let Some(expr) = element.as_expression() {
        FmtJsonValue { expression: expr }.fmt(f);
    } else {
        // The only remaining variant (after `Elision` and `Expression`) is `SpreadElement`.
        write!(f, FormatInvalidJson(element.span()));
    }
}

/// Returns `true` if the array elements are all numeric literals and can be "fill-printed".
///
/// Accepts both bare numbers (`1`, `0.5`) and signed numbers (`-1`, `+0.5`),
/// the JS parser models the latter as `UnaryExpression(±, NumericLiteral)`,
/// so omitting it would knock common JSON shapes like coordinate or bounding arrays out of fill mode.
fn can_concisely_print(elements: &[ArrayExpressionElement<'_>]) -> bool {
    if elements.is_empty() {
        return false;
    }
    elements.iter().all(|el| {
        let Some(expr) = el.as_expression() else { return false };
        match expr {
            Expression::NumericLiteral(_) => true,
            Expression::UnaryExpression(u) if u.operator.is_arithmetic() => {
                matches!(u.argument, Expression::NumericLiteral(_))
            }
            _ => false,
        }
    })
}

use oxc_allocator::Vec;
use oxc_ast::ast::*;

use crate::{array, group, indent, ir::Doc, softline, text, Format, Prettier};

#[allow(clippy::enum_variant_names)]
pub enum MemberExpressionLike<'a, 'b> {
    ComputedMemberExpression(&'b ComputedMemberExpression<'a>),
    StaticMemberExpression(&'b StaticMemberExpression<'a>),
    PrivateFieldExpression(&'b PrivateFieldExpression<'a>),
}

impl<'a> MemberExpressionLike<'a, '_> {
    pub fn object(&self) -> &Expression<'a> {
        match self {
            MemberExpressionLike::ComputedMemberExpression(expr) => &expr.object,
            MemberExpressionLike::StaticMemberExpression(expr) => &expr.object,
            MemberExpressionLike::PrivateFieldExpression(expr) => &expr.object,
        }
    }
}

pub fn print_member_expression<'a>(
    p: &mut Prettier<'a>,
    member_expr: &MemberExpressionLike<'a, '_>,
) -> Doc<'a> {
    let mut parts = Vec::new_in(p.allocator);

    parts.push(member_expr.object().format(p));

    let lookup_doc = print_member_lookup(p, member_expr);

    // TODO: Calc `shouldInline` with parent
    let should_inline = true;

    if should_inline {
        parts.push(lookup_doc);
    } else {
        parts.push(group!(p, [indent!(p, [softline!(), lookup_doc])]));
    }

    // TODO: Wrap with `label!` when in member-chain
    array!(p, parts)
}

pub fn print_member_lookup<'a>(
    p: &mut Prettier<'a>,
    member_expr: &MemberExpressionLike<'a, '_>,
) -> Doc<'a> {
    match member_expr {
        MemberExpressionLike::StaticMemberExpression(expr) => {
            let mut parts = Vec::new_in(p.allocator);
            if expr.optional {
                parts.push(text!("?"));
            }
            parts.push(text!("."));
            parts.push(expr.property.format(p));
            array!(p, parts)
        }
        MemberExpressionLike::PrivateFieldExpression(expr) => {
            let mut parts = Vec::new_in(p.allocator);
            if expr.optional {
                parts.push(text!("?"));
            }
            parts.push(text!("."));
            parts.push(expr.field.format(p));
            array!(p, parts)
        }
        MemberExpressionLike::ComputedMemberExpression(expr) => {
            if expr.expression.is_number_literal() {
                let mut parts = Vec::new_in(p.allocator);
                if expr.optional {
                    parts.push(text!("?."));
                }
                parts.push(text!("["));
                parts.push(expr.expression.format(p));
                parts.push(text!("]"));
                return array!(p, parts);
            }

            // TODO: Examine this is rellay needed or not
            let mut parts = Vec::new_in(p.allocator);
            if expr.optional {
                parts.push(text!("?."));
            }
            parts.push(text!("["));
            parts.push(indent!(p, [softline!(), expr.expression.format(p)]));
            parts.push(softline!());
            parts.push(text!("]"));

            group!(p, parts)
        }
    }
}

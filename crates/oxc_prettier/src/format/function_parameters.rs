use oxc_ast::{ast::*, AstKind};

use crate::{
    doc::{Doc, DocBuilder},
    ss, Format, Prettier,
};

pub(super) fn print_function_parameters<'a>(
    p: &mut Prettier<'a>,
    params: &FormalParameters<'a>,
) -> Doc<'a> {
    let mut parts = p.vec();
    let is_arrow_function = matches!(p.parent_kind(), AstKind::ArrowExpression(_));
    let need_parens =
        !is_arrow_function || p.options.arrow_parens.is_always() || params.items.len() != 1;
    if need_parens {
        parts.push(ss!("("));
    }

    for (i, param) in params.items.iter().enumerate() {
        parts.push(param.format(p));
        if i < params.items.len() - 1 {
            parts.push(ss!(", "));
        }
    }

    if let Some(rest) = &params.rest {
        if !params.items.is_empty() {
            parts.push(ss!(", "));
        }
        parts.push(rest.format(p));
    }

    if need_parens {
        parts.push(ss!(")"));
    }

    Doc::Array(parts)
}

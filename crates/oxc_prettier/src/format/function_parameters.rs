#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

use crate::{doc::Doc, ss, Format, Prettier};

pub(super) fn print_function_parameters<'a>(
    p: &mut Prettier<'a>,
    params: &FormalParameters<'a>,
) -> Doc<'a> {
    let mut parts = p.vec();
    parts.push(ss!("("));

    for (i, param) in params.items.iter().enumerate() {
        parts.push(param.format(p));
        if i < params.items.len() - 1 {
            parts.push(ss!(", "));
        }
    }

    if let Some(rest) = &params.rest {
        parts.push(ss!(", "));
        parts.push(rest.format(p));
    }

    parts.push(ss!(")"));
    Doc::Array(parts)
}

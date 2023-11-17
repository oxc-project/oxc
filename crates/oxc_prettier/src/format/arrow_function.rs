#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

use crate::{doc::Doc, ss, Format, Prettier};

pub(super) fn print_arrow_function<'a>(
    p: &mut Prettier<'a>,
    expr: &ArrowExpression<'a>,
) -> Doc<'a> {
    let mut parts = p.vec();

    parts.push(ss!("() => "));
    parts.push(expr.body.format(p));

    Doc::Array(parts)
}

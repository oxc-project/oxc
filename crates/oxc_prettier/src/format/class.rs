#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

use crate::{doc::Doc, ss, Format, Prettier};

pub(super) fn print_class<'a>(p: &mut Prettier<'a>, class: &Class<'a>) -> Doc<'a> {
    let mut parts = p.vec();
    parts.push(ss!("class "));
    if let Some(id) = &class.id {
        parts.push(id.format(p));
    }
    parts.push(ss!(" "));
    parts.push(class.body.format(p));
    Doc::Array(parts)
}

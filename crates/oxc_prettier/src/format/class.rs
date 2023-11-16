#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

use crate::{doc::Doc, ss, Format, Prettier};

impl<'a> Prettier<'a> {
    pub(super) fn print_class(&mut self, class: &Class<'a>) -> Doc<'a> {
        let mut parts = self.vec();
        parts.push(ss!("class "));
        if let Some(id) = &class.id {
            parts.push(id.format(self));
        }
        parts.push(ss!(" "));
        parts.push(class.body.format(self));
        Doc::Array(parts)
    }
}

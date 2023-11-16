#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

use crate::{doc::Doc, ss, Format, Prettier};

impl<'a> Prettier<'a> {
    pub(super) fn print_function_parameters(&mut self, params: &FormalParameters<'a>) -> Doc<'a> {
        let mut parts = self.vec();
        parts.push(ss!("("));

        for (i, param) in params.items.iter().enumerate() {
            parts.push(param.format(self));
            if i < params.items.len() - 1 {
                parts.push(ss!(", "));
            }
        }

        if let Some(rest) = &params.rest {
            parts.push(ss!(", "));
            parts.push(rest.format(self));
        }

        parts.push(ss!(")"));
        Doc::Array(parts)
    }
}

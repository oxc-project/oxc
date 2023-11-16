#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

use crate::{doc::Doc, group, hardline, if_break, indent, softline, ss, Format, Prettier};

impl<'a> Prettier<'a> {
    pub(super) fn print_function(&mut self, func: &Function<'a>) -> Doc<'a> {
        let mut parts = self.vec();
        if let Some(comments) = self.print_leading_comments(func.span) {
            parts.push(comments);
        }
        if func.r#async {
            parts.push(ss!("async "));
        }
        if func.generator {
            parts.push(ss!("function* "));
        } else {
            parts.push(ss!("function "));
        }
        if let Some(type_params) = &func.type_parameters {
            parts.push(type_params.format(self));
        }
        if let Some(id) = &func.id {
            parts.push(self.str(id.name.as_str()));
        }
        parts.push(func.params.format(self));
        if let Some(body) = &func.body {
            parts.push(ss!(" "));
            parts.push(body.format(self));
        }
        Doc::Array(parts)
    }

    pub(super) fn print_return_or_throw_argument(
        &mut self,
        argument: Option<&Expression<'a>>,
        is_return: bool,
    ) -> Doc<'a> {
        let mut parts = self.vec();

        parts.push(ss!(if is_return { "return" } else { "throw" }));

        if let Some(argument) = argument {
            parts.push(ss!(" "));
            parts.push(group![
                self,
                if_break!(self, "("),
                indent!(self, softline!(), argument.format(self)),
                softline!(),
                if_break!(self, ")")
            ]);
        }

        parts.push(self.str(";"));
        parts.push(hardline!());
        Doc::Array(parts)
    }
}

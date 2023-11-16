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
        if should_group_function_parameters(func) {
            parts.push(group!(self, func.params.format(self)));
        } else {
            parts.push(func.params.format(self));
        }
        if let Some(body) = &func.body {
            parts.push(ss!(" "));
            parts.push(body.format(self));
        }
        if self.options.semi && (func.is_ts_declare_function() || func.body.is_none()) {
            parts.push(self.str(";"));
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

fn should_group_function_parameters(func: &Function) -> bool {
    let Some(return_type) = &func.return_type else {
        return false;
    };
    let type_parameters = func.type_parameters.as_ref().map(|x| &x.params);

    if let Some(type_parameter) = type_parameters {
        if type_parameter.len() > 1 {
            return false;
        }

        if let Some(type_parameter) = type_parameter.first() {
            if type_parameter.constraint.is_some() || type_parameter.default.is_some() {
                return false;
            }
        }
    }

    // TODO: need union `willBreak`
    func.params.parameters_count() == 1
        && (matches!(
            return_type.type_annotation,
            TSType::TSTypeLiteral(_) | TSType::TSMappedType(_)
        ))
}

use oxc_allocator::Vec;
#[allow(clippy::wildcard_imports)]
use oxc_ast::ast::*;

use crate::{doc::Doc, hardline, indent, ss, Prettier};

impl<'a> Prettier<'a> {
    pub(super) fn print_block(
        &mut self,
        stmts: &Vec<'a, Statement<'a>>,
        directives: Option<&Vec<'a, Directive>>,
        is_static_block: bool,
    ) -> Doc<'a> {
        let mut parts = self.vec();
        if is_static_block {
            parts.push(ss!("static "));
        }
        parts.push(ss!("{"));
        if let Some(doc) = self.print_block_body(stmts, directives) {
            parts.push(indent![self, hardline!(), doc]);
            parts.push(hardline!());
        }
        parts.push(ss!("}"));
        Doc::Array(parts)
    }

    pub(super) fn print_block_body(
        &mut self,
        stmts: &Vec<'a, Statement<'a>>,
        directives: Option<&Vec<'a, Directive>>,
    ) -> Option<Doc<'a>> {
        let has_directives = directives.is_some_and(|directives| !directives.is_empty());
        let has_body = stmts.iter().any(|stmt| !matches!(stmt, Statement::EmptyStatement(_)));

        if !has_body && !has_directives {
            return None;
        }

        let mut parts = self.vec();

        if has_directives {
            if let Some(directives) = directives {
                parts.extend(self.print_statement_sequence(directives));
            }
        }

        if !stmts.is_empty() {
            parts.extend(self.print_statement_sequence(stmts));
        }

        Some(Doc::Array(parts))
    }
}

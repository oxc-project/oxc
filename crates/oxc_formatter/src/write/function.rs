use oxc_ast::ast::*;

use super::FormatWrite;
use crate::{
    format_args,
    formatter::{Buffer, FormatResult, Formatter, prelude::*, trivia::DanglingIndentMode},
    generated::ast_nodes::AstNode,
    write,
};

impl<'a> FormatWrite<'a> for AstNode<'a, Function<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.r#async() {
            write!(f, ["async", space()]);
        }
        write!(f, "function");
        if self.generator() {
            write!(f, "*");
        }
        write!(f, [space(), self.id(), group(&self.params()), space(), self.body()]);
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, FunctionBody<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let statements = self.statements();
        let directives = self.directives();
        if statements.is_empty() && directives.is_empty() {
            write!(f, ["{", format_dangling_comments(self.span).with_block_indent(), "}"])
        } else {
            write!(f, ["{", block_indent(&format_args!(directives, statements)), "}"])
        }
    }
}

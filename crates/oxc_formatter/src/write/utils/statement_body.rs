use oxc_ast::ast::Statement;

use crate::{
    format_args,
    formatter::{
        Buffer, Format, FormatResult, Formatter,
        prelude::{indent, soft_line_break_or_space, space},
    },
    generated::ast_nodes::{AstNode, AstNodes},
    write,
    write::FormatWrite,
};

pub struct FormatStatementBody<'a, 'b> {
    body: &'b AstNode<'a, Statement<'a>>,
    force_space: bool,
}

impl<'a, 'b> FormatStatementBody<'a, 'b> {
    pub fn new(body: &'b AstNode<'a, Statement<'a>>) -> Self {
        Self { body, force_space: false }
    }

    /// Prevents that the consequent is formatted on its own line and indented by one level and
    /// instead gets separated by a space.
    pub fn with_forced_space(mut self, forced: bool) -> Self {
        self.force_space = forced;
        self
    }
}

impl<'a> Format<'a> for FormatStatementBody<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if let AstNodes::EmptyStatement(empty) = self.body.as_ast_nodes() {
            write!(f, empty)
        } else if let AstNodes::BlockStatement(block) = self.body.as_ast_nodes() {
            write!(f, [space()]);
            // Use `write` instead of `format` to avoid printing leading comments of the block.
            // Those comments should be printed inside the block statement.
            block.write(f)
        } else if self.force_space {
            write!(f, [space(), self.body])
        } else {
            write!(f, [indent(&format_args!(soft_line_break_or_space(), &self.body))])
        }
    }
}

use oxc_ast::ast::Statement;
use oxc_span::GetSpan;

use crate::{
    format_args,
    formatter::{
        Buffer, Format, FormatResult, Formatter,
        prelude::{
            format_once, group, indent, soft_line_break_or_space, soft_line_indent_or_space, space,
        },
        trivia::FormatTrailingComments,
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
            write!(
                f,
                [indent(&format_args!(
                    &soft_line_break_or_space(),
                    &format_once(|f| {
                        let is_consequent_of_if_statement_parent = matches!(
                            self.body.parent,
                            AstNodes::IfStatement(if_stmt)
                            if if_stmt.consequent.span() == self.body.span() && if_stmt.alternate.is_some()
                        );
                        if is_consequent_of_if_statement_parent
                            && let AstNodes::ExpressionStatement(expression) =
                                self.body.as_ast_nodes()
                        {
                            expression.format_leading_comments(f)?;
                            expression.write(f)?;
                            let comments = f
                                .context()
                                .comments()
                                .comments_before_character(expression.span.end, b'\n');
                            FormatTrailingComments::Comments(comments).fmt(f)
                        } else {
                            self.body.fmt(f)
                        }
                    })
                ))]
            )
        }
    }
}

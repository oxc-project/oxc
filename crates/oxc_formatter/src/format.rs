use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    formatter::{Buffer, Format, FormatContext, FormatResult, Formatter, prelude::*},
    write,
};

impl<'a> Format for Program<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult<()> {
        write!(
            f,
            [
                self.hashbang,
                self.body,
                format_leading_comments(self.span),
                self.directives,
                hard_line_break()
            ]
        )
    }
}

impl<'a> Format for Hashbang<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult<()> {
        write!(f, [text("#!"), dynamic_text(self.value.as_str(), self.span.start)])
    }
}

impl<'a> Format for Vec<'a, Directive<'a>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult<()> {
        let source_text = f.context().source_text();
        let mut join = f.join_nodes_with_hardline();
        for directive in self {
            join.entry(directive.span, source_text, directive);
        }
        join.finish()
    }
}

impl<'a> Format for Directive<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult<()> {
        let source_text = f.context().source_text();
        write!(f, [located_token_text(self.span, source_text)])
    }
}

impl<'a> Format for Vec<'a, Statement<'a>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult<()> {
        let source_text = f.context().source_text();
        let mut join = f.join_nodes_with_hardline();
        for stmt in self {
            join.entry(stmt.span(), source_text, stmt);
        }
        join.finish()
    }
}

impl<'a> Format for Statement<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult<()> {
        match self {
            Statement::VariableDeclaration(stmt) => stmt.fmt(f),
            Statement::BlockStatement(stmt) => stmt.fmt(f),
            _ => write!(f, [text("// TODO"), hard_line_break()]),
        }
    }
}

impl<'a> Format for VariableDeclaration<'a> {
    fn fmt(&self, f: &mut Formatter) -> FormatResult<()> {
        write!(
            f,
            [text("// TODO: VariableDeclaration @"), text(self.kind.as_str()), hard_line_break()]
        )
    }
}

impl<'a> Format for BlockStatement<'a> {
    fn fmt(&self, f: &mut Formatter) -> FormatResult<()> {
        write!(f, [text("{")])?;

        if block_statement::is_empty_block(self) {
            let comments = f.comments();
            let has_dangling_comments = comments.has_dangling_comments(self.span);
            if has_dangling_comments {
            } else if block_statement::is_non_collapsible() {
                write!(f, [hard_line_break()])?;
            }
        } else {
            write!(f, [text("{")])?
        }

        write!(f, [text("}")])
    }
}

mod block_statement {
    use super::BlockStatement;

    pub fn is_empty_block(block: &BlockStatement<'_>) -> bool {
        true
    }

    pub fn is_non_collapsible() -> bool {
        false
    }
}

impl<'a> Format for StringLiteral<'a> {
    fn fmt(&self, f: &mut Formatter) -> FormatResult<()> {
        write!(f, [text("\""), dynamic_text(self.value.as_str(), self.span.start), text("\";")])
    }
}

use oxc_allocator::Vec;
use oxc_ast::{AstKind, ast::*};
use oxc_span::GetSpan;

use crate::{
    formatter::{Buffer, Format, FormatContext, FormatResult, Formatter, prelude::*},
    write,
    write::FormatWrite,
};

/// A hack for erasing the lifetime requirement.
pub fn hack<'ast, T>(t: &T) -> &'ast T {
    // SAFETY: This is not safe :-)
    unsafe { std::mem::transmute(t) }
}

impl<'ast> Format<'ast> for Program<'ast> {
    fn fmt(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()> {
        // f.format_program(self)
        Ok(())
    }
}

impl<'ast> Format<'ast> for Hashbang<'ast> {
    fn fmt(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()> {
        f.state_mut().stack.push(AstKind::Hashbang(hack(self)));
        // let result = block_stmt.fmt(self);
        let result = self.write(f);
        unsafe { f.state_mut().stack.pop_unchecked() };
        result
    }
}

impl<'ast> Format<'ast> for Vec<'ast, Directive<'ast>> {
    fn fmt(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()> {
        let source_text = f.context().source_text();
        let mut join = f.join_nodes_with_hardline();
        for directive in self {
            join.entry(directive.span, source_text, directive);
        }
        join.finish()
    }
}

impl<'ast> Format<'ast> for Directive<'ast> {
    fn fmt(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()> {
        let source_text = f.context().source_text();
        write!(f, [located_token_text(self.span, source_text)])
    }
}

impl<'ast> Format<'ast> for Vec<'ast, Statement<'ast>> {
    fn fmt(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()> {
        let source_text = f.context().source_text();
        let mut join = f.join_nodes_with_hardline();
        for stmt in self {
            join.entry(stmt.span(), source_text, stmt);
        }
        join.finish()
    }
}

impl<'ast> Format<'ast> for Statement<'ast> {
    fn fmt(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()> {
        match self {
            Statement::VariableDeclaration(stmt) => stmt.fmt(f),
            Statement::BlockStatement(stmt) => stmt.fmt(f),
            _ => write!(f, [text("// TODO"), hard_line_break()]),
        }
    }
}

impl<'ast> Format<'ast> for VariableDeclaration<'ast> {
    fn fmt(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()> {
        write!(
            f,
            [text("// TODO: VariableDeclaration @"), text(self.kind.as_str()), hard_line_break()]
        )
    }
}

impl<'ast> Format<'ast> for BlockStatement<'ast> {
    fn fmt(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()> {
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

impl<'ast> Format<'ast> for StringLiteral<'ast> {
    fn fmt(&self, f: &mut Formatter<'_, 'ast>) -> FormatResult<()> {
        write!(f, [text("\""), dynamic_text(self.value.as_str(), self.span.start), text("\";")])
    }
}

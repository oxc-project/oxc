use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::formatter::{Buffer, Format, FormatResult, prelude::*};
use crate::{JsFormatContext, write};

pub type JsFormatter<'ast, 'buf> = Formatter<'buf, JsFormatContext<'ast>>;

impl<'ast> Format<JsFormatContext<'ast>> for Program<'ast> {
    fn fmt(&self, f: &mut JsFormatter<'ast, '_>) -> FormatResult<()> {
        let Program { body, .. } = self;
        body.fmt(f)
    }
}

impl<'a> Format<JsFormatContext<'a>> for Vec<'a, Statement<'a>> {
    fn fmt(&self, f: &mut JsFormatter<'a, '_>) -> FormatResult<()> {
        let source_text = f.context().source_text();
        let mut join = f.join_nodes_with_hardline();
        for stmt in self {
            join.entry(stmt.span(), source_text, stmt);
        }
        join.finish()
    }
}

impl<'a> Format<JsFormatContext<'a>> for Statement<'a> {
    fn fmt(&self, f: &mut JsFormatter<'a, '_>) -> FormatResult<()> {
        match self {
            Statement::VariableDeclaration(stmt) => stmt.fmt(f),
            _ => write!(f, [text("// TODO"), hard_line_break()]),
        }
    }
}

impl<'a> Format<JsFormatContext<'a>> for VariableDeclaration<'a> {
    fn fmt(&self, f: &mut JsFormatter) -> FormatResult<()> {
        let VariableDeclaration { kind, .. } = self;

        write!(
            f,
            [text("// TODO: VariableDeclaration @"), text(kind.as_str()), hard_line_break()]
        )?;

        Ok(())
    }
}

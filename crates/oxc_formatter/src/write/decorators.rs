use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    Format, FormatResult, format_args,
    formatter::{Formatter, prelude::*},
    generated::ast_nodes::{AstNode, AstNodes},
    write,
};

use super::FormatWrite;

impl<'a> Format<'a> for AstNode<'a, Vec<'a, Decorator<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.is_empty() {
            return Ok(());
        }

        // Check parent to determine formatting context
        match self.parent {
            AstNodes::PropertyDefinition(_)
            | AstNodes::MethodDefinition(_)
            | AstNodes::AccessorProperty(_) => {
                return write!(
                    f,
                    [group(&format_args!(
                        format_once(|f| {
                            f.join_nodes_with_soft_line().entries(self.iter()).finish()
                        }),
                        soft_line_break_or_space()
                    ))
                    .should_expand(should_expand_decorators(self, f))]
                );
            }
            // Parameter decorators
            AstNodes::FormalParameter(_) => {
                write!(f, should_expand_decorators(self, f).then_some(expand_parent()))?;
            }
            AstNodes::ExportNamedDeclaration(_) | AstNodes::ExportDefaultDeclaration(_) => {
                write!(f, [hard_line_break()])?;
            }
            _ => {
                write!(f, [expand_parent()])?;
            }
        }

        f.join_with(&soft_line_break_or_space()).entries(self.iter()).finish()?;
        write!(f, [soft_line_break_or_space()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, Decorator<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        write!(f, ["@", self.expression()])
    }
}

/// Check if decorators should expand (have newlines between them)
#[inline]
fn should_expand_decorators<'a>(
    decorators: &AstNode<'a, Vec<'a, Decorator<'a>>>,
    f: &Formatter<'_, 'a>,
) -> bool {
    decorators.iter().any(|decorator| get_lines_after(decorator.span().end, f.source_text()) > 0)
}

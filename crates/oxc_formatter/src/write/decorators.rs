use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    Format, FormatResult,
    ast_nodes::{AstNode, AstNodes},
    format_args,
    formatter::{
        Formatter,
        prelude::*,
        trivia::{FormatLeadingComments, FormatTrailingComments},
    },
    write,
};

use super::FormatWrite;

impl<'a> Format<'a> for AstNode<'a, Vec<'a, Decorator<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Some(last) = self.last() else {
            return Ok(());
        };

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

fn is_identifier_or_static_member_only(callee: &Expression) -> bool {
    let mut expr = callee;
    loop {
        match expr {
            Expression::Identifier(_) => return true,
            Expression::StaticMemberExpression(static_member) => {
                expr = &static_member.object;
            }
            _ => break,
        }
    }

    false
}

impl<'a> FormatWrite<'a> for AstNode<'a, Decorator<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.format_leading_comments(f)?;
        write!(f, ["@"]);

        // Determine if parentheses are required around decorator expressions
        let needs_parentheses = match &self.expression {
            // Identifiers: `@decorator` needs no parens
            Expression::Identifier(_) => false,
            // Call expressions: `@obj.method()` needs no parens, `@(complex().method)()` needs parens
            Expression::CallExpression(call) => !is_identifier_or_static_member_only(&call.callee),
            // Static member expressions: `@obj.prop` needs no parens, `@(complex[key])` needs parens
            Expression::StaticMemberExpression(static_member) => {
                !is_identifier_or_static_member_only(&static_member.object)
            }
            // All other expressions need parentheses: `@(a + b)`, `@(condition ? a : b)`
            _ => true,
        };

        if needs_parentheses {
            write!(f, "(")?;
        }
        write!(f, [self.expression()])?;
        if needs_parentheses {
            write!(f, ")")?;
        }
        Ok(())
    }
}

/// Check if decorators should expand (have newlines between them)
#[inline]
fn should_expand_decorators<'a>(
    decorators: &AstNode<'a, Vec<'a, Decorator<'a>>>,
    f: &Formatter<'_, 'a>,
) -> bool {
    decorators.iter().any(|decorator| f.source_text().lines_after(decorator.span().end) > 0)
}

use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    Format, FormatResult, format_args,
    formatter::{
        Formatter,
        prelude::*,
        trivia::{FormatLeadingComments, FormatTrailingComments},
    },
    generated::ast_nodes::{AstNode, AstNodes},
    write,
};

use super::FormatWrite;

impl<'a> Format<'a> for AstNode<'a, Vec<'a, Decorator<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let Some(last) = self.last() else {
            return Ok(());
        };

        let format_decorators = format_once(|f| {
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
        });

        format_decorators.fmt(f)?;
        format_trailing_comments_for_last_decorator(last.span.end, f)
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
        write!(f, ["@"])?;

        // Check if we need to manually add parentheses for cases not handled by NeedsParentheses
        let needs_manual_parentheses = match &self.expression {
            // These expressions don't need manual parentheses:
            // - ParenthesizedExpression already has parens
            // - CallExpression, ComputedMemberExpression, StaticMemberExpression handled by NeedsParentheses
            // - Identifiers don't need parens
            Expression::ParenthesizedExpression(_)
            | Expression::CallExpression(_)
            | Expression::ComputedMemberExpression(_)
            | Expression::StaticMemberExpression(_)
            | Expression::Identifier(_) => false,
            // All other complex expressions need parentheses
            _ => true,
        };

        if needs_manual_parentheses {
            write!(f, "(")?;
        }
        write!(f, [self.expression()])?;
        if needs_manual_parentheses {
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

pub fn format_trailing_comments_for_last_decorator(
    mut start: u32,
    f: &mut Formatter<'_, '_>,
) -> FormatResult<()> {
    let mut comments = f.context().comments().unprinted_comments();

    for (i, comment) in comments.iter().enumerate() {
        if !f.source_text().all_bytes_match(start, comment.span.start, |b| b.is_ascii_whitespace())
        {
            comments = &comments[..i];
            break;
        }

        start = comment.span.end;
    }

    if !comments.is_empty() {
        write!(
            f,
            [group(&format_args!(
                FormatTrailingComments::Comments(comments),
                soft_line_break_or_space()
            ))]
        )?;
    }

    Ok(())
}

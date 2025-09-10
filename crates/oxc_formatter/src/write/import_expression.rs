use oxc_ast::ast::*;

use crate::{
    Format, FormatResult, TrailingSeparator, best_fitting, format_args,
    formatter::{Formatter, prelude::*},
    generated::ast_nodes::AstNode,
    write,
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, ImportExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["import"])?;
        if let Some(phase) = &self.phase() {
            write!(f, [".", phase.as_str()])?;
        }

        // The formatting implementation of `source` and `options` picks from `call_arguments`.
        if self.options.is_none()
            && (!matches!(
                self.source,
                Expression::StringLiteral(_)
                    | Expression::TemplateLiteral(_)
                    // Theoretically dynamic import shouldn't have this.
                    | Expression::TaggedTemplateExpression(_)
            ) || f.comments().has_comment_before(self.span.end))
        {
            return write!(
                f,
                [
                    "(",
                    group(&soft_block_indent(&format_once(|f| {
                        write!(f, [self.source()])?;
                        if let Some(options) = self.options() {
                            write!(
                                f,
                                [
                                    ",",
                                    soft_line_break_or_space(),
                                    group(&options).should_expand(true)
                                ]
                            )?;
                        }
                        Ok(())
                    }))),
                    ")"
                ]
            );
        }

        let source = self.source().memoized();
        let options = self.options().memoized();

        best_fitting![
            group(&format_once(|f| {
                write!(f, ["(", source])?;
                if self.options().is_some() {
                    write!(f, [",", space(), group(&options).should_expand(true)])?;
                }
                write!(f, ")")
            })),
            group(&format_args!(
                "(",
                &soft_block_indent(&format_once(|f| {
                    write!(f, [source])?;
                    if self.options.is_some() {
                        write!(f, [",", soft_line_break_or_space(), options])?;
                    }
                    Ok(())
                })),
                ")"
            ))
            .should_expand(true),
        ]
        .fmt(f)
    }
}

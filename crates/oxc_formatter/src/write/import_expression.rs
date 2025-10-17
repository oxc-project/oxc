use oxc_allocator::Vec;
use oxc_ast::ast::*;

use crate::{
    Format, FormatResult, TrailingSeparator,
    ast_nodes::AstNode,
    best_fitting, format_args,
    formatter::{Formatter, prelude::*},
    write,
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, ImportExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, ["import"])?;
        if let Some(phase) = &self.phase() {
            write!(f, [".", phase.as_str()])?;
        }

        // Use the same logic as CallExpression arguments formatting
        write!(f, self.to_arguments())
    }
}

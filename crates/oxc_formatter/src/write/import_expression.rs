use oxc_ast::ast::*;

use crate::{
    ast_nodes::AstNode,
    formatter::{Formatter, prelude::*},
    write,
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, ImportExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) {
        write!(f, ["import"]);
        if let Some(phase) = &self.phase() {
            write!(f, [".", phase.as_str()]);
        }

        // Use the same logic as CallExpression arguments formatting
        write!(f, self.to_arguments());
    }
}

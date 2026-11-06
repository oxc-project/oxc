//! An embedded JS/TS part ([`crate::format_to_ir`]), printed as its [`JsRoot`] says.
//! A kind's own rules live in its module; this one dispatches on the root.

mod program;

use oxc_ast::ast::*;
use oxc_formatter_core::Format;

use crate::{
    JsRoot,
    ast_nodes::AstNode,
    formatter::{JsFormatContext, JsFormatter},
};

/// An embedded part's program (see [`crate::format_to_ir`]), printed as the code its root names.
pub struct FormatRoot<'a, 'b> {
    pub program: &'b AstNode<'b, 'a, Program<'a>>,
    pub root: JsRoot,
}

impl<'a> Format<'a, JsFormatContext<'a>> for FormatRoot<'a, '_> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
        match self.root {
            JsRoot::Program { semicolons, layout } => {
                program::write_program(self.program, semicolons, layout, f);
            }
        }
    }
}

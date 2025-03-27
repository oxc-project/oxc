use oxc_ast::{AstKind, ast::*};

use crate::{
    formatter::{Buffer, Format, Formatter},
    write,
};

impl<'buf, 'ast> Formatter<'buf, 'ast> {
    pub fn format_program(&mut self, program: &'ast Program<'ast>) {
        // self.context_mut().stack.push(AstKind::Program(program));
        let result = program.fmt(self);
        // self.context_mut().leave_node();
    }

    pub fn format_block_statement(&mut self, block_stmt: &'ast BlockStatement<'ast>) {
        self.state_mut().stack.push(AstKind::BlockStatement(block_stmt));
        let result = block_stmt.fmt(self);
        unsafe { self.state_mut().stack.pop_unchecked() };
    }
}

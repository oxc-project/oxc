use oxc_allocator::ArenaVec;
use oxc_ast::ast::{BlockStatement, IfStatement, Statement};

pub trait IsTerminated {
    fn is_terminated(&self) -> bool;
}

impl IsTerminated for Statement<'_> {
    fn is_terminated(&self) -> bool {
        match self {
            Statement::IfStatement(stmt) => stmt.is_terminated(),
            Statement::BlockStatement(stmt) => stmt.is_terminated(),
            _ => self.is_jump_statement(),
        }
    }
}

impl IsTerminated for IfStatement<'_> {
    fn is_terminated(&self) -> bool {
        self.consequent.is_terminated() && self.alternate.is_terminated()
    }
}

impl IsTerminated for BlockStatement<'_> {
    fn is_terminated(&self) -> bool {
        self.body.is_terminated()
    }
}

impl IsTerminated for ArenaVec<'_, Statement<'_>> {
    fn is_terminated(&self) -> bool {
        self.iter().last().is_some_and(Statement::is_terminated)
    }
}

impl IsTerminated for Option<Statement<'_>> {
    fn is_terminated(&self) -> bool {
        self.as_ref().is_some_and(Statement::is_terminated)
    }
}

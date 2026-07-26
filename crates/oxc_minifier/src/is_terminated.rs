use oxc_allocator::ArenaVec;
use oxc_ast::ast::{BlockStatement, IfStatement, Statement};

pub trait IsTerminated {
    fn is_terminated(&self) -> bool;
}

impl<'a> IsTerminated for Statement<'a> {
    fn is_terminated(&self) -> bool {
        match self {
            Statement::IfStatement(stmt) => stmt.is_terminated(),
            Statement::BlockStatement(stmt) => stmt.is_terminated(),
            _ => self.is_jump_statement(),
        }
    }
}

impl<'a> IsTerminated for IfStatement<'a> {
    fn is_terminated(&self) -> bool {
        self.consequent.is_terminated() && self.alternate.is_terminated()
    }
}

impl<'a> IsTerminated for BlockStatement<'a> {
    fn is_terminated(&self) -> bool {
        self.body.is_terminated()
    }
}

impl<'a> IsTerminated for ArenaVec<'a, Statement<'a>> {
    fn is_terminated(&self) -> bool {
        self.iter().last().is_some_and(|stmt| stmt.is_terminated())
    }
}

impl<'a> IsTerminated for Option<Statement<'a>> {
    fn is_terminated(&self) -> bool {
        self.as_ref().map_or(false, |stmt| stmt.is_terminated())
    }
}

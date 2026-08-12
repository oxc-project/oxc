use oxc_allocator::ArenaVec;
use oxc_ast::ast::{BlockStatement, IfStatement, Statement, TryStatement};

/// A statement that never completes normally — a direct jump, a kept block
/// ending in a jump, an if/else where every branch jumps, or a try/catch where
/// every path jumps — makes the rest of the list unreachable.
pub trait IsTerminated {
    fn is_terminated(&self) -> bool;
}

impl IsTerminated for Statement<'_> {
    fn is_terminated(&self) -> bool {
        match self {
            Statement::IfStatement(stmt) => stmt.is_terminated(),
            Statement::BlockStatement(stmt) => stmt.is_terminated(),
            Statement::TryStatement(stmt) => stmt.is_terminated(),
            _ => self.is_jump_statement(),
        }
    }
}

impl IsTerminated for IfStatement<'_> {
    fn is_terminated(&self) -> bool {
        self.consequent.is_terminated() && self.alternate.is_terminated()
    }
}

impl IsTerminated for TryStatement<'_> {
    fn is_terminated(&self) -> bool {
        // A finalizer that aborts overrides however the other blocks complete.
        // Otherwise the try block must abort, and so must the catch block when
        // present (an exception thrown before the try block's jump lands there).
        self.finalizer.as_ref().is_some_and(|f| f.is_terminated())
            || (self.block.is_terminated()
                && self.handler.as_ref().is_some_and(|h| h.body.is_terminated()))
    }
}

impl IsTerminated for BlockStatement<'_> {
    fn is_terminated(&self) -> bool {
        self.body.is_terminated()
    }
}

impl IsTerminated for ArenaVec<'_, Statement<'_>> {
    fn is_terminated(&self) -> bool {
        // A minimized dead zone keeps only hoisting survivors after the jump:
        // `function` declarations and the initializer-less `var` stub
        // re-emitted by `KeepVar`. Their bindings initialize at scope entry
        // and nothing after an aborting statement ever runs, so skip them
        // from the back before testing how the block terminates.
        self.iter()
            .rev()
            .find(|stmt| match stmt {
                Statement::FunctionDeclaration(_) => false,
                Statement::VariableDeclaration(decl) => {
                    !(decl.kind.is_var() && decl.declarations.iter().all(|d| d.init.is_none()))
                }
                _ => true,
            })
            .is_some_and(Statement::is_terminated)
    }
}

impl<T: IsTerminated> IsTerminated for Option<T> {
    fn is_terminated(&self) -> bool {
        self.as_ref().is_some_and(IsTerminated::is_terminated)
    }
}

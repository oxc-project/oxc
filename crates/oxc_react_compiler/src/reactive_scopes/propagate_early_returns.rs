/// Propagate early returns in reactive scopes.
///
/// Port of `ReactiveScopes/PropagateEarlyReturns.ts` from the React Compiler.
///
/// Handles the case where a reactive scope contains a return statement.
/// When this happens, the scope needs a special early-return variable
/// to store the return value, and the return is replayed when the
/// cache is hit.
use crate::hir::{
    ReactiveBlock, ReactiveFunction, ReactiveStatement, ReactiveTerminal,
};

/// Propagate early returns from reactive scopes.
pub fn propagate_early_returns(func: &mut ReactiveFunction) {
    propagate_in_block(&mut func.body);
}

fn propagate_in_block(block: &mut ReactiveBlock) {
    for stmt in block.iter_mut() {
        match stmt {
            ReactiveStatement::Scope(scope) => {
                propagate_in_block(&mut scope.instructions);
                // Check if any instruction in this scope is a return
                let has_return = contains_return(&scope.instructions);
                if has_return {
                    // In the full implementation, we'd:
                    // 1. Create an early return variable
                    // 2. Replace return statements with assignments to the variable
                    // 3. Add a check after the scope to replay the return
                }
            }
            ReactiveStatement::PrunedScope(scope) => {
                propagate_in_block(&mut scope.instructions);
            }
            ReactiveStatement::Terminal(term) => {
                propagate_in_terminal(&mut term.terminal);
            }
            ReactiveStatement::Instruction(_) => {}
        }
    }
}

fn propagate_in_terminal(terminal: &mut ReactiveTerminal) {
    match terminal {
        ReactiveTerminal::If(t) => {
            propagate_in_block(&mut t.consequent);
            if let Some(alt) = &mut t.alternate {
                propagate_in_block(alt);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &mut t.cases {
                if let Some(block) = &mut case.block {
                    propagate_in_block(block);
                }
            }
        }
        ReactiveTerminal::While(t) => propagate_in_block(&mut t.r#loop),
        ReactiveTerminal::DoWhile(t) => propagate_in_block(&mut t.r#loop),
        ReactiveTerminal::For(t) => propagate_in_block(&mut t.r#loop),
        ReactiveTerminal::ForOf(t) => propagate_in_block(&mut t.r#loop),
        ReactiveTerminal::ForIn(t) => propagate_in_block(&mut t.r#loop),
        ReactiveTerminal::Label(t) => propagate_in_block(&mut t.block),
        ReactiveTerminal::Try(t) => {
            propagate_in_block(&mut t.block);
            propagate_in_block(&mut t.handler);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}

fn contains_return(block: &ReactiveBlock) -> bool {
    for stmt in block {
        match stmt {
            ReactiveStatement::Terminal(term) => {
                if matches!(term.terminal, ReactiveTerminal::Return(_)) {
                    return true;
                }
                if terminal_contains_return(&term.terminal) {
                    return true;
                }
            }
            ReactiveStatement::Scope(scope) => {
                if contains_return(&scope.instructions) {
                    return true;
                }
            }
            ReactiveStatement::PrunedScope(scope) => {
                if contains_return(&scope.instructions) {
                    return true;
                }
            }
            ReactiveStatement::Instruction(_) => {}
        }
    }
    false
}

fn terminal_contains_return(terminal: &ReactiveTerminal) -> bool {
    match terminal {
        ReactiveTerminal::If(t) => {
            contains_return(&t.consequent)
                || t.alternate.as_ref().is_some_and(contains_return)
        }
        ReactiveTerminal::Switch(t) => {
            t.cases.iter().any(|c| c.block.as_ref().is_some_and(contains_return))
        }
        ReactiveTerminal::While(t) => contains_return(&t.r#loop),
        ReactiveTerminal::DoWhile(t) => contains_return(&t.r#loop),
        ReactiveTerminal::For(t) => contains_return(&t.r#loop),
        ReactiveTerminal::ForOf(t) => contains_return(&t.r#loop),
        ReactiveTerminal::ForIn(t) => contains_return(&t.r#loop),
        ReactiveTerminal::Label(t) => contains_return(&t.block),
        ReactiveTerminal::Try(t) => contains_return(&t.block) || contains_return(&t.handler),
        ReactiveTerminal::Return(_) => true,
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Throw(_) => false,
    }
}

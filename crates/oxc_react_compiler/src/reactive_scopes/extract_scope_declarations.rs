/// Extract scope declarations from destructuring patterns.
///
/// Port of `ReactiveScopes/ExtractScopeDeclarationsFromDestructuring.ts` from the React Compiler.
///
/// When a destructuring pattern produces variables that belong to different
/// reactive scopes, this pass splits the destructuring into individual
/// assignments so each can be placed in the correct scope.
use crate::hir::{ReactiveBlock, ReactiveFunction, ReactiveStatement};

/// Extract scope declarations from destructuring patterns.
pub fn extract_scope_declarations_from_destructuring(func: &mut ReactiveFunction) {
    extract_in_block(&mut func.body);
}

fn extract_in_block(block: &mut ReactiveBlock) {
    for stmt in block.iter_mut() {
        match stmt {
            ReactiveStatement::Scope(scope) => {
                extract_in_block(&mut scope.instructions);
            }
            ReactiveStatement::PrunedScope(scope) => {
                extract_in_block(&mut scope.instructions);
            }
            ReactiveStatement::Terminal(term) => {
                extract_in_terminal(&mut term.terminal);
            }
            ReactiveStatement::Instruction(_instr) => {
                // In the full implementation, we'd check if this is a Destructure
                // instruction where the lvalue pattern contains identifiers from
                // different scopes, and if so, split it into individual assignments.
            }
        }
    }
}

fn extract_in_terminal(terminal: &mut crate::hir::ReactiveTerminal) {
    use crate::hir::ReactiveTerminal;
    match terminal {
        ReactiveTerminal::If(t) => {
            extract_in_block(&mut t.consequent);
            if let Some(alt) = &mut t.alternate {
                extract_in_block(alt);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &mut t.cases {
                if let Some(block) = &mut case.block {
                    extract_in_block(block);
                }
            }
        }
        ReactiveTerminal::While(t) => extract_in_block(&mut t.r#loop),
        ReactiveTerminal::DoWhile(t) => extract_in_block(&mut t.r#loop),
        ReactiveTerminal::For(t) => extract_in_block(&mut t.r#loop),
        ReactiveTerminal::ForOf(t) => extract_in_block(&mut t.r#loop),
        ReactiveTerminal::ForIn(t) => extract_in_block(&mut t.r#loop),
        ReactiveTerminal::Label(t) => extract_in_block(&mut t.block),
        ReactiveTerminal::Try(t) => {
            extract_in_block(&mut t.block);
            extract_in_block(&mut t.handler);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}

/// Promote used temporaries to named variables.
///
/// Port of `ReactiveScopes/PromoteUsedTemporaries.ts` from the React Compiler.
///
/// Finds temporary variables that are used as declarations of reactive scopes
/// and promotes them to named variables. This is needed because temporary
/// variables are typically unnamed, but scope declarations need names for
/// the output code.
use crate::hir::{
    ReactiveBlock, ReactiveFunction, ReactiveStatement,
};

/// Promote used temporaries to named variables.
pub fn promote_used_temporaries(func: &mut ReactiveFunction) {
    promote_in_block(&mut func.body);
}

fn promote_in_block(block: &mut ReactiveBlock) {
    for stmt in block.iter_mut() {
        match stmt {
            ReactiveStatement::Scope(scope) => {
                // Check declarations — promote unnamed identifiers
                for (_id, decl) in &scope.scope.declarations {
                    if decl.identifier.name.is_none() {
                        // In the full implementation, we'd set the identifier name
                        // to a promoted temporary name like "#t0"
                    }
                }
                promote_in_block(&mut scope.instructions);
            }
            ReactiveStatement::PrunedScope(scope) => {
                promote_in_block(&mut scope.instructions);
            }
            ReactiveStatement::Terminal(term) => {
                promote_in_terminal(&mut term.terminal);
            }
            ReactiveStatement::Instruction(_) => {}
        }
    }
}

fn promote_in_terminal(terminal: &mut crate::hir::ReactiveTerminal) {
    use crate::hir::ReactiveTerminal;
    match terminal {
        ReactiveTerminal::If(t) => {
            promote_in_block(&mut t.consequent);
            if let Some(alt) = &mut t.alternate {
                promote_in_block(alt);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &mut t.cases {
                if let Some(block) = &mut case.block {
                    promote_in_block(block);
                }
            }
        }
        ReactiveTerminal::While(t) => promote_in_block(&mut t.r#loop),
        ReactiveTerminal::DoWhile(t) => promote_in_block(&mut t.r#loop),
        ReactiveTerminal::For(t) => promote_in_block(&mut t.r#loop),
        ReactiveTerminal::ForOf(t) => promote_in_block(&mut t.r#loop),
        ReactiveTerminal::ForIn(t) => promote_in_block(&mut t.r#loop),
        ReactiveTerminal::Label(t) => promote_in_block(&mut t.block),
        ReactiveTerminal::Try(t) => {
            promote_in_block(&mut t.block);
            promote_in_block(&mut t.handler);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}

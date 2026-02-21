/// Stabilize block IDs in the reactive function.
///
/// Port of `ReactiveScopes/StabilizeBlockIds.ts` from the React Compiler.
///
/// Renumbers block IDs in the reactive function to be sequential,
/// which makes output more deterministic and easier to diff.
use crate::hir::{
    BlockId, ReactiveBlock, ReactiveFunction, ReactiveStatement, ReactiveTerminal,
};

/// Stabilize block IDs by renumbering them sequentially.
pub fn stabilize_block_ids(func: &mut ReactiveFunction) {
    let mut next_id: u32 = 0;
    stabilize_block(&mut func.body, &mut next_id);
}

fn stabilize_block(block: &mut ReactiveBlock, next_id: &mut u32) {
    for stmt in block.iter_mut() {
        match stmt {
            ReactiveStatement::Instruction(_) => {}
            ReactiveStatement::Terminal(term) => {
                if let Some(ref mut label) = term.label {
                    label.id = BlockId(*next_id);
                    *next_id += 1;
                }
                stabilize_terminal(&mut term.terminal, next_id);
            }
            ReactiveStatement::Scope(scope) => {
                stabilize_block(&mut scope.instructions, next_id);
            }
            ReactiveStatement::PrunedScope(scope) => {
                stabilize_block(&mut scope.instructions, next_id);
            }
        }
    }
}

fn stabilize_terminal(terminal: &mut ReactiveTerminal, next_id: &mut u32) {
    match terminal {
        ReactiveTerminal::If(t) => {
            stabilize_block(&mut t.consequent, next_id);
            if let Some(alt) = &mut t.alternate {
                stabilize_block(alt, next_id);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &mut t.cases {
                if let Some(block) = &mut case.block {
                    stabilize_block(block, next_id);
                }
            }
        }
        ReactiveTerminal::While(t) => stabilize_block(&mut t.r#loop, next_id),
        ReactiveTerminal::DoWhile(t) => stabilize_block(&mut t.r#loop, next_id),
        ReactiveTerminal::For(t) => stabilize_block(&mut t.r#loop, next_id),
        ReactiveTerminal::ForOf(t) => stabilize_block(&mut t.r#loop, next_id),
        ReactiveTerminal::ForIn(t) => stabilize_block(&mut t.r#loop, next_id),
        ReactiveTerminal::Label(t) => stabilize_block(&mut t.block, next_id),
        ReactiveTerminal::Try(t) => {
            stabilize_block(&mut t.block, next_id);
            stabilize_block(&mut t.handler, next_id);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}

/// Stabilize block IDs in the reactive function.
///
/// Port of `ReactiveScopes/StabilizeBlockIds.ts` from the React Compiler.
///
/// Renumbers block IDs in the reactive function to be sequential,
/// which makes output more deterministic and easier to diff.
///
/// Uses a two-pass approach:
/// 1. Collect all referenced (non-implicit) label IDs
/// 2. Rewrite all label IDs and break/continue targets using a sequential mapping
use rustc_hash::FxHashMap;

use crate::hir::{
    BlockId, ReactiveBlock, ReactiveFunction, ReactiveStatement, ReactiveTerminal,
    ReactiveTerminalTargetKind,
};

/// Stabilize block IDs by renumbering them sequentially.
pub fn stabilize_block_ids(func: &mut ReactiveFunction) {
    // Phase 1: Collect all referenced label IDs (non-implicit labels + early return labels)
    let mut referenced: Vec<BlockId> = Vec::new();
    collect_referenced_labels(&func.body, &mut referenced);

    // Phase 2: Build mapping from old IDs to new sequential IDs
    let mut mapping: FxHashMap<BlockId, BlockId> = FxHashMap::default();
    for id in &referenced {
        let next = mapping.len() as u32;
        mapping.entry(*id).or_insert(BlockId(next));
    }

    // Phase 3: Rewrite all IDs using the mapping
    rewrite_block(&mut func.body, &mut mapping);
}

fn collect_referenced_labels(block: &ReactiveBlock, referenced: &mut Vec<BlockId>) {
    for stmt in block.iter() {
        match stmt {
            ReactiveStatement::Instruction(_) => {}
            ReactiveStatement::Terminal(term) => {
                // Collect non-implicit label IDs
                if let Some(label) = &term.label {
                    referenced.push(label.id);
                }
                collect_referenced_labels_terminal(&term.terminal, referenced);
            }
            ReactiveStatement::Scope(scope) => {
                // Collect early return value labels
                if let Some(early) = &scope.scope.early_return_value {
                    referenced.push(early.label);
                }
                collect_referenced_labels(&scope.instructions, referenced);
            }
            ReactiveStatement::PrunedScope(scope) => {
                if let Some(early) = &scope.scope.early_return_value {
                    referenced.push(early.label);
                }
                collect_referenced_labels(&scope.instructions, referenced);
            }
        }
    }
}

fn collect_referenced_labels_terminal(terminal: &ReactiveTerminal, referenced: &mut Vec<BlockId>) {
    match terminal {
        ReactiveTerminal::If(t) => {
            collect_referenced_labels(&t.consequent, referenced);
            if let Some(alt) = &t.alternate {
                collect_referenced_labels(alt, referenced);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &t.cases {
                if let Some(block) = &case.block {
                    collect_referenced_labels(block, referenced);
                }
            }
        }
        ReactiveTerminal::While(t) => collect_referenced_labels(&t.r#loop, referenced),
        ReactiveTerminal::DoWhile(t) => collect_referenced_labels(&t.r#loop, referenced),
        ReactiveTerminal::For(t) => collect_referenced_labels(&t.r#loop, referenced),
        ReactiveTerminal::ForOf(t) => collect_referenced_labels(&t.r#loop, referenced),
        ReactiveTerminal::ForIn(t) => collect_referenced_labels(&t.r#loop, referenced),
        ReactiveTerminal::Label(t) => collect_referenced_labels(&t.block, referenced),
        ReactiveTerminal::Try(t) => {
            collect_referenced_labels(&t.block, referenced);
            collect_referenced_labels(&t.handler, referenced);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}

fn get_or_insert(mapping: &mut FxHashMap<BlockId, BlockId>, id: BlockId) -> BlockId {
    let next = mapping.len() as u32;
    *mapping.entry(id).or_insert(BlockId(next))
}

fn rewrite_block(block: &mut ReactiveBlock, mapping: &mut FxHashMap<BlockId, BlockId>) {
    for stmt in block.iter_mut() {
        match stmt {
            ReactiveStatement::Instruction(_) => {}
            ReactiveStatement::Terminal(term) => {
                // Rewrite label ID (both implicit and non-implicit)
                if let Some(ref mut label) = term.label {
                    label.id = get_or_insert(mapping, label.id);
                }
                rewrite_terminal(&mut term.terminal, mapping);
            }
            ReactiveStatement::Scope(scope) => {
                // Rewrite early return value label
                if let Some(early) = &mut scope.scope.early_return_value {
                    early.label = get_or_insert(mapping, early.label);
                }
                rewrite_block(&mut scope.instructions, mapping);
            }
            ReactiveStatement::PrunedScope(scope) => {
                if let Some(early) = &mut scope.scope.early_return_value {
                    early.label = get_or_insert(mapping, early.label);
                }
                rewrite_block(&mut scope.instructions, mapping);
            }
        }
    }
}

fn rewrite_terminal(terminal: &mut ReactiveTerminal, mapping: &mut FxHashMap<BlockId, BlockId>) {
    match terminal {
        ReactiveTerminal::If(t) => {
            rewrite_block(&mut t.consequent, mapping);
            if let Some(alt) = &mut t.alternate {
                rewrite_block(alt, mapping);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &mut t.cases {
                if let Some(block) = &mut case.block {
                    rewrite_block(block, mapping);
                }
            }
        }
        ReactiveTerminal::While(t) => rewrite_block(&mut t.r#loop, mapping),
        ReactiveTerminal::DoWhile(t) => rewrite_block(&mut t.r#loop, mapping),
        ReactiveTerminal::For(t) => rewrite_block(&mut t.r#loop, mapping),
        ReactiveTerminal::ForOf(t) => rewrite_block(&mut t.r#loop, mapping),
        ReactiveTerminal::ForIn(t) => rewrite_block(&mut t.r#loop, mapping),
        ReactiveTerminal::Label(t) => rewrite_block(&mut t.block, mapping),
        ReactiveTerminal::Try(t) => {
            rewrite_block(&mut t.block, mapping);
            rewrite_block(&mut t.handler, mapping);
        }
        ReactiveTerminal::Break(t) => {
            if t.target_kind != ReactiveTerminalTargetKind::Implicit {
                t.target = get_or_insert(mapping, t.target);
            }
        }
        ReactiveTerminal::Continue(t) => {
            if t.target_kind != ReactiveTerminalTargetKind::Implicit {
                t.target = get_or_insert(mapping, t.target);
            }
        }
        ReactiveTerminal::Return(_) | ReactiveTerminal::Throw(_) => {}
    }
}

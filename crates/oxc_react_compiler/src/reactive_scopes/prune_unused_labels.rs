/// Prune unused labels from the reactive function.
///
/// Port of `ReactiveScopes/PruneUnusedLabels.ts` from the React Compiler.
///
/// Flattens labeled terminals where the label is not reachable via any
/// break/continue, and marks labels as implicit when they are unused.
use rustc_hash::FxHashSet;

use crate::hir::{
    BlockId, ReactiveBlock, ReactiveFunction, ReactiveStatement, ReactiveTerminal,
    ReactiveTerminalTargetKind,
};

/// Prune unused labels from the reactive function.
pub fn prune_unused_labels(func: &mut ReactiveFunction) {
    let mut labels: FxHashSet<BlockId> = FxHashSet::default();
    // First pass: collect all targeted labels
    collect_targeted_labels(&func.body, &mut labels);
    // Second pass: prune unused labels
    transform_block(&mut func.body, &labels);
}

fn collect_targeted_labels(block: &ReactiveBlock, labels: &mut FxHashSet<BlockId>) {
    for stmt in block {
        match stmt {
            ReactiveStatement::Terminal(term_stmt) => {
                match &term_stmt.terminal {
                    ReactiveTerminal::Break(t) => {
                        if t.target_kind == ReactiveTerminalTargetKind::Labeled {
                            labels.insert(t.target);
                        }
                    }
                    ReactiveTerminal::Continue(t) => {
                        if t.target_kind == ReactiveTerminalTargetKind::Labeled {
                            labels.insert(t.target);
                        }
                    }
                    _ => {}
                }
                collect_targeted_labels_in_terminal(&term_stmt.terminal, labels);
            }
            ReactiveStatement::Scope(scope) => {
                collect_targeted_labels(&scope.instructions, labels);
            }
            ReactiveStatement::PrunedScope(scope) => {
                collect_targeted_labels(&scope.instructions, labels);
            }
            ReactiveStatement::Instruction(_) => {}
        }
    }
}

fn collect_targeted_labels_in_terminal(
    terminal: &ReactiveTerminal,
    labels: &mut FxHashSet<BlockId>,
) {
    match terminal {
        ReactiveTerminal::If(t) => {
            collect_targeted_labels(&t.consequent, labels);
            if let Some(alt) = &t.alternate {
                collect_targeted_labels(alt, labels);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &t.cases {
                if let Some(block) = &case.block {
                    collect_targeted_labels(block, labels);
                }
            }
        }
        ReactiveTerminal::While(t) => collect_targeted_labels(&t.r#loop, labels),
        ReactiveTerminal::DoWhile(t) => collect_targeted_labels(&t.r#loop, labels),
        ReactiveTerminal::For(t) => collect_targeted_labels(&t.r#loop, labels),
        ReactiveTerminal::ForOf(t) => collect_targeted_labels(&t.r#loop, labels),
        ReactiveTerminal::ForIn(t) => collect_targeted_labels(&t.r#loop, labels),
        ReactiveTerminal::Label(t) => collect_targeted_labels(&t.block, labels),
        ReactiveTerminal::Try(t) => {
            collect_targeted_labels(&t.block, labels);
            collect_targeted_labels(&t.handler, labels);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}

fn transform_block(block: &mut ReactiveBlock, labels: &FxHashSet<BlockId>) {
    let mut i = 0;
    while i < block.len() {
        let should_flatten = {
            if let ReactiveStatement::Terminal(term_stmt) = &block[i] {
                let is_reachable_label =
                    term_stmt.label.as_ref().is_some_and(|l| labels.contains(&l.id));

                if let ReactiveTerminal::Label(_) = &term_stmt.terminal {
                    !is_reachable_label
                } else {
                    false
                }
            } else {
                false
            }
        };

        if should_flatten {
            // Extract the label terminal's block and flatten it
            let stmt = block.remove(i);
            if let ReactiveStatement::Terminal(term_stmt) = stmt
                && let ReactiveTerminal::Label(label_term) = term_stmt.terminal
            {
                let mut inner_block = label_term.block;
                // Remove trailing break with null target (implicit break)
                if let Some(last) = inner_block.last()
                    && let ReactiveStatement::Terminal(last_term) = last
                    && let ReactiveTerminal::Break(b) = &last_term.terminal
                    && b.target_kind == ReactiveTerminalTargetKind::Implicit
                {
                    inner_block.pop();
                }
                // Transform children before inserting
                transform_block(&mut inner_block, labels);
                let count = inner_block.len();
                for (j, s) in inner_block.into_iter().enumerate() {
                    block.insert(i + j, s);
                }
                i += count;
            }
        } else {
            // Mark unused labels as implicit
            if let ReactiveStatement::Terminal(term_stmt) = &mut block[i] {
                let is_reachable_label =
                    term_stmt.label.as_ref().is_some_and(|l| labels.contains(&l.id));
                if !is_reachable_label && let Some(ref mut label) = term_stmt.label {
                    label.implicit = true;
                }
                transform_terminal_children(&mut term_stmt.terminal, labels);
            }
            match &mut block[i] {
                ReactiveStatement::Scope(scope) => {
                    transform_block(&mut scope.instructions, labels);
                }
                ReactiveStatement::PrunedScope(scope) => {
                    transform_block(&mut scope.instructions, labels);
                }
                _ => {}
            }
            i += 1;
        }
    }
}

fn transform_terminal_children(terminal: &mut ReactiveTerminal, labels: &FxHashSet<BlockId>) {
    match terminal {
        ReactiveTerminal::If(t) => {
            transform_block(&mut t.consequent, labels);
            if let Some(alt) = &mut t.alternate {
                transform_block(alt, labels);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &mut t.cases {
                if let Some(block) = &mut case.block {
                    transform_block(block, labels);
                }
            }
        }
        ReactiveTerminal::While(t) => transform_block(&mut t.r#loop, labels),
        ReactiveTerminal::DoWhile(t) => transform_block(&mut t.r#loop, labels),
        ReactiveTerminal::For(t) => transform_block(&mut t.r#loop, labels),
        ReactiveTerminal::ForOf(t) => transform_block(&mut t.r#loop, labels),
        ReactiveTerminal::ForIn(t) => transform_block(&mut t.r#loop, labels),
        ReactiveTerminal::Label(t) => transform_block(&mut t.block, labels),
        ReactiveTerminal::Try(t) => {
            transform_block(&mut t.block, labels);
            transform_block(&mut t.handler, labels);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}

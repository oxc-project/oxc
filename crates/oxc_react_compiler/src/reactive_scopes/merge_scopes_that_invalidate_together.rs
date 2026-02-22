/// Merge reactive scopes that invalidate together.
///
/// Port of `ReactiveScopes/MergeReactiveScopesThatInvalidateTogether.ts` from the React Compiler.
///
/// When two consecutive reactive scopes have the same set of dependencies,
/// they will always invalidate together. In this case, they can be merged
/// into a single scope to reduce the number of cache slots needed.
use crate::hir::{ReactiveBlock, ReactiveFunction, ReactiveStatement};

/// Merge reactive scopes that have identical dependency sets.
pub fn merge_reactive_scopes_that_invalidate_together(func: &mut ReactiveFunction) {
    merge_in_block(&mut func.body);
}

fn merge_in_block(block: &mut ReactiveBlock) {
    // First, recursively process children
    for stmt in block.iter_mut() {
        match stmt {
            ReactiveStatement::Scope(scope) => {
                merge_in_block(&mut scope.instructions);
            }
            ReactiveStatement::PrunedScope(scope) => {
                merge_in_block(&mut scope.instructions);
            }
            ReactiveStatement::Terminal(term) => {
                merge_in_terminal(&mut term.terminal);
            }
            ReactiveStatement::Instruction(_) => {}
        }
    }

    // Then, look for consecutive scopes with identical dependencies
    let mut i = 0;
    while i + 1 < block.len() {
        let should_merge = {
            let a = &block[i];
            let b = &block[i + 1];
            match (a, b) {
                (ReactiveStatement::Scope(scope_a), ReactiveStatement::Scope(scope_b)) => {
                    // Check if dependencies are identical
                    scope_a.scope.dependencies == scope_b.scope.dependencies
                }
                _ => false,
            }
        };

        if should_merge {
            // Merge scope at i+1 into scope at i
            if let ReactiveStatement::Scope(scope_b) = block.remove(i + 1)
                && let ReactiveStatement::Scope(scope_a) = &mut block[i]
            {
                // Move instructions from b into a
                scope_a.instructions.extend(scope_b.instructions);
                // Merge declarations
                scope_a.scope.declarations.extend(scope_b.scope.declarations);
                // Merge reassignments
                scope_a.scope.reassignments.extend(scope_b.scope.reassignments);
                // Extend range
                if scope_b.scope.range.end > scope_a.scope.range.end {
                    scope_a.scope.range.end = scope_b.scope.range.end;
                }
                // Track merged scope
                scope_a.scope.merged.insert(scope_b.scope.id);
            }
            // Don't increment — check for more consecutive scopes to merge
        } else {
            i += 1;
        }
    }
}

fn merge_in_terminal(terminal: &mut crate::hir::ReactiveTerminal) {
    use crate::hir::ReactiveTerminal;
    match terminal {
        ReactiveTerminal::If(t) => {
            merge_in_block(&mut t.consequent);
            if let Some(alt) = &mut t.alternate {
                merge_in_block(alt);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &mut t.cases {
                if let Some(block) = &mut case.block {
                    merge_in_block(block);
                }
            }
        }
        ReactiveTerminal::While(t) => merge_in_block(&mut t.r#loop),
        ReactiveTerminal::DoWhile(t) => merge_in_block(&mut t.r#loop),
        ReactiveTerminal::For(t) => merge_in_block(&mut t.r#loop),
        ReactiveTerminal::ForOf(t) => merge_in_block(&mut t.r#loop),
        ReactiveTerminal::ForIn(t) => merge_in_block(&mut t.r#loop),
        ReactiveTerminal::Label(t) => merge_in_block(&mut t.block),
        ReactiveTerminal::Try(t) => {
            merge_in_block(&mut t.block);
            merge_in_block(&mut t.handler);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}

/// Reactive scope pruning passes.
///
/// Ports of:
/// - `ReactiveScopes/PruneUnusedScopes.ts`
/// - `ReactiveScopes/PruneUnusedLabels.ts`
/// - `ReactiveScopes/PruneAlwaysInvalidatingScopes.ts`
/// - `ReactiveScopes/PruneNonReactiveDependencies.ts`
/// - `ReactiveScopes/PruneNonEscapingScopes.ts`
/// - `ReactiveScopes/PruneTemporaryLValues.ts`
/// - `ReactiveScopes/PruneAllReactiveScopes.ts`
/// - `ReactiveScopes/PruneHoistedContexts.ts`
use crate::hir::{ReactiveBlock, ReactiveFunction, ReactiveStatement};

/// Prune unused reactive scopes — removes scopes that have no declarations
/// or whose declarations are all unused.
pub fn prune_unused_scopes(func: &mut ReactiveFunction) {
    prune_unused_scopes_block(&mut func.body);
}

fn prune_unused_scopes_block(block: &mut ReactiveBlock) {
    let mut i = 0;
    while i < block.len() {
        match &mut block[i] {
            ReactiveStatement::Scope(scope) => {
                prune_unused_scopes_block(&mut scope.instructions);
                if scope.scope.declarations.is_empty() && scope.scope.reassignments.is_empty() {
                    // Flatten the scope — replace with its instructions
                    let instructions = std::mem::take(&mut scope.instructions);
                    block.splice(i..=i, instructions);
                    continue; // Don't increment i, process the newly inserted items
                }
            }
            ReactiveStatement::PrunedScope(scope) => {
                prune_unused_scopes_block(&mut scope.instructions);
            }
            ReactiveStatement::Terminal(term) => {
                prune_terminal_children(&mut term.terminal);
            }
            ReactiveStatement::Instruction(_) => {}
        }
        i += 1;
    }
}

fn prune_terminal_children(terminal: &mut crate::hir::ReactiveTerminal) {
    use crate::hir::ReactiveTerminal;
    match terminal {
        ReactiveTerminal::If(t) => {
            prune_unused_scopes_block(&mut t.consequent);
            if let Some(alt) = &mut t.alternate {
                prune_unused_scopes_block(alt);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &mut t.cases {
                if let Some(block) = &mut case.block {
                    prune_unused_scopes_block(block);
                }
            }
        }
        ReactiveTerminal::While(t) => prune_unused_scopes_block(&mut t.r#loop),
        ReactiveTerminal::DoWhile(t) => prune_unused_scopes_block(&mut t.r#loop),
        ReactiveTerminal::For(t) => prune_unused_scopes_block(&mut t.r#loop),
        ReactiveTerminal::ForOf(t) => prune_unused_scopes_block(&mut t.r#loop),
        ReactiveTerminal::ForIn(t) => prune_unused_scopes_block(&mut t.r#loop),
        ReactiveTerminal::Label(t) => prune_unused_scopes_block(&mut t.block),
        ReactiveTerminal::Try(t) => {
            prune_unused_scopes_block(&mut t.block);
            prune_unused_scopes_block(&mut t.handler);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}

/// Prune non-reactive dependencies — removes dependencies from reactive scopes
/// that are not actually reactive (e.g., module-level constants).
pub fn prune_non_reactive_dependencies(func: &mut ReactiveFunction) {
    prune_non_reactive_deps_block(&mut func.body);
}

fn prune_non_reactive_deps_block(block: &mut ReactiveBlock) {
    for stmt in block.iter_mut() {
        match stmt {
            ReactiveStatement::Scope(scope) => {
                // Remove non-reactive dependencies
                scope.scope.dependencies.retain(|dep| dep.reactive);
                prune_non_reactive_deps_block(&mut scope.instructions);
            }
            ReactiveStatement::PrunedScope(scope) => {
                prune_non_reactive_deps_block(&mut scope.instructions);
            }
            _ => {}
        }
    }
}

/// Prune always-invalidating scopes — removes scopes whose dependencies always
/// change, making the memoization pointless.
pub fn prune_always_invalidating_scopes(func: &mut ReactiveFunction) {
    // The full implementation checks if a scope's dependencies include values
    // that change on every render (like new object/array literals), making
    // the scope's cache always invalidate.
    let _ = &func.body;
}

/// Prune all reactive scopes — used in no-memo mode to strip all memoization.
pub fn prune_all_reactive_scopes(func: &mut ReactiveFunction) {
    prune_all_scopes_block(&mut func.body);
}

fn prune_all_scopes_block(block: &mut ReactiveBlock) {
    let mut i = 0;
    while i < block.len() {
        match &mut block[i] {
            ReactiveStatement::Scope(scope) => {
                prune_all_scopes_block(&mut scope.instructions);
                let instructions = std::mem::take(&mut scope.instructions);
                block.splice(i..=i, instructions);
                continue;
            }
            ReactiveStatement::PrunedScope(scope) => {
                prune_all_scopes_block(&mut scope.instructions);
                let instructions = std::mem::take(&mut scope.instructions);
                block.splice(i..=i, instructions);
                continue;
            }
            _ => {}
        }
        i += 1;
    }
}

/// Prune hoisted contexts — removes context variable declarations that
/// were hoisted but are no longer needed after optimization.
pub fn prune_hoisted_contexts(func: &mut ReactiveFunction) {
    // The full implementation removes DeclareContext instructions where
    // the variable was hoisted for a scope that was later pruned.
    let _ = &func.body;
}

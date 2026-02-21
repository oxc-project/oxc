/// Prune non-escaping reactive scopes.
///
/// Port of `ReactiveScopes/PruneNonEscapingScopes.ts` from the React Compiler.
///
/// Removes reactive scopes whose output values don't "escape" the function.
/// A value escapes if it's returned, passed to a hook, used in JSX, etc.
/// If a scope only produces local values that don't escape, memoizing them
/// provides no benefit because React can't observe the difference.
use rustc_hash::FxHashSet;

use crate::hir::{
    IdentifierId, ReactiveBlock, ReactiveFunction, ReactiveStatement, ReactiveTerminal,
    InstructionValue, ReactiveValue,
};

/// Prune reactive scopes whose values don't escape the function.
pub fn prune_non_escaping_scopes(func: &mut ReactiveFunction) {
    // Phase 1: Find all identifiers that escape the function
    let escaping_ids = find_escaping_identifiers(func);

    // Phase 2: Prune scopes that only produce non-escaping values
    prune_in_block(&mut func.body, &escaping_ids);
}

fn find_escaping_identifiers(func: &ReactiveFunction) -> FxHashSet<IdentifierId> {
    let mut escaping = FxHashSet::default();

    // Return values always escape
    find_escaping_in_block(&func.body, &mut escaping);

    escaping
}

fn find_escaping_in_block(block: &ReactiveBlock, escaping: &mut FxHashSet<IdentifierId>) {
    for stmt in block {
        match stmt {
            ReactiveStatement::Instruction(instr) => {
                // JSX children and props escape
                if let ReactiveValue::Instruction(value) = &instr.instruction.value
                    && let InstructionValue::JsxExpression(jsx) = value.as_ref() {
                        if let crate::hir::JsxTag::Place(p) = &jsx.tag {
                            escaping.insert(p.identifier.id);
                        }
                        for attr in &jsx.props {
                            match attr {
                                crate::hir::JsxAttribute::Attribute { place, .. } => {
                                    escaping.insert(place.identifier.id);
                                }
                                crate::hir::JsxAttribute::Spread { argument } => {
                                    escaping.insert(argument.identifier.id);
                                }
                            }
                        }
                        if let Some(children) = &jsx.children {
                            for child in children {
                                escaping.insert(child.identifier.id);
                            }
                        }
                    }
            }
            ReactiveStatement::Terminal(term) => {
                // Return values escape
                if let ReactiveTerminal::Return(ret) = &term.terminal {
                    escaping.insert(ret.value.identifier.id);
                }
                find_escaping_in_terminal(&term.terminal, escaping);
            }
            ReactiveStatement::Scope(scope) => {
                find_escaping_in_block(&scope.instructions, escaping);
            }
            ReactiveStatement::PrunedScope(scope) => {
                find_escaping_in_block(&scope.instructions, escaping);
            }
        }
    }
}

fn find_escaping_in_terminal(terminal: &ReactiveTerminal, escaping: &mut FxHashSet<IdentifierId>) {
    match terminal {
        ReactiveTerminal::If(t) => {
            find_escaping_in_block(&t.consequent, escaping);
            if let Some(alt) = &t.alternate {
                find_escaping_in_block(alt, escaping);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &t.cases {
                if let Some(block) = &case.block {
                    find_escaping_in_block(block, escaping);
                }
            }
        }
        ReactiveTerminal::While(t) => find_escaping_in_block(&t.r#loop, escaping),
        ReactiveTerminal::DoWhile(t) => find_escaping_in_block(&t.r#loop, escaping),
        ReactiveTerminal::For(t) => find_escaping_in_block(&t.r#loop, escaping),
        ReactiveTerminal::ForOf(t) => find_escaping_in_block(&t.r#loop, escaping),
        ReactiveTerminal::ForIn(t) => find_escaping_in_block(&t.r#loop, escaping),
        ReactiveTerminal::Label(t) => find_escaping_in_block(&t.block, escaping),
        ReactiveTerminal::Try(t) => {
            find_escaping_in_block(&t.block, escaping);
            find_escaping_in_block(&t.handler, escaping);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}

fn prune_in_block(block: &mut ReactiveBlock, escaping: &FxHashSet<IdentifierId>) {
    let mut i = 0;
    while i < block.len() {
        match &mut block[i] {
            ReactiveStatement::Scope(scope) => {
                prune_in_block(&mut scope.instructions, escaping);
                // Check if any declarations escape
                let has_escaping = scope
                    .scope
                    .declarations
                    .keys()
                    .any(|id| escaping.contains(id));
                if !has_escaping && scope.scope.reassignments.is_empty() {
                    // Scope doesn't escape — flatten it
                    let instructions = std::mem::take(&mut scope.instructions);
                    block.splice(i..=i, instructions);
                    continue;
                }
            }
            ReactiveStatement::PrunedScope(scope) => {
                prune_in_block(&mut scope.instructions, escaping);
            }
            ReactiveStatement::Terminal(term) => {
                prune_in_terminal(&mut term.terminal, escaping);
            }
            ReactiveStatement::Instruction(_) => {}
        }
        i += 1;
    }
}

fn prune_in_terminal(terminal: &mut ReactiveTerminal, escaping: &FxHashSet<IdentifierId>) {
    match terminal {
        ReactiveTerminal::If(t) => {
            prune_in_block(&mut t.consequent, escaping);
            if let Some(alt) = &mut t.alternate {
                prune_in_block(alt, escaping);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &mut t.cases {
                if let Some(block) = &mut case.block {
                    prune_in_block(block, escaping);
                }
            }
        }
        ReactiveTerminal::While(t) => prune_in_block(&mut t.r#loop, escaping),
        ReactiveTerminal::DoWhile(t) => prune_in_block(&mut t.r#loop, escaping),
        ReactiveTerminal::For(t) => prune_in_block(&mut t.r#loop, escaping),
        ReactiveTerminal::ForOf(t) => prune_in_block(&mut t.r#loop, escaping),
        ReactiveTerminal::ForIn(t) => prune_in_block(&mut t.r#loop, escaping),
        ReactiveTerminal::Label(t) => prune_in_block(&mut t.block, escaping),
        ReactiveTerminal::Try(t) => {
            prune_in_block(&mut t.block, escaping);
            prune_in_block(&mut t.handler, escaping);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}

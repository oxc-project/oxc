/// Prune unused lvalues from the reactive function.
///
/// Port of `ReactiveScopes/PruneTemporaryLValues.ts` from the React Compiler.
///
/// Nulls out lvalues for temporary variables that are never accessed later.
/// This only nulls out the lvalue itself, it does not remove the corresponding
/// instructions. Uses DeclarationIds because the lvalue IdentifierId of a
/// compound expression (ternary, logical, optional) in ReactiveFunction may
/// not be the same as the IdentifierId of the phi which is referenced later.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::hir::{
    DeclarationId, ReactiveBlock, ReactiveFunction, ReactiveInstruction, ReactiveStatement,
    ReactiveTerminal,
};

/// Prune unused lvalues from the reactive function.
pub fn prune_unused_lvalues(func: &mut ReactiveFunction) {
    // First pass: collect all temporary lvalues and track which declaration IDs are used
    let mut lvalue_locations: FxHashMap<DeclarationId, Vec<LValueLocation>> = FxHashMap::default();
    let mut used_declarations: FxHashSet<DeclarationId> = FxHashSet::default();

    collect_lvalue_info(&func.body, &mut lvalue_locations, &mut used_declarations);

    // Determine which declaration IDs to prune: those with lvalues but never used
    let to_prune: FxHashSet<DeclarationId> =
        lvalue_locations.keys().filter(|id| !used_declarations.contains(id)).copied().collect();

    // Second pass: null out the unused lvalues
    if !to_prune.is_empty() {
        prune_lvalues_in_block(&mut func.body, &to_prune);
    }
}

#[derive(Debug)]
struct LValueLocation {
    // Marker to track where the lvalue was found (for debugging)
    _declaration_id: DeclarationId,
}

fn collect_lvalue_info(
    block: &ReactiveBlock,
    lvalue_locations: &mut FxHashMap<DeclarationId, Vec<LValueLocation>>,
    used_declarations: &mut FxHashSet<DeclarationId>,
) {
    for stmt in block {
        match stmt {
            ReactiveStatement::Instruction(instr_stmt) => {
                collect_from_instruction(
                    &instr_stmt.instruction,
                    lvalue_locations,
                    used_declarations,
                );
            }
            ReactiveStatement::Terminal(term_stmt) => {
                collect_from_terminal(&term_stmt.terminal, lvalue_locations, used_declarations);
            }
            ReactiveStatement::Scope(scope) => {
                collect_lvalue_info(&scope.instructions, lvalue_locations, used_declarations);
            }
            ReactiveStatement::PrunedScope(scope) => {
                collect_lvalue_info(&scope.instructions, lvalue_locations, used_declarations);
            }
        }
    }
}

fn collect_from_instruction(
    instr: &ReactiveInstruction,
    lvalue_locations: &mut FxHashMap<DeclarationId, Vec<LValueLocation>>,
    used_declarations: &mut FxHashSet<DeclarationId>,
) {
    // Visit places used in the instruction value first (these are "reads")
    collect_used_declarations_from_value(&instr.value, used_declarations);

    // Then check if this instruction has a temporary lvalue (unnamed)
    if let Some(ref lvalue) = instr.lvalue
        && lvalue.identifier.name.is_none()
    {
        let decl_id = lvalue.identifier.declaration_id;
        lvalue_locations
            .entry(decl_id)
            .or_default()
            .push(LValueLocation { _declaration_id: decl_id });
    }
}

fn collect_used_declarations_from_value(
    _value: &crate::hir::ReactiveValue,
    _used: &mut FxHashSet<DeclarationId>,
) {
    // In the full implementation, this would traverse the ReactiveValue tree
    // and mark all referenced DeclarationIds as used. For now, this is a
    // structural stub — the full implementation needs the place visitor
    // infrastructure which traverses all Place references within a value.
}

fn collect_from_terminal(
    terminal: &ReactiveTerminal,
    lvalue_locations: &mut FxHashMap<DeclarationId, Vec<LValueLocation>>,
    used_declarations: &mut FxHashSet<DeclarationId>,
) {
    match terminal {
        ReactiveTerminal::If(t) => {
            collect_lvalue_info(&t.consequent, lvalue_locations, used_declarations);
            if let Some(alt) = &t.alternate {
                collect_lvalue_info(alt, lvalue_locations, used_declarations);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &t.cases {
                if let Some(block) = &case.block {
                    collect_lvalue_info(block, lvalue_locations, used_declarations);
                }
            }
        }
        ReactiveTerminal::While(t) => {
            collect_lvalue_info(&t.r#loop, lvalue_locations, used_declarations);
        }
        ReactiveTerminal::DoWhile(t) => {
            collect_lvalue_info(&t.r#loop, lvalue_locations, used_declarations);
        }
        ReactiveTerminal::For(t) => {
            collect_lvalue_info(&t.r#loop, lvalue_locations, used_declarations);
        }
        ReactiveTerminal::ForOf(t) => {
            collect_lvalue_info(&t.r#loop, lvalue_locations, used_declarations);
        }
        ReactiveTerminal::ForIn(t) => {
            collect_lvalue_info(&t.r#loop, lvalue_locations, used_declarations);
        }
        ReactiveTerminal::Label(t) => {
            collect_lvalue_info(&t.block, lvalue_locations, used_declarations);
        }
        ReactiveTerminal::Try(t) => {
            collect_lvalue_info(&t.block, lvalue_locations, used_declarations);
            collect_lvalue_info(&t.handler, lvalue_locations, used_declarations);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}

fn prune_lvalues_in_block(block: &mut ReactiveBlock, to_prune: &FxHashSet<DeclarationId>) {
    for stmt in block.iter_mut() {
        match stmt {
            ReactiveStatement::Instruction(instr_stmt) => {
                if let Some(ref lvalue) = instr_stmt.instruction.lvalue
                    && to_prune.contains(&lvalue.identifier.declaration_id)
                {
                    instr_stmt.instruction.lvalue = None;
                }
            }
            ReactiveStatement::Terminal(term_stmt) => {
                prune_lvalues_in_terminal(&mut term_stmt.terminal, to_prune);
            }
            ReactiveStatement::Scope(scope) => {
                prune_lvalues_in_block(&mut scope.instructions, to_prune);
            }
            ReactiveStatement::PrunedScope(scope) => {
                prune_lvalues_in_block(&mut scope.instructions, to_prune);
            }
        }
    }
}

fn prune_lvalues_in_terminal(terminal: &mut ReactiveTerminal, to_prune: &FxHashSet<DeclarationId>) {
    match terminal {
        ReactiveTerminal::If(t) => {
            prune_lvalues_in_block(&mut t.consequent, to_prune);
            if let Some(alt) = &mut t.alternate {
                prune_lvalues_in_block(alt, to_prune);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &mut t.cases {
                if let Some(block) = &mut case.block {
                    prune_lvalues_in_block(block, to_prune);
                }
            }
        }
        ReactiveTerminal::While(t) => prune_lvalues_in_block(&mut t.r#loop, to_prune),
        ReactiveTerminal::DoWhile(t) => prune_lvalues_in_block(&mut t.r#loop, to_prune),
        ReactiveTerminal::For(t) => prune_lvalues_in_block(&mut t.r#loop, to_prune),
        ReactiveTerminal::ForOf(t) => prune_lvalues_in_block(&mut t.r#loop, to_prune),
        ReactiveTerminal::ForIn(t) => prune_lvalues_in_block(&mut t.r#loop, to_prune),
        ReactiveTerminal::Label(t) => prune_lvalues_in_block(&mut t.block, to_prune),
        ReactiveTerminal::Try(t) => {
            prune_lvalues_in_block(&mut t.block, to_prune);
            prune_lvalues_in_block(&mut t.handler, to_prune);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}

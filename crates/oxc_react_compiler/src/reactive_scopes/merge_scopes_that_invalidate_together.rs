/// Merge reactive scopes that invalidate together.
///
/// Port of `ReactiveScopes/MergeReactiveScopesThatInvalidateTogether.ts` from the React Compiler.
///
/// Merges in two main cases:
/// 1. **Consecutive scopes** with the same dependencies (or where outputs of one are inputs to the next)
/// 2. **Nested scopes** where inner scope has same dependencies as parent (flattened)
///
/// Allows safe intervening instructions between scopes (LoadLocal, PropertyLoad, etc.)
/// as long as their lvalues are only used within the next scope.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::compiler_error::SourceLocation;
use crate::hir::object_shape::{
    BUILT_IN_ARRAY_ID, BUILT_IN_FUNCTION_ID, BUILT_IN_JSX_ID, BUILT_IN_OBJECT_ID,
};
use crate::hir::types::{FunctionType, ObjectType, Type};
use crate::hir::{
    DeclarationId, InstructionId, InstructionKind, InstructionValue, ReactiveBlock,
    ReactiveFunction, ReactiveInstruction, ReactiveScope, ReactiveScopeBlock,
    ReactiveScopeDependency, ReactiveStatement,
};

/// Entry point: merge reactive scopes that invalidate together.
pub fn merge_reactive_scopes_that_invalidate_together(func: &mut ReactiveFunction) {
    // Pass 1: Pre-compute last usage of each declaration across the entire reactive tree
    let last_usage = find_last_usage(&func.body);

    // Pass 2: Flatten nested scopes with same deps as parent, then merge consecutive scopes
    merge_in_block(&mut func.body, None, &last_usage, &mut FxHashMap::default());
}

// =============================================================================
// Pass 1: FindLastUsageVisitor
// =============================================================================

/// Build a map from DeclarationId -> max InstructionId where that declaration is referenced.
fn find_last_usage(block: &ReactiveBlock) -> FxHashMap<DeclarationId, InstructionId> {
    let mut last_usage = FxHashMap::default();
    visit_block_for_usage(block, &mut last_usage);
    last_usage
}

fn visit_block_for_usage(
    block: &ReactiveBlock,
    last_usage: &mut FxHashMap<DeclarationId, InstructionId>,
) {
    for stmt in block {
        visit_stmt_for_usage(stmt, last_usage);
    }
}

fn visit_stmt_for_usage(
    stmt: &ReactiveStatement,
    last_usage: &mut FxHashMap<DeclarationId, InstructionId>,
) {
    match stmt {
        ReactiveStatement::Instruction(instr_stmt) => {
            visit_instruction_for_usage(&instr_stmt.instruction, last_usage);
        }
        ReactiveStatement::Terminal(term_stmt) => {
            // Visit terminal operands
            visit_terminal_for_usage(&term_stmt.terminal, last_usage);
        }
        ReactiveStatement::Scope(scope_block) => {
            visit_block_for_usage(&scope_block.instructions, last_usage);
        }
        ReactiveStatement::PrunedScope(pruned) => {
            visit_block_for_usage(&pruned.instructions, last_usage);
        }
    }
}

fn visit_instruction_for_usage(
    instr: &ReactiveInstruction,
    last_usage: &mut FxHashMap<DeclarationId, InstructionId>,
) {
    // Visit lvalue
    if let Some(lvalue) = &instr.lvalue {
        update_last_usage(last_usage, lvalue.identifier.declaration_id, instr.id);
    }
    // Visit operands in the value
    visit_reactive_value_for_usage(&instr.value, instr.id, last_usage);
}

fn visit_reactive_value_for_usage(
    value: &crate::hir::ReactiveValue,
    id: InstructionId,
    last_usage: &mut FxHashMap<DeclarationId, InstructionId>,
) {
    use crate::hir::ReactiveValue;
    match value {
        ReactiveValue::Instruction(instr_value) => {
            for place in crate::hir::visitors::each_instruction_value_operand(instr_value) {
                update_last_usage(last_usage, place.identifier.declaration_id, id);
            }
        }
        ReactiveValue::Logical(logical) => {
            visit_reactive_value_for_usage(&logical.left, id, last_usage);
            visit_reactive_value_for_usage(&logical.right, id, last_usage);
        }
        ReactiveValue::Sequence(seq) => {
            for instr in &seq.instructions {
                visit_instruction_for_usage(instr, last_usage);
            }
            visit_reactive_value_for_usage(&seq.value, id, last_usage);
        }
        ReactiveValue::Ternary(tern) => {
            visit_reactive_value_for_usage(&tern.test, id, last_usage);
            visit_reactive_value_for_usage(&tern.consequent, id, last_usage);
            visit_reactive_value_for_usage(&tern.alternate, id, last_usage);
        }
        ReactiveValue::OptionalCall(opt) => {
            visit_reactive_value_for_usage(&opt.value, id, last_usage);
        }
    }
}

fn visit_terminal_for_usage(
    terminal: &crate::hir::ReactiveTerminal,
    last_usage: &mut FxHashMap<DeclarationId, InstructionId>,
) {
    use crate::hir::ReactiveTerminal;

    let visit_place = |place: &crate::hir::Place,
                       id: InstructionId,
                       lu: &mut FxHashMap<DeclarationId, InstructionId>| {
        update_last_usage(lu, place.identifier.declaration_id, id);
    };

    let visit_rv = |rv: &crate::hir::ReactiveValue,
                    id: InstructionId,
                    lu: &mut FxHashMap<DeclarationId, InstructionId>| {
        visit_reactive_value_for_usage(rv, id, lu);
    };

    match terminal {
        ReactiveTerminal::If(t) => {
            visit_place(&t.test, t.id, last_usage);
            visit_block_for_usage(&t.consequent, last_usage);
            if let Some(alt) = &t.alternate {
                visit_block_for_usage(alt, last_usage);
            }
        }
        ReactiveTerminal::Switch(t) => {
            visit_place(&t.test, t.id, last_usage);
            for case in &t.cases {
                if let Some(test) = &case.test {
                    visit_place(test, t.id, last_usage);
                }
                if let Some(block) = &case.block {
                    visit_block_for_usage(block, last_usage);
                }
            }
        }
        ReactiveTerminal::While(t) => {
            visit_rv(&t.test, t.id, last_usage);
            visit_block_for_usage(&t.r#loop, last_usage);
        }
        ReactiveTerminal::DoWhile(t) => {
            visit_rv(&t.test, t.id, last_usage);
            visit_block_for_usage(&t.r#loop, last_usage);
        }
        ReactiveTerminal::For(t) => {
            visit_rv(&t.init, t.id, last_usage);
            visit_rv(&t.test, t.id, last_usage);
            if let Some(update) = &t.update {
                visit_rv(update, t.id, last_usage);
            }
            visit_block_for_usage(&t.r#loop, last_usage);
        }
        ReactiveTerminal::ForOf(t) => {
            visit_rv(&t.init, t.id, last_usage);
            visit_rv(&t.test, t.id, last_usage);
            visit_block_for_usage(&t.r#loop, last_usage);
        }
        ReactiveTerminal::ForIn(t) => {
            visit_rv(&t.init, t.id, last_usage);
            visit_block_for_usage(&t.r#loop, last_usage);
        }
        ReactiveTerminal::Label(t) => {
            visit_block_for_usage(&t.block, last_usage);
        }
        ReactiveTerminal::Try(t) => {
            visit_block_for_usage(&t.block, last_usage);
            visit_block_for_usage(&t.handler, last_usage);
        }
        ReactiveTerminal::Return(t) => {
            visit_place(&t.value, t.id, last_usage);
        }
        ReactiveTerminal::Throw(t) => {
            visit_place(&t.value, t.id, last_usage);
        }
        ReactiveTerminal::Break(_) | ReactiveTerminal::Continue(_) => {}
    }
}

fn update_last_usage(
    last_usage: &mut FxHashMap<DeclarationId, InstructionId>,
    decl_id: DeclarationId,
    id: InstructionId,
) {
    let entry = last_usage.entry(decl_id).or_insert(id);
    if id.0 > entry.0 {
        *entry = id;
    }
}

// =============================================================================
// Pass 2: Merge consecutive + flatten nested scopes
// =============================================================================

/// Process a block: first recurse into children (with nested scope flattening),
/// then merge consecutive scopes within this block.
fn merge_in_block(
    block: &mut ReactiveBlock,
    parent_deps: Option<&FxHashSet<ReactiveScopeDependency>>,
    last_usage: &FxHashMap<DeclarationId, InstructionId>,
    temporaries: &mut FxHashMap<DeclarationId, DeclarationId>,
) {
    // Sub-pass 1: Recurse into children, flattening nested scopes with same deps as parent
    let mut i = 0;
    while i < block.len() {
        match &mut block[i] {
            ReactiveStatement::Scope(scope) => {
                let scope_deps = scope.scope.dependencies.clone();
                // Recurse into the scope's instructions with this scope's deps as parent
                merge_in_block(&mut scope.instructions, Some(&scope_deps), last_usage, temporaries);

                // Nested scope flattening: if parent deps match this scope's deps, flatten
                if let Some(pdeps) = parent_deps {
                    if are_equal_dependencies(pdeps, &scope.scope.dependencies) {
                        // Replace this scope with its instructions
                        let instructions =
                            std::mem::take(&mut block[i].as_scope_mut().unwrap().instructions);
                        block.remove(i);
                        let count = instructions.len();
                        for (j, instr) in instructions.into_iter().enumerate() {
                            block.insert(i + j, instr);
                        }
                        i += count;
                        continue;
                    }
                }
                i += 1;
            }
            ReactiveStatement::PrunedScope(pruned) => {
                merge_in_block(&mut pruned.instructions, None, last_usage, temporaries);
                i += 1;
            }
            ReactiveStatement::Terminal(term) => {
                merge_in_terminal(&mut term.terminal, last_usage, temporaries);
                i += 1;
            }
            ReactiveStatement::Instruction(_) => {
                i += 1;
            }
        }
    }

    // Sub-pass 2: Identify consecutive scopes for merging
    struct MergedScope {
        from: usize,
        to: usize,
        lvalues: FxHashSet<DeclarationId>,
    }

    let mut current: Option<MergedScope> = None;
    let mut merged: Vec<MergedScope> = Vec::new();

    let reset = |current: &mut Option<MergedScope>, merged: &mut Vec<MergedScope>| {
        if let Some(c) = current.take() {
            if c.to > c.from + 1 {
                merged.push(c);
            }
        }
    };

    for i in 0..block.len() {
        match &block[i] {
            ReactiveStatement::Terminal(_) => {
                // Don't merge across terminals
                reset(&mut current, &mut merged);
            }
            ReactiveStatement::PrunedScope(_) => {
                // Don't merge across pruned scopes
                reset(&mut current, &mut merged);
            }
            ReactiveStatement::Instruction(instr_stmt) => {
                let instr = &instr_stmt.instruction;
                if current.is_some() {
                    let is_safe = match &instr.value {
                        crate::hir::ReactiveValue::Instruction(iv) => match iv.as_ref() {
                            InstructionValue::BinaryExpression(_)
                            | InstructionValue::ComputedLoad(_)
                            | InstructionValue::JsxText(_)
                            | InstructionValue::LoadGlobal(_)
                            | InstructionValue::LoadLocal(_)
                            | InstructionValue::LoadContext(_)
                            | InstructionValue::Primitive(_)
                            | InstructionValue::PropertyLoad(_)
                            | InstructionValue::TemplateLiteral(_)
                            | InstructionValue::UnaryExpression(_) => true,
                            InstructionValue::StoreLocal(sl) => {
                                sl.lvalue.kind == InstructionKind::Const
                            }
                            _ => false,
                        },
                        _ => false,
                    };

                    if is_safe {
                        let c = current.as_mut().unwrap();
                        // Track lvalue
                        if let Some(lvalue) = &instr.lvalue {
                            c.lvalues.insert(lvalue.identifier.declaration_id);
                        }
                        // Track temporaries for LoadLocal / LoadContext
                        if let crate::hir::ReactiveValue::Instruction(iv) = &instr.value {
                            match iv.as_ref() {
                                InstructionValue::LoadLocal(ll) => {
                                    if let Some(lvalue) = &instr.lvalue {
                                        temporaries.insert(
                                            lvalue.identifier.declaration_id,
                                            ll.place.identifier.declaration_id,
                                        );
                                    }
                                }
                                InstructionValue::LoadContext(lc) => {
                                    // LoadContext is the Rust equivalent of LoadLocal
                                    // for context variables (captured by closures).
                                    // The TS reference uses LoadLocal for these.
                                    if let Some(lvalue) = &instr.lvalue {
                                        temporaries.insert(
                                            lvalue.identifier.declaration_id,
                                            lc.place.identifier.declaration_id,
                                        );
                                    }
                                }
                                InstructionValue::StoreLocal(sl) => {
                                    // Track lvalues from StoreLocal
                                    c.lvalues.insert(sl.lvalue.place.identifier.declaration_id);
                                    // Follow temporary chain
                                    let source_decl = sl.value.identifier.declaration_id;
                                    let resolved = temporaries
                                        .get(&source_decl)
                                        .copied()
                                        .unwrap_or(source_decl);
                                    temporaries.insert(
                                        sl.lvalue.place.identifier.declaration_id,
                                        resolved,
                                    );
                                }
                                _ => {}
                            }
                        }
                    } else {
                        // Unsafe instruction: reset merge candidate
                        reset(&mut current, &mut merged);
                    }
                }
            }
            ReactiveStatement::Scope(scope_block) => {
                if let Some(ref c) = current {
                    let from_idx = c.from;
                    let current_block = block[from_idx].as_scope().unwrap();
                    let can_merge = can_merge_scopes(current_block, scope_block, temporaries);
                    let lvalues_ok =
                        are_lvalues_last_used_by_scope(&scope_block.scope, &c.lvalues, last_usage);

                    if can_merge && lvalues_ok {
                        // Record merge (actual mutation happens in Pass 3)
                        let c = current.as_mut().unwrap();
                        c.to = i + 1;
                        c.lvalues.clear();

                        // Check if the just-merged scope is eligible for further merging
                        if !scope_is_eligible_for_merging(scope_block) {
                            reset(&mut current, &mut merged);
                        }
                    } else {
                        // Can't merge: reset and maybe start new candidate
                        reset(&mut current, &mut merged);
                        if scope_is_eligible_for_merging(scope_block) {
                            current = Some(MergedScope {
                                from: i,
                                to: i + 1,
                                lvalues: FxHashSet::default(),
                            });
                        }
                    }
                } else {
                    // No current merge candidate: start one if eligible
                    if scope_is_eligible_for_merging(scope_block) {
                        current =
                            Some(MergedScope { from: i, to: i + 1, lvalues: FxHashSet::default() });
                    }
                }
            }
        }
    }
    reset(&mut current, &mut merged);

    // Sub-pass 3: Physically merge scopes
    if merged.is_empty() {
        return;
    }

    let mut next_instructions: ReactiveBlock = Vec::new();
    let mut index = 0;
    // We need to drain the block to move ownership
    let old_block: Vec<ReactiveStatement> = std::mem::take(block);

    for entry in &merged {
        // Copy statements before this merge range
        while index < entry.from {
            next_instructions.push(old_block[index].clone());
            index += 1;
        }
        // The first statement is the surviving scope
        let mut surviving = old_block[index].clone();
        index += 1;
        // Absorb subsequent statements into the surviving scope.
        // Note: `entry.to` is exclusive (set to `i + 1` when the last scope
        // at index `i` is merged), matching the TS reference which uses
        // `while (index < entry.to)`.
        while index < entry.to {
            if index >= old_block.len() {
                break;
            }
            let stmt = &old_block[index];
            match stmt {
                ReactiveStatement::Scope(absorbed_scope) => {
                    if let ReactiveStatement::Scope(ref mut surv) = surviving {
                        // Extend range
                        if absorbed_scope.scope.range.end > surv.scope.range.end {
                            surv.scope.range.end = absorbed_scope.scope.range.end;
                        }
                        // Merge declarations
                        for (key, value) in &absorbed_scope.scope.declarations {
                            surv.scope.declarations.insert(*key, value.clone());
                        }
                        // Prune declarations no longer needed
                        update_scope_declarations(&mut surv.scope, last_usage);
                        // Move instructions
                        surv.instructions.extend(absorbed_scope.instructions.clone());
                        surv.scope.merged.insert(absorbed_scope.scope.id);
                    }
                }
                _ => {
                    // Intervening instruction — absorb into the surviving scope
                    if let ReactiveStatement::Scope(ref mut surv) = surviving {
                        surv.instructions.push(stmt.clone());
                    }
                }
            }
            index += 1;
        }
        next_instructions.push(surviving);
    }
    // Copy remaining statements after the last merge range
    while index < old_block.len() {
        next_instructions.push(old_block[index].clone());
        index += 1;
    }

    *block = next_instructions;
}

// =============================================================================
// Helper: Merge in terminals (recurse into branches)
// =============================================================================

fn merge_in_terminal(
    terminal: &mut crate::hir::ReactiveTerminal,
    last_usage: &FxHashMap<DeclarationId, InstructionId>,
    temporaries: &mut FxHashMap<DeclarationId, DeclarationId>,
) {
    use crate::hir::ReactiveTerminal;
    match terminal {
        ReactiveTerminal::If(t) => {
            merge_in_block(&mut t.consequent, None, last_usage, temporaries);
            if let Some(alt) = &mut t.alternate {
                merge_in_block(alt, None, last_usage, temporaries);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &mut t.cases {
                if let Some(block) = &mut case.block {
                    merge_in_block(block, None, last_usage, temporaries);
                }
            }
        }
        ReactiveTerminal::While(t) => merge_in_block(&mut t.r#loop, None, last_usage, temporaries),
        ReactiveTerminal::DoWhile(t) => {
            merge_in_block(&mut t.r#loop, None, last_usage, temporaries);
        }
        ReactiveTerminal::For(t) => merge_in_block(&mut t.r#loop, None, last_usage, temporaries),
        ReactiveTerminal::ForOf(t) => {
            merge_in_block(&mut t.r#loop, None, last_usage, temporaries);
        }
        ReactiveTerminal::ForIn(t) => {
            merge_in_block(&mut t.r#loop, None, last_usage, temporaries);
        }
        ReactiveTerminal::Label(t) => {
            merge_in_block(&mut t.block, None, last_usage, temporaries);
        }
        ReactiveTerminal::Try(t) => {
            merge_in_block(&mut t.block, None, last_usage, temporaries);
            merge_in_block(&mut t.handler, None, last_usage, temporaries);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}

// =============================================================================
// canMergeScopes
// =============================================================================

/// Check if two scopes can be merged.
fn can_merge_scopes(
    current: &ReactiveScopeBlock,
    next: &ReactiveScopeBlock,
    temporaries: &FxHashMap<DeclarationId, DeclarationId>,
) -> bool {
    // Don't merge scopes with reassignments
    if !current.scope.reassignments.is_empty() || !next.scope.reassignments.is_empty() {
        return false;
    }
    // Case 1: Identical dependencies
    if are_equal_dependencies(&current.scope.dependencies, &next.scope.dependencies) {
        return true;
    }
    // Case 2: Outputs of current are inputs to next
    // 2a: Direct match — declarations of current == dependencies of next
    let current_decl_as_deps: FxHashSet<ReactiveScopeDependency> = current
        .scope
        .declarations
        .values()
        .map(|decl| ReactiveScopeDependency {
            identifier: decl.identifier.clone(),
            reactive: true,
            path: Vec::new(),
            loc: SourceLocation::Generated,
        })
        .collect();
    if are_equal_dependencies(&current_decl_as_deps, &next.scope.dependencies) {
        return true;
    }
    // 2b: Via temporaries with always-invalidating type check
    if !next.scope.dependencies.is_empty()
        && next.scope.dependencies.iter().all(|dep| {
            dep.path.is_empty()
                && is_always_invalidating_type(&dep.identifier.type_)
                && current.scope.declarations.values().any(|decl| {
                    decl.identifier.declaration_id == dep.identifier.declaration_id
                        || temporaries.get(&dep.identifier.declaration_id)
                            == Some(&decl.identifier.declaration_id)
                })
        })
    {
        return true;
    }
    false
}

// =============================================================================
// areEqualDependencies
// =============================================================================

/// Compare two dependency sets for equality.
/// Uses DeclarationId + path for comparison (matching TS reference).
fn are_equal_dependencies(
    a: &FxHashSet<ReactiveScopeDependency>,
    b: &FxHashSet<ReactiveScopeDependency>,
) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for a_dep in a {
        let found = b.iter().any(|b_dep| {
            a_dep.identifier.declaration_id == b_dep.identifier.declaration_id
                && a_dep.path == b_dep.path
        });
        if !found {
            return false;
        }
    }
    true
}

// =============================================================================
// areLValuesLastUsedByScope
// =============================================================================

/// Check that all intermediate lvalues are last-used within the next scope.
fn are_lvalues_last_used_by_scope(
    scope: &ReactiveScope,
    lvalues: &FxHashSet<DeclarationId>,
    last_usage: &FxHashMap<DeclarationId, InstructionId>,
) -> bool {
    for lvalue in lvalues {
        if let Some(&last_used_at) = last_usage.get(lvalue) {
            if last_used_at >= scope.range.end {
                return false;
            }
        }
    }
    true
}

// =============================================================================
// updateScopeDeclarations
// =============================================================================

/// Prune declarations that are no longer used after the scope's (now-extended) range.
fn update_scope_declarations(
    scope: &mut ReactiveScope,
    last_usage: &FxHashMap<DeclarationId, InstructionId>,
) {
    scope.declarations.retain(|_, decl| {
        if let Some(&last_used_at) = last_usage.get(&decl.identifier.declaration_id) {
            last_used_at >= scope.range.end
        } else {
            true // keep if we don't know usage
        }
    });
}

// =============================================================================
// scopeIsEligibleForMerging
// =============================================================================

/// A scope is eligible as a merge candidate if its output is guaranteed to change
/// when its input changes. This is true when:
/// - The scope has no dependencies (output never changes)
/// - At least one declaration is an "always invalidating type"
fn scope_is_eligible_for_merging(scope_block: &ReactiveScopeBlock) -> bool {
    if scope_block.scope.dependencies.is_empty() {
        return true;
    }
    scope_block
        .scope
        .declarations
        .values()
        .any(|decl| is_always_invalidating_type(&decl.identifier.type_))
}

// =============================================================================
// isAlwaysInvalidatingType
// =============================================================================

/// Check if a type is guaranteed to produce a new value when re-evaluated.
fn is_always_invalidating_type(type_: &Type) -> bool {
    match type_ {
        Type::Object(ObjectType { shape_id: Some(id) }) => {
            matches!(
                id.as_str(),
                BUILT_IN_ARRAY_ID | BUILT_IN_OBJECT_ID | BUILT_IN_FUNCTION_ID | BUILT_IN_JSX_ID
            )
        }
        Type::Function(FunctionType { .. }) => true,
        _ => false,
    }
}

// =============================================================================
// Helper: as_scope / as_scope_mut on ReactiveStatement
// =============================================================================

trait ReactiveStatementExt {
    fn as_scope(&self) -> Option<&ReactiveScopeBlock>;
    fn as_scope_mut(&mut self) -> Option<&mut ReactiveScopeBlock>;
}

impl ReactiveStatementExt for ReactiveStatement {
    fn as_scope(&self) -> Option<&ReactiveScopeBlock> {
        match self {
            ReactiveStatement::Scope(s) => Some(s),
            _ => None,
        }
    }
    fn as_scope_mut(&mut self) -> Option<&mut ReactiveScopeBlock> {
        match self {
            ReactiveStatement::Scope(s) => Some(s),
            _ => None,
        }
    }
}

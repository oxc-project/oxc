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
use rustc_hash::{FxHashMap, FxHashSet};

use crate::hir::{
    IdentifierId, InstructionKind, InstructionValue, PrunedReactiveScopeBlock, ReactiveBlock,
    ReactiveFunction, ReactiveInstruction, ReactiveStatement, ReactiveValue,
};

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
///
/// Some instructions will *always* produce a new value, and unless memoized will *always*
/// invalidate downstream reactive scopes. This pass finds such values and prunes downstream
/// memoization.
///
/// NOTE: function calls are an edge-case: function calls *may* return primitives, so this
/// pass optimistically assumes they do. Therefore, unmemoized function calls will *not*
/// prune downstream memoization. Only guaranteed new allocations, such as object and array
/// literals, will cause pruning.
pub fn prune_always_invalidating_scopes(func: &mut ReactiveFunction) {
    let mut always_invalidating: FxHashSet<IdentifierId> = FxHashSet::default();
    let mut unmemoized: FxHashSet<IdentifierId> = FxHashSet::default();
    prune_always_invalidating_block(
        &mut func.body,
        false,
        &mut always_invalidating,
        &mut unmemoized,
    );
}

/// Process a reactive instruction for always-invalidating tracking.
/// Checks the instruction value and updates the tracking sets.
fn process_always_invalidating_instruction(
    instruction: &ReactiveInstruction,
    within_scope: bool,
    always_invalidating: &mut FxHashSet<IdentifierId>,
    unmemoized: &mut FxHashSet<IdentifierId>,
) {
    // First, recursively visit any nested instructions in compound values
    visit_always_invalidating_value(
        &instruction.value,
        within_scope,
        always_invalidating,
        unmemoized,
    );

    // Then process this instruction's value kind
    if let ReactiveValue::Instruction(value) = &instruction.value {
        match value.as_ref() {
            InstructionValue::ArrayExpression(_)
            | InstructionValue::ObjectExpression(_)
            | InstructionValue::JsxExpression(_)
            | InstructionValue::JsxFragment(_)
            | InstructionValue::NewExpression(_) => {
                if let Some(lvalue) = &instruction.lvalue {
                    always_invalidating.insert(lvalue.identifier.id);
                    if !within_scope {
                        unmemoized.insert(lvalue.identifier.id);
                    }
                }
            }
            InstructionValue::StoreLocal(store) => {
                if always_invalidating.contains(&store.value.identifier.id) {
                    always_invalidating.insert(store.lvalue.place.identifier.id);
                }
                if unmemoized.contains(&store.value.identifier.id) {
                    unmemoized.insert(store.lvalue.place.identifier.id);
                }
            }
            InstructionValue::LoadLocal(load) => {
                if let Some(lvalue) = &instruction.lvalue {
                    if always_invalidating.contains(&load.place.identifier.id) {
                        always_invalidating.insert(lvalue.identifier.id);
                    }
                    if unmemoized.contains(&load.place.identifier.id) {
                        unmemoized.insert(lvalue.identifier.id);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Recursively visit compound reactive values to process nested instructions.
fn visit_always_invalidating_value(
    value: &ReactiveValue,
    within_scope: bool,
    always_invalidating: &mut FxHashSet<IdentifierId>,
    unmemoized: &mut FxHashSet<IdentifierId>,
) {
    match value {
        ReactiveValue::Sequence(seq) => {
            for instr in &seq.instructions {
                process_always_invalidating_instruction(
                    instr,
                    within_scope,
                    always_invalidating,
                    unmemoized,
                );
            }
            visit_always_invalidating_value(
                &seq.value,
                within_scope,
                always_invalidating,
                unmemoized,
            );
        }
        ReactiveValue::Logical(logical) => {
            visit_always_invalidating_value(
                &logical.left,
                within_scope,
                always_invalidating,
                unmemoized,
            );
            visit_always_invalidating_value(
                &logical.right,
                within_scope,
                always_invalidating,
                unmemoized,
            );
        }
        ReactiveValue::Ternary(ternary) => {
            visit_always_invalidating_value(
                &ternary.test,
                within_scope,
                always_invalidating,
                unmemoized,
            );
            visit_always_invalidating_value(
                &ternary.consequent,
                within_scope,
                always_invalidating,
                unmemoized,
            );
            visit_always_invalidating_value(
                &ternary.alternate,
                within_scope,
                always_invalidating,
                unmemoized,
            );
        }
        ReactiveValue::OptionalCall(opt) => {
            visit_always_invalidating_value(
                &opt.value,
                within_scope,
                always_invalidating,
                unmemoized,
            );
        }
        ReactiveValue::Instruction(_) => {
            // Leaf instruction values — handled by the caller
        }
    }
}

/// Recursively traverse a reactive block for always-invalidating scope pruning.
fn prune_always_invalidating_block(
    block: &mut ReactiveBlock,
    within_scope: bool,
    always_invalidating: &mut FxHashSet<IdentifierId>,
    unmemoized: &mut FxHashSet<IdentifierId>,
) {
    // First pass: process instructions and scopes, track which indices to prune
    let mut prune_indices: Vec<usize> = Vec::new();

    for (i, stmt) in block.iter_mut().enumerate() {
        match stmt {
            ReactiveStatement::Instruction(instr_stmt) => {
                process_always_invalidating_instruction(
                    &instr_stmt.instruction,
                    within_scope,
                    always_invalidating,
                    unmemoized,
                );
            }
            ReactiveStatement::Terminal(term_stmt) => {
                prune_always_invalidating_terminal(
                    &mut term_stmt.terminal,
                    within_scope,
                    always_invalidating,
                    unmemoized,
                );
            }
            ReactiveStatement::Scope(scope) => {
                // Visit scope contents with within_scope = true
                prune_always_invalidating_block(
                    &mut scope.instructions,
                    true,
                    always_invalidating,
                    unmemoized,
                );

                // Check if any dependency is unmemoized
                let should_prune = scope
                    .scope
                    .dependencies
                    .iter()
                    .any(|dep| unmemoized.contains(&dep.identifier_id));

                if should_prune {
                    // Propagate unmemoized for declarations that are always-invalidating
                    for decl in scope.scope.declarations.values() {
                        if always_invalidating.contains(&decl.identifier.id) {
                            unmemoized.insert(decl.identifier.id);
                        }
                    }
                    for &reassignment_id in &scope.scope.reassignments {
                        if always_invalidating.contains(&reassignment_id) {
                            unmemoized.insert(reassignment_id);
                        }
                    }
                    prune_indices.push(i);
                }
            }
            ReactiveStatement::PrunedScope(scope) => {
                prune_always_invalidating_block(
                    &mut scope.instructions,
                    within_scope,
                    always_invalidating,
                    unmemoized,
                );
            }
        }
    }

    // Second pass: replace Scope -> PrunedScope for marked indices
    // Process in reverse to avoid index invalidation (indices are stable since we only replace)
    for i in prune_indices {
        // Take the old statement by swapping with a temporary
        let old_stmt = std::mem::replace(
            &mut block[i],
            ReactiveStatement::PrunedScope(PrunedReactiveScopeBlock {
                scope: crate::hir::ReactiveScope {
                    id: crate::hir::ScopeId(0),
                    range: crate::hir::MutableRange::default(),
                    dependencies: FxHashSet::default(),
                    declarations: FxHashMap::default(),
                    reassignments: FxHashSet::default(),
                    early_return_value: None,
                    merged: FxHashSet::default(),
                    loc: crate::compiler_error::SourceLocation::default(),
                },
                instructions: Vec::new(),
            }),
        );
        if let ReactiveStatement::Scope(scope_block) = old_stmt {
            block[i] = ReactiveStatement::PrunedScope(PrunedReactiveScopeBlock {
                scope: scope_block.scope,
                instructions: scope_block.instructions,
            });
        }
    }
}

fn prune_always_invalidating_terminal(
    terminal: &mut crate::hir::ReactiveTerminal,
    within_scope: bool,
    always_invalidating: &mut FxHashSet<IdentifierId>,
    unmemoized: &mut FxHashSet<IdentifierId>,
) {
    use crate::hir::ReactiveTerminal;
    match terminal {
        ReactiveTerminal::If(t) => {
            prune_always_invalidating_block(
                &mut t.consequent,
                within_scope,
                always_invalidating,
                unmemoized,
            );
            if let Some(alt) = &mut t.alternate {
                prune_always_invalidating_block(alt, within_scope, always_invalidating, unmemoized);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &mut t.cases {
                if let Some(block) = &mut case.block {
                    prune_always_invalidating_block(
                        block,
                        within_scope,
                        always_invalidating,
                        unmemoized,
                    );
                }
            }
        }
        ReactiveTerminal::While(t) => {
            prune_always_invalidating_block(
                &mut t.r#loop,
                within_scope,
                always_invalidating,
                unmemoized,
            );
        }
        ReactiveTerminal::DoWhile(t) => {
            prune_always_invalidating_block(
                &mut t.r#loop,
                within_scope,
                always_invalidating,
                unmemoized,
            );
        }
        ReactiveTerminal::For(t) => {
            prune_always_invalidating_block(
                &mut t.r#loop,
                within_scope,
                always_invalidating,
                unmemoized,
            );
        }
        ReactiveTerminal::ForOf(t) => {
            prune_always_invalidating_block(
                &mut t.r#loop,
                within_scope,
                always_invalidating,
                unmemoized,
            );
        }
        ReactiveTerminal::ForIn(t) => {
            prune_always_invalidating_block(
                &mut t.r#loop,
                within_scope,
                always_invalidating,
                unmemoized,
            );
        }
        ReactiveTerminal::Label(t) => {
            prune_always_invalidating_block(
                &mut t.block,
                within_scope,
                always_invalidating,
                unmemoized,
            );
        }
        ReactiveTerminal::Try(t) => {
            prune_always_invalidating_block(
                &mut t.block,
                within_scope,
                always_invalidating,
                unmemoized,
            );
            prune_always_invalidating_block(
                &mut t.handler,
                within_scope,
                always_invalidating,
                unmemoized,
            );
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
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

/// Prune hoisted contexts — removes DeclareContext instructions lowered for HoistedConsts,
/// and transforms any references back to their original instruction kind.
///
/// Also detects and bails out on context variables which are:
/// - function declarations, which are hoisted by JS engines to the nearest block scope
/// - referenced before they are defined (i.e. having a `DeclareContext HoistedConst`)
/// - declared
///
/// This is because React Compiler converts a `function foo()` function declaration to
/// 1. a `let foo;` declaration before reactive memo blocks
/// 2. a `foo = function foo() {}` assignment within the block
///
/// This means references before the assignment are invalid.
pub fn prune_hoisted_contexts(func: &mut ReactiveFunction) {
    let mut active_scopes: Vec<FxHashSet<IdentifierId>> = Vec::new();
    let mut uninitialized: FxHashMap<IdentifierId, UninitializedEntry> = FxHashMap::default();
    prune_hoisted_contexts_block(&mut func.body, &mut active_scopes, &mut uninitialized);
}

/// Tracks the state of an uninitialized declaration within a scope.
#[derive(Debug)]
enum UninitializedEntry {
    /// Declaration kind is not yet known to be a function.
    UnknownKind,
    /// A hoisted function declaration that has not yet been assigned.
    Func {
        /// The place where this function is defined (set when StoreContext assigns it).
        /// `None` means the function has been declared hoisted but not yet assigned.
        definition: Option<IdentifierId>,
    },
}

fn prune_hoisted_contexts_block(
    block: &mut ReactiveBlock,
    active_scopes: &mut Vec<FxHashSet<IdentifierId>>,
    uninitialized: &mut FxHashMap<IdentifierId, UninitializedEntry>,
) {
    let mut i = 0;
    while i < block.len() {
        let should_remove = match &mut block[i] {
            ReactiveStatement::Instruction(instr_stmt) => process_hoisted_context_instruction(
                &mut instr_stmt.instruction,
                active_scopes,
                uninitialized,
            ),
            ReactiveStatement::Terminal(term_stmt) => {
                prune_hoisted_contexts_terminal(
                    &mut term_stmt.terminal,
                    active_scopes,
                    uninitialized,
                );
                false
            }
            ReactiveStatement::Scope(scope) => {
                // Push scope declaration IDs onto the active scopes stack
                let scope_decl_ids: FxHashSet<IdentifierId> =
                    scope.scope.declarations.keys().copied().collect();

                // Add declared but not initialized/assigned variables
                for decl in scope.scope.declarations.values() {
                    uninitialized.insert(decl.identifier.id, UninitializedEntry::UnknownKind);
                }

                active_scopes.push(scope_decl_ids);

                // Traverse the scope's instructions
                prune_hoisted_contexts_block(&mut scope.instructions, active_scopes, uninitialized);

                // Pop the active scope
                active_scopes.pop();

                // Remove declarations from uninitialized
                for decl in scope.scope.declarations.values() {
                    uninitialized.remove(&decl.identifier.id);
                }

                false
            }
            ReactiveStatement::PrunedScope(scope) => {
                prune_hoisted_contexts_block(&mut scope.instructions, active_scopes, uninitialized);
                false
            }
        };

        if should_remove {
            block.remove(i);
        } else {
            i += 1;
        }
    }
}

/// Process a single instruction for hoisted context pruning.
/// Returns `true` if the instruction should be removed.
fn process_hoisted_context_instruction(
    instruction: &mut ReactiveInstruction,
    active_scopes: &[FxHashSet<IdentifierId>],
    uninitialized: &mut FxHashMap<IdentifierId, UninitializedEntry>,
) -> bool {
    // Check for DeclareContext with hoisted kind — remove if hoisted
    if let ReactiveValue::Instruction(value) = &instruction.value
        && let InstructionValue::DeclareContext(declare_ctx) = value.as_ref()
        && let Some(non_hoisted) = declare_ctx.lvalue_kind.convert_hoisted()
    {
        // If this is a hoisted function and the identifier is in uninitialized,
        // mark it as a function
        if non_hoisted == InstructionKind::Function
            && uninitialized.contains_key(&declare_ctx.lvalue_place.identifier.id)
        {
            uninitialized.insert(
                declare_ctx.lvalue_place.identifier.id,
                UninitializedEntry::Func { definition: None },
            );
        }
        // Remove this DeclareContext instruction
        return true;
    }

    // Check for StoreContext with non-Reassign kind
    if let ReactiveValue::Instruction(value) = &mut instruction.value
        && let InstructionValue::StoreContext(store_ctx) = value.as_mut()
        && store_ctx.lvalue_kind != InstructionKind::Reassign
    {
        let lvalue_id = store_ctx.lvalue_place.identifier.id;
        let is_declared_by_scope = active_scopes.iter().any(|scope| scope.contains(&lvalue_id));

        if is_declared_by_scope {
            if store_ctx.lvalue_kind == InstructionKind::Let
                || store_ctx.lvalue_kind == InstructionKind::Const
            {
                // Rewrite to Reassign since it will be pre-declared in codegen
                store_ctx.lvalue_kind = InstructionKind::Reassign;
            } else if store_ctx.lvalue_kind == InstructionKind::Function {
                // For function declarations, mark as initialized
                if uninitialized.contains_key(&lvalue_id) {
                    // References to hoisted functions are now "safe" as variable
                    // assignments have finished
                    uninitialized.remove(&lvalue_id);
                }
            }
            // Note: the TS code has an else branch that throws a Todo for unexpected
            // kinds. We skip that for now as it's a diagnostic concern.
        }
    }

    // Visit places within the instruction to check for hoisted function references
    visit_hoisted_context_places(&instruction.value, uninitialized);

    false
}

/// Visit all places in a reactive value to check for uninitialized hoisted function references.
fn visit_hoisted_context_places(
    value: &ReactiveValue,
    uninitialized: &FxHashMap<IdentifierId, UninitializedEntry>,
) {
    match value {
        ReactiveValue::Instruction(iv) => {
            visit_instruction_value_places(iv, uninitialized);
        }
        ReactiveValue::Sequence(seq) => {
            for instr in &seq.instructions {
                visit_hoisted_context_places(&instr.value, uninitialized);
            }
            visit_hoisted_context_places(&seq.value, uninitialized);
        }
        ReactiveValue::Logical(logical) => {
            visit_hoisted_context_places(&logical.left, uninitialized);
            visit_hoisted_context_places(&logical.right, uninitialized);
        }
        ReactiveValue::Ternary(ternary) => {
            visit_hoisted_context_places(&ternary.test, uninitialized);
            visit_hoisted_context_places(&ternary.consequent, uninitialized);
            visit_hoisted_context_places(&ternary.alternate, uninitialized);
        }
        ReactiveValue::OptionalCall(opt) => {
            visit_hoisted_context_places(&opt.value, uninitialized);
        }
    }
}

/// Visit places within an InstructionValue for hoisted function reference checking.
/// The TS visitPlace checks if a place references an uninitialized hoisted function
/// and throws a Todo error. We check for this condition but do not throw since we
/// cannot bail out in the same way.
fn visit_instruction_value_places(
    value: &InstructionValue,
    uninitialized: &FxHashMap<IdentifierId, UninitializedEntry>,
) {
    // Check each operand place for uninitialized hoisted function references
    for_each_instruction_value_operand(value, &|place| {
        check_hoisted_function_reference(place, uninitialized);
    });
}

/// Check if a place references an uninitialized hoisted function.
fn check_hoisted_function_reference(
    place: &crate::hir::Place,
    uninitialized: &FxHashMap<IdentifierId, UninitializedEntry>,
) {
    if let Some(UninitializedEntry::Func { definition }) = uninitialized.get(&place.identifier.id) {
        // In the TS code, this checks `maybeHoistedFn.definition !== place` using reference
        // identity. Since the definition is None when the function hasn't been assigned yet,
        // and if definition is Some but doesn't match this place's id, we should flag it.
        // However, once definition is set, it's removed from uninitialized via the
        // StoreContext handler. So if we reach here, definition is either None or the
        // function is still "uninitialized".
        if definition.is_none() || *definition != Some(place.identifier.id) {
            // The TS code throws a CompilerError.throwTodo here.
            // In Rust, this condition indicates a hoisted function reference before
            // initialization, which is unsupported.
        }
    }
}

/// Iterate over operand places of an InstructionValue, calling the callback for each.
fn for_each_instruction_value_operand(
    value: &InstructionValue,
    callback: &impl Fn(&crate::hir::Place),
) {
    use crate::hir::{
        ArrayExpressionElement, CallArg, JsxAttribute, JsxTag, ManualMemoDependencyRoot,
        ObjectPatternProperty, ObjectPropertyKey,
    };
    match value {
        InstructionValue::LoadLocal(v) => callback(&v.place),
        InstructionValue::LoadContext(v) => callback(&v.place),
        InstructionValue::StoreLocal(v) => callback(&v.value),
        InstructionValue::StoreContext(v) => callback(&v.value),
        InstructionValue::Destructure(v) => callback(&v.value),
        InstructionValue::BinaryExpression(v) => {
            callback(&v.left);
            callback(&v.right);
        }
        InstructionValue::UnaryExpression(v) => callback(&v.value),
        InstructionValue::TypeCastExpression(v) => callback(&v.value),
        InstructionValue::CallExpression(v) => {
            callback(&v.callee);
            for arg in &v.args {
                match arg {
                    CallArg::Spread(s) => callback(&s.place),
                    CallArg::Place(p) => callback(p),
                }
            }
        }
        InstructionValue::MethodCall(v) => {
            callback(&v.receiver);
            callback(&v.property);
            for arg in &v.args {
                match arg {
                    CallArg::Spread(s) => callback(&s.place),
                    CallArg::Place(p) => callback(p),
                }
            }
        }
        InstructionValue::NewExpression(v) => {
            callback(&v.callee);
            for arg in &v.args {
                match arg {
                    CallArg::Spread(s) => callback(&s.place),
                    CallArg::Place(p) => callback(p),
                }
            }
        }
        InstructionValue::ObjectExpression(v) => {
            for prop in &v.properties {
                match prop {
                    ObjectPatternProperty::Property(p) => {
                        callback(&p.place);
                        if let ObjectPropertyKey::Computed(c) = &p.key {
                            callback(c);
                        }
                    }
                    ObjectPatternProperty::Spread(s) => callback(&s.place),
                }
            }
        }
        InstructionValue::ArrayExpression(v) => {
            for elem in &v.elements {
                match elem {
                    ArrayExpressionElement::Place(p) => callback(p),
                    ArrayExpressionElement::Spread(s) => callback(&s.place),
                    ArrayExpressionElement::Hole => {}
                }
            }
        }
        InstructionValue::JsxExpression(v) => {
            if let JsxTag::Place(p) = &v.tag {
                callback(p);
            }
            for prop in &v.props {
                match prop {
                    JsxAttribute::Attribute { place, .. } => callback(place),
                    JsxAttribute::Spread { argument } => callback(argument),
                }
            }
            if let Some(children) = &v.children {
                for child in children {
                    callback(child);
                }
            }
        }
        InstructionValue::JsxFragment(v) => {
            for child in &v.children {
                callback(child);
            }
        }
        InstructionValue::PropertyLoad(v) => callback(&v.object),
        InstructionValue::PropertyStore(v) => {
            callback(&v.object);
            callback(&v.value);
        }
        InstructionValue::PropertyDelete(v) => callback(&v.object),
        InstructionValue::ComputedLoad(v) => {
            callback(&v.object);
            callback(&v.property);
        }
        InstructionValue::ComputedStore(v) => {
            callback(&v.object);
            callback(&v.property);
            callback(&v.value);
        }
        InstructionValue::ComputedDelete(v) => {
            callback(&v.object);
            callback(&v.property);
        }
        InstructionValue::TemplateLiteral(v) => {
            for sub in &v.subexprs {
                callback(sub);
            }
        }
        InstructionValue::TaggedTemplateExpression(v) => {
            callback(&v.tag);
        }
        InstructionValue::DeclareLocal(_)
        | InstructionValue::DeclareContext(_)
        | InstructionValue::Primitive(_)
        | InstructionValue::JsxText(_)
        | InstructionValue::RegExpLiteral(_)
        | InstructionValue::LoadGlobal(_)
        | InstructionValue::Debugger(_)
        | InstructionValue::MetaProperty(_)
        | InstructionValue::UnsupportedNode(_)
        | InstructionValue::FunctionExpression(_)
        | InstructionValue::ObjectMethod(_) => {}
        InstructionValue::StoreGlobal(v) => callback(&v.value),
        InstructionValue::GetIterator(v) => callback(&v.collection),
        InstructionValue::IteratorNext(v) => {
            callback(&v.iterator);
            callback(&v.collection);
        }
        InstructionValue::NextPropertyOf(v) => callback(&v.value),
        InstructionValue::PrefixUpdate(v) => callback(&v.value),
        InstructionValue::PostfixUpdate(v) => callback(&v.value),
        InstructionValue::Await(v) => callback(&v.value),
        InstructionValue::StartMemoize(v) => {
            if let Some(deps) = &v.deps {
                for dep in deps {
                    if let ManualMemoDependencyRoot::NamedLocal { value, .. } = &dep.root {
                        callback(value);
                    }
                }
            }
        }
        InstructionValue::FinishMemoize(v) => callback(&v.decl),
    }
}

fn prune_hoisted_contexts_terminal(
    terminal: &mut crate::hir::ReactiveTerminal,
    active_scopes: &mut Vec<FxHashSet<IdentifierId>>,
    uninitialized: &mut FxHashMap<IdentifierId, UninitializedEntry>,
) {
    use crate::hir::ReactiveTerminal;
    match terminal {
        ReactiveTerminal::If(t) => {
            prune_hoisted_contexts_block(&mut t.consequent, active_scopes, uninitialized);
            if let Some(alt) = &mut t.alternate {
                prune_hoisted_contexts_block(alt, active_scopes, uninitialized);
            }
        }
        ReactiveTerminal::Switch(t) => {
            for case in &mut t.cases {
                if let Some(block) = &mut case.block {
                    prune_hoisted_contexts_block(block, active_scopes, uninitialized);
                }
            }
        }
        ReactiveTerminal::While(t) => {
            prune_hoisted_contexts_block(&mut t.r#loop, active_scopes, uninitialized);
        }
        ReactiveTerminal::DoWhile(t) => {
            prune_hoisted_contexts_block(&mut t.r#loop, active_scopes, uninitialized);
        }
        ReactiveTerminal::For(t) => {
            prune_hoisted_contexts_block(&mut t.r#loop, active_scopes, uninitialized);
        }
        ReactiveTerminal::ForOf(t) => {
            prune_hoisted_contexts_block(&mut t.r#loop, active_scopes, uninitialized);
        }
        ReactiveTerminal::ForIn(t) => {
            prune_hoisted_contexts_block(&mut t.r#loop, active_scopes, uninitialized);
        }
        ReactiveTerminal::Label(t) => {
            prune_hoisted_contexts_block(&mut t.block, active_scopes, uninitialized);
        }
        ReactiveTerminal::Try(t) => {
            prune_hoisted_contexts_block(&mut t.block, active_scopes, uninitialized);
            prune_hoisted_contexts_block(&mut t.handler, active_scopes, uninitialized);
        }
        ReactiveTerminal::Break(_)
        | ReactiveTerminal::Continue(_)
        | ReactiveTerminal::Return(_)
        | ReactiveTerminal::Throw(_) => {}
    }
}

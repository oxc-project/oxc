/// Align reactive scopes to block scopes and method call scopes.
///
/// Ports of:
/// - `ReactiveScopes/AlignMethodCallScopes.ts`
/// - `ReactiveScopes/AlignObjectMethodScopes.ts`
/// - `ReactiveScopes/AlignReactiveScopesToBlockScopesHIR.ts`
///
/// These passes adjust reactive scope boundaries to align with JavaScript's
/// block scoping rules, ensuring memoized code blocks form valid JS scopes.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    hir::{
        BlockId, BlockKind, HIRFunction, IdentifierId, InstructionId, InstructionValue,
        MutableRange, Place, ReactiveScope, ScopeId,
        visitors::{
            each_instruction_lvalue, each_instruction_value_operand, each_terminal_operand,
            each_terminal_successor, terminal_fallthrough,
        },
    },
    utils::disjoint_set::DisjointSet,
};

// =====================================================================================
// Helper: get_place_scope
// =====================================================================================

/// Returns the identifier's scope if the instruction `id` is within the scope's range.
///
/// Port of `getPlaceScope` from `HIR/HIR.ts`.
fn get_place_scope(id: InstructionId, place: &Place) -> Option<&ReactiveScope> {
    let scope = place.identifier.scope.as_ref()?;
    if id >= scope.range.start && id < scope.range.end { Some(scope) } else { None }
}

// =====================================================================================
// 3e-i: align_method_call_scopes
// =====================================================================================

/// Align method call scopes -- ensures method calls share the same reactive scope
/// as their receiver object.
///
/// Port of `AlignMethodCallScopes.ts`.
///
/// Ensures that method call instructions have scopes such that either:
/// - Both the MethodCall and its property have the same scope
/// - OR neither has a scope
pub fn align_method_call_scopes(func: &mut HIRFunction) {
    // scope_mapping: IdentifierId -> Some(ScopeId) to assign, or None to remove scope
    let mut scope_mapping: FxHashMap<IdentifierId, Option<ScopeId>> = FxHashMap::default();
    let mut merged_scopes: DisjointSet<ScopeId> = DisjointSet::new();
    // Track all scope ranges by ScopeId for later merging
    let mut scope_ranges: FxHashMap<ScopeId, MutableRange> = FxHashMap::default();

    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    for block_id in &block_ids {
        let Some(block) = func.body.blocks.get(block_id) else {
            continue;
        };
        for instr in &block.instructions {
            let lvalue_scope_id = instr.lvalue.identifier.scope.as_ref().map(|s| s.id);

            if let InstructionValue::MethodCall(method_call) = &instr.value {
                let property_scope = method_call.property.identifier.scope.as_ref();
                let property_scope_id = property_scope.map(|s| s.id);

                if let Some(lvalue_sid) = lvalue_scope_id {
                    if let Some(lvalue_scope) = &instr.lvalue.identifier.scope {
                        scope_ranges.entry(lvalue_sid).or_insert(lvalue_scope.range);
                    }

                    if let Some(property_sid) = property_scope_id {
                        // Both have a scope: merge the scopes
                        if let Some(prop_scope) = property_scope {
                            scope_ranges.entry(property_sid).or_insert(prop_scope.range);
                        }
                        merged_scopes.union(&[lvalue_sid, property_sid]);
                    } else {
                        // Call has scope but property doesn't —
                        // record that property should get this scope
                        scope_mapping.insert(method_call.property.identifier.id, Some(lvalue_sid));
                    }
                } else if property_scope_id.is_some() {
                    // Property has scope but call doesn't — property doesn't need a scope
                    scope_mapping.insert(method_call.property.identifier.id, None);
                }
            }
        }
    }

    // Recurse into nested functions (mutable access)
    for block_id in &block_ids {
        let Some(block) = func.body.blocks.get_mut(block_id) else {
            continue;
        };
        for instr in &mut block.instructions {
            match &mut instr.value {
                InstructionValue::FunctionExpression(fe) => {
                    align_method_call_scopes(&mut fe.lowered_func.func);
                }
                InstructionValue::ObjectMethod(om) => {
                    align_method_call_scopes(&mut om.lowered_func.func);
                }
                _ => {}
            }
        }
    }

    // Merge scope ranges: for each DisjointSet group, extend root scope to cover all
    let mut root_ranges: FxHashMap<ScopeId, MutableRange> = FxHashMap::default();
    merged_scopes.for_each(|scope_id, root_id| {
        if scope_id == root_id {
            // Initialize root range
            if let Some(&range) = scope_ranges.get(scope_id) {
                root_ranges.entry(*root_id).or_insert(range);
            }
            return;
        }
        if let Some(&scope_range) = scope_ranges.get(scope_id) {
            let root_range = root_ranges
                .entry(*root_id)
                .or_insert_with(|| scope_ranges.get(root_id).copied().unwrap_or(scope_range));
            root_range.start = InstructionId(root_range.start.0.min(scope_range.start.0));
            root_range.end = InstructionId(root_range.end.0.max(scope_range.end.0));
        }
    });

    // Apply: rewrite identifier scopes using scope_mapping and merged_scopes
    for block_id in &block_ids {
        let Some(block) = func.body.blocks.get_mut(block_id) else {
            continue;
        };
        for instr in &mut block.instructions {
            let id = instr.lvalue.identifier.id;

            if let Some(mapped) = scope_mapping.get(&id) {
                match mapped {
                    Some(target_scope_id) => {
                        // Assign the lvalue to this scope
                        if let Some(scope) = &mut instr.lvalue.identifier.scope {
                            scope.id = *target_scope_id;
                            if let Some(range) = root_ranges.get(target_scope_id) {
                                scope.range = *range;
                            }
                        } else {
                            // Need to create a scope — find a template from scope_ranges
                            if let Some(&range) = root_ranges
                                .get(target_scope_id)
                                .or_else(|| scope_ranges.get(target_scope_id))
                            {
                                instr.lvalue.identifier.scope = Some(Box::new(ReactiveScope {
                                    id: *target_scope_id,
                                    range,
                                    dependencies: FxHashSet::default(),
                                    declarations: FxHashMap::default(),
                                    reassignments: FxHashSet::default(),
                                    early_return_value: None,
                                    merged: FxHashSet::default(),
                                    loc: instr.lvalue.identifier.loc,
                                }));
                            }
                        }
                    }
                    None => {
                        instr.lvalue.identifier.scope = None;
                    }
                }
            } else if let Some(scope) = &mut instr.lvalue.identifier.scope {
                let current_id = scope.id;
                if let Some(root_id) = merged_scopes.find(&current_id) {
                    scope.id = root_id;
                    if let Some(range) = root_ranges.get(&root_id) {
                        scope.range = *range;
                    }
                }
            }
        }
    }
}

// =====================================================================================
// 3e-ii: align_object_method_scopes
// =====================================================================================

/// Find scopes that need to be merged for object method alignment.
///
/// Port of `findScopesToMerge` from `AlignObjectMethodScopes.ts`.
fn find_scopes_to_merge(
    func: &HIRFunction,
) -> (DisjointSet<ScopeId>, FxHashMap<ScopeId, MutableRange>) {
    let mut object_method_decls: FxHashSet<IdentifierId> = FxHashSet::default();
    let mut merge_scopes_builder: DisjointSet<ScopeId> = DisjointSet::new();
    let mut scope_ranges: FxHashMap<ScopeId, MutableRange> = FxHashMap::default();

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            if matches!(&instr.value, InstructionValue::ObjectMethod(_)) {
                object_method_decls.insert(instr.lvalue.identifier.id);
            } else if matches!(&instr.value, InstructionValue::ObjectExpression(_)) {
                for operand in each_instruction_value_operand(&instr.value) {
                    if object_method_decls.contains(&operand.identifier.id) {
                        let operand_scope = operand.identifier.scope.as_ref();
                        let lvalue_scope = instr.lvalue.identifier.scope.as_ref();

                        // Both should have non-null scopes (invariant from TS)
                        if let (Some(op_scope), Some(lv_scope)) = (operand_scope, lvalue_scope) {
                            scope_ranges.entry(op_scope.id).or_insert(op_scope.range);
                            scope_ranges.entry(lv_scope.id).or_insert(lv_scope.range);
                            merge_scopes_builder.union(&[op_scope.id, lv_scope.id]);
                        }
                    }
                }
            }
        }
    }
    (merge_scopes_builder, scope_ranges)
}

/// Align object method scopes -- ensures object methods share the same reactive scope
/// as their containing object.
///
/// Port of `AlignObjectMethodScopes.ts`.
///
/// Aligns scopes of object method values to that of their enclosing object expressions.
/// To produce a well-formed JS program in Codegen, object methods and object expressions
/// must be in the same ReactiveBlock as object method definitions must be inlined.
pub fn align_object_method_scopes(func: &mut HIRFunction) {
    // Handle inner functions first: recurse into nested functions
    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    for block_id in &block_ids {
        let Some(block) = func.body.blocks.get_mut(block_id) else {
            continue;
        };
        for instr in &mut block.instructions {
            match &mut instr.value {
                InstructionValue::ObjectMethod(om) => {
                    align_object_method_scopes(&mut om.lowered_func.func);
                }
                InstructionValue::FunctionExpression(fe) => {
                    align_object_method_scopes(&mut fe.lowered_func.func);
                }
                _ => {}
            }
        }
    }

    let (mut merge_scopes_builder, scope_ranges) = find_scopes_to_merge(func);

    // Step 1: Merge affected scopes to their canonical root
    let scope_groups_map = merge_scopes_builder.canonicalize();

    let mut root_ranges: FxHashMap<ScopeId, MutableRange> = FxHashMap::default();
    for (&scope_id, &root_id) in &scope_groups_map {
        if scope_id != root_id {
            if let Some(&scope_range) = scope_ranges.get(&scope_id) {
                let root_range = root_ranges
                    .entry(root_id)
                    .or_insert_with(|| scope_ranges.get(&root_id).copied().unwrap_or(scope_range));
                root_range.start = InstructionId(root_range.start.0.min(scope_range.start.0));
                root_range.end = InstructionId(root_range.end.0.max(scope_range.end.0));
            }
        } else if let Some(&range) = scope_ranges.get(&scope_id) {
            root_ranges.entry(root_id).or_insert(range);
        }
    }

    // Step 2: Repoint identifiers whose scopes were merged
    for block_id in &block_ids {
        let Some(block) = func.body.blocks.get_mut(block_id) else {
            continue;
        };
        for instr in &mut block.instructions {
            if let Some(scope) = &mut instr.lvalue.identifier.scope
                && let Some(&root_id) = scope_groups_map.get(&scope.id)
            {
                scope.id = root_id;
                if let Some(&range) = root_ranges.get(&root_id) {
                    scope.range = range;
                }
            }
        }
    }
}

// =====================================================================================
// 3e-iii: align_reactive_scopes_to_block_scopes_hir
// =====================================================================================

/// Align reactive scopes to block scopes -- adjusts reactive scope boundaries
/// to align with JavaScript block scopes.
///
/// Port of `AlignReactiveScopesToBlockScopesHIR.ts`.
///
/// This is critical because reactive scopes must correspond to valid JS blocks
/// in the output code. A reactive scope cannot start in the middle of an if-else
/// and end after it, for example.
///
/// The more general rule is that a reactive scope may only end at the same block
/// scope as it began: this pass therefore finds, for each scope, the block where
/// that scope started and finds the first instruction after the scope's mutable
/// range in that same block scope (which will be the updated end for that scope).
pub fn align_reactive_scopes_to_block_scopes_hir(func: &mut HIRFunction) {
    // Helper: get scope range from updates map (current range for a scope)
    fn get_scope_range(
        updates: &FxHashMap<ScopeId, MutableRange>,
        scope_id: ScopeId,
    ) -> Option<MutableRange> {
        updates.get(&scope_id).copied()
    }

    // Helper: update scope range in the updates map
    fn update_scope_range(
        updates: &mut FxHashMap<ScopeId, MutableRange>,
        scope_id: ScopeId,
        new_range: MutableRange,
    ) {
        updates.insert(scope_id, new_range);
    }

    // We need to collect scope modifications and apply them after iteration,
    // since we can't mutate scopes while iterating over blocks.
    //
    // Strategy: collect all scope range modifications (keyed by ScopeId) during
    // the traversal, then apply them in a final pass over all identifiers.

    let mut scope_range_updates: FxHashMap<ScopeId, MutableRange> = FxHashMap::default();

    // Collect initial scope ranges from all identifiers
    {
        for block in func.body.blocks.values() {
            for instr in &block.instructions {
                if let Some(scope) = &instr.lvalue.identifier.scope {
                    scope_range_updates.entry(scope.id).or_insert(scope.range);
                }
                for operand in each_instruction_value_operand(&instr.value) {
                    if let Some(scope) = &operand.identifier.scope {
                        scope_range_updates.entry(scope.id).or_insert(scope.range);
                    }
                }
                for lvalue in each_instruction_lvalue(instr) {
                    if let Some(scope) = &lvalue.identifier.scope {
                        scope_range_updates.entry(scope.id).or_insert(scope.range);
                    }
                }
            }
            for operand in each_terminal_operand(&block.terminal) {
                if let Some(scope) = &operand.identifier.scope {
                    scope_range_updates.entry(scope.id).or_insert(scope.range);
                }
            }
        }
    }

    // Active scope tracking (using ScopeId + current range from updates)
    let mut active_block_fallthrough_ranges: Vec<FallthroughRange> = Vec::new();
    let mut active_scopes: FxHashSet<ScopeId> = FxHashSet::default();
    let mut seen: FxHashSet<ScopeId> = FxHashSet::default();
    let mut value_block_nodes: FxHashMap<BlockId, ValueBlockNodeRef> = FxHashMap::default();

    // We track "place scopes" — mapping from place identity to scope ID
    // (not used for anything in the output but kept for TS fidelity)

    // We need sorted block iteration (reverse postorder, approximated by sorted BlockIds)
    let mut block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    block_ids.sort();

    for block_id in &block_ids {
        let Some(block) = func.body.blocks.get(block_id) else {
            continue;
        };

        let starting_id = block.instructions.first().map_or_else(|| block.terminal.id(), |i| i.id);

        // Filter active scopes — remove expired ones
        active_scopes.retain(|scope_id| {
            if let Some(range) = get_scope_range(&scope_range_updates, *scope_id) {
                range.end > starting_id
            } else {
                false
            }
        });

        // Handle fallthrough targets: extend active scope starts
        let top = active_block_fallthrough_ranges.last().cloned();
        if let Some(ref top) = top
            && top.fallthrough == *block_id
        {
            active_block_fallthrough_ranges.pop();
            for &scope_id in &active_scopes {
                if let Some(range) = get_scope_range(&scope_range_updates, scope_id) {
                    let new_start = InstructionId(range.start.0.min(top.range.start.0));
                    update_scope_range(
                        &mut scope_range_updates,
                        scope_id,
                        MutableRange { start: new_start, end: range.end },
                    );
                }
            }
        }

        // Get the value block node for this block (if any)
        let node_ref = value_block_nodes.get(block_id).copied();

        // Process instructions
        for instr in &block.instructions {
            // Record lvalues
            for lvalue in each_instruction_lvalue(instr) {
                record_place(
                    instr.id,
                    lvalue,
                    node_ref.as_ref(),
                    &mut active_scopes,
                    &mut seen,
                    &mut scope_range_updates,
                );
            }
            // Record operands
            for operand in each_instruction_value_operand(&instr.value) {
                record_place(
                    instr.id,
                    operand,
                    node_ref.as_ref(),
                    &mut active_scopes,
                    &mut seen,
                    &mut scope_range_updates,
                );
            }
        }

        // Process terminal operands
        let terminal = &block.terminal;
        for operand in each_terminal_operand(terminal) {
            record_place(
                terminal.id(),
                operand,
                node_ref.as_ref(),
                &mut active_scopes,
                &mut seen,
                &mut scope_range_updates,
            );
        }

        // Handle terminal fallthroughs and successors
        let fallthrough = terminal_fallthrough(terminal);
        let terminal_kind = terminal_kind_str(terminal);
        let terminal_id = terminal.id();

        if let Some(ft) = fallthrough {
            if terminal_kind != "branch" {
                // Extend active scopes that overlap the block-fallthrough range
                let fallthrough_block = func.body.blocks.get(&ft);
                let next_id = fallthrough_block
                    .and_then(|b| b.instructions.first().map(|i| i.id))
                    .unwrap_or_else(|| fallthrough_block.map_or(terminal_id, |b| b.terminal.id()));

                for &scope_id in &active_scopes {
                    if let Some(range) = get_scope_range(&scope_range_updates, scope_id)
                        && range.end > terminal_id
                    {
                        let new_end = InstructionId(range.end.0.max(next_id.0));
                        update_scope_range(
                            &mut scope_range_updates,
                            scope_id,
                            MutableRange { start: range.start, end: new_end },
                        );
                    }
                }

                // Record block-fallthrough range for future scopes
                active_block_fallthrough_ranges.push(FallthroughRange {
                    fallthrough: ft,
                    range: MutableRange { start: terminal_id, end: next_id },
                });

                // Propagate value block node to fallthrough
                if let Some(nr) = node_ref
                    && !value_block_nodes.contains_key(&ft)
                {
                    value_block_nodes.insert(ft, nr);
                }
            }
        } else if terminal_kind == "goto" {
            // Handle goto to a label (not natural fallthrough)
            if let crate::hir::Terminal::Goto(goto) = terminal {
                let goto_target = goto.block;
                let found = active_block_fallthrough_ranges
                    .iter()
                    .position(|r| r.fallthrough == goto_target);
                let is_last = found.map(|i| i == active_block_fallthrough_ranges.len() - 1);

                if let Some(idx) = found
                    && is_last != Some(true)
                {
                    let start = &active_block_fallthrough_ranges[idx];
                    let start_range = start.range;
                    let start_fallthrough = start.fallthrough;

                    let fallthrough_block = func.body.blocks.get(&start_fallthrough);
                    let first_id = fallthrough_block
                        .and_then(|b| b.instructions.first().map(|i| i.id))
                        .unwrap_or_else(|| {
                            fallthrough_block.map_or(terminal_id, |b| b.terminal.id())
                        });

                    for &scope_id in &active_scopes {
                        if let Some(range) = get_scope_range(&scope_range_updates, scope_id) {
                            if range.end <= terminal_id {
                                continue;
                            }
                            let new_start = InstructionId(start_range.start.0.min(range.start.0));
                            let new_end = InstructionId(first_id.0.max(range.end.0));
                            update_scope_range(
                                &mut scope_range_updates,
                                scope_id,
                                MutableRange { start: new_start, end: new_end },
                            );
                        }
                    }
                }
            }
        }

        // Visit all successors to set value block nodes
        let successors = each_terminal_successor(terminal);
        for successor in successors {
            if value_block_nodes.contains_key(&successor) {
                continue;
            }

            let successor_block = func.body.blocks.get(&successor);
            let successor_kind = successor_block.map(|b| b.kind);

            if matches!(successor_kind, Some(BlockKind::Block | BlockKind::Catch)) {
                // Block or catch — don't create value block node
            } else if node_ref.is_none()
                || terminal_kind == "ternary"
                || terminal_kind == "logical"
                || terminal_kind == "optional"
            {
                // Create a new value block node
                let value_range = if node_ref.is_none() {
                    // Transition from block -> value block
                    if let Some(ft) = fallthrough {
                        let fallthrough_block = func.body.blocks.get(&ft);
                        let next_id = fallthrough_block
                            .and_then(|b| b.instructions.first().map(|i| i.id))
                            .unwrap_or_else(|| {
                                fallthrough_block.map_or(terminal_id, |b| b.terminal.id())
                            });
                        MutableRange { start: terminal_id, end: next_id }
                    } else {
                        // Fallthrough should exist for value blocks
                        MutableRange { start: terminal_id, end: terminal_id }
                    }
                } else {
                    // Value -> value transition, reuse the range
                    node_ref
                        .as_ref()
                        .map_or(MutableRange { start: terminal_id, end: terminal_id }, |n| {
                            n.value_range
                        })
                };

                value_block_nodes.insert(successor, ValueBlockNodeRef { value_range });
            } else if let Some(ref nr) = node_ref {
                // Value -> value block transition, reuse the node
                value_block_nodes.insert(successor, *nr);
            }
        }
    }

    // Final pass: apply the updated scope ranges to all identifiers
    apply_scope_range_updates(func, &scope_range_updates);
}

/// Record a place during the align-reactive-scopes pass.
///
/// Port of the `recordPlace` inner function from `AlignReactiveScopesToBlockScopesHIR.ts`.
fn record_place(
    id: InstructionId,
    place: &Place,
    node_ref: Option<&ValueBlockNodeRef>,
    active_scopes: &mut FxHashSet<ScopeId>,
    seen: &mut FxHashSet<ScopeId>,
    scope_range_updates: &mut FxHashMap<ScopeId, MutableRange>,
) {
    let scope = get_place_scope(id, place);
    let Some(scope) = scope else { return };
    let scope_id = scope.id;

    active_scopes.insert(scope_id);

    if seen.contains(&scope_id) {
        return;
    }
    seen.insert(scope_id);

    if let Some(nr) = node_ref {
        // Extend scope range to cover the value block range
        if let Some(range) = scope_range_updates.get(&scope_id).copied() {
            let new_start = InstructionId(nr.value_range.start.0.min(range.start.0));
            let new_end = InstructionId(nr.value_range.end.0.max(range.end.0));
            scope_range_updates.insert(scope_id, MutableRange { start: new_start, end: new_end });
        }
    }
}

/// Apply final scope range updates to all identifiers in the HIR.
fn apply_scope_range_updates(
    func: &mut HIRFunction,
    scope_range_updates: &FxHashMap<ScopeId, MutableRange>,
) {
    fn apply_to_place(place: &mut Place, updates: &FxHashMap<ScopeId, MutableRange>) {
        if let Some(scope) = &mut place.identifier.scope
            && let Some(&new_range) = updates.get(&scope.id)
        {
            scope.range = new_range;
        }
    }

    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        let Some(block) = func.body.blocks.get_mut(&block_id) else {
            continue;
        };

        // Apply to phi nodes
        for phi in &mut block.phis {
            apply_to_place(&mut phi.place, scope_range_updates);
            for operand in phi.operands.values_mut() {
                apply_to_place(operand, scope_range_updates);
            }
        }

        // Apply to instructions
        for instr in &mut block.instructions {
            apply_to_place(&mut instr.lvalue, scope_range_updates);
            apply_to_instruction_value(&mut instr.value, scope_range_updates);
        }

        // Apply to terminal operands
        apply_to_terminal(&mut block.terminal, scope_range_updates);
    }
}

/// Apply scope range updates within an instruction value.
fn apply_to_instruction_value(
    value: &mut InstructionValue,
    updates: &FxHashMap<ScopeId, MutableRange>,
) {
    fn apply(place: &mut Place, updates: &FxHashMap<ScopeId, MutableRange>) {
        if let Some(scope) = &mut place.identifier.scope
            && let Some(&new_range) = updates.get(&scope.id)
        {
            scope.range = new_range;
        }
    }

    fn apply_call_args(
        args: &mut [crate::hir::CallArg],
        updates: &FxHashMap<ScopeId, MutableRange>,
    ) {
        for arg in args.iter_mut() {
            match arg {
                crate::hir::CallArg::Place(p) => apply(p, updates),
                crate::hir::CallArg::Spread(s) => apply(&mut s.place, updates),
            }
        }
    }

    fn apply_pattern(
        pattern: &mut crate::hir::Pattern,
        updates: &FxHashMap<ScopeId, MutableRange>,
    ) {
        match pattern {
            crate::hir::Pattern::Array(arr) => {
                for item in &mut arr.items {
                    match item {
                        crate::hir::ArrayPatternElement::Place(p) => {
                            apply(p, updates);
                        }
                        crate::hir::ArrayPatternElement::Spread(s) => {
                            apply(&mut s.place, updates);
                        }
                        crate::hir::ArrayPatternElement::Hole => {}
                    }
                }
            }
            crate::hir::Pattern::Object(obj) => {
                for prop in &mut obj.properties {
                    match prop {
                        crate::hir::ObjectPatternProperty::Property(p) => {
                            if let crate::hir::ObjectPropertyKey::Computed(ref mut place) = p.key {
                                apply(place, updates);
                            }
                            apply(&mut p.place, updates);
                        }
                        crate::hir::ObjectPatternProperty::Spread(s) => {
                            apply(&mut s.place, updates);
                        }
                    }
                }
            }
        }
    }

    match value {
        InstructionValue::BinaryExpression(v) => {
            apply(&mut v.left, updates);
            apply(&mut v.right, updates);
        }
        InstructionValue::UnaryExpression(v) => {
            apply(&mut v.value, updates);
        }
        InstructionValue::LoadLocal(v) => {
            apply(&mut v.place, updates);
        }
        InstructionValue::LoadContext(v) => {
            apply(&mut v.place, updates);
        }
        InstructionValue::StoreLocal(v) => {
            apply(&mut v.lvalue.place, updates);
            apply(&mut v.value, updates);
        }
        InstructionValue::StoreContext(v) => {
            apply(&mut v.lvalue_place, updates);
            apply(&mut v.value, updates);
        }
        InstructionValue::StoreGlobal(v) => {
            apply(&mut v.value, updates);
        }
        InstructionValue::Destructure(v) => {
            apply_pattern(&mut v.lvalue.pattern, updates);
            apply(&mut v.value, updates);
        }
        InstructionValue::PropertyLoad(v) => {
            apply(&mut v.object, updates);
        }
        InstructionValue::PropertyStore(v) => {
            apply(&mut v.object, updates);
            apply(&mut v.value, updates);
        }
        InstructionValue::PropertyDelete(v) => {
            apply(&mut v.object, updates);
        }
        InstructionValue::ComputedLoad(v) => {
            apply(&mut v.object, updates);
            apply(&mut v.property, updates);
        }
        InstructionValue::ComputedStore(v) => {
            apply(&mut v.object, updates);
            apply(&mut v.property, updates);
            apply(&mut v.value, updates);
        }
        InstructionValue::ComputedDelete(v) => {
            apply(&mut v.object, updates);
            apply(&mut v.property, updates);
        }
        InstructionValue::CallExpression(v) => {
            apply(&mut v.callee, updates);
            apply_call_args(&mut v.args, updates);
        }
        InstructionValue::NewExpression(v) => {
            apply(&mut v.callee, updates);
            apply_call_args(&mut v.args, updates);
        }
        InstructionValue::MethodCall(v) => {
            apply(&mut v.receiver, updates);
            apply(&mut v.property, updates);
            apply_call_args(&mut v.args, updates);
        }
        InstructionValue::TypeCastExpression(v) => {
            apply(&mut v.value, updates);
        }
        InstructionValue::JsxExpression(v) => {
            if let crate::hir::JsxTag::Place(ref mut p) = v.tag {
                apply(p, updates);
            }
            for attr in &mut v.props {
                match attr {
                    crate::hir::JsxAttribute::Attribute { place, .. } => {
                        apply(place, updates);
                    }
                    crate::hir::JsxAttribute::Spread { argument } => {
                        apply(argument, updates);
                    }
                }
            }
            if let Some(children) = &mut v.children {
                for child in children.iter_mut() {
                    apply(child, updates);
                }
            }
        }
        InstructionValue::JsxFragment(v) => {
            for child in &mut v.children {
                apply(child, updates);
            }
        }
        InstructionValue::ObjectExpression(v) => {
            for prop in &mut v.properties {
                match prop {
                    crate::hir::ObjectPatternProperty::Property(p) => {
                        if let crate::hir::ObjectPropertyKey::Computed(ref mut place) = p.key {
                            apply(place, updates);
                        }
                        apply(&mut p.place, updates);
                    }
                    crate::hir::ObjectPatternProperty::Spread(s) => {
                        apply(&mut s.place, updates);
                    }
                }
            }
        }
        InstructionValue::ArrayExpression(v) => {
            for elem in &mut v.elements {
                match elem {
                    crate::hir::ArrayExpressionElement::Place(p) => {
                        apply(p, updates);
                    }
                    crate::hir::ArrayExpressionElement::Spread(s) => {
                        apply(&mut s.place, updates);
                    }
                    crate::hir::ArrayExpressionElement::Hole => {}
                }
            }
        }
        InstructionValue::FunctionExpression(v) => {
            for ctx in &mut v.lowered_func.func.context {
                apply(ctx, updates);
            }
        }
        InstructionValue::ObjectMethod(v) => {
            for ctx in &mut v.lowered_func.func.context {
                apply(ctx, updates);
            }
        }
        InstructionValue::TaggedTemplateExpression(v) => {
            apply(&mut v.tag, updates);
        }
        InstructionValue::TemplateLiteral(v) => {
            for subexpr in &mut v.subexprs {
                apply(subexpr, updates);
            }
        }
        InstructionValue::Await(v) => {
            apply(&mut v.value, updates);
        }
        InstructionValue::GetIterator(v) => {
            apply(&mut v.collection, updates);
        }
        InstructionValue::IteratorNext(v) => {
            apply(&mut v.iterator, updates);
            apply(&mut v.collection, updates);
        }
        InstructionValue::NextPropertyOf(v) => {
            apply(&mut v.value, updates);
        }
        InstructionValue::PrefixUpdate(v) => {
            apply(&mut v.lvalue, updates);
            apply(&mut v.value, updates);
        }
        InstructionValue::PostfixUpdate(v) => {
            apply(&mut v.lvalue, updates);
            apply(&mut v.value, updates);
        }
        InstructionValue::StartMemoize(v) => {
            if let Some(deps) = &mut v.deps {
                for dep in deps {
                    if let crate::hir::ManualMemoDependencyRoot::NamedLocal {
                        ref mut value, ..
                    } = dep.root
                    {
                        apply(value, updates);
                    }
                }
            }
        }
        InstructionValue::FinishMemoize(v) => {
            apply(&mut v.decl, updates);
        }
        InstructionValue::DeclareLocal(v) => {
            apply(&mut v.lvalue.place, updates);
        }
        InstructionValue::DeclareContext(v) => {
            apply(&mut v.lvalue_place, updates);
        }
        InstructionValue::LoadGlobal(_)
        | InstructionValue::Primitive(_)
        | InstructionValue::JsxText(_)
        | InstructionValue::RegExpLiteral(_)
        | InstructionValue::MetaProperty(_)
        | InstructionValue::Debugger(_)
        | InstructionValue::UnsupportedNode(_) => {}
    }
}

/// Apply scope range updates to terminal operands.
fn apply_to_terminal(
    terminal: &mut crate::hir::Terminal,
    updates: &FxHashMap<ScopeId, MutableRange>,
) {
    fn apply(place: &mut Place, updates: &FxHashMap<ScopeId, MutableRange>) {
        if let Some(scope) = &mut place.identifier.scope
            && let Some(&new_range) = updates.get(&scope.id)
        {
            scope.range = new_range;
        }
    }

    match terminal {
        crate::hir::Terminal::Throw(t) => apply(&mut t.value, updates),
        crate::hir::Terminal::Return(t) => apply(&mut t.value, updates),
        crate::hir::Terminal::If(t) => apply(&mut t.test, updates),
        crate::hir::Terminal::Branch(t) => apply(&mut t.test, updates),
        crate::hir::Terminal::Switch(t) => {
            apply(&mut t.test, updates);
            for case in &mut t.cases {
                if let Some(ref mut test) = case.test {
                    apply(test, updates);
                }
            }
        }
        crate::hir::Terminal::Try(t) => {
            if let Some(ref mut binding) = t.handler_binding {
                apply(binding, updates);
            }
        }
        _ => {}
    }
}

// =====================================================================================
// Helper types and functions
// =====================================================================================

/// A lightweight reference to a value block node, carrying only the value range.
///
/// In the TS code, value block nodes form a tree structure with children tracking
/// nested scopes. For the Rust port, we only need the value range for scope alignment.
#[derive(Clone, Copy)]
struct ValueBlockNodeRef {
    value_range: MutableRange,
}

/// A fallthrough range entry.
#[derive(Clone)]
struct FallthroughRange {
    fallthrough: BlockId,
    range: MutableRange,
}

/// Get a string representation of the terminal kind for comparison.
fn terminal_kind_str(terminal: &crate::hir::Terminal) -> &'static str {
    match terminal {
        crate::hir::Terminal::Unsupported(_) => "unsupported",
        crate::hir::Terminal::Unreachable(_) => "unreachable",
        crate::hir::Terminal::Throw(_) => "throw",
        crate::hir::Terminal::Return(_) => "return",
        crate::hir::Terminal::Goto(_) => "goto",
        crate::hir::Terminal::If(_) => "if",
        crate::hir::Terminal::Branch(_) => "branch",
        crate::hir::Terminal::Switch(_) => "switch",
        crate::hir::Terminal::For(_) => "for",
        crate::hir::Terminal::ForOf(_) => "forOf",
        crate::hir::Terminal::ForIn(_) => "forIn",
        crate::hir::Terminal::DoWhile(_) => "doWhile",
        crate::hir::Terminal::While(_) => "while",
        crate::hir::Terminal::Logical(_) => "logical",
        crate::hir::Terminal::Ternary(_) => "ternary",
        crate::hir::Terminal::Optional(_) => "optional",
        crate::hir::Terminal::Label(_) => "label",
        crate::hir::Terminal::Sequence(_) => "sequence",
        crate::hir::Terminal::MaybeThrow(_) => "maybeThrow",
        crate::hir::Terminal::Try(_) => "try",
        crate::hir::Terminal::Scope(_) => "scope",
        crate::hir::Terminal::PrunedScope(_) => "prunedScope",
    }
}

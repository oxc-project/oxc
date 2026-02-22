/// Merge overlapping reactive scopes in the HIR.
///
/// Port of `HIR/MergeOverlappingReactiveScopesHIR.ts` from the React Compiler.
///
/// While previous passes ensure that reactive scopes span valid sets of program
/// blocks, pairs of reactive scopes may still be inconsistent with respect to
/// each other.
///
/// (a) Reactive scopes ranges must form valid blocks in the resulting javascript
/// program. Any two scopes must either be entirely disjoint or one scope must be
/// nested within the other.
///
/// (b) A scope's own instructions may only mutate that scope. If an instruction
/// inside an inner scope mutates an outer scope, the two must be merged.
///
/// This pass detects overlapping scopes and merges them using a DisjointSet.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    hir::{
        HIRFunction, InstructionId, InstructionValue, MutableRange, Place, ReactiveScope, ScopeId,
        visitors::{each_instruction_lvalue, each_instruction_operand, each_terminal_operand},
    },
    reactive_scopes::infer_reactive_scope_variables::is_mutable,
    utils::disjoint_set::DisjointSet,
};

/// Merge overlapping reactive scopes.
pub fn merge_overlapping_reactive_scopes_hir(func: &mut HIRFunction) {
    // Collect all scopes eagerly because some scopes begin before the first
    // instruction that references them (due to alignReactiveScopesToBlocks).
    let mut scopes_info = collect_scope_info(func);

    // Iterate through scopes and instructions to find which should be merged.
    let mut joined_scopes = get_overlapping_reactive_scopes(func, &mut scopes_info);

    // Merge scope ranges: for each group, extend the root scope range to cover
    // all merged scopes (start = min, end = max).
    let mut merged_ranges: FxHashMap<ScopeId, MutableRange> = FxHashMap::default();
    joined_scopes.for_each(|scope_id, group_id| {
        let scope_range = scopes_info.scope_ranges.get(scope_id).copied();
        if let Some(range) = scope_range {
            let entry = merged_ranges.entry(*group_id).or_insert(range);
            entry.start = InstructionId(entry.start.0.min(range.start.0));
            entry.end = InstructionId(entry.end.0.max(range.end.0));
        }
    });

    // Build a mapping from each scope id to its merged root scope id.
    let scope_mapping = joined_scopes.canonicalize();

    // Rewrite scope references: walk all identifiers again and replace each
    // identifier's scope with the merged root scope.
    rewrite_scopes(func, &scopes_info, &scope_mapping, &merged_ranges);
}

// =============================================================================
// Scope info collection
// =============================================================================

/// Information about a scope's start or end point, used as stack entries
/// sorted in descending order by instruction id (so we can pop from the end).
struct ScopeIdEntry {
    id: InstructionId,
    scopes: FxHashSet<ScopeId>,
}

/// Collected information about all scopes in the function.
struct ScopeInfo {
    /// Scope start points, sorted descending by instruction id.
    scope_starts: Vec<ScopeIdEntry>,
    /// Scope end points, sorted descending by instruction id.
    scope_ends: Vec<ScopeIdEntry>,
    /// Map from scope id to its original range.
    scope_ranges: FxHashMap<ScopeId, MutableRange>,
    /// Set of (IdentifierId) that have scopes, for rewriting. We also track which
    /// scope id was originally assigned to each place, keyed by a place identity
    /// token (block_idx, instr_idx, place_idx, category).
    place_scopes: Vec<PlaceScopeEntry>,
}

/// Identifies a specific place in the HIR for later rewriting.
struct PlaceScopeEntry {
    original_scope_id: ScopeId,
    place_location: PlaceLocation,
}

/// Where a place lives in the HIR, for later rewriting.
#[derive(Clone, Copy)]
enum PlaceLocation {
    InstructionLValue { block_idx: usize, instr_idx: usize },
    InstructionLValueInner { block_idx: usize, instr_idx: usize, inner_idx: usize },
    InstructionOperand { block_idx: usize, instr_idx: usize, operand_idx: usize },
    TerminalOperand { block_idx: usize, operand_idx: usize },
}

/// Get the reactive scope for a place if the scope is active at the given instruction.
fn get_place_scope(id: InstructionId, place: &Place) -> Option<&ReactiveScope> {
    if let Some(ref scope) = place.identifier.scope
        && id >= scope.range.start
        && id < scope.range.end
    {
        return Some(scope);
    }
    None
}

fn collect_scope_info(func: &HIRFunction) -> ScopeInfo {
    let mut scope_starts_map: FxHashMap<InstructionId, FxHashSet<ScopeId>> = FxHashMap::default();
    let mut scope_ends_map: FxHashMap<InstructionId, FxHashSet<ScopeId>> = FxHashMap::default();
    let mut scope_ranges: FxHashMap<ScopeId, MutableRange> = FxHashMap::default();
    let mut place_scopes: Vec<PlaceScopeEntry> = Vec::new();

    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();

    for (block_idx, block_id) in block_ids.iter().enumerate() {
        let block = &func.body.blocks[block_id];

        for (instr_idx, instr) in block.instructions.iter().enumerate() {
            // Collect from lvalues
            {
                // The instruction lvalue itself (index 0 in each_instruction_lvalue)
                if let Some(ref scope) = instr.lvalue.identifier.scope {
                    let sid = scope.id;
                    let range = scope.range;
                    place_scopes.push(PlaceScopeEntry {
                        original_scope_id: sid,
                        place_location: PlaceLocation::InstructionLValue { block_idx, instr_idx },
                    });
                    if range.start != range.end {
                        scope_starts_map.entry(range.start).or_default().insert(sid);
                        scope_ends_map.entry(range.end).or_default().insert(sid);
                    }
                    scope_ranges.entry(sid).or_insert(range);
                }

                // Inner lvalues from the instruction value
                let inner_lvalues = collect_inner_lvalue_scopes(&instr.value);
                for (inner_idx, (sid, range)) in inner_lvalues.iter().enumerate() {
                    place_scopes.push(PlaceScopeEntry {
                        original_scope_id: *sid,
                        place_location: PlaceLocation::InstructionLValueInner {
                            block_idx,
                            instr_idx,
                            inner_idx,
                        },
                    });
                    if range.start != range.end {
                        scope_starts_map.entry(range.start).or_default().insert(*sid);
                        scope_ends_map.entry(range.end).or_default().insert(*sid);
                    }
                    scope_ranges.entry(*sid).or_insert(*range);
                }
            }

            // Collect from operands
            let operands = each_instruction_operand(instr);
            for (operand_idx, operand) in operands.iter().enumerate() {
                if let Some(ref scope) = operand.identifier.scope {
                    let sid = scope.id;
                    let range = scope.range;
                    place_scopes.push(PlaceScopeEntry {
                        original_scope_id: sid,
                        place_location: PlaceLocation::InstructionOperand {
                            block_idx,
                            instr_idx,
                            operand_idx,
                        },
                    });
                    if range.start != range.end {
                        scope_starts_map.entry(range.start).or_default().insert(sid);
                        scope_ends_map.entry(range.end).or_default().insert(sid);
                    }
                    scope_ranges.entry(sid).or_insert(range);
                }
            }
        }

        // Collect from terminal operands
        let term_operands = each_terminal_operand(&block.terminal);
        for (operand_idx, operand) in term_operands.iter().enumerate() {
            if let Some(ref scope) = operand.identifier.scope {
                let sid = scope.id;
                let range = scope.range;
                place_scopes.push(PlaceScopeEntry {
                    original_scope_id: sid,
                    place_location: PlaceLocation::TerminalOperand { block_idx, operand_idx },
                });
                if range.start != range.end {
                    scope_starts_map.entry(range.start).or_default().insert(sid);
                    scope_ends_map.entry(range.end).or_default().insert(sid);
                }
                scope_ranges.entry(sid).or_insert(range);
            }
        }
    }

    // Sort scope starts descending by instruction id (so last() gives the smallest).
    let mut scope_starts: Vec<ScopeIdEntry> =
        scope_starts_map.into_iter().map(|(id, scopes)| ScopeIdEntry { id, scopes }).collect();
    scope_starts.sort_by(|a, b| b.id.cmp(&a.id));

    // Sort scope ends descending by instruction id.
    let mut scope_ends: Vec<ScopeIdEntry> =
        scope_ends_map.into_iter().map(|(id, scopes)| ScopeIdEntry { id, scopes }).collect();
    scope_ends.sort_by(|a, b| b.id.cmp(&a.id));

    ScopeInfo { scope_starts, scope_ends, scope_ranges, place_scopes }
}

/// Collect scope info from inner lvalues of an instruction value (not the
/// instruction's own lvalue, but lvalues inside DeclareLocal, StoreLocal, etc.)
fn collect_inner_lvalue_scopes(value: &InstructionValue) -> Vec<(ScopeId, MutableRange)> {
    let mut result = Vec::new();
    let places: Vec<&Place> = match value {
        InstructionValue::DeclareLocal(v) => vec![&v.lvalue.place],
        InstructionValue::DeclareContext(v) => vec![&v.lvalue_place],
        InstructionValue::StoreLocal(v) => vec![&v.lvalue.place],
        InstructionValue::StoreContext(v) => vec![&v.lvalue_place],
        InstructionValue::Destructure(v) => collect_pattern_places(&v.lvalue.pattern),
        InstructionValue::PrefixUpdate(v) => vec![&v.lvalue],
        InstructionValue::PostfixUpdate(v) => vec![&v.lvalue],
        _ => vec![],
    };
    for place in places {
        if let Some(ref scope) = place.identifier.scope {
            result.push((scope.id, scope.range));
        }
    }
    result
}

fn collect_pattern_places(pattern: &super::hir_types::Pattern) -> Vec<&Place> {
    let mut out = Vec::new();
    match pattern {
        super::hir_types::Pattern::Array(arr) => {
            for item in &arr.items {
                match item {
                    super::hir_types::ArrayPatternElement::Place(p) => out.push(p),
                    super::hir_types::ArrayPatternElement::Spread(s) => out.push(&s.place),
                    super::hir_types::ArrayPatternElement::Hole => {}
                }
            }
        }
        super::hir_types::Pattern::Object(obj) => {
            for prop in &obj.properties {
                match prop {
                    super::hir_types::ObjectPatternProperty::Property(p) => out.push(&p.place),
                    super::hir_types::ObjectPatternProperty::Spread(s) => out.push(&s.place),
                }
            }
        }
    }
    out
}

// =============================================================================
// Overlap detection
// =============================================================================

struct TraversalState {
    joined: DisjointSet<ScopeId>,
    /// Stack of active scopes, stored as (ScopeId, MutableRange).
    active_scopes: Vec<(ScopeId, MutableRange)>,
}

fn visit_instruction_id(id: InstructionId, scope_info: &mut ScopeInfo, state: &mut TraversalState) {
    // Handle all scopes that end at this instruction.
    let should_process_ends = scope_info.scope_ends.last().is_some_and(|top| top.id <= id);

    if should_process_ends {
        let scope_end_top = scope_info.scope_ends.pop();
        if let Some(scope_end_top) = scope_end_top {
            // Sort scopes that end here by start descending, because the
            // active_scopes stack is ordered with later-starting scopes at the end.
            let mut scopes_sorted_start_descending: Vec<ScopeId> =
                scope_end_top.scopes.into_iter().collect();
            scopes_sorted_start_descending.sort_by(|a, b| {
                let range_a = scope_info.scope_ranges.get(a).map_or(InstructionId(0), |r| r.start);
                let range_b = scope_info.scope_ranges.get(b).map_or(InstructionId(0), |r| r.start);
                range_b.cmp(&range_a)
            });

            for scope_id in &scopes_sorted_start_descending {
                let idx = state.active_scopes.iter().position(|(sid, _)| sid == scope_id);
                if let Some(idx) = idx {
                    // Detect and merge all overlapping scopes. active_scopes is ordered
                    // by scope start, so every active scope between a completed scope s
                    // and the top of the stack (1) started later than s and (2) completes
                    // after s.
                    if idx != state.active_scopes.len() - 1 {
                        let mut to_union: Vec<ScopeId> = vec![*scope_id];
                        for (sid, _) in &state.active_scopes[idx + 1..] {
                            to_union.push(*sid);
                        }
                        state.joined.union(&to_union);
                    }
                    state.active_scopes.remove(idx);
                }
            }
        }
    }

    // Handle all scopes that begin at this instruction by adding them
    // to the scopes stack.
    let should_process_starts = scope_info.scope_starts.last().is_some_and(|top| top.id <= id);

    if should_process_starts {
        let scope_start_top = scope_info.scope_starts.pop();
        if let Some(scope_start_top) = scope_start_top {
            // Sort scopes that start here by end descending.
            let mut scopes_sorted_end_descending: Vec<ScopeId> =
                scope_start_top.scopes.into_iter().collect();
            scopes_sorted_end_descending.sort_by(|a, b| {
                let range_a = scope_info.scope_ranges.get(a).map_or(InstructionId(0), |r| r.end);
                let range_b = scope_info.scope_ranges.get(b).map_or(InstructionId(0), |r| r.end);
                range_b.cmp(&range_a)
            });

            for scope_id in &scopes_sorted_end_descending {
                let range = scope_info.scope_ranges.get(scope_id).copied().unwrap_or_default();
                state.active_scopes.push((*scope_id, range));
            }

            // Merge all identical scopes (ones with the same start and end),
            // as they end up with the same reactive block.
            for i in 1..scopes_sorted_end_descending.len() {
                let prev = scopes_sorted_end_descending[i - 1];
                let curr = scopes_sorted_end_descending[i];
                let prev_range = scope_info.scope_ranges.get(&prev);
                let curr_range = scope_info.scope_ranges.get(&curr);
                if let (Some(pr), Some(cr)) = (prev_range, curr_range)
                    && pr.end == cr.end
                {
                    state.joined.union(&[prev, curr]);
                }
            }
        }
    }
}

fn visit_place(id: InstructionId, place: &Place, state: &mut TraversalState) {
    // If an instruction mutates an outer scope, flatten all scopes from the top
    // of the stack to the mutated outer scope.
    let place_scope = get_place_scope(id, place);
    if let Some(scope) = place_scope
        && is_mutable(&place.identifier, id)
    {
        let place_scope_id = scope.id;
        let place_scope_idx =
            state.active_scopes.iter().position(|(sid, _)| *sid == place_scope_id);
        if let Some(idx) = place_scope_idx
            && idx != state.active_scopes.len() - 1
        {
            let mut to_union: Vec<ScopeId> = vec![place_scope_id];
            for (sid, _) in &state.active_scopes[idx + 1..] {
                to_union.push(*sid);
            }
            state.joined.union(&to_union);
        }
    }
}

fn get_overlapping_reactive_scopes(
    func: &HIRFunction,
    scopes_info: &mut ScopeInfo,
) -> DisjointSet<ScopeId> {
    let mut state = TraversalState { joined: DisjointSet::new(), active_scopes: Vec::new() };

    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();

    for block_id in &block_ids {
        let block = &func.body.blocks[block_id];

        for instr in &block.instructions {
            visit_instruction_id(instr.id, scopes_info, &mut state);

            for place in each_instruction_operand(instr) {
                // Skip primitive operands of function expressions and object methods.
                if matches!(
                    instr.value,
                    InstructionValue::FunctionExpression(_) | InstructionValue::ObjectMethod(_)
                ) && place.identifier.is_primitive_type()
                {
                    continue;
                }
                visit_place(instr.id, place, &mut state);
            }

            for place in each_instruction_lvalue(instr) {
                visit_place(instr.id, place, &mut state);
            }
        }

        visit_instruction_id(block.terminal.id(), scopes_info, &mut state);
        for place in each_terminal_operand(&block.terminal) {
            visit_place(block.terminal.id(), place, &mut state);
        }
    }

    state.joined
}

// =============================================================================
// Scope rewriting
// =============================================================================

/// Rewrite all scope references in the function to point to merged root scopes.
fn rewrite_scopes(
    func: &mut HIRFunction,
    scopes_info: &ScopeInfo,
    scope_mapping: &FxHashMap<ScopeId, ScopeId>,
    merged_ranges: &FxHashMap<ScopeId, MutableRange>,
) {
    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();

    for place_entry in &scopes_info.place_scopes {
        let original_scope_id = place_entry.original_scope_id;
        let root_scope_id = match scope_mapping.get(&original_scope_id) {
            Some(root) if *root != original_scope_id => *root,
            _ => continue,
        };

        // Get the merged range for the root scope.
        let merged_range = match merged_ranges.get(&root_scope_id) {
            Some(range) => *range,
            None => continue,
        };

        // Get a mutable reference to the place and update its scope.
        let block_id = block_ids[match place_entry.place_location {
            PlaceLocation::InstructionLValue { block_idx, .. }
            | PlaceLocation::InstructionLValueInner { block_idx, .. }
            | PlaceLocation::InstructionOperand { block_idx, .. }
            | PlaceLocation::TerminalOperand { block_idx, .. } => block_idx,
        }];

        let Some(block) = func.body.blocks.get_mut(&block_id) else { continue };

        match place_entry.place_location {
            PlaceLocation::InstructionLValue { instr_idx, .. } => {
                let instr = &mut block.instructions[instr_idx];
                update_scope(&mut instr.lvalue, root_scope_id, merged_range);
            }
            PlaceLocation::InstructionLValueInner { instr_idx, inner_idx, .. } => {
                let instr = &mut block.instructions[instr_idx];
                update_inner_lvalue_scope(&mut instr.value, inner_idx, root_scope_id, merged_range);
            }
            PlaceLocation::InstructionOperand { instr_idx, operand_idx, .. } => {
                let instr = &mut block.instructions[instr_idx];
                update_operand_scope(&mut instr.value, operand_idx, root_scope_id, merged_range);
            }
            PlaceLocation::TerminalOperand { operand_idx, .. } => {
                update_terminal_operand_scope(
                    &mut block.terminal,
                    operand_idx,
                    root_scope_id,
                    merged_range,
                );
            }
        }
    }
}

fn update_scope(place: &mut Place, root_scope_id: ScopeId, merged_range: MutableRange) {
    if let Some(ref mut scope) = place.identifier.scope {
        scope.id = root_scope_id;
        scope.range = merged_range;
    }
}

fn update_inner_lvalue_scope(
    value: &mut InstructionValue,
    inner_idx: usize,
    root_scope_id: ScopeId,
    merged_range: MutableRange,
) {
    let mut places: Vec<&mut Place> = match value {
        InstructionValue::DeclareLocal(v) => vec![&mut v.lvalue.place],
        InstructionValue::DeclareContext(v) => vec![&mut v.lvalue_place],
        InstructionValue::StoreLocal(v) => vec![&mut v.lvalue.place],
        InstructionValue::StoreContext(v) => vec![&mut v.lvalue_place],
        InstructionValue::Destructure(v) => collect_pattern_places_mut(&mut v.lvalue.pattern),
        InstructionValue::PrefixUpdate(v) => vec![&mut v.lvalue],
        InstructionValue::PostfixUpdate(v) => vec![&mut v.lvalue],
        _ => vec![],
    };
    if let Some(place) = places.get_mut(inner_idx) {
        update_scope(place, root_scope_id, merged_range);
    }
}

fn update_operand_scope(
    value: &mut InstructionValue,
    operand_idx: usize,
    root_scope_id: ScopeId,
    merged_range: MutableRange,
) {
    let mut operands = collect_instruction_value_operands_mut(value);
    if let Some(place) = operands.get_mut(operand_idx) {
        update_scope(place, root_scope_id, merged_range);
    }
}

fn update_terminal_operand_scope(
    terminal: &mut super::hir_types::Terminal,
    operand_idx: usize,
    root_scope_id: ScopeId,
    merged_range: MutableRange,
) {
    let mut operands = collect_terminal_operands_mut(terminal);
    if let Some(place) = operands.get_mut(operand_idx) {
        update_scope(place, root_scope_id, merged_range);
    }
}

// =============================================================================
// Mutable operand/lvalue collection (for rewriting)
// =============================================================================

fn collect_instruction_value_operands_mut(value: &mut InstructionValue) -> Vec<&mut Place> {
    let mut operands = Vec::new();
    match value {
        InstructionValue::CallExpression(v) => {
            operands.push(&mut v.callee);
            collect_call_args_mut(&mut v.args, &mut operands);
        }
        InstructionValue::NewExpression(v) => {
            operands.push(&mut v.callee);
            collect_call_args_mut(&mut v.args, &mut operands);
        }
        InstructionValue::MethodCall(v) => {
            operands.push(&mut v.receiver);
            operands.push(&mut v.property);
            collect_call_args_mut(&mut v.args, &mut operands);
        }
        InstructionValue::BinaryExpression(v) => {
            operands.push(&mut v.left);
            operands.push(&mut v.right);
        }
        InstructionValue::UnaryExpression(v) => {
            operands.push(&mut v.value);
        }
        InstructionValue::TypeCastExpression(v) => {
            operands.push(&mut v.value);
        }
        InstructionValue::LoadLocal(v) => {
            operands.push(&mut v.place);
        }
        InstructionValue::LoadContext(v) => {
            operands.push(&mut v.place);
        }
        InstructionValue::StoreLocal(v) => {
            operands.push(&mut v.value);
        }
        InstructionValue::StoreContext(v) => {
            operands.push(&mut v.value);
        }
        InstructionValue::Destructure(v) => {
            operands.push(&mut v.value);
        }
        InstructionValue::ObjectExpression(v) => {
            collect_object_properties_mut(&mut v.properties, &mut operands);
        }
        InstructionValue::ArrayExpression(v) => {
            for elem in &mut v.elements {
                match elem {
                    super::hir_types::ArrayExpressionElement::Place(p) => operands.push(p),
                    super::hir_types::ArrayExpressionElement::Spread(s) => {
                        operands.push(&mut s.place);
                    }
                    super::hir_types::ArrayExpressionElement::Hole => {}
                }
            }
        }
        InstructionValue::JsxExpression(v) => {
            if let super::hir_types::JsxTag::Place(p) = &mut v.tag {
                operands.push(p);
            }
            for attr in &mut v.props {
                match attr {
                    super::hir_types::JsxAttribute::Spread { argument } => {
                        operands.push(argument);
                    }
                    super::hir_types::JsxAttribute::Attribute { place, .. } => {
                        operands.push(place);
                    }
                }
            }
            if let Some(ref mut children) = v.children {
                for child in children {
                    operands.push(child);
                }
            }
        }
        InstructionValue::JsxFragment(v) => {
            for child in &mut v.children {
                operands.push(child);
            }
        }
        InstructionValue::PropertyLoad(v) => {
            operands.push(&mut v.object);
        }
        InstructionValue::PropertyStore(v) => {
            operands.push(&mut v.object);
            operands.push(&mut v.value);
        }
        InstructionValue::PropertyDelete(v) => {
            operands.push(&mut v.object);
        }
        InstructionValue::ComputedLoad(v) => {
            operands.push(&mut v.object);
            operands.push(&mut v.property);
        }
        InstructionValue::ComputedStore(v) => {
            operands.push(&mut v.object);
            operands.push(&mut v.property);
            operands.push(&mut v.value);
        }
        InstructionValue::ComputedDelete(v) => {
            operands.push(&mut v.object);
            operands.push(&mut v.property);
        }
        InstructionValue::TemplateLiteral(v) => {
            for sub in &mut v.subexprs {
                operands.push(sub);
            }
        }
        InstructionValue::TaggedTemplateExpression(v) => {
            operands.push(&mut v.tag);
        }
        InstructionValue::FunctionExpression(v) => {
            for ctx in &mut v.lowered_func.func.context {
                operands.push(ctx);
            }
        }
        InstructionValue::ObjectMethod(v) => {
            for ctx in &mut v.lowered_func.func.context {
                operands.push(ctx);
            }
        }
        InstructionValue::GetIterator(v) => {
            operands.push(&mut v.collection);
        }
        InstructionValue::IteratorNext(v) => {
            operands.push(&mut v.iterator);
            operands.push(&mut v.collection);
        }
        InstructionValue::NextPropertyOf(v) => {
            operands.push(&mut v.value);
        }
        InstructionValue::PrefixUpdate(v) => {
            operands.push(&mut v.value);
        }
        InstructionValue::PostfixUpdate(v) => {
            operands.push(&mut v.value);
        }
        InstructionValue::Await(v) => {
            operands.push(&mut v.value);
        }
        InstructionValue::StartMemoize(v) => {
            if let Some(ref mut deps) = v.deps {
                for dep in deps {
                    if let super::hir_types::ManualMemoDependencyRoot::NamedLocal {
                        ref mut value,
                        ..
                    } = dep.root
                    {
                        operands.push(value);
                    }
                }
            }
        }
        InstructionValue::FinishMemoize(v) => {
            operands.push(&mut v.decl);
        }
        InstructionValue::Primitive(_)
        | InstructionValue::JsxText(_)
        | InstructionValue::RegExpLiteral(_)
        | InstructionValue::LoadGlobal(_)
        | InstructionValue::StoreGlobal(_)
        | InstructionValue::DeclareLocal(_)
        | InstructionValue::DeclareContext(_)
        | InstructionValue::Debugger(_)
        | InstructionValue::MetaProperty(_)
        | InstructionValue::UnsupportedNode(_) => {}
    }
    operands
}

fn collect_call_args_mut<'a>(
    args: &'a mut [super::hir_types::CallArg],
    out: &mut Vec<&'a mut Place>,
) {
    for arg in args {
        match arg {
            super::hir_types::CallArg::Place(p) => out.push(p),
            super::hir_types::CallArg::Spread(s) => out.push(&mut s.place),
        }
    }
}

fn collect_object_properties_mut<'a>(
    props: &'a mut [super::hir_types::ObjectPatternProperty],
    out: &mut Vec<&'a mut Place>,
) {
    for prop in props {
        match prop {
            super::hir_types::ObjectPatternProperty::Property(p) => {
                if let super::hir_types::ObjectPropertyKey::Computed(ref mut place) = p.key {
                    out.push(place);
                }
                out.push(&mut p.place);
            }
            super::hir_types::ObjectPatternProperty::Spread(s) => {
                out.push(&mut s.place);
            }
        }
    }
}

fn collect_terminal_operands_mut(terminal: &mut super::hir_types::Terminal) -> Vec<&mut Place> {
    let mut operands = Vec::new();
    match terminal {
        super::hir_types::Terminal::Throw(t) => operands.push(&mut t.value),
        super::hir_types::Terminal::Return(t) => operands.push(&mut t.value),
        super::hir_types::Terminal::If(t) => operands.push(&mut t.test),
        super::hir_types::Terminal::Branch(t) => operands.push(&mut t.test),
        super::hir_types::Terminal::Switch(t) => {
            operands.push(&mut t.test);
            for case in &mut t.cases {
                if let Some(ref mut test) = case.test {
                    operands.push(test);
                }
            }
        }
        super::hir_types::Terminal::Try(t) => {
            if let Some(ref mut binding) = t.handler_binding {
                operands.push(binding);
            }
        }
        super::hir_types::Terminal::Unsupported(_)
        | super::hir_types::Terminal::Unreachable(_)
        | super::hir_types::Terminal::Goto(_)
        | super::hir_types::Terminal::For(_)
        | super::hir_types::Terminal::ForOf(_)
        | super::hir_types::Terminal::ForIn(_)
        | super::hir_types::Terminal::DoWhile(_)
        | super::hir_types::Terminal::While(_)
        | super::hir_types::Terminal::Logical(_)
        | super::hir_types::Terminal::Ternary(_)
        | super::hir_types::Terminal::Optional(_)
        | super::hir_types::Terminal::Label(_)
        | super::hir_types::Terminal::Sequence(_)
        | super::hir_types::Terminal::MaybeThrow(_)
        | super::hir_types::Terminal::Scope(_)
        | super::hir_types::Terminal::PrunedScope(_) => {}
    }
    operands
}

fn collect_pattern_places_mut(pattern: &mut super::hir_types::Pattern) -> Vec<&mut Place> {
    let mut out = Vec::new();
    match pattern {
        super::hir_types::Pattern::Array(arr) => {
            for item in &mut arr.items {
                match item {
                    super::hir_types::ArrayPatternElement::Place(p) => out.push(p),
                    super::hir_types::ArrayPatternElement::Spread(s) => out.push(&mut s.place),
                    super::hir_types::ArrayPatternElement::Hole => {}
                }
            }
        }
        super::hir_types::Pattern::Object(obj) => {
            for prop in &mut obj.properties {
                match prop {
                    super::hir_types::ObjectPatternProperty::Property(p) => out.push(&mut p.place),
                    super::hir_types::ObjectPatternProperty::Spread(s) => out.push(&mut s.place),
                }
            }
        }
    }
    out
}

/// Infer reactive places in the HIR.
///
/// Port of `Inference/InferReactivePlaces.ts` from the React Compiler.
///
/// Determines which places (variables) in the function are "reactive" -- meaning
/// they may change between renders and therefore need memoization consideration.
///
/// This pass:
/// 1. Marks function parameters as reactive (props change between renders)
/// 2. Marks hook return values as reactive (hooks access state/context)
/// 3. Uses disjoint sets to group mutably-aliased values so reactivity flows
///    across the alias group
/// 4. Tracks stable values from hooks (e.g., `useRef` return, `useState` setter)
///    so they are NOT marked reactive
/// 5. Propagates reactivity through data flow using a fixpoint iteration:
///    - Forward: reactive operands make lvalues reactive
///    - Backward: mutable operands in reactive instructions become reactive
///    - Control: values assigned under reactive conditions become reactive
/// 6. Propagates reactivity into inner (nested) functions
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    hir::{
        BlockId, Effect, HIRFunction, Identifier, IdentifierId, InstructionValue, Place,
        ReactiveParam, hir_builder::compute_rpo_order,
        object_shape::{BUILT_IN_USE_OPERATOR_ID, HookKind},
        visitors::{each_instruction_lvalue, each_instruction_value_operand},
    },
    reactive_scopes::infer_reactive_scope_variables::{find_disjoint_mutable_values, is_mutable},
    utils::disjoint_set::DisjointSet,
};

use super::control_dominators::create_control_dominators;

// =====================================================================================
// StableSidemap
// =====================================================================================

/// Side map to track and propagate sources of stability (i.e. hook calls such as
/// `useRef()` and property reads such as `useState()[1]`).
///
/// Note that this requires forward data flow analysis since stability is not part
/// of the React Compiler's type system.
struct StableSidemap {
    map: FxHashMap<IdentifierId, StableEntry>,
}

#[derive(Debug, Clone, Copy)]
struct StableEntry {
    is_stable: bool,
}

impl StableSidemap {
    fn new() -> Self {
        Self { map: FxHashMap::default() }
    }

    fn handle_instruction(&mut self, instr: &crate::hir::Instruction) {
        let lvalue = &instr.lvalue;
        match &instr.value {
            InstructionValue::CallExpression(_) | InstructionValue::MethodCall(_) => {
                // Sources of stability are known hook calls
                if evaluates_to_stable_type_or_container(instr) {
                    if is_stable_type(&lvalue.identifier) {
                        self.map.insert(lvalue.identifier.id, StableEntry { is_stable: true });
                    } else {
                        self.map.insert(lvalue.identifier.id, StableEntry { is_stable: false });
                    }
                }
            }
            InstructionValue::Destructure(v) => {
                // PropertyLoads/Destructures from stable containers may produce stable values
                let source = v.value.identifier.id;
                if self.map.contains_key(&source) {
                    for lv in each_instruction_lvalue(instr) {
                        if is_stable_type_container(&lv.identifier) {
                            self.map.insert(lv.identifier.id, StableEntry { is_stable: false });
                        } else if is_stable_type(&lv.identifier) {
                            self.map.insert(lv.identifier.id, StableEntry { is_stable: true });
                        }
                    }
                }
            }
            InstructionValue::PropertyLoad(v) => {
                let source = v.object.identifier.id;
                if self.map.contains_key(&source) {
                    for lv in each_instruction_lvalue(instr) {
                        if is_stable_type_container(&lv.identifier) {
                            self.map.insert(lv.identifier.id, StableEntry { is_stable: false });
                        } else if is_stable_type(&lv.identifier) {
                            self.map.insert(lv.identifier.id, StableEntry { is_stable: true });
                        }
                    }
                }
            }
            InstructionValue::StoreLocal(v) => {
                if let Some(&entry) = self.map.get(&v.value.identifier.id) {
                    self.map.insert(lvalue.identifier.id, entry);
                    self.map.insert(v.lvalue.place.identifier.id, entry);
                }
            }
            InstructionValue::LoadLocal(v) => {
                if let Some(&entry) = self.map.get(&v.place.identifier.id) {
                    self.map.insert(lvalue.identifier.id, entry);
                }
            }
            _ => {}
        }
    }

    fn is_stable(&self, id: IdentifierId) -> bool {
        self.map.get(&id).is_some_and(|entry| entry.is_stable)
    }
}

// =====================================================================================
// ReactivityMap
// =====================================================================================

/// Tracks which identifiers are reactive, using a disjoint set to propagate
/// reactivity across mutably-aliased groups of identifiers.
struct ReactivityMap {
    has_changes: bool,
    reactive: FxHashSet<IdentifierId>,
    aliased_identifiers: DisjointSet<IdentifierId>,
}

impl ReactivityMap {
    fn new(aliased_identifiers: DisjointSet<IdentifierId>) -> Self {
        Self { has_changes: false, reactive: FxHashSet::default(), aliased_identifiers }
    }

    /// Check reactivity of an identifier id.
    fn is_reactive_id(&mut self, id: IdentifierId) -> bool {
        let canonical = self.aliased_identifiers.find(&id).unwrap_or(id);
        self.reactive.contains(&canonical)
    }

    /// Mark an identifier id as reactive.
    fn mark_reactive_id(&mut self, id: IdentifierId) {
        let canonical = self.aliased_identifiers.find(&id).unwrap_or(id);
        if self.reactive.insert(canonical) {
            self.has_changes = true;
        }
    }

    /// Snapshot the change status and reset. Returns `true` if changes occurred
    /// since the last snapshot, indicating another iteration is needed.
    fn snapshot(&mut self) -> bool {
        let has_changes = self.has_changes;
        self.has_changes = false;
        has_changes
    }
}

// =====================================================================================
// Type-checking helpers (ports of TS type predicates)
// =====================================================================================

/// Check if an identifier's type represents a stable type (e.g., setState, dispatch, useRef).
fn is_stable_type(id: &Identifier) -> bool {
    is_set_state_type(id)
        || is_set_action_state_type(id)
        || is_dispatcher_type(id)
        || is_use_ref_type(id)
        || is_start_transition_type(id)
        || is_set_optimistic_type(id)
}

/// Check if an identifier's type represents a stable type container (e.g., useState return).
fn is_stable_type_container(id: &Identifier) -> bool {
    use crate::hir::types::Type;
    let Type::Object(ref obj) = id.type_ else {
        return false;
    };
    is_use_state_type(id)
        || is_use_action_state_type(id)
        || is_use_reducer_type(id)
        || is_use_optimistic_type(id)
        || obj.shape_id.as_deref() == Some("BuiltInUseTransition")
}

fn is_set_state_type(id: &Identifier) -> bool {
    matches!(&id.type_, crate::hir::types::Type::Function(f) if f.shape_id.as_deref() == Some("BuiltInSetState"))
}

fn is_set_action_state_type(id: &Identifier) -> bool {
    matches!(&id.type_, crate::hir::types::Type::Function(f) if f.shape_id.as_deref() == Some("BuiltInSetActionState"))
}

fn is_dispatcher_type(id: &Identifier) -> bool {
    matches!(&id.type_, crate::hir::types::Type::Function(f) if f.shape_id.as_deref() == Some("BuiltInDispatch"))
}

fn is_use_ref_type(id: &Identifier) -> bool {
    matches!(&id.type_, crate::hir::types::Type::Object(o) if o.shape_id.as_deref() == Some("BuiltInUseRefId"))
}

fn is_start_transition_type(id: &Identifier) -> bool {
    matches!(&id.type_, crate::hir::types::Type::Function(f) if f.shape_id.as_deref() == Some("BuiltInStartTransition"))
}

fn is_set_optimistic_type(id: &Identifier) -> bool {
    matches!(&id.type_, crate::hir::types::Type::Function(f) if f.shape_id.as_deref() == Some("BuiltInSetOptimistic"))
}

fn is_use_state_type(id: &Identifier) -> bool {
    matches!(&id.type_, crate::hir::types::Type::Object(o) if o.shape_id.as_deref() == Some("BuiltInUseState"))
}

fn is_use_action_state_type(id: &Identifier) -> bool {
    matches!(&id.type_, crate::hir::types::Type::Object(o) if o.shape_id.as_deref() == Some("BuiltInUseActionState"))
}

fn is_use_reducer_type(id: &Identifier) -> bool {
    matches!(&id.type_, crate::hir::types::Type::Function(f) if f.shape_id.as_deref() == Some("BuiltInUseReducer"))
}

fn is_use_optimistic_type(id: &Identifier) -> bool {
    matches!(&id.type_, crate::hir::types::Type::Object(o) if o.shape_id.as_deref() == Some("BuiltInUseOptimistic"))
}

/// Check if an instruction evaluates to a stable type or container.
/// Only call/method-call instructions to known hooks produce stable values.
fn evaluates_to_stable_type_or_container(instr: &crate::hir::Instruction) -> bool {
    match &instr.value {
        InstructionValue::CallExpression(call) => {
            get_hook_kind_for_identifier(&call.callee.identifier).is_some_and(is_stable_hook_kind)
        }
        InstructionValue::MethodCall(call) => {
            get_hook_kind_for_identifier(&call.property.identifier).is_some_and(is_stable_hook_kind)
        }
        _ => false,
    }
}

fn is_stable_hook_kind(kind: HookKind) -> bool {
    matches!(
        kind,
        HookKind::UseState
            | HookKind::UseReducer
            | HookKind::UseActionState
            | HookKind::UseRef
            | HookKind::UseTransition
            | HookKind::UseOptimistic
    )
}

/// Get the hook kind for an identifier by looking up its type's shape in the registry.
fn get_hook_kind(env: &crate::hir::environment::Environment, id: &Identifier) -> Option<HookKind> {
    let shape_id = match &id.type_ {
        crate::hir::types::Type::Function(f) => f.shape_id.as_deref()?,
        _ => return None,
    };
    let shape = env.shapes.get(shape_id)?;
    shape.function_type.as_ref()?.hook_kind
}

/// Get the hook kind based on the identifier's type alone (without environment lookup).
fn get_hook_kind_for_identifier(id: &Identifier) -> Option<HookKind> {
    let shape_id = match &id.type_ {
        crate::hir::types::Type::Function(f) => f.shape_id.as_deref()?,
        _ => return None,
    };
    // Map well-known shape IDs to hook kinds
    match shape_id {
        "BuiltInUseStateHook" => Some(HookKind::UseState),
        "BuiltInUseReducerHook" => Some(HookKind::UseReducer),
        "BuiltInUseActionStateHook" => Some(HookKind::UseActionState),
        "BuiltInUseRefHook" => Some(HookKind::UseRef),
        "BuiltInUseEffectHook" => Some(HookKind::UseEffect),
        "BuiltInUseLayoutEffectHook" => Some(HookKind::UseLayoutEffect),
        "BuiltInUseInsertionEffectHook" => Some(HookKind::UseInsertionEffect),
        "BuiltInUseContextHook" => Some(HookKind::UseContext),
        "BuiltInUseTransitionHook" => Some(HookKind::UseTransition),
        "BuiltInUseOptimisticHook" => Some(HookKind::UseOptimistic),
        "BuiltInUseEffectEvent" => Some(HookKind::UseEffectEvent),
        _ => None,
    }
}

/// Check if an identifier's type is the `use` operator.
fn is_use_operator(id: &Identifier) -> bool {
    matches!(
        &id.type_,
        crate::hir::types::Type::Function(f) if f.shape_id.as_deref() == Some(BUILT_IN_USE_OPERATOR_ID)
    )
}

// =====================================================================================
// Main entry point
// =====================================================================================

/// Infer which places in the function are reactive.
///
/// Uses a fixpoint iteration to propagate reactivity forward through
/// the control-flow graph. Reactivity propagates through:
/// - Function parameters (props)
/// - Hook return values (state/context access)
/// - Data flow (reactive operands -> lvalues)
/// - Mutation with reactive operands (mutable values that could capture reactive data)
/// - Conditional assignment (values assigned under reactive conditions)
/// - Mutable aliasing (mutably-aliased groups share reactivity)
pub fn infer_reactive_places(func: &mut HIRFunction) {
    // Build disjoint sets of mutably-aliased identifiers. These form the basis
    // for propagating reactivity: if one identifier in a group becomes reactive,
    // all identifiers in the group become reactive.
    let aliased_identifiers = find_disjoint_mutable_values(func);
    let mut reactive_identifiers = ReactivityMap::new(aliased_identifiers);
    let mut stable_identifier_sources = StableSidemap::new();

    // Mark all function parameters as reactive (they may change between renders)
    for param in &mut func.params {
        let place = match param {
            ReactiveParam::Place(p) => p,
            ReactiveParam::Spread(s) => &mut s.place,
        };
        reactive_identifiers.mark_reactive_id(place.identifier.id);
        place.reactive = true;
    }

    // Build a RPO-ordered list of block IDs for deterministic iteration.
    let block_ids: Vec<BlockId> = compute_rpo_order(func.body.entry, &func.body.blocks);

    loop {
        // Phase 1: Pre-compute control dominator info for all blocks.
        // We need to do this before mutating blocks because create_control_dominators
        // borrows func immutably.
        let reactive_control_map: FxHashMap<BlockId, bool> = {
            let reactive_snapshot: FxHashSet<IdentifierId> = reactive_identifiers.reactive.clone();
            let canonical_map = {
                let mut aliased_for_control = find_disjoint_mutable_values(func);
                aliased_for_control.canonicalize()
            };

            let is_reactive_for_control = |place: &Place| -> bool {
                let canonical =
                    canonical_map.get(&place.identifier.id).copied().unwrap_or(place.identifier.id);
                reactive_snapshot.contains(&canonical)
            };

            let is_reactive_controlled_block =
                create_control_dominators(func, &is_reactive_for_control);

            // Also pre-compute for each phi's predecessor blocks
            let mut map = FxHashMap::default();
            for &block_id in &block_ids {
                map.insert(block_id, is_reactive_controlled_block(block_id));
            }
            // We also need to check predecessor blocks of phis, which may not
            // be in block_ids. Collect all unique predecessor block ids from phis.
            for &block_id in &block_ids {
                if let Some(block) = func.body.blocks.get(&block_id) {
                    for phi in &block.phis {
                        for &pred in phi.operands.keys() {
                            map.entry(pred).or_insert_with(|| is_reactive_controlled_block(pred));
                        }
                    }
                }
            }
            map
        };

        // Phase 2: Propagate reactivity through data flow (mutating blocks).
        for &block_id in &block_ids {
            let has_reactive_control =
                reactive_control_map.get(&block_id).copied().unwrap_or(false);

            // Process phi nodes
            if let Some(block) = func.body.blocks.get(&block_id) {
                let phi_updates: Vec<(IdentifierId, bool)> = block
                    .phis
                    .iter()
                    .map(|phi| {
                        let phi_id = phi.place.identifier.id;

                        // Already reactive?
                        if reactive_identifiers.is_reactive_id(phi_id) {
                            return (phi_id, false); // no new change needed
                        }

                        // Any operand reactive?
                        let mut is_phi_reactive = false;
                        for operand in phi.operands.values() {
                            if reactive_identifiers.is_reactive_id(operand.identifier.id) {
                                is_phi_reactive = true;
                                break;
                            }
                        }

                        if !is_phi_reactive {
                            // Check if any predecessor is reactively controlled
                            for &pred in phi.operands.keys() {
                                if reactive_control_map.get(&pred).copied().unwrap_or(false) {
                                    is_phi_reactive = true;
                                    break;
                                }
                            }
                        }

                        (phi_id, is_phi_reactive)
                    })
                    .collect();

                // Apply phi reactivity updates
                for (phi_id, should_mark) in phi_updates {
                    if should_mark {
                        reactive_identifiers.mark_reactive_id(phi_id);
                    }
                }
            }

            // Analyze instructions (read-only pass that updates ReactivityMap)
            {
                let Some(block) = func.body.blocks.get(&block_id) else {
                    continue;
                };

                for instr in &block.instructions {
                    analyze_instruction(
                        instr,
                        &func.env,
                        &mut reactive_identifiers,
                        &mut stable_identifier_sources,
                        has_reactive_control,
                    );
                }
            }

            // Apply reactive marks to instruction places (mutation pass)
            {
                let Some(block) = func.body.blocks.get_mut(&block_id) else {
                    continue;
                };

                for instr in &mut block.instructions {
                    apply_reactive_marks(instr, &mut reactive_identifiers);
                }
            }
        }

        // Phase 3: Mark reactive on phi places and terminal operands
        for &block_id in &block_ids {
            let Some(block) = func.body.blocks.get_mut(&block_id) else {
                continue;
            };

            // Mark phi places
            for phi in &mut block.phis {
                if reactive_identifiers.is_reactive_id(phi.place.identifier.id) {
                    phi.place.reactive = true;
                }
                for operand in phi.operands.values_mut() {
                    if reactive_identifiers.is_reactive_id(operand.identifier.id) {
                        operand.reactive = true;
                    }
                }
            }

            // Process terminal operands
            crate::hir::visitors::map_terminal_operands(&mut block.terminal, &mut |mut place| {
                if reactive_identifiers.is_reactive_id(place.identifier.id) {
                    place.reactive = true;
                }
                place
            });
        }

        if !reactive_identifiers.snapshot() {
            break;
        }
    }

    // Propagate reactivity for inner functions, as we eventually hoist and dedupe
    // dependency instructions for scopes.
    propagate_reactivity_to_inner_functions(func, true, &mut reactive_identifiers);
}

// =====================================================================================
// Per-instruction analysis (read-only pass)
// =====================================================================================

/// Analyze a single instruction for reactivity (read-only pass).
///
/// Updates the `ReactivityMap` and `StableSidemap` but does NOT modify the instruction.
/// The actual reactive flag marking on places happens in `apply_reactive_marks`.
fn analyze_instruction(
    instr: &crate::hir::Instruction,
    env: &crate::hir::environment::Environment,
    reactive_identifiers: &mut ReactivityMap,
    stable_sources: &mut StableSidemap,
    has_reactive_control: bool,
) {
    stable_sources.handle_instruction(instr);

    // Check all operands for reactivity (no short-circuiting so all are checked)
    let mut has_reactive_input = false;
    for operand in each_instruction_value_operand(&instr.value) {
        if reactive_identifiers.is_reactive_id(operand.identifier.id) {
            has_reactive_input = true;
        }
    }

    // Hook calls and the `use` operator are sources of reactivity
    match &instr.value {
        InstructionValue::CallExpression(call) => {
            if get_hook_kind(env, &call.callee.identifier).is_some()
                || is_use_operator(&call.callee.identifier)
            {
                has_reactive_input = true;
            }
        }
        InstructionValue::MethodCall(call) => {
            if get_hook_kind(env, &call.property.identifier).is_some()
                || is_use_operator(&call.property.identifier)
            {
                has_reactive_input = true;
            }
        }
        _ => {}
    }

    // Mark reactive lvalues in the ReactivityMap (skip stable ones)
    if has_reactive_input {
        for lv in each_instruction_lvalue(instr) {
            if stable_sources.is_stable(lv.identifier.id) {
                continue;
            }
            reactive_identifiers.mark_reactive_id(lv.identifier.id);
        }
    }

    // Propagate reactivity to mutable operands
    if has_reactive_input || has_reactive_control {
        for operand in each_instruction_value_operand(&instr.value) {
            match operand.effect {
                Effect::Capture
                | Effect::Store
                | Effect::ConditionallyMutate
                | Effect::ConditionallyMutateIterator
                | Effect::Mutate => {
                    if is_mutable(&operand.identifier, instr.id) {
                        reactive_identifiers.mark_reactive_id(operand.identifier.id);
                    }
                }
                // Freeze, Read, and Unknown effects don't propagate reactivity.
                // Note: The TS version throws an invariant error for Unknown,
                // but we silently skip since some identifiers may not have had
                // effects inferred yet.
                Effect::Freeze | Effect::Read | Effect::Unknown => {}
            }
        }
    }
}

/// Apply reactive marks to all places in an instruction based on current ReactivityMap.
fn apply_reactive_marks(
    instr: &mut crate::hir::Instruction,
    reactive_identifiers: &mut ReactivityMap,
) {
    // Mark operand places
    crate::hir::visitors::map_instruction_value_operands(&mut instr.value, &mut |mut place| {
        if reactive_identifiers.is_reactive_id(place.identifier.id) {
            place.reactive = true;
        }
        place
    });

    // Mark lvalue places
    if reactive_identifiers.is_reactive_id(instr.lvalue.identifier.id) {
        instr.lvalue.reactive = true;
    }
    crate::hir::visitors::map_instruction_lvalues(instr, &mut |mut place| {
        if reactive_identifiers.is_reactive_id(place.identifier.id) {
            place.reactive = true;
        }
        place
    });
}

// =====================================================================================
// Inner function propagation
// =====================================================================================

/// Propagate reactivity markings into inner (nested) functions.
fn propagate_reactivity_to_inner_functions(
    func: &mut HIRFunction,
    is_outermost: bool,
    reactive_identifiers: &mut ReactivityMap,
) {
    let block_ids = compute_rpo_order(func.body.entry, &func.body.blocks);
    for block_id in block_ids {
        // First pass: collect which instructions have nested functions
        let nested_func_indices: Vec<usize> = {
            let Some(block) = func.body.blocks.get(&block_id) else {
                continue;
            };
            block
                .instructions
                .iter()
                .enumerate()
                .filter_map(|(i, instr)| {
                    matches!(
                        instr.value,
                        InstructionValue::FunctionExpression(_) | InstructionValue::ObjectMethod(_)
                    )
                    .then_some(i)
                })
                .collect()
        };

        // If not outermost, mark all operands with their reactive status
        if !is_outermost {
            let Some(block) = func.body.blocks.get_mut(&block_id) else {
                continue;
            };
            for instr in &mut block.instructions {
                crate::hir::visitors::map_instruction_operands(instr, &mut |mut place| {
                    if reactive_identifiers.is_reactive_id(place.identifier.id) {
                        place.reactive = true;
                    }
                    place
                });
            }
        }

        // Recurse into nested functions
        for idx in nested_func_indices {
            let Some(block) = func.body.blocks.get_mut(&block_id) else {
                break;
            };
            let instr = &mut block.instructions[idx];
            match &mut instr.value {
                InstructionValue::FunctionExpression(v) => {
                    propagate_reactivity_to_inner_functions(
                        &mut v.lowered_func.func,
                        false,
                        reactive_identifiers,
                    );
                }
                InstructionValue::ObjectMethod(v) => {
                    propagate_reactivity_to_inner_functions(
                        &mut v.lowered_func.func,
                        false,
                        reactive_identifiers,
                    );
                }
                _ => {}
            }
        }

        // Mark terminal operands for inner functions
        if !is_outermost {
            let Some(block) = func.body.blocks.get_mut(&block_id) else {
                continue;
            };
            crate::hir::visitors::map_terminal_operands(&mut block.terminal, &mut |mut place| {
                if reactive_identifiers.is_reactive_id(place.identifier.id) {
                    place.reactive = true;
                }
                place
            });
        }
    }
}

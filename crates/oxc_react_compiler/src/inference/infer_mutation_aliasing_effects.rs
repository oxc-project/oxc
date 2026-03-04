/// Infer mutation and aliasing effects for instructions and terminals.
///
/// Port of `Inference/InferMutationAliasingEffects.ts` from the React Compiler.
///
/// This is the largest and most complex analysis pass. It performs abstract
/// interpretation over the HIR to determine:
/// - The abstract kind of each value (mutable, primitive, frozen, etc.)
/// - The set of values pointed to by each identifier (aliasing)
/// - The effects each instruction has on values (mutation, freezing, etc.)
///
/// The approach:
/// 1. Determine candidate effects based on instruction syntax and types
/// 2. Abstract interpretation with fixpoint iteration
/// 3. Track abstract value kinds and aliasing through the CFG
/// 4. Apply/rewrite effects based on abstract state
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    compiler_error::{CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory},
    hir::{
        BlockId, Effect, FunctionExpressionValue, HIRFunction, Identifier, IdentifierId,
        Instruction, InstructionKind, InstructionValue, Pattern, Phi, Place, ReactFunctionType,
        ReactiveParam, ValueKind, ValueReason,
        hir_builder::{compute_rpo_order, each_terminal_successor},
        visitors::{PatternItem, each_instruction_value_operand, each_pattern_item},
    },
    inference::aliasing_effects::{AliasingEffect, AliasingSignature, MutationReason},
};

/// Abstract value representation for a place during inference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbstractValue {
    pub kind: ValueKind,
    pub reason: FxHashSet<ValueReason>,
}

impl AbstractValue {
    fn primitive() -> Self {
        Self { kind: ValueKind::Primitive, reason: FxHashSet::default() }
    }

    fn mutable() -> Self {
        let mut reason = FxHashSet::default();
        reason.insert(ValueReason::Other);
        Self { kind: ValueKind::Mutable, reason }
    }

    fn frozen(reasons: FxHashSet<ValueReason>) -> Self {
        Self { kind: ValueKind::Frozen, reason: reasons }
    }

    fn context() -> Self {
        let mut reason = FxHashSet::default();
        reason.insert(ValueReason::Other);
        Self { kind: ValueKind::Context, reason }
    }
}

/// The abstract state during inference — tracks value kinds and aliasing.
#[derive(Debug, Clone)]
struct InferenceState {
    /// Map from identifier ID to its abstract value kind
    values: FxHashMap<IdentifierId, AbstractValue>,
    /// Map from identifier ID to the set of identifiers it may alias
    aliases: FxHashMap<IdentifierId, FxHashSet<IdentifierId>>,
    /// Identity aliases: tracks LoadLocal/StoreLocal chains where identifiers
    /// share the same underlying InstructionValue (in the TS model).
    /// Freeze propagation only follows these identity links, not capture aliases.
    identity_aliases: FxHashMap<IdentifierId, FxHashSet<IdentifierId>>,
    /// Map from identifier ID to the FunctionExpressionValue it was defined as.
    /// Used to resolve Apply effects for locally-defined functions with known
    /// aliasing effects (port of TS `state.values(place)` returning FunctionExpression).
    function_values: FxHashMap<IdentifierId, FunctionExpressionValue>,
}

impl InferenceState {
    fn empty() -> Self {
        Self {
            values: FxHashMap::default(),
            aliases: FxHashMap::default(),
            identity_aliases: FxHashMap::default(),
            function_values: FxHashMap::default(),
        }
    }

    /// Define a place with a given abstract value.
    fn define(&mut self, place: &Place, value: AbstractValue) {
        self.values.insert(place.identifier.id, value);
    }

    /// Get the abstract value for a place.
    fn get(&self, place: &Place) -> Option<&AbstractValue> {
        self.values.get(&place.identifier.id)
    }

    /// Record an alias relationship: `into` may alias `from`.
    fn add_alias(&mut self, from: IdentifierId, into: IdentifierId) {
        self.aliases.entry(into).or_default().insert(from);
    }

    /// Record an identity alias: `into` shares the same underlying value as `from`.
    /// This corresponds to the TS `assign()` operation where identifiers share
    /// InstructionValue references. Freeze propagation follows identity aliases.
    fn add_identity_alias(&mut self, from: IdentifierId, into: IdentifierId) {
        self.aliases.entry(into).or_default().insert(from);
        self.identity_aliases.entry(into).or_default().insert(from);
    }

    /// Freeze a place: transition its abstract value from Mutable/Context/MaybeFrozen to Frozen.
    /// Returns true if the value was actually frozen (was not already frozen/global/primitive).
    ///
    /// Port of `InferenceState.freeze()` from `InferMutationAliasingEffects.ts` (lines 1435-1458).
    /// In the TS code, values are stored per-InstructionValue (shared identity), so freezing one
    /// place automatically freezes all aliases. In Rust, values are per-IdentifierId, so we
    /// must explicitly propagate the freeze to all connected aliases.
    fn freeze(&mut self, id: IdentifierId, reason: ValueReason) -> bool {
        // Check if the target is freezable
        let freezable = match self.values.get(&id) {
            Some(av) => {
                matches!(av.kind, ValueKind::Mutable | ValueKind::Context | ValueKind::MaybeFrozen)
            }
            None => false,
        };
        if !freezable {
            return false;
        }

        // Collect all connected identifiers via identity aliases only.
        // Identity aliases represent LoadLocal/StoreLocal and phi-node connections where
        // values share the same InstructionValue in the TS (shared object identity).
        // Regular aliases (PropertyLoad, Call args) represent capture relationships
        // where the target is a DIFFERENT value — freezing should NOT propagate through them.
        let mut to_freeze = Vec::new();
        to_freeze.push(id);
        self.collect_identity_group(id, &mut to_freeze);
        // Freeze all collected identifiers
        let mut reasons = FxHashSet::default();
        reasons.insert(reason);
        let frozen_value = AbstractValue::frozen(reasons);
        for &freeze_id in &to_freeze {
            if let Some(av) = self.values.get(&freeze_id)
                && matches!(
                    av.kind,
                    ValueKind::Mutable | ValueKind::Context | ValueKind::MaybeFrozen
                )
            {
                self.values.insert(freeze_id, frozen_value.clone());
            }
        }

        // Transitively freeze FunctionExpression context captures.
        // Port of TS `freezeValue()` (lines 1461-1474): when a FunctionExpression is frozen,
        // all of its context captures are also recursively frozen.
        // Both enablePreserveExistingMemoizationGuarantees and
        // enableTransitivelyFreezeFunctionExpressions default to true.
        let context_ids: Vec<IdentifierId> = to_freeze
            .iter()
            .filter_map(|&freeze_id| {
                self.function_values.get(&freeze_id).map(|fn_val| {
                    fn_val
                        .lowered_func
                        .func
                        .context
                        .iter()
                        .map(|p| p.identifier.id)
                        .collect::<Vec<_>>()
                })
            })
            .flatten()
            .collect();
        for ctx_id in context_ids {
            self.freeze(ctx_id, reason);
        }

        true
    }

    /// Freeze a place through identity aliases only (LoadLocal/StoreLocal chains).
    /// Used for JSX operand freezing where we need to propagate freeze through
    /// value-sharing links (TS `assign()` semantics) but NOT through capture
    /// aliases (Call arguments, PropertyLoad etc.) which would over-freeze.
    fn freeze_through_identity(&mut self, id: IdentifierId, reason: ValueReason) -> bool {
        let freezable = match self.values.get(&id) {
            Some(av) => {
                matches!(av.kind, ValueKind::Mutable | ValueKind::Context | ValueKind::MaybeFrozen)
            }
            None => false,
        };
        if !freezable {
            return false;
        }

        let mut to_freeze = Vec::new();
        to_freeze.push(id);
        self.collect_identity_group(id, &mut to_freeze);

        let mut reasons = FxHashSet::default();
        reasons.insert(reason);
        let frozen_value = AbstractValue::frozen(reasons);
        for &freeze_id in &to_freeze {
            if let Some(av) = self.values.get(&freeze_id)
                && matches!(
                    av.kind,
                    ValueKind::Mutable | ValueKind::Context | ValueKind::MaybeFrozen
                )
            {
                self.values.insert(freeze_id, frozen_value.clone());
            }
        }

        // Transitively freeze FunctionExpression context captures (same as in freeze()).
        let context_ids: Vec<IdentifierId> = to_freeze
            .iter()
            .filter_map(|&freeze_id| {
                self.function_values.get(&freeze_id).map(|fn_val| {
                    fn_val
                        .lowered_func
                        .func
                        .context
                        .iter()
                        .map(|p| p.identifier.id)
                        .collect::<Vec<_>>()
                })
            })
            .flatten()
            .collect();
        for ctx_id in context_ids {
            self.freeze(ctx_id, reason);
        }

        true
    }

    /// Collect all identifiers transitively connected to `id` via identity aliases only.
    /// Identity aliases represent LoadLocal/StoreLocal chains where the TS code would
    /// share InstructionValue references (via `assign()`/`appendAlias()`). This is used
    /// by freeze to propagate freeze only through value-sharing links, not through
    /// capture/mutation links (Call arguments, PropertyLoad, etc.).
    fn collect_identity_group(&self, id: IdentifierId, group: &mut Vec<IdentifierId>) {
        let mut visited = FxHashSet::default();
        visited.insert(id);
        let mut stack = vec![id];

        while let Some(current) = stack.pop() {
            // Forward: identity_aliases[current] = {a, b, ...}
            if let Some(alias_set) = self.identity_aliases.get(&current) {
                for &aliased in alias_set {
                    if visited.insert(aliased) {
                        group.push(aliased);
                        stack.push(aliased);
                    }
                }
            }
            // Reverse: find any identifier whose identity alias set contains current
            for (&other_id, other_set) in &self.identity_aliases {
                if other_set.contains(&current) && visited.insert(other_id) {
                    group.push(other_id);
                    stack.push(other_id);
                }
            }
        }
    }

    /// Process a phi node: merge abstract values from all defined operands into the phi's
    /// place identifier. Operands that are not yet defined (backedges in loops) are skipped
    /// — they will be handled later by the fixpoint iteration.
    ///
    /// Port of `InferenceState.inferPhi()` from `InferMutationAliasingEffects.ts`.
    /// The TypeScript version merges the `Set<InstructionValue>` entries for each operand
    /// into the phi's place. In Rust, we instead merge abstract value kinds directly.
    ///
    /// Crucially, we also add identity alias edges from each defined operand to the phi
    /// result. In TypeScript, `inferPhi` copies the same shared `InstructionValue` objects
    /// into the phi's variable set, so when any of those objects gets frozen (e.g., because
    /// `x` is passed as a JSX prop), the phi result is automatically frozen too. In Rust,
    /// we replicate this by making each operand an identity alias of the phi: when
    /// `freeze_through_identity` propagates a freeze to an operand, it follows the
    /// (bidirectional) identity alias graph to reach the phi result as well.
    ///
    /// Without this, a phi like `y$73 = phi(y$44, y$65)` would NOT be frozen when
    /// `y$65` is frozen, causing `y.push` after the phi to be treated as a mutation
    /// of a Mutable value instead of a MaybeFrozen one. That incorrectly extends `y`'s
    /// mutable range through the push call, merging all variables into one reactive scope.
    fn infer_phi(&mut self, phi: &Phi) {
        let phi_id = phi.place.identifier.id;
        let mut merged_value: Option<AbstractValue> = None;
        let mut merged_fn_val: Option<FunctionExpressionValue> = None;

        for operand in phi.operands.values() {
            let op_id = operand.identifier.id;
            if let Some(op_val) = self.values.get(&op_id).cloned() {
                merged_value = Some(match merged_value {
                    None => op_val,
                    Some(existing) => {
                        let kind = merge_value_kinds(existing.kind, op_val.kind);
                        let mut reasons = existing.reason;
                        reasons.extend(op_val.reason.iter().copied());
                        AbstractValue { kind, reason: reasons }
                    }
                });
                // Add an identity alias so that freeze propagation (freeze_through_identity)
                // reaches the phi result when any operand is frozen. This mirrors the TS
                // behavior where the phi's variable set contains the same InstructionValue
                // objects as its operands — freezing one object freezes all sharers.
                self.add_identity_alias(op_id, phi_id);
            }
            // Propagate function_values through phi nodes
            if merged_fn_val.is_none()
                && let Some(fn_val) = self.function_values.get(&op_id).cloned()
            {
                merged_fn_val = Some(fn_val);
            }
        }

        if let Some(value) = merged_value {
            self.values.insert(phi_id, value);
        }

        if let Some(fn_val) = merged_fn_val {
            self.function_values.entry(phi_id).or_insert(fn_val);
        }
    }

    /// Merge another state into this one. Returns true if anything changed.
    fn merge(&mut self, other: &InferenceState) -> bool {
        let mut changed = false;

        for (&id, other_value) in &other.values {
            if let Some(existing) = self.values.get(&id) {
                // Merge value kinds — take the more conservative one
                let merged_kind = merge_value_kinds(existing.kind, other_value.kind);
                if merged_kind != existing.kind {
                    let mut merged_reasons = existing.reason.clone();
                    merged_reasons.extend(other_value.reason.iter().copied());
                    self.values
                        .insert(id, AbstractValue { kind: merged_kind, reason: merged_reasons });
                    changed = true;
                }
            } else {
                self.values.insert(id, other_value.clone());
                changed = true;
            }
        }

        for (&id, other_aliases) in &other.aliases {
            let aliases = self.aliases.entry(id).or_default();
            for &alias in other_aliases {
                if aliases.insert(alias) {
                    changed = true;
                }
            }
        }

        for (&id, other_aliases) in &other.identity_aliases {
            let aliases = self.identity_aliases.entry(id).or_default();
            for &alias in other_aliases {
                if aliases.insert(alias) {
                    changed = true;
                }
            }
        }

        for (&id, other_fn) in &other.function_values {
            if let std::collections::hash_map::Entry::Vacant(e) = self.function_values.entry(id) {
                e.insert(other_fn.clone());
                changed = true;
            }
        }

        changed
    }
}

/// Merge two value kinds using the TS join lattice from `mergeValueKinds`.
///
/// Join lattice:
///   - immutable | mutable   => mutable (callers can distinguish primitive vs object)
///   - frozen    | mutable   => maybe-frozen (callers cannot distinguish frozen vs mutable)
///   - immutable | frozen    => frozen
///   - <any>     | maybe-frozen => maybe-frozen
///   - immutable | context   => context
///   - mutable   | context   => context
///   - frozen    | context   => maybe-frozen
fn merge_value_kinds(a: ValueKind, b: ValueKind) -> ValueKind {
    if a == b {
        return a;
    }
    // 1. Either is MaybeFrozen -> MaybeFrozen
    if a == ValueKind::MaybeFrozen || b == ValueKind::MaybeFrozen {
        return ValueKind::MaybeFrozen;
    }
    // 2. Either is Mutable
    if a == ValueKind::Mutable || b == ValueKind::Mutable {
        if a == ValueKind::Frozen || b == ValueKind::Frozen {
            // frozen | mutable -> MaybeFrozen
            return ValueKind::MaybeFrozen;
        } else if a == ValueKind::Context || b == ValueKind::Context {
            // context | mutable -> Context
            return ValueKind::Context;
        }
        // mutable | immutable (Global/Primitive) -> Mutable
        return ValueKind::Mutable;
    }
    // 3. Either is Context (neither is Mutable/MaybeFrozen at this point)
    if a == ValueKind::Context || b == ValueKind::Context {
        if a == ValueKind::Frozen || b == ValueKind::Frozen {
            // frozen | context -> MaybeFrozen
            return ValueKind::MaybeFrozen;
        }
        // context | immutable -> Context
        return ValueKind::Context;
    }
    // 4. Either is Frozen (neither is Mutable/Context/MaybeFrozen)
    if a == ValueKind::Frozen || b == ValueKind::Frozen {
        // frozen | immutable -> Frozen
        return ValueKind::Frozen;
    }
    // 5. Either is Global
    if a == ValueKind::Global || b == ValueKind::Global {
        return ValueKind::Global;
    }
    // 6. Both Primitive
    ValueKind::Primitive
}

/// Finds objects created via ObjectPattern spread destructuring
/// (`const {x, ...spread} = ...`) where a) the rvalue is known frozen and
/// b) the spread value cannot possibly be directly mutated. The idea is that
/// for this set of values, we can treat the spread object as frozen.
///
/// Port of `findNonMutatedDestructureSpreads` from `InferMutationAliasingEffects.ts`.
fn find_non_mutating_spreads(func: &HIRFunction) -> FxHashSet<IdentifierId> {
    let mut known_frozen = FxHashSet::default();
    if func.fn_type == ReactFunctionType::Component {
        if let Some(param) = func.params.first()
            && let ReactiveParam::Place(p) = param
        {
            known_frozen.insert(p.identifier.id);
        }
    } else {
        for param in &func.params {
            if let ReactiveParam::Place(p) = param {
                known_frozen.insert(p.identifier.id);
            }
        }
    }

    // Map of temporaries to identifiers for spread objects
    let mut candidate_non_mutating_spreads: FxHashMap<IdentifierId, IdentifierId> =
        FxHashMap::default();
    for block in func.body.blocks.values() {
        if !candidate_non_mutating_spreads.is_empty() {
            for phi in &block.phis {
                for operand in phi.operands.values() {
                    if let Some(&spread) =
                        candidate_non_mutating_spreads.get(&operand.identifier.id)
                    {
                        candidate_non_mutating_spreads.remove(&spread);
                    }
                }
            }
        }
        for instr in &block.instructions {
            let lvalue = &instr.lvalue;
            match &instr.value {
                InstructionValue::Destructure(v) => {
                    if !known_frozen.contains(&v.value.identifier.id)
                        || !matches!(v.lvalue.kind, InstructionKind::Let | InstructionKind::Const)
                    {
                        continue;
                    }
                    if let Pattern::Object(obj) = &v.lvalue.pattern {
                        for prop in &obj.properties {
                            if let crate::hir::ObjectPatternProperty::Spread(s) = prop {
                                candidate_non_mutating_spreads
                                    .insert(s.place.identifier.id, s.place.identifier.id);
                            }
                        }
                    }
                }
                InstructionValue::LoadLocal(v) => {
                    if let Some(&spread) =
                        candidate_non_mutating_spreads.get(&v.place.identifier.id)
                    {
                        candidate_non_mutating_spreads.insert(lvalue.identifier.id, spread);
                    }
                }
                InstructionValue::StoreLocal(v) => {
                    if let Some(&spread) =
                        candidate_non_mutating_spreads.get(&v.value.identifier.id)
                    {
                        candidate_non_mutating_spreads.insert(lvalue.identifier.id, spread);
                        candidate_non_mutating_spreads.insert(v.lvalue.place.identifier.id, spread);
                    }
                }
                InstructionValue::JsxFragment(_) | InstructionValue::JsxExpression(_) => {
                    // Passing objects created with spread to jsx can't mutate them
                }
                InstructionValue::PropertyLoad(_) => {
                    // Properties must be frozen since the original value was frozen
                }
                InstructionValue::CallExpression(call) => {
                    if get_hook_kind(&func.env, &call.callee.identifier).is_some() {
                        // Hook calls have frozen arguments, and non-ref returns are frozen
                        if !is_ref_or_ref_value(&lvalue.identifier) {
                            known_frozen.insert(lvalue.identifier.id);
                        }
                    } else if !candidate_non_mutating_spreads.is_empty() {
                        for operand in each_instruction_value_operand(&instr.value) {
                            if let Some(&spread) =
                                candidate_non_mutating_spreads.get(&operand.identifier.id)
                            {
                                candidate_non_mutating_spreads.remove(&spread);
                            }
                        }
                    }
                }
                InstructionValue::MethodCall(call) => {
                    if get_hook_kind(&func.env, &call.property.identifier).is_some() {
                        if !is_ref_or_ref_value(&lvalue.identifier) {
                            known_frozen.insert(lvalue.identifier.id);
                        }
                    } else if !candidate_non_mutating_spreads.is_empty() {
                        for operand in each_instruction_value_operand(&instr.value) {
                            if let Some(&spread) =
                                candidate_non_mutating_spreads.get(&operand.identifier.id)
                            {
                                candidate_non_mutating_spreads.remove(&spread);
                            }
                        }
                    }
                }
                _ => {
                    if !candidate_non_mutating_spreads.is_empty() {
                        for operand in each_instruction_value_operand(&instr.value) {
                            if let Some(&spread) =
                                candidate_non_mutating_spreads.get(&operand.identifier.id)
                            {
                                candidate_non_mutating_spreads.remove(&spread);
                            }
                        }
                    }
                }
            }
        }
    }

    let mut non_mutating_spreads = FxHashSet::default();
    for (&key, &value) in &candidate_non_mutating_spreads {
        if key == value {
            non_mutating_spreads.insert(key);
        }
    }
    non_mutating_spreads
}

/// Get the hook kind for an identifier by looking up its type's shape in the registry.
fn get_hook_kind(
    env: &crate::hir::environment::Environment,
    id: &Identifier,
) -> Option<crate::hir::object_shape::HookKind> {
    let shape_id = match &id.type_ {
        crate::hir::types::Type::Function(f) => f.shape_id.as_deref()?,
        _ => return None,
    };
    let shape = env.shapes.get(shape_id)?;
    shape.function_type.as_ref()?.hook_kind
}

/// Check if an identifier is a ref or ref value type.
fn is_ref_or_ref_value(id: &Identifier) -> bool {
    matches!(
        &id.type_,
        crate::hir::types::Type::Object(crate::hir::types::ObjectType { shape_id: Some(id) })
            if id == crate::hir::object_shape::BUILT_IN_USE_REF_ID
                || id == crate::hir::object_shape::BUILT_IN_REF_VALUE_ID
    )
}

/// Check if a type is a function returning JSX (or a phi containing a JSX return).
/// Used to identify render helper props in JSX expressions that should get Render effects.
fn is_render_function_type(ty: &crate::hir::types::Type) -> bool {
    match ty {
        crate::hir::types::Type::Function(func_type) => is_jsx_or_phi_jsx(&func_type.return_type),
        _ => false,
    }
}

/// Check if a type is JSX or a phi containing JSX.
fn is_jsx_or_phi_jsx(ty: &crate::hir::types::Type) -> bool {
    match ty {
        crate::hir::types::Type::Object(obj) => {
            obj.shape_id.as_deref() == Some(crate::hir::object_shape::BUILT_IN_JSX_ID)
        }
        crate::hir::types::Type::Phi(phi) => phi.operands.iter().any(is_jsx_or_phi_jsx),
        _ => false,
    }
}

/// Infer mutation/aliasing effects for the given function.
pub fn infer_mutation_aliasing_effects(
    func: &mut HIRFunction,
    options: &InferOptions,
) -> Result<(), crate::compiler_error::CompilerError> {
    let mut initial_state = InferenceState::empty();

    // Initialize context variables
    for ctx_ref in &func.context {
        initial_state.define(ctx_ref, AbstractValue::context());
    }

    // Initialize parameters
    let param_kind = if options.is_function_expression {
        AbstractValue::mutable()
    } else {
        let mut reasons = FxHashSet::default();
        reasons.insert(ValueReason::ReactiveFunctionArgument);
        AbstractValue::frozen(reasons)
    };

    if func.fn_type == ReactFunctionType::Component {
        // Components have at most 2 params: props and ref
        for (i, param) in func.params.iter().enumerate() {
            let place = match param {
                ReactiveParam::Place(p) => p,
                ReactiveParam::Spread(s) => &s.place,
            };
            if i == 0 {
                // Props parameter
                initial_state.define(place, param_kind.clone());
            } else {
                // Ref parameter — always mutable
                initial_state.define(place, AbstractValue::mutable());
            }
        }
    } else {
        for param in &func.params {
            let place = match param {
                ReactiveParam::Place(p) => p,
                ReactiveParam::Spread(s) => &s.place,
            };
            initial_state.define(place, param_kind.clone());
        }
    }

    // Compute non-mutating spreads for the Destructure handling.
    // Port of `findNonMutatedDestructureSpreads` from `InferMutationAliasingEffects.ts`.
    // Computed before the fixpoint loop because it only depends on the HIR structure,
    // not on the inference state. Used in both the fixpoint iteration and the effect
    // annotation replay pass.
    let non_mutating_spreads = find_non_mutating_spreads(func);

    // Fixpoint iteration over the CFG
    // Port of TS fixpoint loop from InferMutationAliasingEffects.ts lines 186-211.
    //
    // `states_by_block` stores the INCOMING state for each block (before inferBlock).
    // This matches the TS: `statesByBlock.set(blockId, incomingState)` at line 203.
    // The queue function compares NEW outgoing state against OLD incoming state to
    // detect whether re-processing is needed.
    let mut states_by_block: FxHashMap<BlockId, InferenceState> = FxHashMap::default();
    // Map of blocks to the incoming state (after merge/phis, before instructions).
    // Used for effect annotation replay to compute per-instruction state.
    let mut incoming_states: FxHashMap<BlockId, InferenceState> = FxHashMap::default();
    // Pending incoming states for each block. Merged incrementally as predecessors complete.
    let mut queued_states: FxHashMap<BlockId, InferenceState> = FxHashMap::default();
    queued_states.insert(func.body.entry, initial_state);

    /// Port of TS `queue()` function (InferMutationAliasingEffects.ts lines 157-174).
    /// Enqueues a successor block with the outgoing state from the current block.
    /// If the block is already queued, merges the states. If not, checks whether
    /// the new state has changed relative to the last incoming state for that block.
    fn queue_block(
        queued_states: &mut FxHashMap<BlockId, InferenceState>,
        states_by_block: &FxHashMap<BlockId, InferenceState>,
        block_id: BlockId,
        state: &InferenceState,
    ) {
        if let Some(existing) = queued_states.get_mut(&block_id) {
            // Already queued — merge the new state into the existing queued state
            existing.merge(state);
        } else {
            // First queue for this block — check if there's new info vs last incoming state
            if let Some(prev_state) = states_by_block.get(&block_id) {
                let mut merged = prev_state.clone();
                if merged.merge(state) {
                    // New info found — queue the merged state
                    queued_states.insert(block_id, merged);
                }
                // else: no change, don't re-queue
            } else {
                // Never processed — queue unconditionally
                queued_states.insert(block_id, state.clone());
            }
        }
    }

    let mut iteration_count = 0;
    let max_iterations = 1000; // Safety limit

    // Fixpoint iteration: process blocks in insertion order (deterministic).
    // The fn.body.blocks IndexMap preserves insertion order (equivalent to TypeScript's Map).
    // Each pass iterates all blocks, only processing those with a queued incoming state.
    while !queued_states.is_empty() && iteration_count < max_iterations {
        iteration_count += 1;

        // Collect block IDs in insertion order for deterministic processing.
        let block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();

        for block_id in block_ids {
            // Pop the queued incoming state for this block (if any).
            let mut state = match queued_states.remove(&block_id) {
                Some(s) => s,
                None => continue,
            };

            // Store the incoming state (before phis/instructions) — matches TS line 203.
            states_by_block.insert(block_id, state.clone());

            let block = match func.body.blocks.get(&block_id) {
                Some(b) => b.clone(),
                None => continue,
            };

            // Process phi nodes before instructions — port of TypeScript inferBlock()
            // which calls `state.inferPhi(phi)` for each phi before processing instructions.
            // This merges abstract values from all predecessor operands into the phi's place,
            // handling control flow merge points correctly.
            for phi in &block.phis {
                state.infer_phi(phi);
            }

            // Store the incoming state (after phis, before instructions) for effect annotation.
            incoming_states.insert(block_id, state.clone());

            // Process instructions.
            for instr in &block.instructions {
                infer_instruction_effects(
                    &mut state,
                    instr,
                    options,
                    &func.env,
                    &non_mutating_spreads,
                    false, // fixpoint phase: don't check invariants
                )?;
            }

            // Queue successor blocks — port of TS lines 207-209.
            let successors = each_terminal_successor(&block.terminal);
            for succ_id in successors {
                queue_block(&mut queued_states, &states_by_block, succ_id, &state);
            }
        }
    }

    // Annotate effects on instructions.
    // For blocks reached during fixpoint iteration, compute effects from abstract state.
    // We replay `infer_instruction_effects` to build per-instruction state, matching the
    // TS behavior where effects are computed inline during the fixpoint loop. This ensures
    // state-dependent effect computations (like FunctionExpression capture demotion) use
    // the state at each instruction's position, not the final block state.
    // For unreachable blocks, set empty effects (not None) so downstream passes
    // (infer_mutation_aliasing_ranges) still process them correctly.
    let block_ids = compute_rpo_order(func.body.entry, &func.body.blocks);
    for block_id in block_ids {
        let Some(block) = func.body.blocks.get_mut(&block_id) else {
            continue;
        };
        if let Some(incoming_state) = incoming_states.get(&block_id) {
            // Replay instructions to build per-instruction state.
            // incoming_state already has phis processed.
            let mut replay_state = incoming_state.clone();
            for instr in &mut block.instructions {
                // Compute effects using the state BEFORE this instruction
                let raw_effects = compute_instruction_effects(
                    &replay_state,
                    instr,
                    &func.env,
                    &non_mutating_spreads,
                )?;
                // Filter mutation effects through abstract state: convert Mutate on
                // frozen/global values to MutateFrozen/MutateGlobal error effects.
                // This matches TS applyEffect behavior where unconditional mutations
                // of frozen values generate diagnostics.
                //
                // NOTE: We only filter Mutate/MutateTransitive effects (not the full
                // filter_substituted_effects) because a full filter would incorrectly
                // drop Capture effects whose destination hasn't been created in the
                // abstract state yet (the state reflects BEFORE this instruction, so
                // newly-created lvalues don't exist yet).
                let effects = filter_mutation_effects(&replay_state, raw_effects);
                instr.effects = Some(effects);

                // Match TS applyEffect for CreateFunction: demote context operands
                // from Capture to Read when they have Global/Frozen/Primitive kind in
                // the outer abstract state. This allows validate_no_freezing_known_mutable_functions
                // to skip mutations of known-global context variables.
                match &mut instr.value {
                    InstructionValue::FunctionExpression(v) => {
                        for ctx in &mut v.lowered_func.func.context {
                            if ctx.effect == Effect::Capture
                                && let Some(av) = replay_state.get(ctx)
                                && matches!(
                                    av.kind,
                                    ValueKind::Primitive
                                        | ValueKind::Frozen
                                        | ValueKind::MaybeFrozen
                                        | ValueKind::Global
                                )
                            {
                                ctx.effect = Effect::Read;
                            }
                        }
                    }
                    InstructionValue::ObjectMethod(v) => {
                        for ctx in &mut v.lowered_func.func.context {
                            if ctx.effect == Effect::Capture
                                && let Some(av) = replay_state.get(ctx)
                                && matches!(
                                    av.kind,
                                    ValueKind::Primitive
                                        | ValueKind::Frozen
                                        | ValueKind::MaybeFrozen
                                        | ValueKind::Global
                                )
                            {
                                ctx.effect = Effect::Read;
                            }
                        }
                    }
                    _ => {}
                }

                // Advance the state past this instruction (replay)
                infer_instruction_effects(
                    &mut replay_state,
                    &*instr,
                    options,
                    &func.env,
                    &non_mutating_spreads,
                    true, // replay phase: check invariants
                )?;
            }
        } else {
            // Unreachable block: set empty effects so downstream passes don't skip
            for instr in &mut block.instructions {
                if instr.effects.is_none() {
                    instr.effects = Some(Vec::new());
                }
            }
        }
    }

    Ok(())
}

/// Options for the inference pass.
#[derive(Debug, Clone, Default)]
pub struct InferOptions {
    pub is_function_expression: bool,
}

/// Infer effects of a single instruction on the abstract state.
///
/// When `check_invariants` is true (replay phase), additional invariant checks
/// are enabled that detect invalid states like uninitialized named identifiers.
fn infer_instruction_effects(
    state: &mut InferenceState,
    instr: &Instruction,
    _options: &InferOptions,
    env: &crate::hir::environment::Environment,
    non_mutating_spreads: &FxHashSet<IdentifierId>,
    check_invariants: bool,
) -> Result<(), CompilerError> {
    let lvalue_id = instr.lvalue.identifier.id;

    match &instr.value {
        // Primitives, binary/unary, and template literals produce primitives
        InstructionValue::Primitive(_)
        | InstructionValue::JsxText(_)
        | InstructionValue::BinaryExpression(_)
        | InstructionValue::UnaryExpression(_)
        | InstructionValue::TemplateLiteral(_) => {
            state.define(&instr.lvalue, AbstractValue::primitive());
        }

        // PostfixUpdate/PrefixUpdate: result is primitive, and the updated lvalue is also primitive
        InstructionValue::PostfixUpdate(v) => {
            state.define(&instr.lvalue, AbstractValue::primitive());
            state.define(&v.lvalue, AbstractValue::primitive());
        }
        InstructionValue::PrefixUpdate(v) => {
            state.define(&instr.lvalue, AbstractValue::primitive());
            state.define(&v.lvalue, AbstractValue::primitive());
        }

        // Object/array/function/regexp/iterator values create mutable values
        InstructionValue::ObjectExpression(_)
        | InstructionValue::ArrayExpression(_)
        | InstructionValue::RegExpLiteral(_)
        | InstructionValue::GetIterator(_)
        | InstructionValue::NextPropertyOf(_) => {
            state.define(&instr.lvalue, AbstractValue::mutable());
        }

        // IteratorNext: propagate collection's kind (CreateFrom effect)
        // In TS, CreateFrom copies the source value's kind to the target.
        // If the collection is Frozen/MaybeFrozen, the iterator element is MaybeFrozen.
        InstructionValue::IteratorNext(v) => {
            if let Some(val) = state.get(&v.collection) {
                let result_kind = match val.kind {
                    ValueKind::Primitive | ValueKind::Global => val.kind,
                    ValueKind::Frozen | ValueKind::MaybeFrozen => ValueKind::MaybeFrozen,
                    _ => ValueKind::Mutable,
                };
                state.define(
                    &instr.lvalue,
                    AbstractValue { kind: result_kind, reason: val.reason.clone() },
                );
            } else {
                state.define(&instr.lvalue, AbstractValue::mutable());
            }
        }

        // FunctionExpression / ObjectMethod: mutable or frozen depending on captures.
        //
        // Port of TS CreateFunction applyEffect (InferMutationAliasingEffects.ts lines 791-858):
        // A function expression is considered "mutable" if it has any context variables
        // with Effect::Capture whose outer abstract kind is Mutable or Context, OR if its
        // inner aliasing effects contain MutateFrozen/MutateGlobal/Impure (tracked side effects).
        //
        // If the function is NOT mutable (all captures are frozen/global/primitive), it is
        // considered "frozen". This is critical because:
        //   - When calling a frozen function (e.g. cb() where cb captures only frozen setState),
        //     MutateTransitiveConditionally(cb) is dropped (no range extension for cb).
        //   - This prevents cb's mutable range from spuriously spanning hook calls, which would
        //     cause FlattenScopesWithHooksOrUse to prune cb's reactive scope.
        InstructionValue::FunctionExpression(v) => {
            let is_mutable = is_function_expression_mutable(state, &v.lowered_func.func);
            if is_mutable {
                state.define(&instr.lvalue, AbstractValue::mutable());
            } else {
                state.define(&instr.lvalue, AbstractValue::frozen(FxHashSet::default()));
            }
            state.function_values.insert(lvalue_id, v.clone());
        }
        InstructionValue::ObjectMethod(v) => {
            let is_mutable = is_function_expression_mutable(state, &v.lowered_func.func);
            if is_mutable {
                state.define(&instr.lvalue, AbstractValue::mutable());
            } else {
                state.define(&instr.lvalue, AbstractValue::frozen(FxHashSet::default()));
            }
        }

        // JSX creates a frozen value
        InstructionValue::JsxExpression(_) | InstructionValue::JsxFragment(_) => {
            let mut reasons = FxHashSet::default();
            reasons.insert(ValueReason::JsxCaptured);
            state.define(&instr.lvalue, AbstractValue::frozen(reasons));
            // Freeze all operands (children, props, tag) to match TS applyEffect(Freeze) behavior.
            // Use freeze_through_identity (not freeze) to propagate freeze through
            // LoadLocal/StoreLocal chains (TS InstructionValue sharing) but NOT through
            // Call argument aliases which would over-freeze.
            for operand in each_instruction_value_operand(&instr.value) {
                state.freeze_through_identity(operand.identifier.id, ValueReason::JsxCaptured);
            }
        }

        // LoadLocal propagates the type of the loaded value
        InstructionValue::LoadLocal(v) => {
            if let Some(val) = state.get(&v.place).cloned() {
                state.define(&instr.lvalue, val);
                state.add_identity_alias(v.place.identifier.id, lvalue_id);
            } else if check_invariants && v.place.identifier.name.is_some() {
                // Invariant: named identifiers should be initialized in the state
                // after fixpoint converges. If they aren't, it means something went
                // wrong with shadowing or function declaration hoisting.
                return Err(CompilerError::invariant(
                    "[InferMutationAliasingEffects] Expected value kind to be initialized",
                    Some(&format!("{:?}", v.place.identifier.name)),
                    v.place.loc,
                ));
            }
            // Propagate function value tracking through LoadLocal
            if let Some(fn_val) = state.function_values.get(&v.place.identifier.id).cloned() {
                state.function_values.insert(lvalue_id, fn_val);
            }
        }

        InstructionValue::LoadContext(v) => {
            if let Some(val) = state.get(&v.place).cloned() {
                state.define(&instr.lvalue, val);
            }
        }

        // StoreLocal propagates from value to lvalue
        InstructionValue::StoreLocal(v) => {
            if let Some(val) = state.get(&v.value).cloned() {
                state.define(&v.lvalue.place, val);
                state.add_identity_alias(v.value.identifier.id, v.lvalue.place.identifier.id);
            } else {
                // Fallback: if the value isn't in the state yet, define lvalue as mutable
                state.define(&v.lvalue.place, AbstractValue::mutable());
            }
            // Propagate function value tracking through StoreLocal
            if let Some(fn_val) = state.function_values.get(&v.value.identifier.id).cloned() {
                state.function_values.insert(v.lvalue.place.identifier.id, fn_val);
            }
        }

        InstructionValue::StoreContext(v) => {
            // Context variables are mutable boxes — when declaring (Let/Const), the box
            // itself is always Mutable regardless of the stored value's type. This matches
            // the TypeScript reference which emits Create(x, ValueKind.Mutable) for the
            // non-reassign case in `computeSignatureForInstruction`.
            //
            // For reassign, don't change the box's abstract kind — the existing Context/Mutable
            // kind is preserved. We should NOT propagate the rvalue's abstract type (e.g.
            // Primitive) to the context variable, since that would incorrectly prevent the
            // context variable from being recognized as a mutable capture in FunctionExpression
            // context handling (is_function_expression_mutable checks for Mutable/Context).
            if v.lvalue_kind != crate::hir::InstructionKind::Reassign {
                state.define(&v.lvalue_place, AbstractValue::mutable());
            }
        }

        // Calls may create, mutate, or alias values
        InstructionValue::CallExpression(v) => {
            // Use the callee's return_value_kind if a known signature exists.
            // This is critical for hooks like useState() which return Frozen values:
            // if we always use mutable() here, downstream destructuring of the result
            // uses CreateFrom instead of ImmutableCapture, causing spurious range
            // extensions (e.g. setState's range getting extended transitively when
            // `state` is used in render, causing the second useEffect lambda to get
            // merged into the useState scope and then pruned by FlattenScopesWithHooksOrUse).
            let sig = env.get_function_signature(&v.callee.identifier.type_).cloned();
            let return_kind = sig.as_ref().map_or(ValueKind::Mutable, |s| s.return_value_kind);
            let av = match return_kind {
                ValueKind::Frozen => {
                    let mut reasons = FxHashSet::default();
                    reasons.insert(ValueReason::Other);
                    AbstractValue::frozen(reasons)
                }
                ValueKind::Primitive => AbstractValue::primitive(),
                _ => AbstractValue::mutable(),
            };
            state.define(&instr.lvalue, av);
            // Arguments may be captured/aliased by the callee
            for arg in &v.args {
                if let crate::hir::CallArg::Place(p) = arg {
                    state.add_alias(p.identifier.id, lvalue_id);
                }
            }
            // Apply freeze effects from the function signature to the abstract state.
            // Port of TS applyEffect for Freeze (InferMutationAliasingEffects.ts lines 685-690):
            // When a parameter has Effect::Freeze, call state.freeze() on the corresponding
            // argument so that downstream MutateTransitiveConditionally effects on frozen
            // values are dropped (they return 'none' for Frozen values).
            if let Some(sig) = &sig {
                apply_signature_freeze_effects(state, sig, &v.args);
            }
        }

        InstructionValue::MethodCall(v) => {
            // Use the method's return_value_kind if a known signature exists.
            let sig = env.get_function_signature(&v.property.identifier.type_).cloned();
            let return_kind = sig.as_ref().map_or(ValueKind::Mutable, |s| s.return_value_kind);
            let av = match return_kind {
                ValueKind::Frozen => {
                    let mut reasons = FxHashSet::default();
                    reasons.insert(ValueReason::Other);
                    AbstractValue::frozen(reasons)
                }
                ValueKind::Primitive => AbstractValue::primitive(),
                _ => AbstractValue::mutable(),
            };
            state.define(&instr.lvalue, av);
            state.add_alias(v.receiver.identifier.id, lvalue_id);
            for arg in &v.args {
                if let crate::hir::CallArg::Place(p) = arg {
                    state.add_alias(p.identifier.id, lvalue_id);
                }
            }
            // Apply freeze effects from the method signature to the abstract state.
            if let Some(sig) = &sig {
                apply_signature_freeze_effects(state, sig, &v.args);
            }
        }

        InstructionValue::NewExpression(_v) => {
            state.define(&instr.lvalue, AbstractValue::mutable());
        }

        // Property operations
        //
        // Port of TS `applyEffect(CreateFrom)` (InferMutationAliasingEffects.ts lines 731-789):
        // The result of a PropertyLoad inherits the abstract kind from the source object:
        // - Primitive/Global sources → result keeps the same kind
        // - Frozen/MaybeFrozen sources → result is MaybeFrozen
        // - Mutable/Context sources → result is Mutable
        // This is critical for globals like `globalThis.globalThis.NaN`: each PropertyLoad
        // in the chain must preserve the Global kind so that downstream
        // MutateTransitiveConditionally effects are correctly dropped (they only apply to
        // Mutable/Context values), preventing spurious mutable range extensions.
        InstructionValue::PropertyLoad(v) => {
            if instr.lvalue.identifier.is_primitive_type() {
                state.define(&instr.lvalue, AbstractValue::primitive());
            } else if let Some(val) = state.get(&v.object) {
                let result_kind = match val.kind {
                    ValueKind::Primitive | ValueKind::Global => val.kind,
                    ValueKind::Frozen | ValueKind::MaybeFrozen => ValueKind::MaybeFrozen,
                    _ => ValueKind::Mutable,
                };
                state.define(
                    &instr.lvalue,
                    AbstractValue { kind: result_kind, reason: val.reason.clone() },
                );
                state.add_alias(v.object.identifier.id, lvalue_id);
            } else {
                state.define(&instr.lvalue, AbstractValue::mutable());
            }
        }

        InstructionValue::PropertyStore(_v) => {
            // Storing to a property doesn't change the lvalue type much
            state.define(&instr.lvalue, AbstractValue::primitive());
        }

        InstructionValue::ComputedLoad(v) => {
            if instr.lvalue.identifier.is_primitive_type() {
                state.define(&instr.lvalue, AbstractValue::primitive());
            } else if let Some(val) = state.get(&v.object) {
                state.define(&instr.lvalue, val.clone());
                state.add_alias(v.object.identifier.id, lvalue_id);
            } else {
                state.define(&instr.lvalue, AbstractValue::mutable());
            }
        }

        // Globals are frozen
        InstructionValue::LoadGlobal(_) => {
            let mut reasons = FxHashSet::default();
            reasons.insert(ValueReason::Global);
            state.define(&instr.lvalue, AbstractValue { kind: ValueKind::Global, reason: reasons });
        }

        InstructionValue::StoreGlobal(_v) => {
            state.define(&instr.lvalue, AbstractValue::primitive());
        }

        // Type casts preserve the underlying value's type
        InstructionValue::TypeCastExpression(v) => {
            if let Some(val) = state.get(&v.value).cloned() {
                state.define(&instr.lvalue, val);
            }
        }

        // Await produces the awaited value's type
        InstructionValue::Await(v) => {
            if let Some(val) = state.get(&v.value).cloned() {
                state.define(&instr.lvalue, val);
            } else {
                state.define(&instr.lvalue, AbstractValue::mutable());
            }
        }

        // Destructure: propagate the source's abstract type to pattern lvalues.
        //
        // Port of TS `applyEffect` for Destructure effects:
        // - Identifier items use CreateFrom, which inherits the source's kind
        // - Spread items use Create(Mutable|Frozen), which creates a NEW object:
        //   Mutable unless the spread is in nonMutatingSpreads (frozen)
        //
        // In the Rust code, the compute_instruction_effects Destructure handler
        // already distinguishes these cases in the effect list. Here in state
        // propagation, we must mirror that: spread items get Mutable or Frozen
        // depending on the nonMutatingSpreads set, while identifier items inherit
        // the source type.
        InstructionValue::Destructure(v) => {
            if let Some(val) = state.get(&v.value).cloned() {
                state.define(&instr.lvalue, val.clone());
                // Use each_pattern_item to distinguish spread vs identifier items
                for item in crate::hir::visitors::each_pattern_item(&v.lvalue.pattern) {
                    match item {
                        crate::hir::visitors::PatternItem::Spread(place) => {
                            // Spread creates a new object. If it's in non_mutating_spreads,
                            // define it as Frozen (matching Create(Frozen) in effects).
                            // Otherwise define as Mutable (matching Create(Mutable)).
                            if non_mutating_spreads.contains(&place.identifier.id) {
                                state.define(place, AbstractValue::frozen(FxHashSet::default()));
                            } else {
                                state.define(place, AbstractValue::mutable());
                            }
                        }
                        crate::hir::visitors::PatternItem::Identifier(place) => {
                            // Primitive-typed identifier items are independent copies,
                            // not aliases of the source value.
                            if place.identifier.is_primitive_type() {
                                state.define(place, AbstractValue::primitive());
                            } else {
                                // Non-primitive identifier items inherit the source's abstract type
                                state.define(place, val.clone());
                            }
                        }
                    }
                }
            } else {
                state.define(&instr.lvalue, AbstractValue::mutable());
            }
        }

        // StartMemoize: freeze the deps (matching TS applyEffect for Freeze)
        // In TS, StartMemoize with enablePreserveExistingMemoizationGuarantees
        // iterates eachInstructionValueOperand (the named local deps) and emits
        // Freeze effects. applyEffect then calls state.freeze(dep) which
        // transitively freezes through alias chains and function captures.
        InstructionValue::StartMemoize(v) => {
            if env.config.enable_preserve_existing_memoization_guarantees
                && let Some(deps) = &v.deps
            {
                for dep in deps {
                    if let crate::hir::ManualMemoDependencyRoot::NamedLocal { value, .. } =
                        &dep.root
                    {
                        state.freeze(value.identifier.id, ValueReason::HookCaptured);
                    }
                }
            }
            state.define(&instr.lvalue, AbstractValue::mutable());
        }

        // FinishMemoize: freeze the declared memoized value (matching TS applyEffect for Freeze)
        // In TS, FinishMemoize with enablePreserveExistingMemoizationGuarantees
        // emits Freeze(decl). applyEffect calls state.freeze(decl), which
        // transitively freezes through alias chains and function captures.
        InstructionValue::FinishMemoize(v) => {
            if env.config.enable_preserve_existing_memoization_guarantees {
                state.freeze(v.decl.identifier.id, ValueReason::HookCaptured);
            }
            state.define(&instr.lvalue, AbstractValue::mutable());
        }

        // DeclareLocal: the declared variable is primitive (undefined),
        // matching the TS applyEffect for Create(Primitive).
        // Without this, y's abstract state is None, causing incorrect
        // MutateTransitiveConditionally effects in conservative calls.
        // Exception: Catch bindings hold the thrown exception object, which
        // is an unknown mutable value, not a primitive.
        InstructionValue::DeclareLocal(v) => {
            if v.lvalue.kind == InstructionKind::Catch {
                state.define(&v.lvalue.place, AbstractValue::mutable());
            } else {
                state.define(&v.lvalue.place, AbstractValue::primitive());
            }
            state.define(&instr.lvalue, AbstractValue::mutable());
        }

        // Other values
        InstructionValue::DeclareContext(_)
        | InstructionValue::Debugger(_)
        | InstructionValue::MetaProperty(_)
        | InstructionValue::TaggedTemplateExpression(_)
        | InstructionValue::ComputedStore(_)
        | InstructionValue::ComputedDelete(_)
        | InstructionValue::PropertyDelete(_)
        | InstructionValue::UnsupportedNode(_) => {
            // Default: assume lvalue is a new mutable value
            state.define(&instr.lvalue, AbstractValue::mutable());
        }
    }
    Ok(())
}

/// Apply freeze effects from a function signature to the abstract state.
///
/// Port of TS `applyEffect` for `Freeze` (InferMutationAliasingEffects.ts lines 685-690).
/// When a parameter has `Effect::Freeze`, we call `state.freeze()` on the corresponding
/// argument. This transitions the argument (and all its aliases) from Mutable to Frozen
/// in the abstract state, so that subsequent `MutateTransitiveConditionally` effects
/// on frozen values return 'none' and don't extend mutable ranges.
fn apply_signature_freeze_effects(
    state: &mut InferenceState,
    sig: &crate::hir::object_shape::FunctionSignature,
    args: &[crate::hir::CallArg],
) {
    use crate::hir::{CallArg, Effect};

    for (i, arg) in args.iter().enumerate() {
        let effect = if i < sig.positional_params.len() {
            sig.positional_params[i]
        } else if let Some(rest) = sig.rest_param {
            rest
        } else {
            continue;
        };
        if effect == Effect::Freeze {
            let place = match arg {
                CallArg::Place(p) => p,
                CallArg::Spread(s) => &s.place,
            };
            state.freeze(place.identifier.id, ValueReason::HookCaptured);
        }
    }
}

/// Generate aliasing effects for a function call based on a known `FunctionSignature`.
///
/// Port of `computeEffectsForLegacySignature` from `InferMutationAliasingEffects.ts` (lines 2316-2497).
///
/// For each parameter, the declared effect determines what aliasing effect is generated:
/// - Read → ImmutableCapture (no mutation)
/// - Capture → deferred, then aliased to return or captured into store targets
/// - Freeze → Freeze the argument
/// - ConditionallyMutate → MutateTransitiveConditionally
/// - Mutate → MutateTransitive
/// - Store → Mutate + track as store target for capture resolution
fn effects_from_signature(
    sig: &crate::hir::object_shape::FunctionSignature,
    callee_or_receiver: &Place,
    args: &[crate::hir::CallArg],
    lvalue: &Place,
    state: &InferenceState,
) -> Result<Vec<AliasingEffect>, CompilerError> {
    use crate::hir::{CallArg, Effect};

    let mut effects = Vec::new();
    let mut captures: Vec<Place> = Vec::new();
    let mut stores: Vec<Place> = Vec::new();

    let return_value_reason = sig.return_value_reason.unwrap_or(ValueReason::Other);

    // Create the return value FIRST so that the lvalue node exists before
    // subsequent Alias/Capture effects reference it.
    //
    // The TypeScript reference emits `Create` before `Alias`/`Capture` in the
    // debug output (e.g. `Create $66; Alias $66 <- $64`). If `Create` comes
    // after the alias effects, then when `InferMutationAliasingRanges` processes
    // the `Alias { from: $64, into: $66 }`, the `$66` node does not yet exist
    // and `state.assign()` returns early without creating the edge. This breaks
    // backward mutation propagation through alias chains.
    effects.push(AliasingEffect::Create {
        into: lvalue.clone(),
        value: sig.return_value_kind,
        reason: return_value_reason,
    });

    // Port of TS InferMutationAliasingEffects.ts lines 2224-2243:
    // If the signature is marked as knownIncompatible, throw an error immediately.
    if let Some(ref incompatible_msg) = sig.known_incompatible {
        let mut errors = CompilerError::new();
        errors.push_diagnostic(
            CompilerDiagnostic::create(
                ErrorCategory::IncompatibleLibrary,
                "Use of incompatible library".to_string(),
                Some(
                    "This API returns functions which cannot be memoized without leading to stale UI. \
                     To prevent this, by default React Compiler will skip memoizing this component/hook. \
                     However, you may see issues if values from this API are passed to other components/hooks that are \
                     memoized"
                        .to_string(),
                ),
                None,
            )
            .with_detail(CompilerDiagnosticDetail::Error {
                loc: Some(callee_or_receiver.loc),
                message: Some(incompatible_msg.clone()),
            }),
        );
        return Err(errors);
    }

    // Helper closure to process a single place with a declared effect
    let mut visit = |place: &Place, effect: Effect| {
        match effect {
            Effect::Store => {
                effects.push(AliasingEffect::Mutate { value: place.clone(), reason: None });
                stores.push(place.clone());
            }
            Effect::Capture => {
                captures.push(place.clone());
            }
            Effect::ConditionallyMutate => {
                effects
                    .push(AliasingEffect::MutateTransitiveConditionally { value: place.clone() });
            }
            Effect::ConditionallyMutateIterator => {
                // Port of TypeScript `conditionallyMutateIterator`:
                // For non-builtin-collection types (e.g. Poly iterator), also emit
                // MutateTransitiveConditionally so that mutations propagate through alias chains.
                // For builtin Array/Set/Map types, no mutation (the collection is safe to read).
                if !is_array_or_set_or_map_type(&place.identifier.type_) {
                    effects.push(AliasingEffect::MutateTransitiveConditionally {
                        value: place.clone(),
                    });
                }
                effects.push(AliasingEffect::Capture { from: place.clone(), into: lvalue.clone() });
            }
            Effect::Freeze => {
                effects.push(AliasingEffect::Freeze {
                    value: place.clone(),
                    reason: return_value_reason,
                });
            }
            Effect::Mutate => {
                effects.push(AliasingEffect::MutateTransitive { value: place.clone() });
            }
            Effect::Read => {
                effects.push(AliasingEffect::ImmutableCapture {
                    from: place.clone(),
                    into: lvalue.clone(),
                });
            }
            _ => {
                // Unknown or other: conservative
                effects
                    .push(AliasingEffect::MutateTransitiveConditionally { value: place.clone() });
            }
        }
    };

    // Apply callee effect to the receiver/callee
    visit(callee_or_receiver, sig.callee_effect);

    // Process each argument with its declared effect.
    // Port of TS `getArgumentEffect` (InferMutationAliasingEffects.ts line 2681-2697):
    // When an argument is a Spread and the signature effect is Freeze, throw a
    // Todo error because spreading a mutable iterator into a hook that freezes
    // its arguments is not yet supported.
    for (i, arg) in args.iter().enumerate() {
        let place = match arg {
            CallArg::Place(p) => p,
            CallArg::Spread(s) => &s.place,
        };
        let sig_effect = if i < sig.positional_params.len() {
            sig.positional_params[i]
        } else if let Some(rest) = sig.rest_param {
            rest
        } else {
            // No more declared params — conservative
            Effect::ConditionallyMutate
        };
        // Port of getArgumentEffect: adjust effect for spread arguments
        let effect = if matches!(arg, CallArg::Spread(_)) {
            match sig_effect {
                Effect::Mutate | Effect::ConditionallyMutate => sig_effect,
                Effect::Freeze => {
                    let mut errors = CompilerError::new();
                    errors.push_error_detail(crate::compiler_error::CompilerErrorDetail::new(
                        crate::compiler_error::CompilerErrorDetailOptions {
                            category: crate::compiler_error::ErrorCategory::Todo,
                            reason: "Support spread syntax for hook arguments".to_string(),
                            description: Some(
                                "Support spread syntax for hook arguments".to_string(),
                            ),
                            loc: Some(place.loc.into()),
                            suggestions: None,
                        },
                    ));
                    return Err(errors);
                }
                _ => Effect::Read,
            }
        } else {
            sig_effect
        };
        visit(place, effect);
    }

    // Process captures: if stores exist, capture into stores; otherwise alias to return.
    //
    // Port of TS applyEffect lines 896-903, 928-944:
    // When the source has an abstract kind of Global or Primitive, the TS reference
    // drops Alias/Capture effects entirely (sourceType=null → no-op). We convert to
    // ImmutableCapture which is a no-op in InferMutationAliasingRanges. Similarly,
    // Frozen/MaybeFrozen sources become ImmutableCapture.
    if !captures.is_empty() {
        if stores.is_empty() {
            for cap in &captures {
                let source_kind = state.get(cap).map(|av| av.kind);
                if matches!(
                    source_kind,
                    Some(
                        ValueKind::Global
                            | ValueKind::Primitive
                            | ValueKind::Frozen
                            | ValueKind::MaybeFrozen
                    )
                ) {
                    effects.push(AliasingEffect::ImmutableCapture {
                        from: cap.clone(),
                        into: lvalue.clone(),
                    });
                } else {
                    effects.push(AliasingEffect::Alias { from: cap.clone(), into: lvalue.clone() });
                }
            }
        } else {
            for cap in &captures {
                let source_kind = state.get(cap).map(|av| av.kind);
                if matches!(
                    source_kind,
                    Some(
                        ValueKind::Global
                            | ValueKind::Primitive
                            | ValueKind::Frozen
                            | ValueKind::MaybeFrozen
                    )
                ) {
                    for store in &stores {
                        effects.push(AliasingEffect::ImmutableCapture {
                            from: cap.clone(),
                            into: store.clone(),
                        });
                    }
                } else {
                    for store in &stores {
                        effects.push(AliasingEffect::Capture {
                            from: cap.clone(),
                            into: store.clone(),
                        });
                    }
                }
            }
        }
    }

    Ok(effects)
}

/// Emit conservative (no-signature) call effects for Apply.
///
/// Port of the `else` branch in TS `applyEffect` for `Apply` (lines 1103-1183):
///   1. Create(into=lvalue, Mutable) -- FIRST
///   2. For each operand in [receiver, function, ...args]:
///      a. MutateTransitiveConditionally(operand) -- unless operand===function && !mutatesFunction
///      b. conditionallyMutateIterator(operand) -- if arg is Spread
///      c. MaybeAlias(from=operand, into=lvalue)
///      d. For each other operand: Capture(from=operand, into=other) -- skip self by ref identity
fn emit_conservative_call_effects(
    effects: &mut Vec<AliasingEffect>,
    state: &InferenceState,
    lvalue: &Place,
    receiver: &Place,
    function: &Place,
    mutates_function: bool,
    args: &[crate::hir::CallArg],
) {
    use crate::hir::CallArg;
    // Step 1: Create lvalue FIRST so subsequent edges can reference it.
    effects.push(AliasingEffect::Create {
        into: lvalue.clone(),
        value: ValueKind::Mutable,
        reason: ValueReason::Other,
    });

    // Step 2: Build operand list = [receiver, function, ...args].
    // Use raw pointers to track TS reference identity (receiver===function when they
    // point to the same Place, e.g. callee for CallExpression/NewExpression).
    let mut operands: Vec<(*const Place, &Place, bool)> = Vec::new();
    operands.push((std::ptr::from_ref::<Place>(receiver), receiver, false));
    operands.push((std::ptr::from_ref::<Place>(function), function, true));
    for arg in args {
        let place = match arg {
            CallArg::Place(p) => p,
            CallArg::Spread(s) => &s.place,
        };
        operands.push((std::ptr::from_ref::<Place>(place), place, false));
    }

    let function_ptr = function as *const Place;
    for (idx, &(self_ptr, operand, _is_fn_entry)) in operands.iter().enumerate() {
        // TS applyEffect filters MutateTransitiveConditionally through state.mutate(),
        // which returns 'none' for non-Mutable/Context values (Primitive, Frozen, Global).
        // We replicate that check here to avoid spurious mutable range extensions.
        let is_conditionally_mutable = match state.get(operand) {
            Some(av) => matches!(av.kind, ValueKind::Mutable | ValueKind::Context),
            None => true, // Unknown: conservatively assume mutable
        };
        // TS: if (operand !== effect.function || effect.mutatesFunction)
        // In TS, `operand !== effect.function` compares by reference identity.
        let is_same_as_function = std::ptr::eq(self_ptr, function_ptr);
        if (!is_same_as_function || mutates_function) && is_conditionally_mutable {
            effects.push(AliasingEffect::MutateTransitiveConditionally { value: operand.clone() });
        }
        // TS: conditionallyMutateIterator for Spread args (idx >= 2 = actual call args)
        if idx >= 2
            && is_conditionally_mutable
            && let CallArg::Spread(_) = &args[idx - 2]
            && !is_array_or_set_or_map_type(&operand.identifier.type_)
        {
            effects.push(AliasingEffect::MutateTransitiveConditionally { value: operand.clone() });
        }
        // TS: MaybeAlias(from=operand, into=lvalue)
        effects.push(AliasingEffect::MaybeAlias { from: operand.clone(), into: lvalue.clone() });
        // TS: cross-argument Capture(from=operand, into=other) for all other operands.
        // In TS, each Capture goes through applyEffect which filters based on abstract
        // value kinds (lines 810-878). We replicate that filtering here:
        // - frozen source -> ImmutableCapture
        // - global/primitive source -> drop
        // - context source + any known dest -> MaybeAlias
        // - mutable source + mutable/maybeFrozen dest -> Capture (keep)
        // - mutable source + context dest -> MaybeAlias
        // - mutable source + other dest -> drop
        for (other_idx, &(other_ptr, other, _)) in operands.iter().enumerate() {
            if other_idx == idx {
                continue;
            }
            // TS: `other === arg` skip by reference identity
            if std::ptr::eq(self_ptr, other_ptr) {
                continue;
            }
            // Apply Capture filtering matching TS applyEffect behavior
            let from_kind = state.get(operand).map(|av| av.kind);
            let into_kind = state.get(other).map(|av| av.kind);
            match from_kind {
                Some(ValueKind::Global | ValueKind::Primitive) => {
                    // Drop: global/primitive sources don't need data flow tracking
                }
                Some(ValueKind::Frozen | ValueKind::MaybeFrozen) => {
                    effects.push(AliasingEffect::ImmutableCapture {
                        from: operand.clone(),
                        into: other.clone(),
                    });
                }
                Some(ValueKind::Context) => {
                    if into_kind.is_some() {
                        effects.push(AliasingEffect::MaybeAlias {
                            from: operand.clone(),
                            into: other.clone(),
                        });
                    }
                }
                Some(ValueKind::Mutable) => match into_kind {
                    Some(ValueKind::Mutable | ValueKind::MaybeFrozen) => {
                        effects.push(AliasingEffect::Capture {
                            from: operand.clone(),
                            into: other.clone(),
                        });
                    }
                    Some(ValueKind::Context) => {
                        effects.push(AliasingEffect::MaybeAlias {
                            from: operand.clone(),
                            into: other.clone(),
                        });
                    }
                    _ => {
                        // destination is frozen/global/primitive/unknown -> drop
                    }
                },
                None => {
                    // Unknown source: conservatively keep as Capture
                    effects.push(AliasingEffect::Capture {
                        from: operand.clone(),
                        into: other.clone(),
                    });
                }
            }
        }
    }
}

/// Check if a type is a built-in Array, Set, or Map type.
///
/// Port of TS `isArrayType`, `isSetType`, `isMapType` from HIR.ts.
/// Used in `GetIterator` handling: built-in collections return a fresh iterator
/// (no aliasing), while other types may return `this` as the iterator.
fn is_array_or_set_or_map_type(ty: &crate::hir::types::Type) -> bool {
    use crate::hir::{
        object_shape::{BUILT_IN_ARRAY_ID, BUILT_IN_MAP_ID, BUILT_IN_SET_ID},
        types::Type,
    };
    matches!(ty, Type::Object(obj) if matches!(obj.shape_id.as_deref(), Some(BUILT_IN_ARRAY_ID | BUILT_IN_MAP_ID | BUILT_IN_SET_ID)))
}

/// Check if a place's abstract value is Frozen or MaybeFrozen in the inference state.
/// Used to decide whether to emit ImmutableCapture instead of Capture/Assign/CreateFrom
/// (matching TS applyEffect behavior).
fn is_frozen(state: &InferenceState, place: &crate::hir::Place) -> bool {
    match state.get(place) {
        Some(av) => matches!(av.kind, ValueKind::Frozen | ValueKind::MaybeFrozen),
        None => false,
    }
}

/// Check if all arguments are immutable and non-mutating.
///
/// Port of `areArgumentsImmutableAndNonMutating` from `InferMutationAliasingEffects.ts`.
///
/// Returns true if all args are either:
/// 1. Function types with known signatures that have no mutable param effects, OR
/// 2. Abstract values that are Primitive, Frozen, MaybeFrozen, or Global (i.e. non-mutable)
///
/// Used to implement the `mutable_only_if_operands_are_mutable` check for built-in methods
/// like `Array.filter`, `Array.map`, etc. When all args are immutable, those methods don't
/// need to extend the receiver's mutable range.
fn are_arguments_immutable_and_non_mutating(
    state: &InferenceState,
    env: &crate::hir::environment::Environment,
    args: &[crate::hir::CallArg],
) -> bool {
    use crate::hir::{CallArg, types::Type};

    for arg in args {
        let place = match arg {
            CallArg::Place(p) => p,
            CallArg::Spread(s) => &s.place,
        };

        // If the argument is a Function type with a known signature, check whether any
        // of the signature's params are mutable effects (matching TS isKnownMutableEffect).
        if let Type::Function(_) = &place.identifier.type_
            && let Some(fn_sig) = env.get_function_signature(&place.identifier.type_)
        {
            // isKnownMutableEffect: Store, ConditionallyMutate, ConditionallyMutateIterator, Mutate → true
            // Read, Capture, Freeze → false
            let has_mutable_positional = fn_sig.positional_params.iter().any(|&e| {
                matches!(
                    e,
                    Effect::Store
                        | Effect::ConditionallyMutate
                        | Effect::ConditionallyMutateIterator
                        | Effect::Mutate
                )
            });
            if has_mutable_positional {
                return false;
            }
            if let Some(rest) = fn_sig.rest_param
                && matches!(
                    rest,
                    Effect::Store
                        | Effect::ConditionallyMutate
                        | Effect::ConditionallyMutateIterator
                        | Effect::Mutate
                )
            {
                return false;
            }
            // This function's signature doesn't mutate its inputs — continue checking others
            continue;
        }

        // For non-function types or functions without known signatures, check the abstract value kind.
        // Only Primitive and Frozen are allowed. Globals, module locals, and other locally
        // defined functions may mutate their arguments.
        match state.get(place) {
            Some(av) => {
                if !matches!(av.kind, ValueKind::Primitive | ValueKind::Frozen) {
                    return false;
                }
            }
            None => {
                // Unknown: conservatively not immutable
                return false;
            }
        }

        // For Frozen/MaybeFrozen function expressions: additionally check that the function's
        // params don't have mutable ranges (which would indicate the function mutates its inputs).
        //
        // Port of TS `areArgumentsImmutableAndNonMutating` lines:
        //   for (const value of values) {
        //     if (value.kind === 'FunctionExpression' && value.loweredFunc.func.params.some(
        //       param => { const range = param.place.identifier.mutableRange; return range.end > range.start + 1; }
        //     )) { return false; }
        //   }
        if let Some(func_expr) = state.function_values.get(&place.identifier.id) {
            let params_have_mutable_range =
                func_expr.lowered_func.func.params.iter().any(|param| {
                    let p = match param {
                        crate::hir::ReactiveParam::Place(p) => p,
                        crate::hir::ReactiveParam::Spread(s) => &s.place,
                    };
                    p.identifier.mutable_range.end.0 > p.identifier.mutable_range.start.0 + 1
                });
            if params_have_mutable_range {
                return false;
            }
        }
    }
    true
}

/// Determine whether a function expression should be considered "mutable" or "frozen".
///
/// Port of the `isMutable` computation in TS CreateFunction applyEffect
/// (InferMutationAliasingEffects.ts lines 804-827):
///
/// A function is mutable if:
/// 1. It has any context variable with `Effect::Capture` whose abstract value in the
///    outer inference state is `Mutable` or `Context` (hasCaptures).
/// 2. OR its inner aliasing effects contain `MutateFrozen`, `MutateGlobal`, or `Impure`
///    (hasTrackedSideEffects).
///
/// If a function is NOT mutable (all captures are frozen/global/primitive), it is
/// considered "frozen" — calling it will not mutate the captured environment.
fn is_function_expression_mutable(state: &InferenceState, inner_func: &HIRFunction) -> bool {
    // hasCaptures: any context var with Effect::Capture that is Mutable or Context in outer state
    let has_captures = inner_func.context.iter().any(|ctx| {
        if ctx.effect != Effect::Capture {
            return false;
        }
        match state.get(ctx) {
            Some(av) => matches!(av.kind, ValueKind::Mutable | ValueKind::Context),
            None => true, // Unknown: conservatively assume mutable
        }
    });

    if has_captures {
        return true;
    }

    // hasTrackedSideEffects: inner aliasing effects contain MutateFrozen, MutateGlobal, or Impure
    if let Some(aliasing_effects) = &inner_func.aliasing_effects {
        let has_side_effects = aliasing_effects.iter().any(|effect| {
            matches!(
                effect,
                AliasingEffect::MutateFrozen { .. }
                    | AliasingEffect::MutateGlobal { .. }
                    | AliasingEffect::Impure { .. }
            )
        });
        if has_side_effects {
            return true;
        }
    }

    // capturesRef: for legacy compatibility (matches TS InferMutationAliasingEffects.ts line 824).
    // If the function captures a ref or ref-value type, treat it as mutable.
    // This ensures that function expressions capturing refs get a longer mutable
    // range, which causes InferReactiveScopeVariables to group them with dependent
    // identifiers (call results, objects, arrays) into a single reactive scope.
    let captures_ref = inner_func.context.iter().any(|ctx| is_ref_or_ref_value(&ctx.identifier));
    if captures_ref {
        return true;
    }

    false
}

/// Build an `AliasingSignature` from a locally-defined function expression.
///
/// Port of `buildSignatureFromFunctionExpression` from `InferMutationAliasingEffects.ts`
/// (lines 2758-2779).
///
/// This extracts the parameter identifiers, the return identifier, and the
/// aliasing effects from a function expression that was previously analysed by
/// `analyse_functions`.
fn build_signature_from_function_expression(
    func_expr: &FunctionExpressionValue,
) -> AliasingSignature {
    let inner_func = &func_expr.lowered_func.func;
    let mut params = Vec::new();
    let mut rest = None;
    for param in &inner_func.params {
        match param {
            ReactiveParam::Place(p) => {
                params.push(p.identifier.id);
            }
            ReactiveParam::Spread(s) => {
                rest = Some(s.place.identifier.id);
            }
        }
    }
    // TS uses `makeIdentifierId(0)` for receiver — a dummy ID that won't match
    // any real identifier. We use IdentifierId(0) for the same purpose.
    // If there's no rest param, the TS creates a temporary place. Since we only
    // use this to key substitutions (and unused params are harmless), we use a
    // sentinel value.
    AliasingSignature {
        receiver: IdentifierId(0),
        params,
        rest,
        returns: inner_func.returns.identifier.id,
        effects: inner_func.aliasing_effects.clone().unwrap_or_default(),
        temporaries: Vec::new(),
    }
}

/// Substitute an `AliasingSignature`'s placeholder identifiers with actual
/// call-site values and produce concrete aliasing effects.
///
/// Port of `computeEffectsForSignature` from `InferMutationAliasingEffects.ts`
/// (lines 2563-2756).
///
/// The substitution table maps each formal parameter, receiver, return value,
/// and context variable to the corresponding call-site place. The signature's
/// effects are then rewritten using these substitutions.
fn compute_effects_for_signature(
    signature: &AliasingSignature,
    lvalue: &Place,
    receiver: &Place,
    args: &[crate::hir::CallArg],
    context: &[Place],
) -> Option<Vec<AliasingEffect>> {
    use crate::hir::CallArg;

    // Arity check: not enough args, or too many without a rest param
    if signature.params.len() > args.len() {
        return None;
    }
    if args.len() > signature.params.len() && signature.rest.is_none() {
        return None;
    }

    // Build substitution table: IdentifierId -> Vec<Place>
    let mut substitutions: FxHashMap<IdentifierId, Vec<Place>> = FxHashMap::default();
    substitutions.insert(signature.receiver, vec![receiver.clone()]);
    substitutions.insert(signature.returns, vec![lvalue.clone()]);

    for (i, arg) in args.iter().enumerate() {
        let (place, is_spread) = match arg {
            CallArg::Place(p) => (p, false),
            CallArg::Spread(s) => (&s.place, true),
        };
        if i >= signature.params.len() || is_spread {
            // Goes into rest param
            let Some(rest_id) = signature.rest else {
                return None;
            };
            substitutions.entry(rest_id).or_default().push(place.clone());
        } else {
            substitutions.insert(signature.params[i], vec![place.clone()]);
        }
    }

    // Context variables from function expressions reference identifiers in the
    // outer scope — populate substitutions so they map to themselves.
    for operand in context {
        substitutions.insert(operand.identifier.id, vec![operand.clone()]);
    }

    // Handle temporaries: built-in aliasing signatures can have temporaries
    // (e.g. useEffect has an @effect temporary). The signature's effects include
    // Create effects that establish values at temporary places, and other effects
    // reference them. We map each temporary's placeholder ID to itself so that
    // substitution lookups succeed.
    // Note: TS creates actual temporary places with new IDs via createTemporaryPlace,
    // but since the signature temporaries already have unique placeholder IDs,
    // mapping to themselves achieves the same substitution behavior.
    for temp in &signature.temporaries {
        substitutions.insert(temp.identifier.id, vec![temp.clone()]);
    }

    let mut effects = Vec::new();

    for effect in &signature.effects {
        match effect {
            // Binary effects (from -> into)
            AliasingEffect::MaybeAlias { from, into }
            | AliasingEffect::Assign { from, into }
            | AliasingEffect::ImmutableCapture { from, into }
            | AliasingEffect::Alias { from, into }
            | AliasingEffect::CreateFrom { from, into }
            | AliasingEffect::Capture { from, into } => {
                let from_places =
                    substitutions.get(&from.identifier.id).cloned().unwrap_or_default();
                let into_places =
                    substitutions.get(&into.identifier.id).cloned().unwrap_or_default();
                for from_place in &from_places {
                    for into_place in &into_places {
                        // Reconstruct the same effect variant with substituted places
                        let substituted = match effect {
                            AliasingEffect::MaybeAlias { .. } => AliasingEffect::MaybeAlias {
                                from: from_place.clone(),
                                into: into_place.clone(),
                            },
                            AliasingEffect::Assign { .. } => AliasingEffect::Assign {
                                from: from_place.clone(),
                                into: into_place.clone(),
                            },
                            AliasingEffect::ImmutableCapture { .. } => {
                                AliasingEffect::ImmutableCapture {
                                    from: from_place.clone(),
                                    into: into_place.clone(),
                                }
                            }
                            AliasingEffect::Alias { .. } => AliasingEffect::Alias {
                                from: from_place.clone(),
                                into: into_place.clone(),
                            },
                            AliasingEffect::CreateFrom { .. } => AliasingEffect::CreateFrom {
                                from: from_place.clone(),
                                into: into_place.clone(),
                            },
                            AliasingEffect::Capture { .. } => AliasingEffect::Capture {
                                from: from_place.clone(),
                                into: into_place.clone(),
                            },
                            _ => unreachable!(),
                        };
                        effects.push(substituted);
                    }
                }
            }

            // Error/diagnostic effects (place-based)
            AliasingEffect::Impure { place, error } => {
                let places = substitutions.get(&place.identifier.id).cloned().unwrap_or_default();
                for p in places {
                    effects.push(AliasingEffect::Impure { place: p, error: error.clone() });
                }
            }
            AliasingEffect::MutateFrozen { place, error } => {
                let places = substitutions.get(&place.identifier.id).cloned().unwrap_or_default();
                for p in places {
                    effects.push(AliasingEffect::MutateFrozen { place: p, error: error.clone() });
                }
            }
            AliasingEffect::MutateGlobal { place, error } => {
                let places = substitutions.get(&place.identifier.id).cloned().unwrap_or_default();
                for p in places {
                    effects.push(AliasingEffect::MutateGlobal { place: p, error: error.clone() });
                }
            }
            AliasingEffect::Render { place } => {
                let places = substitutions.get(&place.identifier.id).cloned().unwrap_or_default();
                for p in places {
                    effects.push(AliasingEffect::Render { place: p });
                }
            }

            // Mutation effects (value-based)
            AliasingEffect::Mutate { value, reason } => {
                let places = substitutions.get(&value.identifier.id).cloned().unwrap_or_default();
                for p in places {
                    effects.push(AliasingEffect::Mutate { value: p, reason: reason.clone() });
                }
            }
            AliasingEffect::MutateTransitive { value } => {
                let places = substitutions.get(&value.identifier.id).cloned().unwrap_or_default();
                for p in places {
                    effects.push(AliasingEffect::MutateTransitive { value: p });
                }
            }
            AliasingEffect::MutateTransitiveConditionally { value } => {
                let places = substitutions.get(&value.identifier.id).cloned().unwrap_or_default();
                for p in places {
                    effects.push(AliasingEffect::MutateTransitiveConditionally { value: p });
                }
            }
            AliasingEffect::MutateConditionally { value } => {
                let places = substitutions.get(&value.identifier.id).cloned().unwrap_or_default();
                for p in places {
                    effects.push(AliasingEffect::MutateConditionally { value: p });
                }
            }

            // Freeze
            AliasingEffect::Freeze { value, reason } => {
                let places = substitutions.get(&value.identifier.id).cloned().unwrap_or_default();
                for p in places {
                    effects.push(AliasingEffect::Freeze { value: p, reason: *reason });
                }
            }

            // Create
            AliasingEffect::Create { into, value, reason } => {
                let places = substitutions.get(&into.identifier.id).cloned().unwrap_or_default();
                for p in places {
                    effects.push(AliasingEffect::Create {
                        into: p,
                        value: *value,
                        reason: *reason,
                    });
                }
            }

            // Apply: recursively substitute
            AliasingEffect::Apply {
                receiver: apply_recv,
                function,
                mutates_function,
                args: apply_args,
                into: apply_into,
                signature: apply_sig,
                loc,
            } => {
                let recv_sub = substitutions.get(&apply_recv.identifier.id);
                if recv_sub.is_none_or(|v| v.len() != 1) {
                    return None;
                }
                let fn_sub = substitutions.get(&function.identifier.id);
                if fn_sub.is_none_or(|v| v.len() != 1) {
                    return None;
                }
                let into_sub = substitutions.get(&apply_into.identifier.id);
                if into_sub.is_none_or(|v| v.len() != 1) {
                    return None;
                }
                let mut new_args = Vec::new();
                for arg in apply_args {
                    match arg {
                        crate::inference::aliasing_effects::ApplyArg::Hole => {
                            new_args.push(crate::inference::aliasing_effects::ApplyArg::Hole);
                        }
                        crate::inference::aliasing_effects::ApplyArg::Place(p) => {
                            let arg_sub = substitutions.get(&p.identifier.id);
                            if arg_sub.is_none_or(|v| v.len() != 1) {
                                return None;
                            }
                            new_args.push(crate::inference::aliasing_effects::ApplyArg::Place(
                                arg_sub.unwrap()[0].clone(),
                            ));
                        }
                        crate::inference::aliasing_effects::ApplyArg::Spread(s) => {
                            let arg_sub = substitutions.get(&s.place.identifier.id);
                            if arg_sub.is_none_or(|v| v.len() != 1) {
                                return None;
                            }
                            new_args.push(crate::inference::aliasing_effects::ApplyArg::Spread(
                                crate::hir::SpreadPattern { place: arg_sub.unwrap()[0].clone() },
                            ));
                        }
                    }
                }
                effects.push(AliasingEffect::Apply {
                    receiver: recv_sub.unwrap()[0].clone(),
                    function: fn_sub.unwrap()[0].clone(),
                    mutates_function: *mutates_function,
                    args: new_args,
                    into: Box::new(into_sub.unwrap()[0].clone()),
                    signature: apply_sig.clone(),
                    loc: *loc,
                });
            }

            // CreateFunction: not expected in signatures from function expressions
            AliasingEffect::CreateFunction { .. } => {
                // TS throws a todo error here; we skip it for now
            }
        }
    }

    Some(effects)
}

/// Port of `getWriteErrorReason` from `InferMutationAliasingEffects.ts` (lines 2655-2679).
///
/// Returns a human-readable error message describing why a mutation is invalid,
/// based on the abstract value's reason set.
fn get_write_error_reason(abstract_value: &AbstractValue) -> &'static str {
    if abstract_value.reason.contains(&ValueReason::Global) {
        "Modifying a variable defined outside a component or hook is not allowed. Consider using an effect"
    } else if abstract_value.reason.contains(&ValueReason::JsxCaptured) {
        "Modifying a value used previously in JSX is not allowed. Consider moving the modification before the JSX"
    } else if abstract_value.reason.contains(&ValueReason::Context) {
        "Modifying a value returned from 'useContext()' is not allowed."
    } else if abstract_value.reason.contains(&ValueReason::KnownReturnSignature) {
        "Modifying a value returned from a function whose return value should not be mutated"
    } else if abstract_value.reason.contains(&ValueReason::ReactiveFunctionArgument) {
        "Modifying component props or hook arguments is not allowed. Consider using a local variable instead"
    } else if abstract_value.reason.contains(&ValueReason::State) {
        "Modifying a value returned from 'useState()', which should not be modified directly. Use the setter function to update instead"
    } else if abstract_value.reason.contains(&ValueReason::ReducerState) {
        "Modifying a value returned from 'useReducer()', which should not be modified directly. Use the dispatch function to update instead"
    } else if abstract_value.reason.contains(&ValueReason::Effect) {
        "Modifying a value used previously in an effect function or as an effect dependency is not allowed. Consider moving the modification before calling useEffect()"
    } else if abstract_value.reason.contains(&ValueReason::HookCaptured) {
        "Modifying a value previously passed as an argument to a hook is not allowed. Consider moving the modification before calling the hook"
    } else if abstract_value.reason.contains(&ValueReason::HookReturn) {
        "Modifying a value returned from a hook is not allowed. Consider moving the modification into the hook where the value is constructed"
    } else {
        "This modifies a variable that React considers immutable"
    }
}

fn make_mutation_error_effect(
    value: &Place,
    abstract_value: &AbstractValue,
    reason: &Option<MutationReason>,
) -> AliasingEffect {
    let error_reason = get_write_error_reason(abstract_value);
    let variable = match &value.identifier.name {
        Some(crate::hir::IdentifierName::Named(name)) => format!("`{name}`"),
        _ => "value".to_string(),
    };

    let mut diagnostic = CompilerDiagnostic::create(
        ErrorCategory::Immutability,
        "This value cannot be modified".to_string(),
        Some(error_reason.to_string()),
        None,
    )
    .with_detail(CompilerDiagnosticDetail::Error {
        loc: Some(value.loc),
        message: Some(format!("{variable} cannot be modified")),
    });

    if matches!(reason, Some(MutationReason::AssignCurrentProperty)) {
        diagnostic = diagnostic.with_detail(CompilerDiagnosticDetail::Hint {
            message: "Hint: If this value is a Ref (value returned by `useRef()`), rename the variable to end in \"Ref\".".to_string(),
        });
    }

    if abstract_value.kind == ValueKind::Frozen || abstract_value.kind == ValueKind::MaybeFrozen {
        AliasingEffect::MutateFrozen { place: value.clone(), error: diagnostic }
    } else {
        AliasingEffect::MutateGlobal { place: value.clone(), error: diagnostic }
    }
}

/// Port of TS `applySignature` lines 538-583 from `InferMutationAliasingEffects.ts`.
///
/// For FunctionExpression/ObjectMethod instructions, eagerly validate that the inner
/// function isn't mutating a known-frozen context variable. The inner function's
/// `aliasingEffects` (computed by `analyse_functions`) contain `Mutate`/`MutateTransitive`
/// effects for context variables that are mutated inside the callback. We check the
/// OUTER function's abstract state to see if those context variables are Frozen, and if
/// so, emit `MutateFrozen` error effects.
fn emit_inner_function_frozen_mutation_errors(
    state: &InferenceState,
    inner_func: &crate::hir::HIRFunction,
    effects: &mut Vec<AliasingEffect>,
) {
    let aliasing_effects = match &inner_func.aliasing_effects {
        Some(e) => e,
        None => return,
    };
    let context_ids: rustc_hash::FxHashSet<IdentifierId> =
        inner_func.context.iter().map(|p| p.identifier.id).collect();

    for effect in aliasing_effects {
        let (value, reason) = match effect {
            AliasingEffect::Mutate { value, reason } => (value, reason.clone()),
            AliasingEffect::MutateTransitive { value } => (value, None),
            _ => continue,
        };
        if !context_ids.contains(&value.identifier.id) {
            continue;
        }
        // Check the OUTER state for this context variable
        if let Some(av) = state.get(value) {
            if matches!(av.kind, ValueKind::Frozen | ValueKind::MaybeFrozen) {
                // Create MutateFrozen error matching TS behavior
                let error_reason = get_write_error_reason(av);
                let variable = match &value.identifier.name {
                    Some(crate::hir::IdentifierName::Named(name)) => format!("`{name}`"),
                    _ => "value".to_string(),
                };
                let mut diagnostic = CompilerDiagnostic::create(
                    ErrorCategory::Immutability,
                    "This value cannot be modified".to_string(),
                    Some(error_reason.to_string()),
                    None,
                )
                .with_detail(CompilerDiagnosticDetail::Error {
                    loc: Some(value.loc),
                    message: Some(format!("{variable} cannot be modified")),
                });
                if matches!(reason, Some(MutationReason::AssignCurrentProperty)) {
                    diagnostic = diagnostic.with_detail(CompilerDiagnosticDetail::Hint {
                        message: "Hint: If this value is a Ref (value returned by `useRef()`), rename the variable to end in \"Ref\".".to_string(),
                    });
                }
                effects
                    .push(AliasingEffect::MutateFrozen { place: value.clone(), error: diagnostic });
            }
        }
    }
}

/// Filter mutation effects through the abstract state to detect invalid mutations
/// of frozen/global values.
///
/// This is a lightweight filter applied to ALL instruction effects. It only
/// converts unconditional mutations (`Mutate`/`MutateTransitive`) of frozen or
/// global values to `MutateFrozen`/`MutateGlobal` error effects. All other
/// effects pass through unchanged.
///
/// Unlike `filter_substituted_effects`, this does NOT modify Capture/Alias/Assign
/// effects, because at the point where it's called, the instruction's lvalue has
/// not yet been created in the abstract state. A full Capture filter would
/// incorrectly drop effects whose destination doesn't exist yet.
fn filter_mutation_effects(
    state: &InferenceState,
    raw_effects: Vec<AliasingEffect>,
) -> Vec<AliasingEffect> {
    let mut filtered = Vec::with_capacity(raw_effects.len());

    for effect in raw_effects {
        match &effect {
            AliasingEffect::Mutate { value, reason } => {
                if is_ref_or_ref_value(&value.identifier) {
                    // Ref mutations are handled separately; pass through
                    filtered.push(effect);
                    continue;
                }
                match state.get(value) {
                    Some(av) => match av.kind {
                        ValueKind::Frozen | ValueKind::MaybeFrozen | ValueKind::Global => {
                            filtered.push(make_mutation_error_effect(value, av, reason));
                        }
                        // Mutable, Context, Primitive: keep as-is.
                        // Note: unlike filter_substituted_effects, we do NOT drop
                        // mutations of Primitive values here. That drop is only
                        // correct for signature-substituted effects. For regular
                        // instruction effects, the Mutate must be kept so
                        // InferMutationAliasingRanges can extend mutable ranges.
                        _ => {
                            filtered.push(effect);
                        }
                    },
                    None => {
                        // Unknown: conservatively keep
                        filtered.push(effect);
                    }
                }
            }
            AliasingEffect::MutateTransitive { value } => {
                if is_ref_or_ref_value(&value.identifier) {
                    filtered.push(effect);
                    continue;
                }
                match state.get(value) {
                    Some(av) => match av.kind {
                        ValueKind::Frozen | ValueKind::MaybeFrozen | ValueKind::Global => {
                            filtered.push(make_mutation_error_effect(value, av, &None));
                        }
                        _ => {
                            filtered.push(effect);
                        }
                    },
                    None => {
                        // Unknown: conservatively keep
                        filtered.push(effect);
                    }
                }
            }
            // All other effects pass through unchanged
            _ => {
                filtered.push(effect);
            }
        }
    }

    filtered
}

/// Filter substituted effects from `compute_effects_for_signature` through the
/// abstract state, matching the TS `applyEffect` filtering behavior.
///
/// In the TS compiler, each substituted effect is recursively passed through
/// `applyEffect`, which checks the abstract value kind of places referenced by
/// the effect and may drop, downgrade, or transform effects:
///
/// - `Mutate`/`MutateTransitive`: converted to `MutateFrozen`/`MutateGlobal` error effects
///   if the value is frozen or global; dropped if primitive; kept if mutable/context.
/// - `MutateConditionally`/`MutateTransitiveConditionally`: dropped if the value
///   is not Mutable or Context (e.g. frozen values are not conditionally mutated).
/// - `Alias`/`Capture`: downgraded to `ImmutableCapture` if the source is frozen,
///   or dropped entirely if the source is global/primitive.
/// - `Assign`: downgraded to `ImmutableCapture` if the source is frozen.
/// - `ImmutableCapture`: dropped if the source is global/primitive.
/// - `Freeze`: only kept if the value is Mutable, Context, or MaybeFrozen.
/// - Other effects (`Create`, `Render`, etc.) pass through unchanged.
///
/// NOTE: For built-in hook signatures, use `filter_freeze_effects` instead,
/// which only filters `Freeze` effects without altering data flow effects.
fn filter_substituted_effects(
    state: &InferenceState,
    raw_effects: Vec<AliasingEffect>,
) -> Vec<AliasingEffect> {
    let mut filtered = Vec::with_capacity(raw_effects.len());

    for effect in raw_effects {
        match &effect {
            // Mutate / MutateTransitive:
            // TS applyEffect (lines 1104-1200) + state.mutate() (lines 1372-1426):
            // - ref/ref-value → drop (mutate-ref)
            // - mutable/context → keep (mutate)
            // - primitive → drop (none)
            // - frozen/maybeFrozen → MutateFrozen error
            // - global → MutateGlobal error
            AliasingEffect::Mutate { value, reason } => {
                if is_ref_or_ref_value(&value.identifier) {
                    // mutate-ref: no-op
                    continue;
                }
                match state.get(value) {
                    Some(av) => match av.kind {
                        ValueKind::Mutable | ValueKind::Context => {
                            filtered.push(effect);
                        }
                        ValueKind::Primitive => {
                            // technically an error, but not React-specific: drop
                        }
                        ValueKind::Frozen | ValueKind::MaybeFrozen | ValueKind::Global => {
                            filtered.push(make_mutation_error_effect(value, av, reason));
                        }
                    },
                    None => {
                        // Unknown: conservatively keep
                        filtered.push(effect);
                    }
                }
            }
            AliasingEffect::MutateTransitive { value } => {
                if is_ref_or_ref_value(&value.identifier) {
                    // mutate-ref: no-op
                    continue;
                }
                match state.get(value) {
                    Some(av) => match av.kind {
                        ValueKind::Mutable | ValueKind::Context => {
                            filtered.push(effect);
                        }
                        ValueKind::Primitive => {
                            // technically an error, but not React-specific: drop
                        }
                        ValueKind::Frozen | ValueKind::MaybeFrozen | ValueKind::Global => {
                            filtered.push(make_mutation_error_effect(value, av, &None));
                        }
                    },
                    None => {
                        // Unknown: conservatively keep
                        filtered.push(effect);
                    }
                }
            }

            // MutateConditionally / MutateTransitiveConditionally:
            // TS applyEffect (lines 1490-1501): only kept if value is Mutable or Context.
            AliasingEffect::MutateConditionally { value }
            | AliasingEffect::MutateTransitiveConditionally { value } => {
                let keep = match state.get(value) {
                    Some(av) => matches!(av.kind, ValueKind::Mutable | ValueKind::Context),
                    None => true, // Unknown: conservatively keep
                };
                if keep {
                    filtered.push(effect);
                }
            }

            // Alias / Capture:
            // TS applyEffect (lines 862-944): check source and destination kinds.
            // - source frozen -> ImmutableCapture
            // - source global/primitive -> drop
            // - source mutable & dest mutable -> keep
            // - source context or cross context/mutable -> MaybeAlias
            AliasingEffect::Alias { from, into } | AliasingEffect::Capture { from, into } => {
                let from_kind = state.get(from).map(|av| av.kind);
                match from_kind {
                    Some(ValueKind::Global | ValueKind::Primitive) => {
                        // Drop: global/primitive sources don't need data flow tracking
                    }
                    Some(ValueKind::Frozen | ValueKind::MaybeFrozen) => {
                        // Downgrade to ImmutableCapture
                        filtered.push(AliasingEffect::ImmutableCapture {
                            from: from.clone(),
                            into: into.clone(),
                        });
                    }
                    Some(ValueKind::Context) => {
                        let into_kind = state.get(into).map(|av| av.kind);
                        if into_kind.is_some() {
                            // Context source with any known destination -> MaybeAlias
                            filtered.push(AliasingEffect::MaybeAlias {
                                from: from.clone(),
                                into: into.clone(),
                            });
                        }
                    }
                    Some(ValueKind::Mutable) => {
                        let into_kind = state.get(into).map(|av| av.kind);
                        match into_kind {
                            Some(ValueKind::Mutable | ValueKind::MaybeFrozen) => {
                                filtered.push(effect);
                            }
                            Some(ValueKind::Context) => {
                                filtered.push(AliasingEffect::MaybeAlias {
                                    from: from.clone(),
                                    into: into.clone(),
                                });
                            }
                            _ => {
                                // destination is frozen/global/primitive/unknown -> drop
                            }
                        }
                    }
                    None => {
                        // Unknown source: conservatively keep
                        filtered.push(effect);
                    }
                }
            }

            // Assign:
            // TS applyEffect (lines 947-1015): frozen source -> ImmutableCapture.
            AliasingEffect::Assign { from, into } => {
                let from_kind = state.get(from).map(|av| av.kind);
                match from_kind {
                    Some(ValueKind::Frozen | ValueKind::MaybeFrozen) => {
                        filtered.push(AliasingEffect::ImmutableCapture {
                            from: from.clone(),
                            into: into.clone(),
                        });
                    }
                    _ => {
                        // Global, Primitive, Mutable, Context, Unknown -> keep as-is
                        filtered.push(effect);
                    }
                }
            }

            // ImmutableCapture:
            // TS applyEffect (lines 717-729): drop if source is global/primitive.
            AliasingEffect::ImmutableCapture { from, .. } => {
                let from_kind = state.get(from).map(|av| av.kind);
                match from_kind {
                    Some(ValueKind::Global | ValueKind::Primitive) => {
                        // Drop: no data flow tracking needed for copy types
                    }
                    _ => {
                        filtered.push(effect);
                    }
                }
            }

            // All other effects pass through unchanged.
            _ => {
                filtered.push(effect);
            }
        }
    }

    filtered
}

/// Filter only `Freeze` effects from built-in hook aliasing signatures.
///
/// Compute the aliasing effects for an instruction.
///
/// Port of `computeSignatureForInstruction` from `InferMutationAliasingEffects.ts`.
fn compute_instruction_effects(
    state: &InferenceState,
    instr: &Instruction,
    env: &crate::hir::environment::Environment,
    non_mutating_spreads: &FxHashSet<IdentifierId>,
) -> Result<Vec<AliasingEffect>, CompilerError> {
    use crate::hir::CallArg;
    use crate::inference::aliasing_effects::CreateFunctionKind;

    let lvalue = &instr.lvalue;
    let mut effects = Vec::new();

    match &instr.value {
        // ArrayExpression: Create(Mutable) + Capture each element into the array
        // If an element is frozen, use ImmutableCapture instead of Capture (matching TS applyEffect).
        InstructionValue::ArrayExpression(arr) => {
            effects.push(AliasingEffect::Create {
                into: lvalue.clone(),
                value: ValueKind::Mutable,
                reason: ValueReason::Other,
            });
            for element in &arr.elements {
                match element {
                    crate::hir::ArrayExpressionElement::Place(p) => {
                        if is_frozen(state, p) {
                            effects.push(AliasingEffect::ImmutableCapture {
                                from: p.clone(),
                                into: lvalue.clone(),
                            });
                        } else {
                            effects.push(AliasingEffect::Capture {
                                from: p.clone(),
                                into: lvalue.clone(),
                            });
                        }
                    }
                    crate::hir::ArrayExpressionElement::Spread(s) => {
                        if !is_array_or_set_or_map_type(&s.place.identifier.type_) {
                            effects.push(AliasingEffect::MutateTransitiveConditionally {
                                value: s.place.clone(),
                            });
                        }
                        if is_frozen(state, &s.place) {
                            effects.push(AliasingEffect::ImmutableCapture {
                                from: s.place.clone(),
                                into: lvalue.clone(),
                            });
                        } else {
                            effects.push(AliasingEffect::Capture {
                                from: s.place.clone(),
                                into: lvalue.clone(),
                            });
                        }
                    }
                    crate::hir::ArrayExpressionElement::Hole => {}
                }
            }
        }

        // ObjectExpression: Create(Mutable) + Capture each property value into the object
        // If a property is frozen, use ImmutableCapture instead of Capture (matching TS applyEffect).
        InstructionValue::ObjectExpression(obj) => {
            effects.push(AliasingEffect::Create {
                into: lvalue.clone(),
                value: ValueKind::Mutable,
                reason: ValueReason::Other,
            });
            for prop in &obj.properties {
                match prop {
                    crate::hir::ObjectPatternProperty::Property(p) => {
                        if is_frozen(state, &p.place) {
                            effects.push(AliasingEffect::ImmutableCapture {
                                from: p.place.clone(),
                                into: lvalue.clone(),
                            });
                        } else {
                            effects.push(AliasingEffect::Capture {
                                from: p.place.clone(),
                                into: lvalue.clone(),
                            });
                        }
                    }
                    crate::hir::ObjectPatternProperty::Spread(s) => {
                        if is_frozen(state, &s.place) {
                            effects.push(AliasingEffect::ImmutableCapture {
                                from: s.place.clone(),
                                into: lvalue.clone(),
                            });
                        } else {
                            effects.push(AliasingEffect::Capture {
                                from: s.place.clone(),
                                into: lvalue.clone(),
                            });
                        }
                    }
                }
            }
        }

        // CallExpression / MethodCall / NewExpression
        //
        // Try to resolve a function signature for the callee. If found, generate
        // precise effects based on the declared parameter effects. Otherwise, fall
        // back to the conservative behavior.
        InstructionValue::CallExpression(v) => {
            // 1. Check if callee is a locally-defined FunctionExpression with known effects.
            //    Port of the Apply effect handler from TS applyEffect (lines 1016-1067):
            //    When calling a locally declared function whose aliasing effects are known,
            //    we build a signature from the function expression, substitute params/context
            //    with actual call-site values, and emit the substituted effects instead of
            //    conservative fallback.
            if let Some(func_expr) = state.function_values.get(&v.callee.identifier.id)
                && func_expr.lowered_func.func.aliasing_effects.is_some()
            {
                let signature = build_signature_from_function_expression(func_expr);
                let context_vars = &func_expr.lowered_func.func.context;
                if let Some(sig_effects) = compute_effects_for_signature(
                    &signature,
                    lvalue,
                    &v.callee, // receiver = callee for CallExpression
                    &v.args,
                    context_vars,
                ) {
                    // TS: MutateTransitiveConditionally(function) + substituted effects.
                    // In TS, this goes through applyEffect which filters it out if the
                    // function's abstract kind is not Mutable or Context (e.g. Frozen).
                    // We replicate that filtering here: only emit if callee is mutable.
                    let callee_is_mutable = match state.get(&v.callee) {
                        Some(av) => {
                            matches!(av.kind, ValueKind::Mutable | ValueKind::Context)
                        }
                        None => true, // Unknown: conservatively assume mutable
                    };
                    if callee_is_mutable {
                        effects.push(AliasingEffect::MutateTransitiveConditionally {
                            value: v.callee.clone(),
                        });
                    }
                    // Filter substituted effects through abstract state to match
                    // TS applyEffect behavior: drop mutations on frozen values,
                    // downgrade Alias to ImmutableCapture when source is frozen, etc.
                    let filtered = filter_substituted_effects(state, sig_effects);
                    effects.extend(filtered);
                    return Ok(effects);
                }
            }
            // 2. Try to get function signature from callee's type
            let sig = env.get_function_signature(&v.callee.identifier.type_);
            if let Some(sig) = sig {
                let sig = sig.clone();
                // 2a. Check for new-style aliasing signature first
                if let Some(ref aliasing) = sig.aliasing
                    && let Some(sig_effects) = compute_effects_for_signature(
                        aliasing,
                        lvalue,
                        &v.callee,
                        &v.args,
                        &[], // empty context for built-in signatures
                    )
                {
                    effects.extend(sig_effects);
                    return Ok(effects);
                }
                // 2b. Legacy fallback
                return effects_from_signature(&sig, &v.callee, &v.args, lvalue, state);
            }
            // 3. Conservative fallback: no signature found.
            // TS: receiver=callee, function=callee, mutatesFunction=true
            emit_conservative_call_effects(
                &mut effects,
                state,
                lvalue,
                &v.callee, // receiver = callee
                &v.callee, // function = callee
                true,      // mutatesFunction = true for CallExpression
                &v.args,
            );
        }
        InstructionValue::MethodCall(v) => {
            // 1. Check if the method (property) is a locally-defined FunctionExpression
            if let Some(func_expr) = state.function_values.get(&v.property.identifier.id)
                && func_expr.lowered_func.func.aliasing_effects.is_some()
            {
                let signature = build_signature_from_function_expression(func_expr);
                let context_vars = &func_expr.lowered_func.func.context;
                if let Some(sig_effects) = compute_effects_for_signature(
                    &signature,
                    lvalue,
                    &v.receiver, // receiver for MethodCall
                    &v.args,
                    context_vars,
                ) {
                    // TS: for MethodCall, mutatesCallee=false so no MutateTransitiveConditionally on function
                    // Filter substituted effects through abstract state to match
                    // TS applyEffect behavior.
                    let filtered = filter_substituted_effects(state, sig_effects);
                    effects.extend(filtered);
                    return Ok(effects);
                }
            }
            // 2. Try to get function signature from the property's type (the method)
            let sig = env.get_function_signature(&v.property.identifier.type_);
            if let Some(sig) = sig {
                let sig = sig.clone();
                // 2a. Check for new-style aliasing signature first
                if let Some(ref aliasing) = sig.aliasing
                    && let Some(sig_effects) = compute_effects_for_signature(
                        aliasing,
                        lvalue,
                        &v.receiver, // receiver for MethodCall
                        &v.args,
                        &[], // empty context for built-in signatures
                    )
                {
                    effects.extend(sig_effects);
                    return Ok(effects);
                }
                // 2b. Port of TS mutableOnlyIfOperandsAreMutable check (InferMutationAliasingEffects.ts).
                // If the method is only mutable when operands are mutable (e.g. Array.filter, Array.map),
                // and all arguments are immutable/non-mutating, use Alias + ImmutableCapture instead
                // of the normal signature effects. This prevents extending the receiver's mutable range
                // unnecessarily when calling methods like `arr.filter(Boolean)`.
                if sig.mutable_only_if_operands_are_mutable
                    && are_arguments_immutable_and_non_mutating(state, env, &v.args)
                {
                    let return_value_reason = sig.return_value_reason.unwrap_or(ValueReason::Other);
                    effects.push(AliasingEffect::Create {
                        into: lvalue.clone(),
                        value: sig.return_value_kind,
                        reason: return_value_reason,
                    });
                    effects.push(AliasingEffect::Alias {
                        from: v.receiver.clone(),
                        into: lvalue.clone(),
                    });
                    for arg in &v.args {
                        let place = match arg {
                            CallArg::Place(p) => p,
                            CallArg::Spread(s) => &s.place,
                        };
                        effects.push(AliasingEffect::ImmutableCapture {
                            from: place.clone(),
                            into: lvalue.clone(),
                        });
                    }
                    return Ok(effects);
                }
                // 2c. Legacy fallback for method calls
                return effects_from_signature(&sig, &v.receiver, &v.args, lvalue, state);
            }
            // 3. Conservative fallback: no signature found.
            // TS: receiver=receiver, function=property, mutatesFunction=false
            emit_conservative_call_effects(
                &mut effects,
                state,
                lvalue,
                &v.receiver, // receiver
                &v.property, // function = property
                false,       // mutatesFunction = false for MethodCall
                &v.args,
            );
        }
        InstructionValue::NewExpression(v) => {
            // Port of TS: NewExpression uses callee as the receiver, and mutatesFunction=false.
            // Try to find a function signature for the callee (e.g. `new Set(...)` uses the
            // Set constructor signature with ConditionallyMutateIterator for its argument).
            let sig = env.get_function_signature(&v.callee.identifier.type_);
            if let Some(sig) = sig {
                let sig = sig.clone();
                // Check for new-style aliasing signature first
                if let Some(ref aliasing) = sig.aliasing
                    && let Some(sig_effects) = compute_effects_for_signature(
                        aliasing,
                        lvalue,
                        &v.callee,
                        &v.args,
                        &[], // empty context for built-in signatures
                    )
                {
                    effects.extend(sig_effects);
                    return Ok(effects);
                }
                // Legacy fallback
                return effects_from_signature(&sig, &v.callee, &v.args, lvalue, state);
            }
            // Conservative fallback when no signature is found.
            // TS: receiver=callee, function=callee, mutatesFunction=false
            emit_conservative_call_effects(
                &mut effects,
                state,
                lvalue,
                &v.callee, // receiver = callee
                &v.callee, // function = callee
                false,     // mutatesFunction = false for NewExpression
                &v.args,
            );
        }

        // PropertyLoad / ComputedLoad: if the lvalue type is Primitive, emit Create(Primitive)
        // instead of CreateFrom, to avoid creating an alias edge that would incorrectly
        // extend the primitive value's mutable range when the object is later mutated.
        // Port of TS `isPrimitiveType(lvalue.identifier)` check in
        // `InferMutationAliasingEffects.ts` lines 1856-1871.
        InstructionValue::PropertyLoad(v) => {
            if lvalue.identifier.is_primitive_type() {
                effects.push(AliasingEffect::Create {
                    into: lvalue.clone(),
                    value: ValueKind::Primitive,
                    reason: ValueReason::Other,
                });
            } else {
                effects.push(AliasingEffect::CreateFrom {
                    from: v.object.clone(),
                    into: lvalue.clone(),
                });
            }
        }
        InstructionValue::ComputedLoad(v) => {
            if lvalue.identifier.is_primitive_type() {
                effects.push(AliasingEffect::Create {
                    into: lvalue.clone(),
                    value: ValueKind::Primitive,
                    reason: ValueReason::Other,
                });
            } else {
                effects.push(AliasingEffect::CreateFrom {
                    from: v.object.clone(),
                    into: lvalue.clone(),
                });
            }
        }

        // PropertyStore / ComputedStore: Mutate(object) + Capture(value->object) + Create(Primitive, lvalue)
        InstructionValue::PropertyStore(v) => {
            let mutation_reason = if matches!(v.property, crate::hir::types::PropertyLiteral::String(ref s) if s == "current")
            {
                Some(crate::inference::aliasing_effects::MutationReason::AssignCurrentProperty)
            } else {
                None
            };
            effects
                .push(AliasingEffect::Mutate { value: v.object.clone(), reason: mutation_reason });
            effects.push(AliasingEffect::Capture { from: v.value.clone(), into: v.object.clone() });
            effects.push(AliasingEffect::Create {
                into: lvalue.clone(),
                value: ValueKind::Primitive,
                reason: ValueReason::Other,
            });
        }
        InstructionValue::ComputedStore(v) => {
            effects.push(AliasingEffect::Mutate { value: v.object.clone(), reason: None });
            effects.push(AliasingEffect::Capture { from: v.value.clone(), into: v.object.clone() });
            effects.push(AliasingEffect::Create {
                into: lvalue.clone(),
                value: ValueKind::Primitive,
                reason: ValueReason::Other,
            });
        }

        // PropertyDelete / ComputedDelete: Create(Primitive) + Mutate(object)
        InstructionValue::PropertyDelete(v) => {
            effects.push(AliasingEffect::Create {
                into: lvalue.clone(),
                value: ValueKind::Primitive,
                reason: ValueReason::Other,
            });
            effects.push(AliasingEffect::Mutate { value: v.object.clone(), reason: None });
        }
        InstructionValue::ComputedDelete(v) => {
            effects.push(AliasingEffect::Create {
                into: lvalue.clone(),
                value: ValueKind::Primitive,
                reason: ValueReason::Other,
            });
            effects.push(AliasingEffect::Mutate { value: v.object.clone(), reason: None });
        }

        // FunctionExpression: CreateFunction with captured context variables
        // The TS version's applyEffect for CreateFunction recursively emits
        // Capture effects for each context variable with Effect::Capture.
        // This is critical for InferMutationAliasingRanges to create capture
        // edges in the alias graph, enabling mutation propagation through
        // captured variables.
        InstructionValue::FunctionExpression(v) => {
            // Port of TS applySignature lines 538-583: eagerly validate that
            // the inner function isn't mutating a known-frozen context variable.
            // The inner function's aliasingEffects contain Mutate/MutateTransitive
            // for context variables that are mutated inside the callback. We check
            // the OUTER state to see if those context variables are Frozen.
            emit_inner_function_frozen_mutation_errors(state, &v.lowered_func.func, &mut effects);

            // Match TS applyEffect for CreateFunction: if a context operand has kind
            // Primitive, Frozen, or Global in the outer state, demote it from Capture to Read
            // (it doesn't need to be mutable-captured). This prevents false-positive
            // Immutability errors when a context variable is a known global/frozen value.
            let captures: Vec<Place> = v
                .lowered_func
                .func
                .context
                .iter()
                .filter(|operand| {
                    if operand.effect != Effect::Capture {
                        return false;
                    }
                    // Check outer-function abstract kind: if Global/Frozen/MaybeFrozen/Primitive,
                    // treat as Read (not a mutable capture).
                    match state.get(operand) {
                        Some(av) => !matches!(
                            av.kind,
                            ValueKind::Primitive
                                | ValueKind::Frozen
                                | ValueKind::MaybeFrozen
                                | ValueKind::Global
                        ),
                        None => true, // Unknown: assume mutable capture
                    }
                })
                .cloned()
                .collect();
            effects.push(AliasingEffect::CreateFunction {
                captures: captures.clone(),
                function: CreateFunctionKind::FunctionExpression(v.clone()),
                into: lvalue.clone(),
            });
            for capture in &captures {
                effects
                    .push(AliasingEffect::Capture { from: capture.clone(), into: lvalue.clone() });
            }
        }
        InstructionValue::ObjectMethod(v) => {
            // Same as FunctionExpression: check inner function's aliasing effects
            // for mutations of frozen context variables.
            emit_inner_function_frozen_mutation_errors(state, &v.lowered_func.func, &mut effects);

            let captures: Vec<Place> = v
                .lowered_func
                .func
                .context
                .iter()
                .filter(|operand| {
                    if operand.effect != Effect::Capture {
                        return false;
                    }
                    match state.get(operand) {
                        Some(av) => !matches!(
                            av.kind,
                            ValueKind::Primitive
                                | ValueKind::Frozen
                                | ValueKind::MaybeFrozen
                                | ValueKind::Global
                        ),
                        None => true,
                    }
                })
                .cloned()
                .collect();
            effects.push(AliasingEffect::CreateFunction {
                captures: captures.clone(),
                function: CreateFunctionKind::ObjectMethod(v.clone()),
                into: lvalue.clone(),
            });
            for capture in &captures {
                effects
                    .push(AliasingEffect::Capture { from: capture.clone(), into: lvalue.clone() });
            }
        }

        // LoadLocal: always Assign (direct value flow).
        // The TS reference unconditionally emits Assign for LoadLocal (line 2203-2205 of
        // InferMutationAliasingEffects.ts). The frozen check is NOT applied here — frozen
        // semantics are handled later during the Assign/CreateFrom resolution in
        // InferMutationAliasingRanges. Emitting ImmutableCapture here would prevent alias
        // edges from being created in the ranges pass, breaking mutation propagation through
        // aliases (e.g. `obj` -> `aliasedObj` via `identity(obj)`).
        InstructionValue::LoadLocal(v) => {
            effects.push(AliasingEffect::Assign { from: v.place.clone(), into: lvalue.clone() });
        }

        // LoadContext: CreateFrom (loading from mutable box), but if source is frozen,
        // emit Create(Frozen) + ImmutableCapture instead (matching TS applyEffect behavior).
        //
        // The TS reference emits `CreateFrom` for LoadContext (line 2128-2135), but then
        // TS's `applyEffect(CreateFrom)` converts it when the source is frozen (line 765-783):
        //   - If fromKind == Frozen: emit Create(Frozen, into) + ImmutableCapture(from, into)
        //   - This prevents the mutation BFS from creating a `createdFrom` back-edge that
        //     would propagate mutations from the LoadContext result back to the frozen source.
        //
        // Example: for a component prop `items` (frozen), `$39 = LoadContext items`:
        // - Without this check: CreateFrom(items -> $39) creates a createdFrom back-edge,
        //   so mutations to $39 (e.g. from GetIterator) propagate back to items, extending
        //   items.mutableRange. This causes items to get a reactive scope ID.
        // - With this check: ImmutableCapture(items -> $39) does nothing in the ranges pass,
        //   so items.mutableRange stays narrow and items remains scopeless.
        InstructionValue::LoadContext(v) => {
            let source_frozen = is_frozen(state, &v.place);
            if source_frozen {
                let source_reason = state
                    .get(&v.place)
                    .and_then(|av| av.reason.iter().next().copied())
                    .unwrap_or(ValueReason::Other);
                // Matching TS applyEffect(CreateFrom) with frozen source:
                // emit Create(Frozen, lvalue) + ImmutableCapture(source, lvalue)
                effects.push(AliasingEffect::Create {
                    into: lvalue.clone(),
                    value: ValueKind::Frozen,
                    reason: source_reason,
                });
                effects.push(AliasingEffect::ImmutableCapture {
                    from: v.place.clone(),
                    into: lvalue.clone(),
                });
            } else {
                effects.push(AliasingEffect::CreateFrom {
                    from: v.place.clone(),
                    into: lvalue.clone(),
                });
            }
        }

        // StoreLocal: Assign to lvalue target + Assign to instruction lvalue
        InstructionValue::StoreLocal(v) => {
            effects.push(AliasingEffect::Assign {
                from: v.value.clone(),
                into: v.lvalue.place.clone(),
            });
            effects.push(AliasingEffect::Assign { from: v.value.clone(), into: lvalue.clone() });
        }

        // StoreContext: Mutate/Create for context box + Capture + Assign
        InstructionValue::StoreContext(v) => {
            if v.lvalue_kind == InstructionKind::Reassign {
                effects
                    .push(AliasingEffect::Mutate { value: v.lvalue_place.clone(), reason: None });
            } else {
                effects.push(AliasingEffect::Create {
                    into: v.lvalue_place.clone(),
                    value: ValueKind::Mutable,
                    reason: ValueReason::Other,
                });
            }
            effects.push(AliasingEffect::Capture {
                from: v.value.clone(),
                into: v.lvalue_place.clone(),
            });
            effects.push(AliasingEffect::Assign { from: v.value.clone(), into: lvalue.clone() });
        }

        // DeclareLocal: Create(Primitive) for both lvalue place and instruction lvalue
        InstructionValue::DeclareLocal(v) => {
            effects.push(AliasingEffect::Create {
                into: v.lvalue.place.clone(),
                value: ValueKind::Primitive,
                reason: ValueReason::Other,
            });
            effects.push(AliasingEffect::Create {
                into: lvalue.clone(),
                value: ValueKind::Primitive,
                reason: ValueReason::Other,
            });
        }

        // DeclareContext: Create(Mutable) or Mutate + Create(Primitive) for lvalue
        InstructionValue::DeclareContext(v) => {
            effects.push(AliasingEffect::Create {
                into: v.lvalue_place.clone(),
                value: ValueKind::Mutable,
                reason: ValueReason::Other,
            });
            effects.push(AliasingEffect::Create {
                into: lvalue.clone(),
                value: ValueKind::Primitive,
                reason: ValueReason::Other,
            });
        }

        // Destructure: per pattern item, differentiate between primitive, identifier, and spread.
        // Port of `computeSignatureForInstruction` case 'Destructure' from
        // `InferMutationAliasingEffects.ts` lines 2091-2127.
        InstructionValue::Destructure(v) => {
            for pattern_item in each_pattern_item(&v.lvalue.pattern) {
                let place = pattern_item.place();
                if place.identifier.is_primitive_type() {
                    // Path 1: Primitive type → Create(Primitive)
                    effects.push(AliasingEffect::Create {
                        into: place.clone(),
                        value: ValueKind::Primitive,
                        reason: ValueReason::Other,
                    });
                } else if matches!(pattern_item, PatternItem::Identifier(_)) {
                    // Path 2: Non-primitive identifier → CreateFrom
                    effects.push(AliasingEffect::CreateFrom {
                        from: v.value.clone(),
                        into: place.clone(),
                    });
                } else {
                    // Path 3: Spread → Create(Frozen|Mutable) + Capture
                    let value_kind = if non_mutating_spreads.contains(&place.identifier.id) {
                        ValueKind::Frozen
                    } else {
                        ValueKind::Mutable
                    };
                    effects.push(AliasingEffect::Create {
                        into: place.clone(),
                        value: value_kind,
                        reason: ValueReason::Other,
                    });
                    effects.push(AliasingEffect::Capture {
                        from: v.value.clone(),
                        into: place.clone(),
                    });
                }
            }
            effects.push(AliasingEffect::Assign { from: v.value.clone(), into: lvalue.clone() });
        }

        // PostfixUpdate / PrefixUpdate: Create(Primitive) for both lvalue and updated place
        InstructionValue::PostfixUpdate(v) => {
            effects.push(AliasingEffect::Create {
                into: lvalue.clone(),
                value: ValueKind::Primitive,
                reason: ValueReason::Other,
            });
            effects.push(AliasingEffect::Create {
                into: v.lvalue.clone(),
                value: ValueKind::Primitive,
                reason: ValueReason::Other,
            });
        }
        InstructionValue::PrefixUpdate(v) => {
            effects.push(AliasingEffect::Create {
                into: lvalue.clone(),
                value: ValueKind::Primitive,
                reason: ValueReason::Other,
            });
            effects.push(AliasingEffect::Create {
                into: v.lvalue.clone(),
                value: ValueKind::Primitive,
                reason: ValueReason::Other,
            });
        }

        // TypeCastExpression: Assign (transparent)
        InstructionValue::TypeCastExpression(v) => {
            effects.push(AliasingEffect::Assign { from: v.value.clone(), into: lvalue.clone() });
        }

        // LoadGlobal: Create(Global)
        InstructionValue::LoadGlobal(_) => {
            effects.push(AliasingEffect::Create {
                into: lvalue.clone(),
                value: ValueKind::Global,
                reason: ValueReason::Global,
            });
        }

        // StoreGlobal: MutateGlobal error + Assign to lvalue
        // Port of TS InferMutationAliasingEffects.ts lines 2106-2122:
        // Reassigning a variable declared outside the component/hook is a side effect.
        InstructionValue::StoreGlobal(v) => {
            let variable = format!("`{}`", v.name);
            effects.push(AliasingEffect::MutateGlobal {
                place: v.value.clone(),
                error: CompilerDiagnostic::create(
                    ErrorCategory::Globals,
                    "Cannot reassign variables declared outside of the component/hook".to_string(),
                    Some(format!(
                        "Variable {variable} is declared outside of the component/hook. \
                         Reassigning this value during render is a form of side effect, \
                         which can cause unpredictable behavior depending on when the component \
                         happens to re-render. If this variable is used in rendering, use useState \
                         instead. Otherwise, consider updating it in an effect. \
                         (https://react.dev/reference/rules/components-and-hooks-must-be-pure\
                         #side-effects-must-run-outside-of-render)"
                    )),
                    None,
                )
                .with_detail(CompilerDiagnosticDetail::Error {
                    loc: Some(instr.loc),
                    message: Some(format!("{variable} cannot be reassigned")),
                }),
            });
            effects.push(AliasingEffect::Assign { from: v.value.clone(), into: lvalue.clone() });
        }

        // GetIterator: Create(Mutable) + Capture/Alias depending on collection type.
        //
        // Port of TS InferMutationAliasingEffects.ts `GetIterator` case:
        // - BuiltIn Array/Map/Set collections return a *fresh* iterator on each call,
        //   so the iterator does not alias the collection → use Capture.
        // - For all other types the object may return *itself* as the iterator
        //   (e.g. custom iterables that `return this` from `[Symbol.iterator]()`),
        //   so we must assume the result directly aliases the collection and that
        //   calling the iterator method could mutate the collection → Alias +
        //   MutateTransitiveConditionally.
        InstructionValue::GetIterator(v) => {
            effects.push(AliasingEffect::Create {
                into: lvalue.clone(),
                value: ValueKind::Mutable,
                reason: ValueReason::Other,
            });
            let is_builtin_collection = is_array_or_set_or_map_type(&v.collection.identifier.type_);
            if is_builtin_collection {
                effects.push(AliasingEffect::Capture {
                    from: v.collection.clone(),
                    into: lvalue.clone(),
                });
            } else {
                effects.push(AliasingEffect::Alias {
                    from: v.collection.clone(),
                    into: lvalue.clone(),
                });
                effects.push(AliasingEffect::MutateTransitiveConditionally {
                    value: v.collection.clone(),
                });
            }
        }

        // IteratorNext: MutateConditionally(iterator) + CreateFrom(collection -> lvalue)
        InstructionValue::IteratorNext(v) => {
            effects.push(AliasingEffect::MutateConditionally { value: v.iterator.clone() });
            effects.push(AliasingEffect::CreateFrom {
                from: v.collection.clone(),
                into: lvalue.clone(),
            });
        }

        // NextPropertyOf: Create(Primitive) -- property name string
        InstructionValue::NextPropertyOf(_) => {
            effects.push(AliasingEffect::Create {
                into: lvalue.clone(),
                value: ValueKind::Primitive,
                reason: ValueReason::Other,
            });
        }

        // AwaitExpression: Create(Mutable) + MutateTransitiveConditionally + Capture
        InstructionValue::Await(v) => {
            effects.push(AliasingEffect::Create {
                into: lvalue.clone(),
                value: ValueKind::Mutable,
                reason: ValueReason::Other,
            });
            effects.push(AliasingEffect::MutateTransitiveConditionally { value: v.value.clone() });
            effects.push(AliasingEffect::Capture { from: v.value.clone(), into: lvalue.clone() });
        }

        // JsxExpression: Create(Frozen) + Freeze + Capture per operand
        InstructionValue::JsxExpression(jsx) => {
            effects.push(AliasingEffect::Create {
                into: lvalue.clone(),
                value: ValueKind::Frozen,
                reason: ValueReason::JsxCaptured,
            });
            for operand in each_instruction_value_operand(&instr.value) {
                effects.push(AliasingEffect::Freeze {
                    value: operand.clone(),
                    reason: ValueReason::JsxCaptured,
                });
                effects
                    .push(AliasingEffect::Capture { from: operand.clone(), into: lvalue.clone() });
            }
            // Render effects for tag (component references)
            if let crate::hir::JsxTag::Place(tag_place) = &jsx.tag {
                effects.push(AliasingEffect::Render { place: tag_place.clone() });
            }
            // Render effects for children
            if let Some(ref children) = jsx.children {
                for child in children {
                    effects.push(AliasingEffect::Render { place: child.clone() });
                }
            }
            // Render effects for props that are functions returning JSX
            for prop in &jsx.props {
                if let crate::hir::JsxAttribute::Attribute { place, .. } = prop {
                    if is_render_function_type(&place.identifier.type_) {
                        effects.push(AliasingEffect::Render { place: place.clone() });
                    }
                }
            }
        }

        // JsxFragment: Create(Frozen) + Freeze + Capture per child
        InstructionValue::JsxFragment(frag) => {
            effects.push(AliasingEffect::Create {
                into: lvalue.clone(),
                value: ValueKind::Frozen,
                reason: ValueReason::JsxCaptured,
            });
            for child in &frag.children {
                effects.push(AliasingEffect::Freeze {
                    value: child.clone(),
                    reason: ValueReason::JsxCaptured,
                });
                effects.push(AliasingEffect::Capture { from: child.clone(), into: lvalue.clone() });
            }
        }

        // StartMemoize / FinishMemoize: Create(Primitive), with optional Freeze
        InstructionValue::StartMemoize(memo) => {
            if env.config.enable_preserve_existing_memoization_guarantees
                && let Some(deps) = &memo.deps
            {
                for dep in deps {
                    if let crate::hir::ManualMemoDependencyRoot::NamedLocal { value, .. } =
                        &dep.root
                    {
                        effects.push(AliasingEffect::Freeze {
                            value: value.clone(),
                            reason: ValueReason::HookCaptured,
                        });
                    }
                }
            }
            effects.push(AliasingEffect::Create {
                into: lvalue.clone(),
                value: ValueKind::Primitive,
                reason: ValueReason::Other,
            });
        }
        InstructionValue::FinishMemoize(memo) => {
            if env.config.enable_preserve_existing_memoization_guarantees {
                effects.push(AliasingEffect::Freeze {
                    value: memo.decl.clone(),
                    reason: ValueReason::HookCaptured,
                });
            }
            effects.push(AliasingEffect::Create {
                into: lvalue.clone(),
                value: ValueKind::Primitive,
                reason: ValueReason::Other,
            });
        }

        // All primitives: Create(Primitive)
        InstructionValue::Primitive(_)
        | InstructionValue::JsxText(_)
        | InstructionValue::BinaryExpression(_)
        | InstructionValue::UnaryExpression(_)
        | InstructionValue::TaggedTemplateExpression(_)
        | InstructionValue::TemplateLiteral(_)
        | InstructionValue::RegExpLiteral(_)
        | InstructionValue::Debugger(_)
        | InstructionValue::MetaProperty(_)
        | InstructionValue::UnsupportedNode(_) => {
            effects.push(AliasingEffect::Create {
                into: lvalue.clone(),
                value: ValueKind::Primitive,
                reason: ValueReason::Other,
            });
        }
    }

    // Post-process: convert/drop mutation effects based on abstract state.
    //
    // Port of the `applyEffect` / `state.mutate()` behavior from `InferMutationAliasingEffects.ts`
    // (lines 1104-1200, 1372-1426):
    // 1. ref/ref-value targets → drop (mutate-ref)
    // 2. frozen/global targets → convert Mutate/MutateTransitive to MutateFrozen/MutateGlobal error
    // 3. conditional mutations on non-mutable values → drop
    // 4. all other mutations → keep
    {
        let mut processed = Vec::with_capacity(effects.len());
        for effect in effects {
            match &effect {
                AliasingEffect::Mutate { value, reason } => {
                    if is_ref_or_ref_value(&value.identifier) {
                        continue; // mutate-ref: drop
                    }
                    match state.get(value) {
                        Some(av)
                            if matches!(
                                av.kind,
                                ValueKind::Frozen | ValueKind::MaybeFrozen | ValueKind::Global
                            ) =>
                        {
                            processed.push(make_mutation_error_effect(value, av, reason));
                        }
                        _ => {
                            processed.push(effect);
                        }
                    }
                }
                AliasingEffect::MutateTransitive { value } => {
                    if is_ref_or_ref_value(&value.identifier) {
                        continue; // mutate-ref: drop
                    }
                    match state.get(value) {
                        Some(av)
                            if matches!(
                                av.kind,
                                ValueKind::Frozen | ValueKind::MaybeFrozen | ValueKind::Global
                            ) =>
                        {
                            processed.push(make_mutation_error_effect(value, av, &None));
                        }
                        _ => {
                            processed.push(effect);
                        }
                    }
                }
                AliasingEffect::MutateTransitiveConditionally { value }
                | AliasingEffect::MutateConditionally { value } => {
                    if is_ref_or_ref_value(&value.identifier) {
                        continue; // mutate-ref: drop
                    }
                    let keep = match state.get(value) {
                        Some(av) => matches!(av.kind, ValueKind::Mutable | ValueKind::Context),
                        None => true,
                    };
                    if keep {
                        processed.push(effect);
                    }
                }
                _ => {
                    processed.push(effect);
                }
            }
        }
        effects = processed;
    }

    // Port of TS applyEffect data-flow conversions (InferMutationAliasingEffects.ts lines 861-944):
    //
    // For Capture/Alias/MaybeAlias, the TS reference classifies source and destination
    // value kinds and then decides whether to keep, convert, or drop the effect:
    //
    //  sourceType:      Frozen/MaybeFrozen → "frozen", Context → "context",
    //                   Global/Primitive → null (drop), default → "mutable"
    //  destinationType: Context → "context", Mutable/MaybeFrozen → "mutable", default → null
    //
    //  Decision:
    //    frozen source              → ImmutableCapture
    //    (mutable src & mutable dst) OR MaybeAlias → keep
    //    context+non-null dst OR mutable src+context dst → convert to MaybeAlias
    //    otherwise (incl. Global/Primitive source)       → drop (no-op)
    //
    // For Assign (lines 947-1014): Frozen source → ImmutableCapture.
    // For CreateFrom (lines 731-789): Frozen source → Create(Frozen) + ImmutableCapture.

    // Collect extra effects that arise from splitting CreateFrom into two effects.
    let mut extra_effects: Vec<AliasingEffect> = Vec::new();

    for effect in &mut effects {
        match effect {
            // Capture, Alias: prune based on source and destination value kinds
            //
            // Port of TS applyEffect lines 861-944:
            //  sourceType:      Frozen/MaybeFrozen → "frozen", Context → "context",
            //                   Global/Primitive → null (drop), default → "mutable"
            //  destinationType: Context → "context", Mutable/MaybeFrozen → "mutable",
            //                   default → null (including Frozen)
            //
            //  Decision:
            //    frozen source              → ImmutableCapture
            //    global/primitive source    → drop (Capture/Alias only, not MaybeAlias)
            //    frozen destination         → drop (Capture/Alias only, not MaybeAlias)
            //    (mutable src & mutable dst) OR MaybeAlias → keep
            //    context+non-null dst OR mutable src+context dst → convert to MaybeAlias
            //    otherwise                  → drop (no-op)
            AliasingEffect::Capture { from, into } | AliasingEffect::Alias { from, into } => {
                if let Some(abstract_val) = state.get(from) {
                    if matches!(abstract_val.kind, ValueKind::Frozen | ValueKind::MaybeFrozen) {
                        *effect = AliasingEffect::ImmutableCapture {
                            from: from.clone(),
                            into: into.clone(),
                        };
                    } else if matches!(abstract_val.kind, ValueKind::Global | ValueKind::Primitive)
                    {
                        // Port of TS applyEffect lines 896-903, 928-944:
                        // When the source is Global or Primitive, sourceType is null.
                        // For Capture/Alias, none of the keep/convert conditions match,
                        // so the effect is dropped entirely. We convert to ImmutableCapture
                        // which is a no-op in InferMutationAliasingRanges.
                        *effect = AliasingEffect::ImmutableCapture {
                            from: from.clone(),
                            into: into.clone(),
                        };
                    }
                }
                // If the destination is Frozen, the effect cannot create meaningful
                // data flow since frozen values are immutable.
                // TS drops these in applyEffect because destinationType=null for Frozen.
                if matches!(effect, AliasingEffect::Capture { .. } | AliasingEffect::Alias { .. }) {
                    let (from, into) = match effect {
                        AliasingEffect::Capture { from, into }
                        | AliasingEffect::Alias { from, into } => (from, into),
                        _ => unreachable!(),
                    };
                    if let Some(abstract_val) = state.get(into)
                        && matches!(abstract_val.kind, ValueKind::Frozen)
                    {
                        *effect = AliasingEffect::ImmutableCapture {
                            from: from.clone(),
                            into: into.clone(),
                        };
                    }
                }
            }
            AliasingEffect::MaybeAlias { from, into } => {
                // MaybeAlias: Frozen/MaybeFrozen source → ImmutableCapture
                // MaybeAlias always survives for Global/Primitive sources per TS line 930.
                if let Some(abstract_val) = state.get(from)
                    && matches!(abstract_val.kind, ValueKind::Frozen | ValueKind::MaybeFrozen)
                {
                    *effect =
                        AliasingEffect::ImmutableCapture { from: from.clone(), into: into.clone() };
                }
            }
            // Assign: Frozen source only → ImmutableCapture
            AliasingEffect::Assign { from, into } => {
                if let Some(abstract_val) = state.get(from)
                    && matches!(abstract_val.kind, ValueKind::Frozen)
                {
                    *effect =
                        AliasingEffect::ImmutableCapture { from: from.clone(), into: into.clone() };
                }
            }
            // CreateFrom: Frozen source only → Create(Frozen) + ImmutableCapture
            AliasingEffect::CreateFrom { from, into } => {
                if let Some(abstract_val) = state.get(from)
                    && matches!(abstract_val.kind, ValueKind::Frozen)
                {
                    let from_clone = from.clone();
                    let into_clone = into.clone();
                    let reason =
                        abstract_val.reason.iter().next().copied().unwrap_or(ValueReason::Other);
                    extra_effects.push(AliasingEffect::ImmutableCapture {
                        from: from_clone,
                        into: into_clone.clone(),
                    });
                    *effect = AliasingEffect::Create {
                        into: into_clone,
                        value: ValueKind::Frozen,
                        reason,
                    };
                }
            }
            _ => {}
        }
    }

    effects.extend(extra_effects);

    Ok(effects)
}

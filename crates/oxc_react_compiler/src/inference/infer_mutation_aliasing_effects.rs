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
    hir::{
        BlockId, Effect, FunctionExpressionValue, HIRFunction, IdentifierId, Instruction,
        InstructionValue, Place, ReactFunctionType, ReactiveParam, ValueKind, ValueReason,
        hir_builder::{compute_rpo_order, each_terminal_successor},
        visitors::each_terminal_operand,
    },
    inference::aliasing_effects::{AliasingEffect, AliasingSignature},
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

        for (&id, other_fn) in &other.function_values {
            if !self.function_values.contains_key(&id) {
                self.function_values.insert(id, other_fn.clone());
                changed = true;
            }
        }

        changed
    }
}

/// Merge two value kinds, returning the more conservative one.
fn merge_value_kinds(a: ValueKind, b: ValueKind) -> ValueKind {
    if a == b {
        return a;
    }
    // Ordering: Mutable > Context > MaybeFrozen > Frozen > Primitive > Global
    match (a, b) {
        (ValueKind::Mutable, _) | (_, ValueKind::Mutable) => ValueKind::Mutable,
        (ValueKind::Context, _) | (_, ValueKind::Context) => ValueKind::Context,
        (ValueKind::MaybeFrozen, _) | (_, ValueKind::MaybeFrozen) => ValueKind::MaybeFrozen,
        (ValueKind::Frozen, _) | (_, ValueKind::Frozen) => ValueKind::Frozen,
        (ValueKind::Primitive, _) | (_, ValueKind::Primitive) => ValueKind::Primitive,
        (ValueKind::Global, ValueKind::Global) => ValueKind::Global,
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

    // Fixpoint iteration over the CFG
    let mut states_by_block: FxHashMap<BlockId, InferenceState> = FxHashMap::default();
    let mut queued_states: FxHashMap<BlockId, InferenceState> = FxHashMap::default();
    queued_states.insert(func.body.entry, initial_state);

    let mut iteration_count = 0;
    let max_iterations = 1000; // Safety limit

    while !queued_states.is_empty() && iteration_count < max_iterations {
        iteration_count += 1;

        // Process queued states
        let blocks_to_process: Vec<(BlockId, InferenceState)> = queued_states.drain().collect();

        for (block_id, mut state) in blocks_to_process {
            // Check if state has changed since last visit
            if let Some(prev_state) = states_by_block.get(&block_id)
                && !state.merge(prev_state)
            {
                continue; // No changes, skip
            }

            let block = match func.body.blocks.get(&block_id) {
                Some(b) => b.clone(),
                None => continue,
            };

            // Process instructions
            for instr in &block.instructions {
                infer_instruction_effects(&mut state, instr, options, &func.env);
            }

            // Store the state for this block
            states_by_block.insert(block_id, state.clone());

            // Queue successor blocks
            let successors = each_terminal_successor(&block.terminal);
            for succ_id in successors {
                let mut succ_state = state.clone();
                // Process terminal operands
                for operand in each_terminal_operand(&block.terminal) {
                    // Terminal operands are typically reads
                    if let Some(val) = state.get(operand) {
                        succ_state.define(operand, val.clone());
                    }
                }
                if let Some(existing) = queued_states.get_mut(&succ_id) {
                    existing.merge(&succ_state);
                } else {
                    queued_states.insert(succ_id, succ_state);
                }
            }
        }
    }

    // Annotate effects on instructions.
    // For blocks reached during fixpoint iteration, compute effects from abstract state.
    // For unreachable blocks, set empty effects (not None) so downstream passes
    // (infer_mutation_aliasing_ranges) still process them correctly.
    let block_ids = compute_rpo_order(func.body.entry, &func.body.blocks);
    for block_id in block_ids {
        let Some(block) = func.body.blocks.get_mut(&block_id) else {
            continue;
        };
        if let Some(state) = states_by_block.get(&block_id) {
            for instr in &mut block.instructions {
                let effects = compute_instruction_effects(state, instr, &func.env);
                instr.effects = Some(effects);

                // Match TS applyEffect for CreateFunction: demote context operands
                // from Capture to Read when they have Global/Frozen/Primitive kind in
                // the outer abstract state. This allows validate_no_freezing_known_mutable_functions
                // to skip mutations of known-global context variables.
                match &mut instr.value {
                    InstructionValue::FunctionExpression(v) => {
                        for ctx in &mut v.lowered_func.func.context {
                            if ctx.effect == Effect::Capture {
                                if let Some(av) = state.get(ctx) {
                                    if matches!(
                                        av.kind,
                                        ValueKind::Primitive
                                            | ValueKind::Frozen
                                            | ValueKind::MaybeFrozen
                                            | ValueKind::Global
                                    ) {
                                        ctx.effect = Effect::Read;
                                    }
                                }
                            }
                        }
                    }
                    InstructionValue::ObjectMethod(v) => {
                        for ctx in &mut v.lowered_func.func.context {
                            if ctx.effect == Effect::Capture {
                                if let Some(av) = state.get(ctx) {
                                    if matches!(
                                        av.kind,
                                        ValueKind::Primitive
                                            | ValueKind::Frozen
                                            | ValueKind::MaybeFrozen
                                            | ValueKind::Global
                                    ) {
                                        ctx.effect = Effect::Read;
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
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
fn infer_instruction_effects(
    state: &mut InferenceState,
    instr: &Instruction,
    _options: &InferOptions,
    env: &crate::hir::environment::Environment,
) {
    let lvalue_id = instr.lvalue.identifier.id;

    match &instr.value {
        // Primitives, update expressions, binary/unary, and template literals produce primitives
        InstructionValue::Primitive(_)
        | InstructionValue::JsxText(_)
        | InstructionValue::PrefixUpdate(_)
        | InstructionValue::PostfixUpdate(_)
        | InstructionValue::BinaryExpression(_)
        | InstructionValue::UnaryExpression(_)
        | InstructionValue::TemplateLiteral(_) => {
            state.define(&instr.lvalue, AbstractValue::primitive());
        }

        // Object/array/function/regexp/iterator values create mutable values
        InstructionValue::ObjectExpression(_)
        | InstructionValue::ArrayExpression(_)
        | InstructionValue::RegExpLiteral(_)
        | InstructionValue::GetIterator(_)
        | InstructionValue::IteratorNext(_)
        | InstructionValue::NextPropertyOf(_) => {
            state.define(&instr.lvalue, AbstractValue::mutable());
        }

        // FunctionExpression / ObjectMethod: mutable + track for Apply resolution
        InstructionValue::FunctionExpression(v) => {
            state.define(&instr.lvalue, AbstractValue::mutable());
            state.function_values.insert(lvalue_id, v.clone());
        }
        InstructionValue::ObjectMethod(_) => {
            state.define(&instr.lvalue, AbstractValue::mutable());
        }

        // JSX creates a frozen value
        InstructionValue::JsxExpression(_) | InstructionValue::JsxFragment(_) => {
            let mut reasons = FxHashSet::default();
            reasons.insert(ValueReason::JsxCaptured);
            state.define(&instr.lvalue, AbstractValue::frozen(reasons));
        }

        // LoadLocal propagates the type of the loaded value
        InstructionValue::LoadLocal(v) => {
            if let Some(val) = state.get(&v.place).cloned() {
                state.define(&instr.lvalue, val);
                state.add_alias(v.place.identifier.id, lvalue_id);
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
                state.add_alias(v.value.identifier.id, v.lvalue.place.identifier.id);
            }
            // Propagate function value tracking through StoreLocal
            if let Some(fn_val) = state.function_values.get(&v.value.identifier.id).cloned() {
                state.function_values.insert(v.lvalue.place.identifier.id, fn_val);
            }
        }

        InstructionValue::StoreContext(v) => {
            if let Some(val) = state.get(&v.value).cloned() {
                state.define(&v.lvalue_place, val);
            }
        }

        // Calls may create, mutate, or alias values
        InstructionValue::CallExpression(v) => {
            // By default, assume the return value is mutable
            state.define(&instr.lvalue, AbstractValue::mutable());
            // Arguments may be captured/aliased by the callee
            for arg in &v.args {
                if let crate::hir::CallArg::Place(p) = arg {
                    state.add_alias(p.identifier.id, lvalue_id);
                }
            }
        }

        InstructionValue::MethodCall(v) => {
            state.define(&instr.lvalue, AbstractValue::mutable());
            state.add_alias(v.receiver.identifier.id, lvalue_id);
            for arg in &v.args {
                if let crate::hir::CallArg::Place(p) = arg {
                    state.add_alias(p.identifier.id, lvalue_id);
                }
            }
        }

        InstructionValue::NewExpression(_v) => {
            state.define(&instr.lvalue, AbstractValue::mutable());
        }

        // Property operations
        InstructionValue::PropertyLoad(v) => {
            // Loading a property may return a mutable value from a mutable object
            if let Some(val) = state.get(&v.object) {
                let result_kind = match val.kind {
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
            if let Some(val) = state.get(&v.object) {
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

        // Destructure: the destructured value may be mutable
        // When the source is frozen, propagate frozen to the pattern lvalues.
        InstructionValue::Destructure(v) => {
            if let Some(val) = state.get(&v.value).cloned() {
                state.define(&instr.lvalue, val.clone());
                // Propagate source type to pattern lvalues
                for place in crate::hir::visitors::each_pattern_operand(&v.lvalue.pattern) {
                    state.define(place, val.clone());
                }
            } else {
                state.define(&instr.lvalue, AbstractValue::mutable());
            }
        }

        // StartMemoize: freeze the deps (matching TS applyEffect for Freeze)
        // In TS, StartMemoize with enablePreserveExistingMemoizationGuarantees
        // iterates eachInstructionValueOperand (the named local deps) and emits
        // Freeze effects. applyEffect then calls state.freeze(dep) which updates
        // the abstract state. We replicate this here to ensure the abstract state
        // is up-to-date for subsequent LoadContext/LoadLocal instructions.
        InstructionValue::StartMemoize(v) => {
            if env.config.enable_preserve_existing_memoization_guarantees {
                if let Some(deps) = &v.deps {
                    for dep in deps {
                        if let crate::hir::ManualMemoDependencyRoot::NamedLocal { value, .. } =
                            &dep.root
                        {
                            let mut reasons = FxHashSet::default();
                            reasons.insert(ValueReason::HookCaptured);
                            state.define(value, AbstractValue::frozen(reasons));
                        }
                    }
                }
            }
            state.define(&instr.lvalue, AbstractValue::mutable());
        }

        // FinishMemoize: freeze the declared memoized value (matching TS applyEffect for Freeze)
        // In TS, FinishMemoize with enablePreserveExistingMemoizationGuarantees
        // emits Freeze(decl). applyEffect calls state.freeze(decl), updating the
        // abstract state so subsequent LoadContext/LoadLocal of the same identifier
        // see a Frozen abstract value and emit ImmutableCapture instead of CreateFrom.
        // This prevents item1/item2 mutable ranges from being extended through the
        // items useMemo scope (fixes type-provider-store-capture.tsx).
        InstructionValue::FinishMemoize(v) => {
            if env.config.enable_preserve_existing_memoization_guarantees {
                let mut reasons = FxHashSet::default();
                reasons.insert(ValueReason::HookCaptured);
                state.define(&v.decl, AbstractValue::frozen(reasons));
            }
            state.define(&instr.lvalue, AbstractValue::mutable());
        }

        // Other values
        InstructionValue::DeclareLocal(_)
        | InstructionValue::DeclareContext(_)
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
) -> Vec<AliasingEffect> {
    use crate::hir::{CallArg, Effect};

    let mut effects = Vec::new();
    let mut captures: Vec<Place> = Vec::new();
    let mut stores: Vec<Place> = Vec::new();

    let return_value_reason = sig.return_value_reason.unwrap_or(ValueReason::Other);

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
                // For iterables: capture into return value
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

    // Process each argument with its declared effect
    for (i, arg) in args.iter().enumerate() {
        let place = match arg {
            CallArg::Place(p) => p,
            CallArg::Spread(s) => &s.place,
        };
        let effect = if i < sig.positional_params.len() {
            sig.positional_params[i]
        } else if let Some(rest) = sig.rest_param {
            rest
        } else {
            // No more declared params — conservative
            Effect::ConditionallyMutate
        };
        visit(place, effect);
    }

    // Process captures: if stores exist, capture into stores; otherwise alias to return
    if !captures.is_empty() {
        if stores.is_empty() {
            for cap in &captures {
                effects.push(AliasingEffect::Alias { from: cap.clone(), into: lvalue.clone() });
            }
        } else {
            for cap in &captures {
                for store in &stores {
                    effects
                        .push(AliasingEffect::Capture { from: cap.clone(), into: store.clone() });
                }
            }
        }
    }

    // Create the return value
    effects.push(AliasingEffect::Create {
        into: lvalue.clone(),
        value: sig.return_value_kind,
        reason: return_value_reason,
    });

    effects
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

    // Create temporaries
    // Note: TS creates actual temporary places with new IDs. For simplicity,
    // we skip temporaries since the signatures from function expressions have
    // none (temporaries is always []).

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
                if recv_sub.map_or(true, |v| v.len() != 1) {
                    return None;
                }
                let fn_sub = substitutions.get(&function.identifier.id);
                if fn_sub.map_or(true, |v| v.len() != 1) {
                    return None;
                }
                let into_sub = substitutions.get(&apply_into.identifier.id);
                if into_sub.map_or(true, |v| v.len() != 1) {
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
                            if arg_sub.map_or(true, |v| v.len() != 1) {
                                return None;
                            }
                            new_args.push(crate::inference::aliasing_effects::ApplyArg::Place(
                                arg_sub.unwrap()[0].clone(),
                            ));
                        }
                        crate::inference::aliasing_effects::ApplyArg::Spread(s) => {
                            let arg_sub = substitutions.get(&s.place.identifier.id);
                            if arg_sub.map_or(true, |v| v.len() != 1) {
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

/// Compute the aliasing effects for an instruction.
///
/// Port of `computeSignatureForInstruction` from `InferMutationAliasingEffects.ts`.
fn compute_instruction_effects(
    state: &InferenceState,
    instr: &Instruction,
    env: &crate::hir::environment::Environment,
) -> Vec<AliasingEffect> {
    use crate::hir::{CallArg, Effect, InstructionKind, visitors::each_instruction_value_operand};
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
            if let Some(func_expr) = state.function_values.get(&v.callee.identifier.id) {
                if func_expr.lowered_func.func.aliasing_effects.is_some() {
                    let signature = build_signature_from_function_expression(func_expr);
                    let context_vars = &func_expr.lowered_func.func.context;
                    if let Some(sig_effects) = compute_effects_for_signature(
                        &signature,
                        lvalue,
                        &v.callee, // receiver = callee for CallExpression
                        &v.args,
                        context_vars,
                    ) {
                        // TS: MutateTransitiveConditionally(function) + substituted effects
                        effects.push(AliasingEffect::MutateTransitiveConditionally {
                            value: v.callee.clone(),
                        });
                        effects.extend(sig_effects);
                        return effects;
                    }
                }
            }
            // 2. Try to get function signature from callee's type (legacy signature)
            let sig = env.get_function_signature(&v.callee.identifier.type_);
            if let Some(sig) = sig {
                let sig = sig.clone();
                return effects_from_signature(&sig, &v.callee, &v.args, lvalue);
            }
            // 3. Conservative fallback: callee may also be mutated (mutatesFunction=true)
            effects.push(AliasingEffect::MutateTransitiveConditionally { value: v.callee.clone() });
            for arg in &v.args {
                let place = match arg {
                    CallArg::Place(p) => p,
                    CallArg::Spread(s) => &s.place,
                };
                effects
                    .push(AliasingEffect::MutateTransitiveConditionally { value: place.clone() });
                effects.push(AliasingEffect::Capture { from: place.clone(), into: lvalue.clone() });
            }
            effects.push(AliasingEffect::Create {
                into: lvalue.clone(),
                value: ValueKind::Mutable,
                reason: ValueReason::Other,
            });
        }
        InstructionValue::MethodCall(v) => {
            // 1. Check if the method (property) is a locally-defined FunctionExpression
            if let Some(func_expr) = state.function_values.get(&v.property.identifier.id) {
                if func_expr.lowered_func.func.aliasing_effects.is_some() {
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
                        effects.extend(sig_effects);
                        return effects;
                    }
                }
            }
            // 2. Try to get function signature from the property's type (the method)
            let sig = env.get_function_signature(&v.property.identifier.type_);
            if let Some(sig) = sig {
                let sig = sig.clone();
                // For method calls, the receiver is the callee_or_receiver
                return effects_from_signature(&sig, &v.receiver, &v.args, lvalue);
            }
            // 3. Conservative fallback: receiver is conditionally mutated
            effects
                .push(AliasingEffect::MutateTransitiveConditionally { value: v.receiver.clone() });
            for arg in &v.args {
                let place = match arg {
                    CallArg::Place(p) => p,
                    CallArg::Spread(s) => &s.place,
                };
                effects
                    .push(AliasingEffect::MutateTransitiveConditionally { value: place.clone() });
                effects.push(AliasingEffect::Capture { from: place.clone(), into: lvalue.clone() });
            }
            effects.push(AliasingEffect::Create {
                into: lvalue.clone(),
                value: ValueKind::Mutable,
                reason: ValueReason::Other,
            });
        }
        InstructionValue::NewExpression(v) => {
            // NewExpression: callee is NOT mutated (mutatesFunction=false)
            for arg in &v.args {
                let place = match arg {
                    CallArg::Place(p) => p,
                    CallArg::Spread(s) => &s.place,
                };
                effects
                    .push(AliasingEffect::MutateTransitiveConditionally { value: place.clone() });
                effects.push(AliasingEffect::Capture { from: place.clone(), into: lvalue.clone() });
            }
            effects.push(AliasingEffect::Create {
                into: lvalue.clone(),
                value: ValueKind::Mutable,
                reason: ValueReason::Other,
            });
        }

        // PropertyLoad / ComputedLoad: CreateFrom (result inherits kind from object)
        InstructionValue::PropertyLoad(v) => {
            effects
                .push(AliasingEffect::CreateFrom { from: v.object.clone(), into: lvalue.clone() });
        }
        InstructionValue::ComputedLoad(v) => {
            effects
                .push(AliasingEffect::CreateFrom { from: v.object.clone(), into: lvalue.clone() });
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

        // LoadContext: always CreateFrom (loading from mutable box).
        // The TS reference unconditionally emits CreateFrom for LoadContext (line 2128-2135).
        // Same reasoning as LoadLocal above — no frozen check here.
        InstructionValue::LoadContext(v) => {
            effects
                .push(AliasingEffect::CreateFrom { from: v.place.clone(), into: lvalue.clone() });
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

        // Destructure: CreateFrom per pattern item + Assign to lvalue
        // If the source is frozen (e.g., props param), use Create(Frozen) + ImmutableCapture
        // instead of CreateFrom (matching TS applyEffect for CreateFrom with frozen source).
        // This prevents the mutation BFS in InferMutationAliasingRanges from propagating
        // mutations through props destructuring.
        InstructionValue::Destructure(v) => {
            let source_frozen = is_frozen(state, &v.value);
            let source_reason = state
                .get(&v.value)
                .and_then(|av| av.reason.iter().next().copied())
                .unwrap_or(ValueReason::Other);
            for place in crate::hir::visitors::each_pattern_operand(&v.lvalue.pattern) {
                if source_frozen {
                    // Matching TS: CreateFrom with frozen source → Create(Frozen) + ImmutableCapture
                    effects.push(AliasingEffect::Create {
                        into: place.clone(),
                        value: ValueKind::Frozen,
                        reason: source_reason,
                    });
                    effects.push(AliasingEffect::ImmutableCapture {
                        from: v.value.clone(),
                        into: place.clone(),
                    });
                } else {
                    effects.push(AliasingEffect::CreateFrom {
                        from: v.value.clone(),
                        into: place.clone(),
                    });
                }
            }
            if source_frozen {
                // Matching TS: Assign with frozen source → ImmutableCapture
                effects.push(AliasingEffect::ImmutableCapture {
                    from: v.value.clone(),
                    into: lvalue.clone(),
                });
            } else {
                effects
                    .push(AliasingEffect::Assign { from: v.value.clone(), into: lvalue.clone() });
            }
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

        // StoreGlobal: Assign to lvalue (mutation is an error, but still track flow)
        InstructionValue::StoreGlobal(v) => {
            effects.push(AliasingEffect::Assign { from: v.value.clone(), into: lvalue.clone() });
        }

        // GetIterator: Create(Mutable) + Capture(collection -> iterator)
        InstructionValue::GetIterator(v) => {
            effects.push(AliasingEffect::Create {
                into: lvalue.clone(),
                value: ValueKind::Mutable,
                reason: ValueReason::Other,
            });
            effects
                .push(AliasingEffect::Capture { from: v.collection.clone(), into: lvalue.clone() });
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
            if env.config.enable_preserve_existing_memoization_guarantees {
                if let Some(deps) = &memo.deps {
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

    // Post-process: drop conditional mutations on frozen values.
    //
    // Port of the `applyEffect` behavior from `InferMutationAliasingEffects.ts`:
    // When a `MutateTransitiveConditionally` or `MutateConditionally` effect targets
    // a value that is frozen (or any non-mutable kind), the mutation is silently
    // dropped because conditional mutations only apply to mutable values.
    effects.retain(|effect| {
        match effect {
            AliasingEffect::MutateTransitiveConditionally { value }
            | AliasingEffect::MutateConditionally { value } => {
                if let Some(abstract_val) = state.get(value) {
                    // Only keep the mutation if the value is Mutable or Context
                    matches!(abstract_val.kind, ValueKind::Mutable | ValueKind::Context)
                } else {
                    // Unknown state — keep the effect conservatively
                    true
                }
            }
            _ => true,
        }
    });

    effects
}

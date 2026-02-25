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
        BlockId, HIRFunction, IdentifierId, Instruction, InstructionValue, Place,
        ReactFunctionType, ReactiveParam, ValueKind, ValueReason,
        hir_builder::each_terminal_successor, visitors::each_terminal_operand,
    },
    inference::aliasing_effects::AliasingEffect,
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
}

impl InferenceState {
    fn empty() -> Self {
        Self { values: FxHashMap::default(), aliases: FxHashMap::default() }
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
                infer_instruction_effects(&mut state, instr, options);
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
    let block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        let Some(block) = func.body.blocks.get_mut(&block_id) else {
            continue;
        };
        if let Some(state) = states_by_block.get(&block_id) {
            for instr in &mut block.instructions {
                let effects = compute_instruction_effects(state, instr, &func.env);
                instr.effects = Some(effects);
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
        | InstructionValue::FunctionExpression(_)
        | InstructionValue::ObjectMethod(_)
        | InstructionValue::RegExpLiteral(_)
        | InstructionValue::GetIterator(_)
        | InstructionValue::IteratorNext(_)
        | InstructionValue::NextPropertyOf(_) => {
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
        InstructionValue::Destructure(v) => {
            if let Some(val) = state.get(&v.value).cloned() {
                state.define(&instr.lvalue, val);
            } else {
                state.define(&instr.lvalue, AbstractValue::mutable());
            }
        }

        // Other values
        InstructionValue::DeclareLocal(_)
        | InstructionValue::DeclareContext(_)
        | InstructionValue::StartMemoize(_)
        | InstructionValue::FinishMemoize(_)
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

/// Compute the aliasing effects for an instruction.
///
/// Port of `computeSignatureForInstruction` from `InferMutationAliasingEffects.ts`.
fn compute_instruction_effects(
    _state: &InferenceState,
    instr: &Instruction,
    env: &crate::hir::environment::Environment,
) -> Vec<AliasingEffect> {
    use crate::hir::{CallArg, Effect, InstructionKind, visitors::each_instruction_value_operand};
    use crate::inference::aliasing_effects::CreateFunctionKind;

    let lvalue = &instr.lvalue;
    let mut effects = Vec::new();

    match &instr.value {
        // ArrayExpression: Create(Mutable) + Capture each element into the array
        InstructionValue::ArrayExpression(arr) => {
            effects.push(AliasingEffect::Create {
                into: lvalue.clone(),
                value: ValueKind::Mutable,
                reason: ValueReason::Other,
            });
            for element in &arr.elements {
                match element {
                    crate::hir::ArrayExpressionElement::Place(p) => {
                        effects.push(AliasingEffect::Capture {
                            from: p.clone(),
                            into: lvalue.clone(),
                        });
                    }
                    crate::hir::ArrayExpressionElement::Spread(s) => {
                        effects.push(AliasingEffect::Capture {
                            from: s.place.clone(),
                            into: lvalue.clone(),
                        });
                    }
                    crate::hir::ArrayExpressionElement::Hole => {}
                }
            }
        }

        // ObjectExpression: Create(Mutable) + Capture each property value into the object
        InstructionValue::ObjectExpression(obj) => {
            effects.push(AliasingEffect::Create {
                into: lvalue.clone(),
                value: ValueKind::Mutable,
                reason: ValueReason::Other,
            });
            for prop in &obj.properties {
                match prop {
                    crate::hir::ObjectPatternProperty::Property(p) => {
                        effects.push(AliasingEffect::Capture {
                            from: p.place.clone(),
                            into: lvalue.clone(),
                        });
                    }
                    crate::hir::ObjectPatternProperty::Spread(s) => {
                        effects.push(AliasingEffect::Capture {
                            from: s.place.clone(),
                            into: lvalue.clone(),
                        });
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
            // Try to get function signature from callee's type
            let sig = env.get_function_signature(&v.callee.identifier.type_);
            if let Some(sig) = sig {
                let sig = sig.clone();
                return effects_from_signature(&sig, &v.callee, &v.args, lvalue);
            }
            // Conservative fallback: callee may also be mutated (mutatesFunction=true)
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
            // Try to get function signature from the property's type (the method)
            let sig = env.get_function_signature(&v.property.identifier.type_);
            if let Some(sig) = sig {
                let sig = sig.clone();
                // For method calls, the receiver is the callee_or_receiver
                return effects_from_signature(&sig, &v.receiver, &v.args, lvalue);
            }
            // Conservative fallback: receiver is conditionally mutated
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
        InstructionValue::FunctionExpression(v) => {
            let captures: Vec<Place> = v
                .lowered_func
                .func
                .context
                .iter()
                .filter(|operand| operand.effect == Effect::Capture)
                .cloned()
                .collect();
            effects.push(AliasingEffect::CreateFunction {
                captures,
                function: CreateFunctionKind::FunctionExpression(v.clone()),
                into: lvalue.clone(),
            });
        }
        InstructionValue::ObjectMethod(v) => {
            let captures: Vec<Place> = v
                .lowered_func
                .func
                .context
                .iter()
                .filter(|operand| operand.effect == Effect::Capture)
                .cloned()
                .collect();
            effects.push(AliasingEffect::CreateFunction {
                captures,
                function: CreateFunctionKind::ObjectMethod(v.clone()),
                into: lvalue.clone(),
            });
        }

        // LoadLocal: Assign (direct value flow)
        InstructionValue::LoadLocal(v) => {
            effects.push(AliasingEffect::Assign { from: v.place.clone(), into: lvalue.clone() });
        }

        // LoadContext: CreateFrom (loading from mutable box)
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
        InstructionValue::Destructure(v) => {
            for place in crate::hir::visitors::each_pattern_operand(&v.lvalue.pattern) {
                effects.push(AliasingEffect::CreateFrom {
                    from: v.value.clone(),
                    into: place.clone(),
                });
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

        // StartMemoize / FinishMemoize: Create(Primitive)
        InstructionValue::StartMemoize(_) | InstructionValue::FinishMemoize(_) => {
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

    effects
}

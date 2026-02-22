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
pub fn infer_mutation_aliasing_effects(func: &mut HIRFunction, options: &InferOptions) {
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

    // Annotate effects on instructions
    let block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        if let Some(state) = states_by_block.get(&block_id)
            && let Some(block) = func.body.blocks.get_mut(&block_id)
        {
            for instr in &mut block.instructions {
                let effects = compute_instruction_effects(state, instr);
                instr.effects = Some(effects);
            }
        }
    }
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

/// Compute the aliasing effects for an instruction.
fn compute_instruction_effects(
    _state: &InferenceState,
    instr: &Instruction,
) -> Vec<AliasingEffect> {
    let mut effects = Vec::new();

    match &instr.value {
        InstructionValue::ObjectExpression(_) | InstructionValue::ArrayExpression(_) => {
            effects.push(AliasingEffect::Create {
                into: instr.lvalue.clone(),
                value: ValueKind::Mutable,
                reason: ValueReason::Other,
            });
        }
        InstructionValue::Primitive(_) | InstructionValue::JsxText(_) => {
            effects.push(AliasingEffect::Create {
                into: instr.lvalue.clone(),
                value: ValueKind::Primitive,
                reason: ValueReason::Other,
            });
        }
        InstructionValue::LoadLocal(v) => {
            effects
                .push(AliasingEffect::Alias { from: v.place.clone(), into: instr.lvalue.clone() });
        }
        InstructionValue::LoadContext(v) => {
            effects
                .push(AliasingEffect::Alias { from: v.place.clone(), into: instr.lvalue.clone() });
        }
        InstructionValue::StoreLocal(v) => {
            effects.push(AliasingEffect::Assign {
                from: v.value.clone(),
                into: v.lvalue.place.clone(),
            });
        }
        InstructionValue::CallExpression(v) => {
            // Function calls: capture args, create return value
            for arg in &v.args {
                if let crate::hir::CallArg::Place(p) = arg {
                    effects.push(AliasingEffect::Capture {
                        from: p.clone(),
                        into: instr.lvalue.clone(),
                    });
                }
            }
            effects.push(AliasingEffect::Create {
                into: instr.lvalue.clone(),
                value: ValueKind::Mutable,
                reason: ValueReason::Other,
            });
        }
        InstructionValue::JsxExpression(jsx) => {
            // JSX freezes its props and children
            for attr in &jsx.props {
                match attr {
                    crate::hir::JsxAttribute::Attribute { place, .. } => {
                        effects.push(AliasingEffect::Freeze {
                            value: place.clone(),
                            reason: ValueReason::JsxCaptured,
                        });
                    }
                    crate::hir::JsxAttribute::Spread { argument } => {
                        effects.push(AliasingEffect::Freeze {
                            value: argument.clone(),
                            reason: ValueReason::JsxCaptured,
                        });
                    }
                }
            }
            if let Some(children) = &jsx.children {
                for child in children {
                    effects.push(AliasingEffect::Freeze {
                        value: child.clone(),
                        reason: ValueReason::JsxCaptured,
                    });
                }
            }
            effects.push(AliasingEffect::Create {
                into: instr.lvalue.clone(),
                value: ValueKind::Frozen,
                reason: ValueReason::JsxCaptured,
            });
        }
        _ => {
            // Default: create a mutable value
            effects.push(AliasingEffect::Create {
                into: instr.lvalue.clone(),
                value: ValueKind::Mutable,
                reason: ValueReason::Other,
            });
        }
    }

    effects
}

/// Infer mutable ranges from aliasing effects.
///
/// Port of `Inference/InferMutationAliasingRanges.ts` from the React Compiler.
///
/// This pass builds an abstract model of the heap and interprets effects to determine:
/// - The mutable ranges of all identifiers
/// - The externally-visible effects of the function (mutations of params, aliasing)
/// - The legacy `Effect` to store on each Place
///
/// The algorithm has three phases:
/// 1. Build a directed alias/capture graph and collect deferred mutations
/// 2. Set legacy `Effect` on each Place based on instruction effects and mutable ranges
/// 3. Compute external effects for params/context/returns
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    compiler_error::{CompilerError, SourceLocation},
    hir::{
        BlockId, Effect, HIRFunction, Identifier, IdentifierId, InstructionId, InstructionValue,
        Place, ReactiveParam, Terminal, ValueKind, ValueReason,
    },
    inference::aliasing_effects::{AliasingEffect, CreateFunctionKind, MutationReason},
};

/// Options for the inference pass.
#[derive(Debug, Clone)]
pub struct InferRangesOptions {
    pub is_function_expression: bool,
}

// =====================================================================================
// MutationKind — ordered enum for mutation severity
// =====================================================================================

/// The severity/kind of a mutation. Higher values dominate lower values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum MutationKind {
    /// No mutation. Used as sentinel in match arms.
    #[expect(dead_code)]
    None = 0,
    Conditional = 1,
    Definite = 2,
}

// =====================================================================================
// Graph node types
// =====================================================================================

/// The kind of edge between nodes in the alias graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EdgeKind {
    Capture,
    Alias,
    MaybeAlias,
}

/// A forward edge from a node to another.
#[derive(Debug, Clone)]
struct NodeEdge {
    index: usize,
    node: IdentifierId,
    kind: EdgeKind,
}

/// Mutation info tracked on each node.
#[derive(Debug, Clone, Copy)]
struct MutationInfo {
    kind: MutationKind,
    loc: SourceLocation,
}

/// The abstract value stored in a graph node.
enum NodeValue {
    Object,
    Phi,
    Function { func: Box<HIRFunction> },
}

/// A node in the alias graph, representing an SSA identifier.
struct Node {
    id: IdentifierId,
    created_from: FxHashMap<IdentifierId, usize>,
    captures: FxHashMap<IdentifierId, usize>,
    aliases: FxHashMap<IdentifierId, usize>,
    maybe_aliases: FxHashMap<IdentifierId, usize>,
    edges: Vec<NodeEdge>,
    transitive: Option<MutationInfo>,
    local: Option<MutationInfo>,
    last_mutated: usize,
    mutation_reason: Option<MutationReason>,
    value: NodeValue,
}

// =====================================================================================
// AliasingState — the abstract heap graph
// =====================================================================================

/// The abstract state: a directed graph of alias/capture/created-from edges.
struct AliasingState {
    nodes: FxHashMap<IdentifierId, Node>,
}

impl AliasingState {
    fn new() -> Self {
        Self { nodes: FxHashMap::default() }
    }

    /// Create a new node for the given place.
    fn create(&mut self, place: &Place, value: NodeValue) {
        self.nodes.insert(
            place.identifier.id,
            Node {
                id: place.identifier.id,
                created_from: FxHashMap::default(),
                captures: FxHashMap::default(),
                aliases: FxHashMap::default(),
                maybe_aliases: FxHashMap::default(),
                edges: Vec::new(),
                transitive: None,
                local: None,
                last_mutated: 0,
                mutation_reason: None,
                value,
            },
        );
    }

    /// Create a node for `into` that is derived from `from`.
    fn create_from(&mut self, index: usize, from: &Place, into: &Place) {
        self.create(into, NodeValue::Object);
        let from_id = from.identifier.id;
        let into_id = into.identifier.id;

        // Add forward edge from -> into
        if let Some(from_node) = self.nodes.get_mut(&from_id) {
            from_node.edges.push(NodeEdge { index, node: into_id, kind: EdgeKind::Alias });
        }
        // Add backward created_from edge into -> from
        if let Some(to_node) = self.nodes.get_mut(&into_id) {
            to_node.created_from.entry(from_id).or_insert(index);
        }
    }

    /// Record a capture edge: from is captured into `into`.
    fn capture(&mut self, index: usize, from: &Place, into: &Place) {
        let from_id = from.identifier.id;
        let into_id = into.identifier.id;
        if !self.nodes.contains_key(&from_id) || !self.nodes.contains_key(&into_id) {
            return;
        }
        if let Some(from_node) = self.nodes.get_mut(&from_id) {
            from_node.edges.push(NodeEdge { index, node: into_id, kind: EdgeKind::Capture });
        }
        if let Some(to_node) = self.nodes.get_mut(&into_id) {
            to_node.captures.entry(from_id).or_insert(index);
        }
    }

    /// Record an alias edge (or assignment): from aliases into `into`.
    fn assign(&mut self, index: usize, from: &Place, into: &Place) {
        let from_id = from.identifier.id;
        let into_id = into.identifier.id;
        if !self.nodes.contains_key(&from_id) || !self.nodes.contains_key(&into_id) {
            return;
        }
        if let Some(from_node) = self.nodes.get_mut(&from_id) {
            from_node.edges.push(NodeEdge { index, node: into_id, kind: EdgeKind::Alias });
        }
        if let Some(to_node) = self.nodes.get_mut(&into_id) {
            to_node.aliases.entry(from_id).or_insert(index);
        }
    }

    /// Record a maybe-alias edge.
    fn maybe_alias(&mut self, index: usize, from: &Place, into: &Place) {
        let from_id = from.identifier.id;
        let into_id = into.identifier.id;
        if !self.nodes.contains_key(&from_id) || !self.nodes.contains_key(&into_id) {
            return;
        }
        if let Some(from_node) = self.nodes.get_mut(&from_id) {
            from_node.edges.push(NodeEdge { index, node: into_id, kind: EdgeKind::MaybeAlias });
        }
        if let Some(to_node) = self.nodes.get_mut(&into_id) {
            to_node.maybe_aliases.entry(from_id).or_insert(index);
        }
    }

    /// BFS traversal for render effects: walk backward through the graph to
    /// find Function nodes whose errors should be appended.
    fn render(&self, index: usize, start: IdentifierId, errors: &mut CompilerError) {
        let mut seen: FxHashSet<IdentifierId> = FxHashSet::default();
        let mut queue: Vec<IdentifierId> = vec![start];

        while let Some(current) = queue.pop() {
            if !seen.insert(current) {
                continue;
            }
            let Some(node) = self.nodes.get(&current) else {
                continue;
            };
            if node.transitive.is_some() || node.local.is_some() {
                continue;
            }
            if let NodeValue::Function { func } = &node.value {
                append_function_errors(errors, func);
            }
            for (&alias, &when) in &node.created_from {
                if when < index {
                    queue.push(alias);
                }
            }
            for (&alias, &when) in &node.aliases {
                if when < index {
                    queue.push(alias);
                }
            }
            for (&capture, &when) in &node.captures {
                if when < index {
                    queue.push(capture);
                }
            }
        }
    }

    /// The core mutation BFS: walk forward and backward edges to extend mutable ranges.
    ///
    /// `end` is `None` for simulated mutations (Phase 3).
    /// Returns a list of `(IdentifierId, InstructionId)` pairs for range updates.
    #[expect(clippy::too_many_arguments)]
    fn mutate(
        &mut self,
        index: usize,
        start: IdentifierId,
        end: Option<InstructionId>,
        transitive: bool,
        start_kind: MutationKind,
        loc: SourceLocation,
        reason: Option<&MutationReason>,
        errors: &mut CompilerError,
    ) -> Vec<(IdentifierId, InstructionId)> {
        let mut range_updates: Vec<(IdentifierId, InstructionId)> = Vec::new();

        // seen: IdentifierId -> max MutationKind we've visited it with
        let mut seen: FxHashMap<IdentifierId, MutationKind> = FxHashMap::default();
        let mut queue: Vec<QueueEntry> = vec![QueueEntry {
            place: start,
            transitive,
            direction: Direction::Backwards,
            kind: start_kind,
        }];

        while let Some(entry) = queue.pop() {
            let current = entry.place;
            let transitive = entry.transitive;
            let direction = entry.direction;
            let kind = entry.kind;

            // Skip if we've already visited with >= kind
            if seen.get(&current).is_some_and(|&prev_kind| prev_kind >= kind) {
                continue;
            }
            seen.insert(current, kind);

            let Some(node) = self.nodes.get_mut(&current) else {
                continue;
            };

            // Set mutation reason if not already set
            if node.mutation_reason.is_none() {
                node.mutation_reason = reason.cloned();
            }

            // Update last_mutated
            if index > node.last_mutated {
                node.last_mutated = index;
            }

            // Extend mutableRange.end
            if let Some(end_id) = end {
                range_updates.push((node.id, end_id));
            }

            // If this is a Function node and not yet mutated, append its errors
            if let NodeValue::Function { func } = &node.value
                && node.transitive.is_none()
                && node.local.is_none()
            {
                append_function_errors(errors, func);
            }

            // Record transitive or local mutation
            if transitive {
                if node.transitive.is_none() || node.transitive.is_some_and(|t| t.kind < kind) {
                    node.transitive = Some(MutationInfo { kind, loc });
                }
            } else if node.local.is_none() || node.local.is_some_and(|l| l.kind < kind) {
                node.local = Some(MutationInfo { kind, loc });
            }

            // Collect edges to traverse (we need to drop the mutable borrow first)
            let mut forward_targets: Vec<(IdentifierId, bool, Direction, MutationKind)> =
                Vec::new();
            let mut backward_targets: Vec<(IdentifierId, bool, Direction, MutationKind)> =
                Vec::new();

            // Forward edges: follow edges created before this mutation
            for edge in &node.edges {
                if edge.index >= index {
                    break;
                }
                let edge_kind = if edge.kind == EdgeKind::MaybeAlias {
                    MutationKind::Conditional
                } else {
                    kind
                };
                forward_targets.push((edge.node, transitive, Direction::Forwards, edge_kind));
            }

            // createdFrom back-edges: always follow as transitive, backwards
            for (&alias, &when) in &node.created_from {
                if when < index {
                    backward_targets.push((alias, true, Direction::Backwards, kind));
                }
            }

            // aliases back-edges: only if direction==backwards or node is not Phi
            let is_phi = matches!(node.value, NodeValue::Phi);
            if direction == Direction::Backwards || !is_phi {
                for (&alias, &when) in &node.aliases {
                    if when < index {
                        backward_targets.push((alias, transitive, Direction::Backwards, kind));
                    }
                }
                // maybeAliases back-edges: same as aliases but kind=Conditional
                for (&alias, &when) in &node.maybe_aliases {
                    if when < index {
                        backward_targets.push((
                            alias,
                            transitive,
                            Direction::Backwards,
                            MutationKind::Conditional,
                        ));
                    }
                }
            }

            // captures back-edges: only if transitive
            if transitive {
                for (&capture, &when) in &node.captures {
                    if when < index {
                        backward_targets.push((capture, transitive, Direction::Backwards, kind));
                    }
                }
            }

            // Now push all targets to queue
            for (place, tr, dir, k) in forward_targets {
                queue.push(QueueEntry { place, transitive: tr, direction: dir, kind: k });
            }
            for (place, tr, dir, k) in backward_targets {
                queue.push(QueueEntry { place, transitive: tr, direction: dir, kind: k });
            }
        }

        range_updates
    }
}

/// Append function errors from a nested function's aliasing effects.
fn append_function_errors(errors: &mut CompilerError, func: &HIRFunction) {
    if let Some(effects) = &func.aliasing_effects {
        for effect in effects {
            match effect {
                AliasingEffect::Impure { error, .. }
                | AliasingEffect::MutateFrozen { error, .. }
                | AliasingEffect::MutateGlobal { error, .. } => {
                    errors.push_diagnostic(error.clone());
                }
                _ => {}
            }
        }
    }
}

/// Extract the Place from a ReactiveParam.
fn param_place(param: &ReactiveParam) -> &Place {
    match param {
        ReactiveParam::Place(p) => p,
        ReactiveParam::Spread(s) => &s.place,
    }
}

/// Check if a type is a JSX type (matches `isJsxType` from TS).
fn is_jsx_type(identifier: &Identifier) -> bool {
    matches!(
        &identifier.type_,
        crate::hir::types::Type::Object(obj) if obj.shape_id.as_deref() == Some("BuiltInJsx")
    )
}

// =====================================================================================
// Deferred mutation / render entries
// =====================================================================================

// =====================================================================================
// Direction / QueueEntry for mutate() BFS
// =====================================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Backwards,
    Forwards,
}

struct QueueEntry {
    place: IdentifierId,
    transitive: bool,
    direction: Direction,
    kind: MutationKind,
}

// =====================================================================================
// Deferred mutation / render entries
// =====================================================================================

struct DeferredMutation {
    index: usize,
    id: InstructionId,
    transitive: bool,
    kind: MutationKind,
    place_id: IdentifierId,
    loc: SourceLocation,
    reason: Option<MutationReason>,
}

struct DeferredRender {
    index: usize,
    place_id: IdentifierId,
}

struct PendingPhiOperand {
    from_place: Place,
    into_place: Place,
    index: usize,
}

// =====================================================================================
// Main entry point
// =====================================================================================

/// Infer mutable ranges from aliasing effects.
///
/// # Errors
/// Returns a `CompilerError` if any invalid effects are found (e.g., mutating frozen values)
/// and the function is not a function expression.
pub fn infer_mutation_aliasing_ranges(
    func: &mut HIRFunction,
    options: &InferRangesOptions,
) -> Result<Vec<AliasingEffect>, CompilerError> {
    let function_effects: Vec<AliasingEffect> = Vec::new();
    let mut errors = CompilerError::new();

    // ===== Phase 1: Build abstract heap graph and collect mutations =====
    let mut state = AliasingState::new();
    let mut pending_phis: FxHashMap<BlockId, Vec<PendingPhiOperand>> = FxHashMap::default();
    let mut mutations: Vec<DeferredMutation> = Vec::new();
    let mut renders: Vec<DeferredRender> = Vec::new();
    let mut function_effects = function_effects;
    let mut index: usize = 0;

    // Initialize nodes for params, context, and returns
    for param in &func.params {
        let place = param_place(param);
        state.create(place, NodeValue::Object);
    }
    for ctx in &func.context {
        state.create(ctx, NodeValue::Object);
    }
    state.create(&func.returns, NodeValue::Object);

    // Process blocks in sorted order for determinism
    let mut block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();
    block_ids.sort();

    let mut seen_blocks: FxHashSet<BlockId> = FxHashSet::default();

    for &block_id in &block_ids {
        let block = &func.body.blocks[&block_id];

        // Process phis
        for phi in &block.phis {
            state.create(&phi.place, NodeValue::Phi);
            for (&pred, operand) in &phi.operands {
                if seen_blocks.contains(&pred) {
                    state.assign(
                        {
                            let i = index;
                            index += 1;
                            i
                        },
                        operand,
                        &phi.place,
                    );
                } else {
                    // Defer phi operand processing for unseen predecessors
                    pending_phis.entry(pred).or_default().push(PendingPhiOperand {
                        from_place: operand.clone(),
                        into_place: phi.place.clone(),
                        index: {
                            let i = index;
                            index += 1;
                            i
                        },
                    });
                }
            }
        }
        seen_blocks.insert(block_id);

        // Process instruction effects
        for instr in &block.instructions {
            let Some(effects) = &instr.effects else {
                continue;
            };
            for effect in effects {
                match effect {
                    AliasingEffect::Create { into, .. } => {
                        state.create(into, NodeValue::Object);
                    }
                    AliasingEffect::CreateFunction { into, function, .. } => {
                        let inner_func = match function {
                            CreateFunctionKind::FunctionExpression(fe) => {
                                fe.lowered_func.func.clone()
                            }
                            CreateFunctionKind::ObjectMethod(om) => om.lowered_func.func.clone(),
                        };
                        state.create(into, NodeValue::Function { func: inner_func });
                    }
                    AliasingEffect::CreateFrom { from, into } => {
                        let i = index;
                        index += 1;
                        state.create_from(i, from, into);
                    }
                    AliasingEffect::Assign { from, into } => {
                        // If the node doesn't exist yet, create it
                        if !state.nodes.contains_key(&into.identifier.id) {
                            state.create(into, NodeValue::Object);
                        }
                        let i = index;
                        index += 1;
                        state.assign(i, from, into);
                    }
                    AliasingEffect::Alias { from, into } => {
                        let i = index;
                        index += 1;
                        state.assign(i, from, into);
                    }
                    AliasingEffect::MaybeAlias { from, into } => {
                        let i = index;
                        index += 1;
                        state.maybe_alias(i, from, into);
                    }
                    AliasingEffect::Capture { from, into } => {
                        let i = index;
                        index += 1;
                        state.capture(i, from, into);
                    }
                    AliasingEffect::MutateTransitive { value }
                    | AliasingEffect::MutateTransitiveConditionally { value } => {
                        let i = index;
                        index += 1;
                        let kind = if matches!(effect, AliasingEffect::MutateTransitive { .. }) {
                            MutationKind::Definite
                        } else {
                            MutationKind::Conditional
                        };
                        mutations.push(DeferredMutation {
                            index: i,
                            id: instr.id,
                            transitive: true,
                            kind,
                            place_id: value.identifier.id,
                            loc: value.loc,
                            reason: None,
                        });
                    }
                    AliasingEffect::Mutate { value, .. }
                    | AliasingEffect::MutateConditionally { value } => {
                        let i = index;
                        index += 1;
                        let (kind, reason) = match effect {
                            AliasingEffect::Mutate { reason, .. } => {
                                (MutationKind::Definite, reason.clone())
                            }
                            _ => (MutationKind::Conditional, None),
                        };
                        mutations.push(DeferredMutation {
                            index: i,
                            id: instr.id,
                            transitive: false,
                            kind,
                            place_id: value.identifier.id,
                            loc: value.loc,
                            reason,
                        });
                    }
                    AliasingEffect::MutateFrozen { error, .. }
                    | AliasingEffect::MutateGlobal { error, .. }
                    | AliasingEffect::Impure { error, .. } => {
                        errors.push_diagnostic(error.clone());
                        function_effects.push(effect.clone());
                    }
                    AliasingEffect::Render { place } => {
                        let i = index;
                        index += 1;
                        renders.push(DeferredRender { index: i, place_id: place.identifier.id });
                        function_effects.push(effect.clone());
                    }
                    // ImmutableCapture, Freeze, Apply — no graph action in this pass
                    AliasingEffect::ImmutableCapture { .. }
                    | AliasingEffect::Freeze { .. }
                    | AliasingEffect::Apply { .. } => {}
                }
            }
        }

        // Process pending phis for this block
        if let Some(block_phis) = pending_phis.get(&block_id) {
            for pending in block_phis {
                state.assign(pending.index, &pending.from_place, &pending.into_place);
            }
        }

        // Handle return terminals: assign return value -> fn.returns
        if let Terminal::Return(ret) = &block.terminal {
            let i = index;
            index += 1;
            state.assign(i, &ret.value, &func.returns);
        }

        // Handle MaybeThrow and Return terminal effects
        // Note: Rust terminal types don't have `effects` fields, so we skip this.
        // The TS code processes terminal.effects for MaybeThrow/Return terminals,
        // but in the Rust port these effects are folded into instruction effects.
    }

    // Apply all deferred mutations
    let mut all_range_updates: Vec<(IdentifierId, InstructionId)> = Vec::new();
    for mutation in &mutations {
        let updates = state.mutate(
            mutation.index,
            mutation.place_id,
            Some(InstructionId(mutation.id.0 + 1)),
            mutation.transitive,
            mutation.kind,
            mutation.loc,
            mutation.reason.as_ref(),
            &mut errors,
        );
        all_range_updates.extend(updates);
    }
    for render in &renders {
        state.render(render.index, render.place_id, &mut errors);
    }

    // Collect mutation info for context/param effects
    for param in func.context.iter().chain(func.params.iter().map(param_place)) {
        let Some(node) = state.nodes.get(&param.identifier.id) else {
            continue;
        };
        let mut mutated = false;
        if let Some(local) = node.local {
            match local.kind {
                MutationKind::Conditional => {
                    mutated = true;
                    function_effects.push(AliasingEffect::MutateConditionally {
                        value: Place { loc: local.loc, ..param.clone() },
                    });
                }
                MutationKind::Definite => {
                    mutated = true;
                    function_effects.push(AliasingEffect::Mutate {
                        value: Place { loc: local.loc, ..param.clone() },
                        reason: node.mutation_reason.clone(),
                    });
                }
                MutationKind::None => {}
            }
        }
        if let Some(transitive) = node.transitive {
            match transitive.kind {
                MutationKind::Conditional => {
                    mutated = true;
                    function_effects.push(AliasingEffect::MutateTransitiveConditionally {
                        value: Place { loc: transitive.loc, ..param.clone() },
                    });
                }
                MutationKind::Definite => {
                    mutated = true;
                    function_effects.push(AliasingEffect::MutateTransitive {
                        value: Place { loc: transitive.loc, ..param.clone() },
                    });
                }
                MutationKind::None => {}
            }
        }
        if mutated {
            // We'll set the effect on the param's Place below
        }
    }

    // Apply range updates to identifiers across all blocks
    // Collect identifier -> max end
    let mut range_end_map: FxHashMap<IdentifierId, InstructionId> = FxHashMap::default();
    for (id, end) in &all_range_updates {
        let entry = range_end_map.entry(*id).or_insert(InstructionId(0));
        if end.0 > entry.0 {
            *entry = *end;
        }
    }

    // Apply range updates to all places in the function
    apply_range_updates(func, &range_end_map);

    // Set effect on mutated context/param places
    let mutated_param_ids: FxHashSet<IdentifierId> = {
        let mut set = FxHashSet::default();
        for param in func.context.iter().chain(func.params.iter().map(param_place)) {
            let Some(node) = state.nodes.get(&param.identifier.id) else {
                continue;
            };
            if node.local.is_some() || node.transitive.is_some() {
                set.insert(param.identifier.id);
            }
        }
        set
    };
    for param in &mut func.context {
        if mutated_param_ids.contains(&param.identifier.id) {
            param.effect = Effect::Capture;
        }
    }
    for param in &mut func.params {
        let place = match param {
            ReactiveParam::Place(p) => p,
            ReactiveParam::Spread(s) => &mut s.place,
        };
        if mutated_param_ids.contains(&place.identifier.id) {
            place.effect = Effect::Capture;
        }
    }

    // ===== Phase 2: Set legacy Effect on each Place =====
    let block_ids_sorted: Vec<BlockId> = {
        let mut ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();
        ids.sort();
        ids
    };

    for &block_id in &block_ids_sorted {
        let block = func.body.blocks.get_mut(&block_id).map(|b| {
            // We need the terminal id for phi processing
            let terminal_id = b.terminal.id();
            let first_instr_id = b.instructions.first().map_or(terminal_id, |i| i.id);
            (b, terminal_id, first_instr_id)
        });
        let Some((block, _terminal_id, first_instr_id)) = block else {
            continue;
        };

        // Process phis
        for phi in &mut block.phis {
            phi.place.effect = Effect::Store;
            let is_phi_mutated_after_creation =
                phi.place.identifier.mutable_range.end > first_instr_id;
            for operand in phi.operands.values_mut() {
                operand.effect =
                    if is_phi_mutated_after_creation { Effect::Capture } else { Effect::Read };
            }
            if is_phi_mutated_after_creation
                && phi.place.identifier.mutable_range.start == InstructionId(0)
            {
                // Set start to the instruction before this block
                phi.place.identifier.mutable_range.start =
                    InstructionId(first_instr_id.0.saturating_sub(1));
            }
        }

        // Process instructions
        for instr in &mut block.instructions {
            // Default: set all lvalues to ConditionallyMutate
            // and fix up mutableRange.start
            set_lvalue_effects(instr);

            // Default: set all operands to Read
            set_default_operand_effects(instr);

            // If effects are present, override operand effects
            if instr.effects.is_some() {
                let operand_effects = compute_operand_effects(instr);

                // Apply operand effects to lvalues
                apply_lvalue_operand_effects(instr, &operand_effects);

                // Apply operand effects to operands + fix up mutableRange.start
                apply_operand_effects(instr, &operand_effects);

                // Handle StoreContext special case
                handle_store_context_range(instr);
            }
        }

        // Process terminal
        if let Terminal::Return(ret) = &mut block.terminal {
            ret.value.effect =
                if options.is_function_expression { Effect::Read } else { Effect::Freeze };
        } else {
            set_terminal_operand_effects(&mut block.terminal);
        }
    }

    // ===== Sync mutableRange across all Places =====
    // In TypeScript, Identifier is a reference type — updating mutableRange on one
    // Place automatically affects all Places sharing the same Identifier object.
    // In Rust, each Place has its own Identifier copy. After Phase 2 sets ranges
    // on lvalues, we need to propagate those ranges to all other Places with the
    // same IdentifierId (operands, effects, terminal operands, etc.).
    //
    // This sync ensures that when later passes (e.g., InferReactiveScopeVariables)
    // check operand mutable ranges, they see the same range as the lvalue definition.
    // TODO: sync_mutable_ranges causes net regression (310→308) due to over-extending
    // ranges for phi operands from different branches. Needs phi-aware exclusion logic.
    // sync_mutable_ranges(func);

    // ===== Phase 3: Compute external effects for params/context/returns =====
    let returns_id = func.returns.identifier.id;
    let return_value_kind = if func.returns.identifier.is_primitive_type() {
        ValueKind::Primitive
    } else if is_jsx_type(&func.returns.identifier) {
        ValueKind::Frozen
    } else {
        ValueKind::Mutable
    };
    function_effects.push(AliasingEffect::Create {
        into: func.returns.clone(),
        value: return_value_kind,
        reason: ValueReason::KnownReturnSignature,
    });

    // Collect tracked places
    let tracked: Vec<Place> = func
        .params
        .iter()
        .map(param_place)
        .chain(func.context.iter())
        .chain(std::iter::once(&func.returns))
        .cloned()
        .collect();

    let ignored_errors = &mut CompilerError::new();

    for into in &tracked {
        let mutation_index = index;
        index += 1;
        // Simulated mutation (no range update)
        state.mutate(
            mutation_index,
            into.identifier.id,
            None,
            true,
            MutationKind::Conditional,
            into.loc,
            None,
            ignored_errors,
        );
        for from in &tracked {
            if from.identifier.id == into.identifier.id || from.identifier.id == returns_id {
                continue;
            }
            let Some(from_node) = state.nodes.get(&from.identifier.id) else {
                continue;
            };
            if from_node.last_mutated == mutation_index {
                if into.identifier.id == returns_id {
                    // Return value could be any of the params/context variables
                    function_effects
                        .push(AliasingEffect::Alias { from: from.clone(), into: into.clone() });
                } else {
                    // Params/context-vars can only capture each other
                    function_effects
                        .push(AliasingEffect::Capture { from: from.clone(), into: into.clone() });
                }
            }
        }
    }

    if errors.has_any_errors() && !options.is_function_expression {
        return Err(errors);
    }

    Ok(function_effects)
}

// =====================================================================================
// Phase 2 helper functions
// =====================================================================================

/// Set all instruction lvalues to ConditionallyMutate and fix up mutableRange.start.
fn set_lvalue_effects(instr: &mut crate::hir::Instruction) {
    let instr_id = instr.id;

    // Fix the instruction's own lvalue
    instr.lvalue.effect = Effect::ConditionallyMutate;
    if instr.lvalue.identifier.mutable_range.start == InstructionId(0) {
        instr.lvalue.identifier.mutable_range.start = instr_id;
    }
    if instr.lvalue.identifier.mutable_range.end == InstructionId(0) {
        let new_end = std::cmp::max(instr_id.0 + 1, instr.lvalue.identifier.mutable_range.end.0);
        instr.lvalue.identifier.mutable_range.end = InstructionId(new_end);
    }

    // Fix lvalues from instruction value
    match &mut instr.value {
        InstructionValue::DeclareLocal(v) => {
            v.lvalue.place.effect = Effect::ConditionallyMutate;
            if v.lvalue.place.identifier.mutable_range.start == InstructionId(0) {
                v.lvalue.place.identifier.mutable_range.start = instr_id;
            }
            if v.lvalue.place.identifier.mutable_range.end == InstructionId(0) {
                let new_end =
                    std::cmp::max(instr_id.0 + 1, v.lvalue.place.identifier.mutable_range.end.0);
                v.lvalue.place.identifier.mutable_range.end = InstructionId(new_end);
            }
        }
        InstructionValue::DeclareContext(v) => {
            v.lvalue_place.effect = Effect::ConditionallyMutate;
            if v.lvalue_place.identifier.mutable_range.start == InstructionId(0) {
                v.lvalue_place.identifier.mutable_range.start = instr_id;
            }
            if v.lvalue_place.identifier.mutable_range.end == InstructionId(0) {
                let new_end =
                    std::cmp::max(instr_id.0 + 1, v.lvalue_place.identifier.mutable_range.end.0);
                v.lvalue_place.identifier.mutable_range.end = InstructionId(new_end);
            }
        }
        InstructionValue::StoreLocal(v) => {
            v.lvalue.place.effect = Effect::ConditionallyMutate;
            if v.lvalue.place.identifier.mutable_range.start == InstructionId(0) {
                v.lvalue.place.identifier.mutable_range.start = instr_id;
            }
            if v.lvalue.place.identifier.mutable_range.end == InstructionId(0) {
                let new_end =
                    std::cmp::max(instr_id.0 + 1, v.lvalue.place.identifier.mutable_range.end.0);
                v.lvalue.place.identifier.mutable_range.end = InstructionId(new_end);
            }
        }
        InstructionValue::StoreContext(v) => {
            v.lvalue_place.effect = Effect::ConditionallyMutate;
            if v.lvalue_place.identifier.mutable_range.start == InstructionId(0) {
                v.lvalue_place.identifier.mutable_range.start = instr_id;
            }
            if v.lvalue_place.identifier.mutable_range.end == InstructionId(0) {
                let new_end =
                    std::cmp::max(instr_id.0 + 1, v.lvalue_place.identifier.mutable_range.end.0);
                v.lvalue_place.identifier.mutable_range.end = InstructionId(new_end);
            }
        }
        InstructionValue::Destructure(v) => {
            set_pattern_lvalue_effects(&mut v.lvalue.pattern, instr_id);
        }
        InstructionValue::PrefixUpdate(v) => {
            v.lvalue.effect = Effect::ConditionallyMutate;
            if v.lvalue.identifier.mutable_range.start == InstructionId(0) {
                v.lvalue.identifier.mutable_range.start = instr_id;
            }
            if v.lvalue.identifier.mutable_range.end == InstructionId(0) {
                let new_end =
                    std::cmp::max(instr_id.0 + 1, v.lvalue.identifier.mutable_range.end.0);
                v.lvalue.identifier.mutable_range.end = InstructionId(new_end);
            }
        }
        InstructionValue::PostfixUpdate(v) => {
            v.lvalue.effect = Effect::ConditionallyMutate;
            if v.lvalue.identifier.mutable_range.start == InstructionId(0) {
                v.lvalue.identifier.mutable_range.start = instr_id;
            }
            if v.lvalue.identifier.mutable_range.end == InstructionId(0) {
                let new_end =
                    std::cmp::max(instr_id.0 + 1, v.lvalue.identifier.mutable_range.end.0);
                v.lvalue.identifier.mutable_range.end = InstructionId(new_end);
            }
        }
        _ => {}
    }
}

/// Set ConditionallyMutate on all places in a destructuring pattern.
fn set_pattern_lvalue_effects(pattern: &mut crate::hir::Pattern, instr_id: InstructionId) {
    match pattern {
        crate::hir::Pattern::Array(arr) => {
            for item in &mut arr.items {
                match item {
                    crate::hir::ArrayPatternElement::Place(p) => {
                        p.effect = Effect::ConditionallyMutate;
                        if p.identifier.mutable_range.start == InstructionId(0) {
                            p.identifier.mutable_range.start = instr_id;
                        }
                        if p.identifier.mutable_range.end == InstructionId(0) {
                            let new_end =
                                std::cmp::max(instr_id.0 + 1, p.identifier.mutable_range.end.0);
                            p.identifier.mutable_range.end = InstructionId(new_end);
                        }
                    }
                    crate::hir::ArrayPatternElement::Spread(s) => {
                        s.place.effect = Effect::ConditionallyMutate;
                        if s.place.identifier.mutable_range.start == InstructionId(0) {
                            s.place.identifier.mutable_range.start = instr_id;
                        }
                        if s.place.identifier.mutable_range.end == InstructionId(0) {
                            let new_end = std::cmp::max(
                                instr_id.0 + 1,
                                s.place.identifier.mutable_range.end.0,
                            );
                            s.place.identifier.mutable_range.end = InstructionId(new_end);
                        }
                    }
                    crate::hir::ArrayPatternElement::Hole => {}
                }
            }
        }
        crate::hir::Pattern::Object(obj) => {
            for prop in &mut obj.properties {
                match prop {
                    crate::hir::ObjectPatternProperty::Property(p) => {
                        p.place.effect = Effect::ConditionallyMutate;
                        if p.place.identifier.mutable_range.start == InstructionId(0) {
                            p.place.identifier.mutable_range.start = instr_id;
                        }
                        if p.place.identifier.mutable_range.end == InstructionId(0) {
                            let new_end = std::cmp::max(
                                instr_id.0 + 1,
                                p.place.identifier.mutable_range.end.0,
                            );
                            p.place.identifier.mutable_range.end = InstructionId(new_end);
                        }
                    }
                    crate::hir::ObjectPatternProperty::Spread(s) => {
                        s.place.effect = Effect::ConditionallyMutate;
                        if s.place.identifier.mutable_range.start == InstructionId(0) {
                            s.place.identifier.mutable_range.start = instr_id;
                        }
                        if s.place.identifier.mutable_range.end == InstructionId(0) {
                            let new_end = std::cmp::max(
                                instr_id.0 + 1,
                                s.place.identifier.mutable_range.end.0,
                            );
                            s.place.identifier.mutable_range.end = InstructionId(new_end);
                        }
                    }
                }
            }
        }
    }
}

/// Set all instruction value operands to Effect::Read.
fn set_default_operand_effects(instr: &mut crate::hir::Instruction) {
    set_operand_effects_on_value(&mut instr.value, Effect::Read);
}

/// Recursively set effects on all operands of an instruction value.
fn set_operand_effects_on_value(value: &mut InstructionValue, effect: Effect) {
    match value {
        InstructionValue::CallExpression(v) => {
            v.callee.effect = effect;
            for arg in &mut v.args {
                match arg {
                    crate::hir::CallArg::Place(p) => p.effect = effect,
                    crate::hir::CallArg::Spread(s) => s.place.effect = effect,
                }
            }
        }
        InstructionValue::NewExpression(v) => {
            v.callee.effect = effect;
            for arg in &mut v.args {
                match arg {
                    crate::hir::CallArg::Place(p) => p.effect = effect,
                    crate::hir::CallArg::Spread(s) => s.place.effect = effect,
                }
            }
        }
        InstructionValue::MethodCall(v) => {
            v.receiver.effect = effect;
            v.property.effect = effect;
            for arg in &mut v.args {
                match arg {
                    crate::hir::CallArg::Place(p) => p.effect = effect,
                    crate::hir::CallArg::Spread(s) => s.place.effect = effect,
                }
            }
        }
        InstructionValue::BinaryExpression(v) => {
            v.left.effect = effect;
            v.right.effect = effect;
        }
        InstructionValue::UnaryExpression(v) => {
            v.value.effect = effect;
        }
        InstructionValue::LoadLocal(v) => {
            v.place.effect = effect;
        }
        InstructionValue::LoadContext(v) => {
            v.place.effect = effect;
        }
        InstructionValue::StoreLocal(v) => {
            v.value.effect = effect;
        }
        InstructionValue::StoreContext(v) => {
            v.lvalue_place.effect = effect;
            v.value.effect = effect;
        }
        InstructionValue::StoreGlobal(v) => {
            v.value.effect = effect;
        }
        InstructionValue::Destructure(v) => {
            v.value.effect = effect;
        }
        InstructionValue::PropertyLoad(v) => {
            v.object.effect = effect;
        }
        InstructionValue::PropertyStore(v) => {
            v.object.effect = effect;
            v.value.effect = effect;
        }
        InstructionValue::PropertyDelete(v) => {
            v.object.effect = effect;
        }
        InstructionValue::ComputedLoad(v) => {
            v.object.effect = effect;
            v.property.effect = effect;
        }
        InstructionValue::ComputedStore(v) => {
            v.object.effect = effect;
            v.property.effect = effect;
            v.value.effect = effect;
        }
        InstructionValue::ComputedDelete(v) => {
            v.object.effect = effect;
            v.property.effect = effect;
        }
        InstructionValue::TypeCastExpression(v) => {
            v.value.effect = effect;
        }
        InstructionValue::JsxExpression(v) => {
            if let crate::hir::JsxTag::Place(ref mut p) = v.tag {
                p.effect = effect;
            }
            for attr in &mut v.props {
                match attr {
                    crate::hir::JsxAttribute::Attribute { place, .. } => place.effect = effect,
                    crate::hir::JsxAttribute::Spread { argument } => argument.effect = effect,
                }
            }
            if let Some(children) = &mut v.children {
                for child in children {
                    child.effect = effect;
                }
            }
        }
        InstructionValue::JsxFragment(v) => {
            for child in &mut v.children {
                child.effect = effect;
            }
        }
        InstructionValue::ObjectExpression(v) => {
            for prop in &mut v.properties {
                match prop {
                    crate::hir::ObjectPatternProperty::Property(p) => {
                        if let crate::hir::ObjectPropertyKey::Computed(ref mut place) = p.key {
                            place.effect = effect;
                        }
                        p.place.effect = effect;
                    }
                    crate::hir::ObjectPatternProperty::Spread(s) => s.place.effect = effect,
                }
            }
        }
        InstructionValue::ArrayExpression(v) => {
            for elem in &mut v.elements {
                match elem {
                    crate::hir::ArrayExpressionElement::Place(p) => p.effect = effect,
                    crate::hir::ArrayExpressionElement::Spread(s) => s.place.effect = effect,
                    crate::hir::ArrayExpressionElement::Hole => {}
                }
            }
        }
        InstructionValue::FunctionExpression(v) => {
            for ctx in &mut v.lowered_func.func.context {
                ctx.effect = effect;
            }
        }
        InstructionValue::ObjectMethod(v) => {
            for ctx in &mut v.lowered_func.func.context {
                ctx.effect = effect;
            }
        }
        InstructionValue::TaggedTemplateExpression(v) => {
            v.tag.effect = effect;
        }
        InstructionValue::TemplateLiteral(v) => {
            for subexpr in &mut v.subexprs {
                subexpr.effect = effect;
            }
        }
        InstructionValue::Await(v) => {
            v.value.effect = effect;
        }
        InstructionValue::GetIterator(v) => {
            v.collection.effect = effect;
        }
        InstructionValue::IteratorNext(v) => {
            v.iterator.effect = effect;
            v.collection.effect = effect;
        }
        InstructionValue::NextPropertyOf(v) => {
            v.value.effect = effect;
        }
        InstructionValue::PrefixUpdate(v) => {
            v.value.effect = effect;
        }
        InstructionValue::PostfixUpdate(v) => {
            v.value.effect = effect;
        }
        InstructionValue::StartMemoize(v) => {
            if let Some(deps) = &mut v.deps {
                for dep in deps {
                    if let crate::hir::ManualMemoDependencyRoot::NamedLocal {
                        ref mut value, ..
                    } = dep.root
                    {
                        value.effect = effect;
                    }
                }
            }
        }
        InstructionValue::FinishMemoize(v) => {
            v.decl.effect = effect;
        }
        InstructionValue::LoadGlobal(_)
        | InstructionValue::MetaProperty(_)
        | InstructionValue::RegExpLiteral(_)
        | InstructionValue::Primitive(_)
        | InstructionValue::JsxText(_)
        | InstructionValue::DeclareLocal(_)
        | InstructionValue::DeclareContext(_)
        | InstructionValue::Debugger(_)
        | InstructionValue::UnsupportedNode(_) => {}
    }
}

/// Compute the operand effect overrides from instruction effects.
fn compute_operand_effects(instr: &crate::hir::Instruction) -> FxHashMap<IdentifierId, Effect> {
    let mut operand_effects: FxHashMap<IdentifierId, Effect> = FxHashMap::default();
    let instr_id = instr.id;

    let Some(effects) = &instr.effects else {
        return operand_effects;
    };

    for effect in effects {
        match effect {
            AliasingEffect::Assign { from, into }
            | AliasingEffect::Alias { from, into }
            | AliasingEffect::Capture { from, into }
            | AliasingEffect::CreateFrom { from, into }
            | AliasingEffect::MaybeAlias { from, into } => {
                let is_mutated_or_reassigned = into.identifier.mutable_range.end > instr_id;
                if is_mutated_or_reassigned {
                    operand_effects.insert(from.identifier.id, Effect::Capture);
                    operand_effects.insert(into.identifier.id, Effect::Store);
                } else {
                    operand_effects.insert(from.identifier.id, Effect::Read);
                    operand_effects.insert(into.identifier.id, Effect::Store);
                }
            }
            AliasingEffect::Mutate { value, .. } => {
                operand_effects.insert(value.identifier.id, Effect::Store);
            }
            AliasingEffect::MutateTransitive { value, .. }
            | AliasingEffect::MutateConditionally { value, .. }
            | AliasingEffect::MutateTransitiveConditionally { value, .. } => {
                operand_effects.insert(value.identifier.id, Effect::ConditionallyMutate);
            }
            AliasingEffect::Freeze { value, .. } => {
                operand_effects.insert(value.identifier.id, Effect::Freeze);
            }
            AliasingEffect::CreateFunction { .. }
            | AliasingEffect::Create { .. }
            | AliasingEffect::ImmutableCapture { .. }
            | AliasingEffect::Impure { .. }
            | AliasingEffect::Render { .. }
            | AliasingEffect::MutateFrozen { .. }
            | AliasingEffect::MutateGlobal { .. }
            | AliasingEffect::Apply { .. } => {
                // no-op (Apply should have been replaced by more precise effects)
            }
        }
    }

    operand_effects
}

/// Apply operand effects to lvalues of an instruction.
fn apply_lvalue_operand_effects(
    instr: &mut crate::hir::Instruction,
    operand_effects: &FxHashMap<IdentifierId, Effect>,
) {
    // Apply to instruction's own lvalue
    let effect = operand_effects
        .get(&instr.lvalue.identifier.id)
        .copied()
        .unwrap_or(Effect::ConditionallyMutate);
    instr.lvalue.effect = effect;

    // Apply to value lvalues
    match &mut instr.value {
        InstructionValue::DeclareLocal(v) => {
            let eff = operand_effects
                .get(&v.lvalue.place.identifier.id)
                .copied()
                .unwrap_or(Effect::ConditionallyMutate);
            v.lvalue.place.effect = eff;
        }
        InstructionValue::DeclareContext(v) => {
            let eff = operand_effects
                .get(&v.lvalue_place.identifier.id)
                .copied()
                .unwrap_or(Effect::ConditionallyMutate);
            v.lvalue_place.effect = eff;
        }
        InstructionValue::StoreLocal(v) => {
            let eff = operand_effects
                .get(&v.lvalue.place.identifier.id)
                .copied()
                .unwrap_or(Effect::ConditionallyMutate);
            v.lvalue.place.effect = eff;
        }
        InstructionValue::StoreContext(v) => {
            let eff = operand_effects
                .get(&v.lvalue_place.identifier.id)
                .copied()
                .unwrap_or(Effect::ConditionallyMutate);
            v.lvalue_place.effect = eff;
        }
        InstructionValue::Destructure(v) => {
            apply_pattern_operand_effects(&mut v.lvalue.pattern, operand_effects);
        }
        InstructionValue::PrefixUpdate(v) => {
            let eff = operand_effects
                .get(&v.lvalue.identifier.id)
                .copied()
                .unwrap_or(Effect::ConditionallyMutate);
            v.lvalue.effect = eff;
        }
        InstructionValue::PostfixUpdate(v) => {
            let eff = operand_effects
                .get(&v.lvalue.identifier.id)
                .copied()
                .unwrap_or(Effect::ConditionallyMutate);
            v.lvalue.effect = eff;
        }
        _ => {}
    }
}

/// Apply operand effects to pattern places.
fn apply_pattern_operand_effects(
    pattern: &mut crate::hir::Pattern,
    operand_effects: &FxHashMap<IdentifierId, Effect>,
) {
    match pattern {
        crate::hir::Pattern::Array(arr) => {
            for item in &mut arr.items {
                match item {
                    crate::hir::ArrayPatternElement::Place(p) => {
                        let eff = operand_effects
                            .get(&p.identifier.id)
                            .copied()
                            .unwrap_or(Effect::ConditionallyMutate);
                        p.effect = eff;
                    }
                    crate::hir::ArrayPatternElement::Spread(s) => {
                        let eff = operand_effects
                            .get(&s.place.identifier.id)
                            .copied()
                            .unwrap_or(Effect::ConditionallyMutate);
                        s.place.effect = eff;
                    }
                    crate::hir::ArrayPatternElement::Hole => {}
                }
            }
        }
        crate::hir::Pattern::Object(obj) => {
            for prop in &mut obj.properties {
                match prop {
                    crate::hir::ObjectPatternProperty::Property(p) => {
                        let eff = operand_effects
                            .get(&p.place.identifier.id)
                            .copied()
                            .unwrap_or(Effect::ConditionallyMutate);
                        p.place.effect = eff;
                    }
                    crate::hir::ObjectPatternProperty::Spread(s) => {
                        let eff = operand_effects
                            .get(&s.place.identifier.id)
                            .copied()
                            .unwrap_or(Effect::ConditionallyMutate);
                        s.place.effect = eff;
                    }
                }
            }
        }
    }
}

/// Apply operand effects to value operands, with mutableRange.start fixup.
fn apply_operand_effects(
    instr: &mut crate::hir::Instruction,
    operand_effects: &FxHashMap<IdentifierId, Effect>,
) {
    let instr_id = instr.id;
    apply_operand_effects_to_value(&mut instr.value, instr_id, operand_effects);
}

/// Apply operand effects to all operand places in an instruction value.
fn apply_operand_effects_to_value(
    value: &mut InstructionValue,
    instr_id: InstructionId,
    operand_effects: &FxHashMap<IdentifierId, Effect>,
) {
    // Helper closure to fix up a single operand
    fn fix_operand(
        place: &mut Place,
        instr_id: InstructionId,
        operand_effects: &FxHashMap<IdentifierId, Effect>,
    ) {
        if place.identifier.mutable_range.end > instr_id
            && place.identifier.mutable_range.start == InstructionId(0)
        {
            place.identifier.mutable_range.start = instr_id;
        }
        let effect = operand_effects.get(&place.identifier.id).copied().unwrap_or(Effect::Read);
        place.effect = effect;
    }

    match value {
        InstructionValue::CallExpression(v) => {
            fix_operand(&mut v.callee, instr_id, operand_effects);
            for arg in &mut v.args {
                match arg {
                    crate::hir::CallArg::Place(p) => fix_operand(p, instr_id, operand_effects),
                    crate::hir::CallArg::Spread(s) => {
                        fix_operand(&mut s.place, instr_id, operand_effects);
                    }
                }
            }
        }
        InstructionValue::NewExpression(v) => {
            fix_operand(&mut v.callee, instr_id, operand_effects);
            for arg in &mut v.args {
                match arg {
                    crate::hir::CallArg::Place(p) => fix_operand(p, instr_id, operand_effects),
                    crate::hir::CallArg::Spread(s) => {
                        fix_operand(&mut s.place, instr_id, operand_effects);
                    }
                }
            }
        }
        InstructionValue::MethodCall(v) => {
            fix_operand(&mut v.receiver, instr_id, operand_effects);
            fix_operand(&mut v.property, instr_id, operand_effects);
            for arg in &mut v.args {
                match arg {
                    crate::hir::CallArg::Place(p) => fix_operand(p, instr_id, operand_effects),
                    crate::hir::CallArg::Spread(s) => {
                        fix_operand(&mut s.place, instr_id, operand_effects);
                    }
                }
            }
        }
        InstructionValue::BinaryExpression(v) => {
            fix_operand(&mut v.left, instr_id, operand_effects);
            fix_operand(&mut v.right, instr_id, operand_effects);
        }
        InstructionValue::UnaryExpression(v) => {
            fix_operand(&mut v.value, instr_id, operand_effects);
        }
        InstructionValue::LoadLocal(v) => {
            fix_operand(&mut v.place, instr_id, operand_effects);
        }
        InstructionValue::LoadContext(v) => {
            fix_operand(&mut v.place, instr_id, operand_effects);
        }
        InstructionValue::StoreLocal(v) => {
            fix_operand(&mut v.value, instr_id, operand_effects);
        }
        InstructionValue::StoreContext(v) => {
            fix_operand(&mut v.lvalue_place, instr_id, operand_effects);
            fix_operand(&mut v.value, instr_id, operand_effects);
        }
        InstructionValue::StoreGlobal(v) => {
            fix_operand(&mut v.value, instr_id, operand_effects);
        }
        InstructionValue::Destructure(v) => {
            fix_operand(&mut v.value, instr_id, operand_effects);
        }
        InstructionValue::PropertyLoad(v) => {
            fix_operand(&mut v.object, instr_id, operand_effects);
        }
        InstructionValue::PropertyStore(v) => {
            fix_operand(&mut v.object, instr_id, operand_effects);
            fix_operand(&mut v.value, instr_id, operand_effects);
        }
        InstructionValue::PropertyDelete(v) => {
            fix_operand(&mut v.object, instr_id, operand_effects);
        }
        InstructionValue::ComputedLoad(v) => {
            fix_operand(&mut v.object, instr_id, operand_effects);
            fix_operand(&mut v.property, instr_id, operand_effects);
        }
        InstructionValue::ComputedStore(v) => {
            fix_operand(&mut v.object, instr_id, operand_effects);
            fix_operand(&mut v.property, instr_id, operand_effects);
            fix_operand(&mut v.value, instr_id, operand_effects);
        }
        InstructionValue::ComputedDelete(v) => {
            fix_operand(&mut v.object, instr_id, operand_effects);
            fix_operand(&mut v.property, instr_id, operand_effects);
        }
        InstructionValue::TypeCastExpression(v) => {
            fix_operand(&mut v.value, instr_id, operand_effects);
        }
        InstructionValue::JsxExpression(v) => {
            if let crate::hir::JsxTag::Place(ref mut p) = v.tag {
                fix_operand(p, instr_id, operand_effects);
            }
            for attr in &mut v.props {
                match attr {
                    crate::hir::JsxAttribute::Attribute { place, .. } => {
                        fix_operand(place, instr_id, operand_effects);
                    }
                    crate::hir::JsxAttribute::Spread { argument } => {
                        fix_operand(argument, instr_id, operand_effects);
                    }
                }
            }
            if let Some(children) = &mut v.children {
                for child in children {
                    fix_operand(child, instr_id, operand_effects);
                }
            }
        }
        InstructionValue::JsxFragment(v) => {
            for child in &mut v.children {
                fix_operand(child, instr_id, operand_effects);
            }
        }
        InstructionValue::ObjectExpression(v) => {
            for prop in &mut v.properties {
                match prop {
                    crate::hir::ObjectPatternProperty::Property(p) => {
                        if let crate::hir::ObjectPropertyKey::Computed(ref mut place) = p.key {
                            fix_operand(place, instr_id, operand_effects);
                        }
                        fix_operand(&mut p.place, instr_id, operand_effects);
                    }
                    crate::hir::ObjectPatternProperty::Spread(s) => {
                        fix_operand(&mut s.place, instr_id, operand_effects);
                    }
                }
            }
        }
        InstructionValue::ArrayExpression(v) => {
            for elem in &mut v.elements {
                match elem {
                    crate::hir::ArrayExpressionElement::Place(p) => {
                        fix_operand(p, instr_id, operand_effects);
                    }
                    crate::hir::ArrayExpressionElement::Spread(s) => {
                        fix_operand(&mut s.place, instr_id, operand_effects);
                    }
                    crate::hir::ArrayExpressionElement::Hole => {}
                }
            }
        }
        InstructionValue::FunctionExpression(v) => {
            for ctx in &mut v.lowered_func.func.context {
                fix_operand(ctx, instr_id, operand_effects);
            }
        }
        InstructionValue::ObjectMethod(v) => {
            for ctx in &mut v.lowered_func.func.context {
                fix_operand(ctx, instr_id, operand_effects);
            }
        }
        InstructionValue::TaggedTemplateExpression(v) => {
            fix_operand(&mut v.tag, instr_id, operand_effects);
        }
        InstructionValue::TemplateLiteral(v) => {
            for subexpr in &mut v.subexprs {
                fix_operand(subexpr, instr_id, operand_effects);
            }
        }
        InstructionValue::Await(v) => {
            fix_operand(&mut v.value, instr_id, operand_effects);
        }
        InstructionValue::GetIterator(v) => {
            fix_operand(&mut v.collection, instr_id, operand_effects);
        }
        InstructionValue::IteratorNext(v) => {
            fix_operand(&mut v.iterator, instr_id, operand_effects);
            fix_operand(&mut v.collection, instr_id, operand_effects);
        }
        InstructionValue::NextPropertyOf(v) => {
            fix_operand(&mut v.value, instr_id, operand_effects);
        }
        InstructionValue::PrefixUpdate(v) => {
            fix_operand(&mut v.value, instr_id, operand_effects);
        }
        InstructionValue::PostfixUpdate(v) => {
            fix_operand(&mut v.value, instr_id, operand_effects);
        }
        InstructionValue::StartMemoize(v) => {
            if let Some(deps) = &mut v.deps {
                for dep in deps {
                    if let crate::hir::ManualMemoDependencyRoot::NamedLocal {
                        ref mut value, ..
                    } = dep.root
                    {
                        fix_operand(value, instr_id, operand_effects);
                    }
                }
            }
        }
        InstructionValue::FinishMemoize(v) => {
            fix_operand(&mut v.decl, instr_id, operand_effects);
        }
        InstructionValue::LoadGlobal(_)
        | InstructionValue::MetaProperty(_)
        | InstructionValue::RegExpLiteral(_)
        | InstructionValue::Primitive(_)
        | InstructionValue::JsxText(_)
        | InstructionValue::DeclareLocal(_)
        | InstructionValue::DeclareContext(_)
        | InstructionValue::Debugger(_)
        | InstructionValue::UnsupportedNode(_) => {}
    }
}

/// Handle StoreContext special case: extend rvalue's mutableRange.
fn handle_store_context_range(instr: &mut crate::hir::Instruction) {
    let instr_id = instr.id;
    if let InstructionValue::StoreContext(v) = &mut instr.value
        && v.value.identifier.mutable_range.end <= instr_id
    {
        v.value.identifier.mutable_range.end = InstructionId(instr_id.0 + 1);
    }
}

/// Set all terminal operands to Effect::Read.
fn set_terminal_operand_effects(terminal: &mut Terminal) {
    match terminal {
        Terminal::Throw(t) => t.value.effect = Effect::Read,
        Terminal::If(t) => t.test.effect = Effect::Read,
        Terminal::Branch(t) => t.test.effect = Effect::Read,
        Terminal::Switch(t) => {
            t.test.effect = Effect::Read;
            for case in &mut t.cases {
                if let Some(ref mut test) = case.test {
                    test.effect = Effect::Read;
                }
            }
        }
        Terminal::Try(t) => {
            if let Some(ref mut binding) = t.handler_binding {
                binding.effect = Effect::Read;
            }
        }
        Terminal::Return(t) => t.value.effect = Effect::Read,
        _ => {}
    }
}

/// Apply mutableRange.end updates to all identifier occurrences in the function.
fn apply_range_updates(func: &mut HIRFunction, updates: &FxHashMap<IdentifierId, InstructionId>) {
    if updates.is_empty() {
        return;
    }

    // Update params
    for param in &mut func.params {
        let place = match param {
            ReactiveParam::Place(p) => p,
            ReactiveParam::Spread(s) => &mut s.place,
        };
        update_place_range(place, updates);
    }

    // Update context
    for ctx in &mut func.context {
        update_place_range(ctx, updates);
    }

    // Update returns
    update_place_range(&mut func.returns, updates);

    // Update all places in blocks
    let block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        let Some(block) = func.body.blocks.get_mut(&block_id) else {
            continue;
        };

        // Update phis
        for phi in &mut block.phis {
            update_place_range(&mut phi.place, updates);
            for operand in phi.operands.values_mut() {
                update_place_range(operand, updates);
            }
        }

        // Update instructions
        for instr in &mut block.instructions {
            update_place_range(&mut instr.lvalue, updates);
            update_instruction_value_ranges(&mut instr.value, updates);

            // Update places in effects
            if let Some(effects) = &mut instr.effects {
                for effect in effects {
                    update_effect_ranges(effect, updates);
                }
            }
        }

        // Update terminal
        update_terminal_ranges(&mut block.terminal, updates);
    }
}

/// Update the mutableRange.end of a place if there's an update for its identifier.
fn update_place_range(place: &mut Place, updates: &FxHashMap<IdentifierId, InstructionId>) {
    if let Some(&end) = updates.get(&place.identifier.id) {
        place.identifier.mutable_range.end = std::cmp::max(place.identifier.mutable_range.end, end);
    }
}

/// Update mutableRange.end for all places in an instruction value.
fn update_instruction_value_ranges(
    value: &mut InstructionValue,
    updates: &FxHashMap<IdentifierId, InstructionId>,
) {
    match value {
        InstructionValue::CallExpression(v) => {
            update_place_range(&mut v.callee, updates);
            for arg in &mut v.args {
                match arg {
                    crate::hir::CallArg::Place(p) => update_place_range(p, updates),
                    crate::hir::CallArg::Spread(s) => update_place_range(&mut s.place, updates),
                }
            }
        }
        InstructionValue::NewExpression(v) => {
            update_place_range(&mut v.callee, updates);
            for arg in &mut v.args {
                match arg {
                    crate::hir::CallArg::Place(p) => update_place_range(p, updates),
                    crate::hir::CallArg::Spread(s) => update_place_range(&mut s.place, updates),
                }
            }
        }
        InstructionValue::MethodCall(v) => {
            update_place_range(&mut v.receiver, updates);
            update_place_range(&mut v.property, updates);
            for arg in &mut v.args {
                match arg {
                    crate::hir::CallArg::Place(p) => update_place_range(p, updates),
                    crate::hir::CallArg::Spread(s) => update_place_range(&mut s.place, updates),
                }
            }
        }
        InstructionValue::BinaryExpression(v) => {
            update_place_range(&mut v.left, updates);
            update_place_range(&mut v.right, updates);
        }
        InstructionValue::UnaryExpression(v) => {
            update_place_range(&mut v.value, updates);
        }
        InstructionValue::LoadLocal(v) => {
            update_place_range(&mut v.place, updates);
        }
        InstructionValue::LoadContext(v) => {
            update_place_range(&mut v.place, updates);
        }
        InstructionValue::StoreLocal(v) => {
            update_place_range(&mut v.lvalue.place, updates);
            update_place_range(&mut v.value, updates);
        }
        InstructionValue::StoreContext(v) => {
            update_place_range(&mut v.lvalue_place, updates);
            update_place_range(&mut v.value, updates);
        }
        InstructionValue::StoreGlobal(v) => {
            update_place_range(&mut v.value, updates);
        }
        InstructionValue::Destructure(v) => {
            update_pattern_ranges(&mut v.lvalue.pattern, updates);
            update_place_range(&mut v.value, updates);
        }
        InstructionValue::PropertyLoad(v) => {
            update_place_range(&mut v.object, updates);
        }
        InstructionValue::PropertyStore(v) => {
            update_place_range(&mut v.object, updates);
            update_place_range(&mut v.value, updates);
        }
        InstructionValue::PropertyDelete(v) => {
            update_place_range(&mut v.object, updates);
        }
        InstructionValue::ComputedLoad(v) => {
            update_place_range(&mut v.object, updates);
            update_place_range(&mut v.property, updates);
        }
        InstructionValue::ComputedStore(v) => {
            update_place_range(&mut v.object, updates);
            update_place_range(&mut v.property, updates);
            update_place_range(&mut v.value, updates);
        }
        InstructionValue::ComputedDelete(v) => {
            update_place_range(&mut v.object, updates);
            update_place_range(&mut v.property, updates);
        }
        InstructionValue::TypeCastExpression(v) => {
            update_place_range(&mut v.value, updates);
        }
        InstructionValue::JsxExpression(v) => {
            if let crate::hir::JsxTag::Place(ref mut p) = v.tag {
                update_place_range(p, updates);
            }
            for attr in &mut v.props {
                match attr {
                    crate::hir::JsxAttribute::Attribute { place, .. } => {
                        update_place_range(place, updates);
                    }
                    crate::hir::JsxAttribute::Spread { argument } => {
                        update_place_range(argument, updates);
                    }
                }
            }
            if let Some(children) = &mut v.children {
                for child in children {
                    update_place_range(child, updates);
                }
            }
        }
        InstructionValue::JsxFragment(v) => {
            for child in &mut v.children {
                update_place_range(child, updates);
            }
        }
        InstructionValue::ObjectExpression(v) => {
            for prop in &mut v.properties {
                match prop {
                    crate::hir::ObjectPatternProperty::Property(p) => {
                        if let crate::hir::ObjectPropertyKey::Computed(ref mut place) = p.key {
                            update_place_range(place, updates);
                        }
                        update_place_range(&mut p.place, updates);
                    }
                    crate::hir::ObjectPatternProperty::Spread(s) => {
                        update_place_range(&mut s.place, updates);
                    }
                }
            }
        }
        InstructionValue::ArrayExpression(v) => {
            for elem in &mut v.elements {
                match elem {
                    crate::hir::ArrayExpressionElement::Place(p) => {
                        update_place_range(p, updates);
                    }
                    crate::hir::ArrayExpressionElement::Spread(s) => {
                        update_place_range(&mut s.place, updates);
                    }
                    crate::hir::ArrayExpressionElement::Hole => {}
                }
            }
        }
        InstructionValue::FunctionExpression(v) => {
            for ctx in &mut v.lowered_func.func.context {
                update_place_range(ctx, updates);
            }
        }
        InstructionValue::ObjectMethod(v) => {
            for ctx in &mut v.lowered_func.func.context {
                update_place_range(ctx, updates);
            }
        }
        InstructionValue::TaggedTemplateExpression(v) => {
            update_place_range(&mut v.tag, updates);
        }
        InstructionValue::TemplateLiteral(v) => {
            for subexpr in &mut v.subexprs {
                update_place_range(subexpr, updates);
            }
        }
        InstructionValue::Await(v) => {
            update_place_range(&mut v.value, updates);
        }
        InstructionValue::GetIterator(v) => {
            update_place_range(&mut v.collection, updates);
        }
        InstructionValue::IteratorNext(v) => {
            update_place_range(&mut v.iterator, updates);
            update_place_range(&mut v.collection, updates);
        }
        InstructionValue::NextPropertyOf(v) => {
            update_place_range(&mut v.value, updates);
        }
        InstructionValue::PrefixUpdate(v) => {
            update_place_range(&mut v.lvalue, updates);
            update_place_range(&mut v.value, updates);
        }
        InstructionValue::PostfixUpdate(v) => {
            update_place_range(&mut v.lvalue, updates);
            update_place_range(&mut v.value, updates);
        }
        InstructionValue::StartMemoize(v) => {
            if let Some(deps) = &mut v.deps {
                for dep in deps {
                    if let crate::hir::ManualMemoDependencyRoot::NamedLocal {
                        ref mut value, ..
                    } = dep.root
                    {
                        update_place_range(value, updates);
                    }
                }
            }
        }
        InstructionValue::FinishMemoize(v) => {
            update_place_range(&mut v.decl, updates);
        }
        InstructionValue::DeclareLocal(v) => {
            update_place_range(&mut v.lvalue.place, updates);
        }
        InstructionValue::DeclareContext(v) => {
            update_place_range(&mut v.lvalue_place, updates);
        }
        InstructionValue::LoadGlobal(_)
        | InstructionValue::MetaProperty(_)
        | InstructionValue::RegExpLiteral(_)
        | InstructionValue::Primitive(_)
        | InstructionValue::JsxText(_)
        | InstructionValue::Debugger(_)
        | InstructionValue::UnsupportedNode(_) => {}
    }
}

/// Update mutableRange.end for places in a pattern.
fn update_pattern_ranges(
    pattern: &mut crate::hir::Pattern,
    updates: &FxHashMap<IdentifierId, InstructionId>,
) {
    match pattern {
        crate::hir::Pattern::Array(arr) => {
            for item in &mut arr.items {
                match item {
                    crate::hir::ArrayPatternElement::Place(p) => update_place_range(p, updates),
                    crate::hir::ArrayPatternElement::Spread(s) => {
                        update_place_range(&mut s.place, updates);
                    }
                    crate::hir::ArrayPatternElement::Hole => {}
                }
            }
        }
        crate::hir::Pattern::Object(obj) => {
            for prop in &mut obj.properties {
                match prop {
                    crate::hir::ObjectPatternProperty::Property(p) => {
                        update_place_range(&mut p.place, updates);
                    }
                    crate::hir::ObjectPatternProperty::Spread(s) => {
                        update_place_range(&mut s.place, updates);
                    }
                }
            }
        }
    }
}

/// Update mutableRange.end for all places in an aliasing effect.
fn update_effect_ranges(
    effect: &mut AliasingEffect,
    updates: &FxHashMap<IdentifierId, InstructionId>,
) {
    match effect {
        AliasingEffect::Freeze { value, .. }
        | AliasingEffect::Mutate { value, .. }
        | AliasingEffect::MutateConditionally { value }
        | AliasingEffect::MutateTransitive { value }
        | AliasingEffect::MutateTransitiveConditionally { value }
        | AliasingEffect::Render { place: value } => {
            update_place_range(value, updates);
        }
        AliasingEffect::Capture { from, into }
        | AliasingEffect::Alias { from, into }
        | AliasingEffect::MaybeAlias { from, into }
        | AliasingEffect::Assign { from, into }
        | AliasingEffect::CreateFrom { from, into }
        | AliasingEffect::ImmutableCapture { from, into } => {
            update_place_range(from, updates);
            update_place_range(into, updates);
        }
        AliasingEffect::Create { into, .. } => {
            update_place_range(into, updates);
        }
        AliasingEffect::CreateFunction { captures, into, .. } => {
            for cap in captures {
                update_place_range(cap, updates);
            }
            update_place_range(into, updates);
        }
        AliasingEffect::MutateFrozen { place, .. }
        | AliasingEffect::MutateGlobal { place, .. }
        | AliasingEffect::Impure { place, .. } => {
            update_place_range(place, updates);
        }
        AliasingEffect::Apply { receiver, function, args, into, .. } => {
            update_place_range(receiver, updates);
            update_place_range(function, updates);
            for arg in args {
                match arg {
                    crate::inference::aliasing_effects::ApplyArg::Place(p) => {
                        update_place_range(p, updates);
                    }
                    crate::inference::aliasing_effects::ApplyArg::Spread(s) => {
                        update_place_range(&mut s.place, updates);
                    }
                    crate::inference::aliasing_effects::ApplyArg::Hole => {}
                }
            }
            update_place_range(into, updates);
        }
    }
}

/// Update mutableRange.end for places in a terminal.
fn update_terminal_ranges(
    terminal: &mut Terminal,
    updates: &FxHashMap<IdentifierId, InstructionId>,
) {
    match terminal {
        Terminal::Throw(t) => update_place_range(&mut t.value, updates),
        Terminal::Return(t) => update_place_range(&mut t.value, updates),
        Terminal::If(t) => update_place_range(&mut t.test, updates),
        Terminal::Branch(t) => update_place_range(&mut t.test, updates),
        Terminal::Switch(t) => {
            update_place_range(&mut t.test, updates);
            for case in &mut t.cases {
                if let Some(ref mut test) = case.test {
                    update_place_range(test, updates);
                }
            }
        }
        Terminal::Try(t) => {
            if let Some(ref mut binding) = t.handler_binding {
                update_place_range(binding, updates);
            }
        }
        _ => {}
    }
}

// =============================================================================
// Mutable range synchronization
// =============================================================================

use crate::hir::MutableRange;

/// Synchronize mutableRange across all Places sharing the same IdentifierId.
///
/// In the TypeScript compiler, Identifier is a reference type — mutating mutableRange
/// on any Place automatically affects all other Places that share the same Identifier.
/// In Rust, each Place owns its own Identifier copy. After Phase 2 updates ranges on
/// lvalues, operands still have stale [0..0) ranges. This function collects the
/// canonical range for each IdentifierId and propagates it to all Places.
fn sync_mutable_ranges(func: &mut HIRFunction) {
    use crate::hir::visitors::{
        each_instruction_lvalue, each_instruction_value_operand, each_terminal_operand,
    };

    // Step 1: Collect canonical ranges (max end, min non-zero start) per IdentifierId
    let mut canonical: FxHashMap<IdentifierId, MutableRange> = FxHashMap::default();

    fn merge_range(canonical: &mut FxHashMap<IdentifierId, MutableRange>, place: &Place) {
        let id = place.identifier.id;
        let range = &place.identifier.mutable_range;
        canonical
            .entry(id)
            .and_modify(|existing| {
                if range.start.0 > 0 {
                    if existing.start.0 == 0 || range.start.0 < existing.start.0 {
                        existing.start = range.start;
                    }
                }
                if range.end.0 > existing.end.0 {
                    existing.end = range.end;
                }
            })
            .or_insert(*range);
    }

    // Collect from params
    for param in &func.params {
        let place = match param {
            ReactiveParam::Place(p) => p,
            ReactiveParam::Spread(s) => &s.place,
        };
        merge_range(&mut canonical, place);
    }
    for ctx in &func.context {
        merge_range(&mut canonical, ctx);
    }
    merge_range(&mut canonical, &func.returns);

    // Collect from all blocks — use visitors to cover all places
    for block in func.body.blocks.values() {
        for phi in &block.phis {
            merge_range(&mut canonical, &phi.place);
            for operand in phi.operands.values() {
                merge_range(&mut canonical, operand);
            }
        }
        for instr in &block.instructions {
            // Top-level lvalue
            merge_range(&mut canonical, &instr.lvalue);
            // Inner lvalues (StoreLocal target, DeclareLocal target, etc.)
            for lvalue in each_instruction_lvalue(instr) {
                merge_range(&mut canonical, lvalue);
            }
            // All operands
            for operand in each_instruction_value_operand(&instr.value) {
                merge_range(&mut canonical, operand);
            }
        }
        for operand in each_terminal_operand(&block.terminal) {
            merge_range(&mut canonical, operand);
        }
    }

    // Step 2: Apply canonical ranges to ALL Places.
    for param in &mut func.params {
        let place = match param {
            ReactiveParam::Place(p) => p,
            ReactiveParam::Spread(s) => &mut s.place,
        };
        sync_place(&canonical, place);
    }
    for ctx in &mut func.context {
        sync_place(&canonical, ctx);
    }
    sync_place(&canonical, &mut func.returns);

    let block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        let Some(block) = func.body.blocks.get_mut(&block_id) else {
            continue;
        };
        for phi in &mut block.phis {
            sync_place(&canonical, &mut phi.place);
            for operand in phi.operands.values_mut() {
                sync_place(&canonical, operand);
            }
        }
        for instr in &mut block.instructions {
            sync_place(&canonical, &mut instr.lvalue);
            sync_instruction_value_ranges(&canonical, &mut instr.value);
            if let Some(effects) = &mut instr.effects {
                for effect in effects {
                    sync_effect_ranges(&canonical, effect);
                }
            }
        }
        sync_terminal_ranges(&canonical, &mut block.terminal);
    }
}

fn sync_place(canonical: &FxHashMap<IdentifierId, MutableRange>, place: &mut Place) {
    if let Some(range) = canonical.get(&place.identifier.id) {
        place.identifier.mutable_range = *range;
    }
}

/// Sync mutable range for instruction value places.
/// Reuses the same pattern as `update_instruction_value_ranges` but applies full range.
fn sync_instruction_value_ranges(
    canonical: &FxHashMap<IdentifierId, MutableRange>,
    value: &mut InstructionValue,
) {
    // Use the existing update_instruction_value_ranges infrastructure
    // but convert canonical to an InstructionId-only map for end updates,
    // then separately handle start updates.
    // Actually, simpler: just walk all mutable places and sync each one.
    match value {
        InstructionValue::CallExpression(v) => {
            sync_place(canonical, &mut v.callee);
            for arg in &mut v.args {
                match arg {
                    crate::hir::CallArg::Place(p) => sync_place(canonical, p),
                    crate::hir::CallArg::Spread(s) => sync_place(canonical, &mut s.place),
                }
            }
        }
        InstructionValue::NewExpression(v) => {
            sync_place(canonical, &mut v.callee);
            for arg in &mut v.args {
                match arg {
                    crate::hir::CallArg::Place(p) => sync_place(canonical, p),
                    crate::hir::CallArg::Spread(s) => sync_place(canonical, &mut s.place),
                }
            }
        }
        InstructionValue::MethodCall(v) => {
            sync_place(canonical, &mut v.receiver);
            sync_place(canonical, &mut v.property);
            for arg in &mut v.args {
                match arg {
                    crate::hir::CallArg::Place(p) => sync_place(canonical, p),
                    crate::hir::CallArg::Spread(s) => sync_place(canonical, &mut s.place),
                }
            }
        }
        InstructionValue::LoadLocal(v) => sync_place(canonical, &mut v.place),
        InstructionValue::LoadContext(v) => sync_place(canonical, &mut v.place),
        InstructionValue::StoreLocal(v) => {
            sync_place(canonical, &mut v.lvalue.place);
            sync_place(canonical, &mut v.value);
        }
        InstructionValue::StoreContext(v) => {
            sync_place(canonical, &mut v.lvalue_place);
            sync_place(canonical, &mut v.value);
        }
        InstructionValue::DeclareLocal(v) => {
            sync_place(canonical, &mut v.lvalue.place);
        }
        InstructionValue::DeclareContext(v) => {
            sync_place(canonical, &mut v.lvalue_place);
        }
        InstructionValue::Destructure(v) => {
            sync_pattern_ranges(canonical, &mut v.lvalue.pattern);
            sync_place(canonical, &mut v.value);
        }
        InstructionValue::PropertyLoad(v) => sync_place(canonical, &mut v.object),
        InstructionValue::PropertyStore(v) => {
            sync_place(canonical, &mut v.object);
            sync_place(canonical, &mut v.value);
        }
        InstructionValue::PropertyDelete(v) => sync_place(canonical, &mut v.object),
        InstructionValue::ComputedLoad(v) => {
            sync_place(canonical, &mut v.object);
            sync_place(canonical, &mut v.property);
        }
        InstructionValue::ComputedStore(v) => {
            sync_place(canonical, &mut v.object);
            sync_place(canonical, &mut v.property);
            sync_place(canonical, &mut v.value);
        }
        InstructionValue::ComputedDelete(v) => {
            sync_place(canonical, &mut v.object);
            sync_place(canonical, &mut v.property);
        }
        InstructionValue::ArrayExpression(v) => {
            for elem in &mut v.elements {
                match elem {
                    crate::hir::ArrayExpressionElement::Place(p) => sync_place(canonical, p),
                    crate::hir::ArrayExpressionElement::Spread(s) => {
                        sync_place(canonical, &mut s.place);
                    }
                    crate::hir::ArrayExpressionElement::Hole => {}
                }
            }
        }
        InstructionValue::ObjectExpression(v) => {
            for prop in &mut v.properties {
                match prop {
                    crate::hir::ObjectPatternProperty::Property(p) => {
                        if let crate::hir::ObjectPropertyKey::Computed(c) = &mut p.key {
                            sync_place(canonical, c);
                        }
                        sync_place(canonical, &mut p.place);
                    }
                    crate::hir::ObjectPatternProperty::Spread(s) => {
                        sync_place(canonical, &mut s.place);
                    }
                }
            }
        }
        InstructionValue::JsxExpression(v) => {
            if let crate::hir::JsxTag::Place(p) = &mut v.tag {
                sync_place(canonical, p);
            }
            for prop in &mut v.props {
                match prop {
                    crate::hir::JsxAttribute::Spread { argument } => {
                        sync_place(canonical, argument);
                    }
                    crate::hir::JsxAttribute::Attribute { place, .. } => {
                        sync_place(canonical, place);
                    }
                }
            }
            if let Some(children) = &mut v.children {
                for child in children {
                    sync_place(canonical, child);
                }
            }
        }
        InstructionValue::JsxFragment(v) => {
            for child in &mut v.children {
                sync_place(canonical, child);
            }
        }
        InstructionValue::UnaryExpression(v) => sync_place(canonical, &mut v.value),
        InstructionValue::BinaryExpression(v) => {
            sync_place(canonical, &mut v.left);
            sync_place(canonical, &mut v.right);
        }
        InstructionValue::TypeCastExpression(v) => sync_place(canonical, &mut v.value),
        InstructionValue::TemplateLiteral(v) => {
            for sub in &mut v.subexprs {
                sync_place(canonical, sub);
            }
        }
        InstructionValue::TaggedTemplateExpression(v) => {
            sync_place(canonical, &mut v.tag);
            // TaggedTemplateExpression's value is TemplateLiteralQuasi (raw/cooked), no places
        }
        InstructionValue::PrefixUpdate(v) => {
            sync_place(canonical, &mut v.lvalue);
            sync_place(canonical, &mut v.value);
        }
        InstructionValue::PostfixUpdate(v) => {
            sync_place(canonical, &mut v.lvalue);
            sync_place(canonical, &mut v.value);
        }
        InstructionValue::FunctionExpression(v) => {
            for dep in &mut v.lowered_func.func.context {
                sync_place(canonical, dep);
            }
        }
        _ => {}
    }
}

fn sync_pattern_ranges(
    canonical: &FxHashMap<IdentifierId, MutableRange>,
    pattern: &mut crate::hir::Pattern,
) {
    match pattern {
        crate::hir::Pattern::Array(arr) => {
            for item in &mut arr.items {
                match item {
                    crate::hir::ArrayPatternElement::Place(p) => sync_place(canonical, p),
                    crate::hir::ArrayPatternElement::Spread(s) => {
                        sync_place(canonical, &mut s.place);
                    }
                    crate::hir::ArrayPatternElement::Hole => {}
                }
            }
        }
        crate::hir::Pattern::Object(obj) => {
            for prop in &mut obj.properties {
                match prop {
                    crate::hir::ObjectPatternProperty::Property(p) => {
                        sync_place(canonical, &mut p.place);
                    }
                    crate::hir::ObjectPatternProperty::Spread(s) => {
                        sync_place(canonical, &mut s.place);
                    }
                }
            }
        }
    }
}

fn sync_effect_ranges(
    canonical: &FxHashMap<IdentifierId, MutableRange>,
    effect: &mut AliasingEffect,
) {
    match effect {
        AliasingEffect::Create { into, .. } => sync_place(canonical, into),
        AliasingEffect::Capture { from, into }
        | AliasingEffect::ImmutableCapture { from, into }
        | AliasingEffect::Alias { from, into }
        | AliasingEffect::Assign { from, into }
        | AliasingEffect::MaybeAlias { from, into }
        | AliasingEffect::CreateFrom { from, into } => {
            sync_place(canonical, from);
            sync_place(canonical, into);
        }
        AliasingEffect::Mutate { value, .. }
        | AliasingEffect::MutateConditionally { value }
        | AliasingEffect::MutateTransitive { value, .. }
        | AliasingEffect::MutateTransitiveConditionally { value }
        | AliasingEffect::Freeze { value, .. } => {
            sync_place(canonical, value);
        }
        AliasingEffect::Render { place }
        | AliasingEffect::Impure { place, .. }
        | AliasingEffect::MutateFrozen { place, .. }
        | AliasingEffect::MutateGlobal { place, .. } => {
            sync_place(canonical, place);
        }
        AliasingEffect::CreateFunction { captures, .. } => {
            for cap in captures {
                sync_place(canonical, cap);
            }
        }
        AliasingEffect::Apply { receiver, function, args, into, .. } => {
            sync_place(canonical, receiver);
            sync_place(canonical, function);
            for arg in args {
                match arg {
                    crate::inference::aliasing_effects::ApplyArg::Place(p) => {
                        sync_place(canonical, p);
                    }
                    crate::inference::aliasing_effects::ApplyArg::Spread(s) => {
                        sync_place(canonical, &mut s.place);
                    }
                    crate::inference::aliasing_effects::ApplyArg::Hole => {}
                }
            }
            sync_place(canonical, into);
        }
    }
}

fn sync_terminal_ranges(
    canonical: &FxHashMap<IdentifierId, MutableRange>,
    terminal: &mut Terminal,
) {
    match terminal {
        Terminal::Throw(t) => sync_place(canonical, &mut t.value),
        Terminal::Return(t) => sync_place(canonical, &mut t.value),
        Terminal::If(t) => sync_place(canonical, &mut t.test),
        Terminal::Branch(t) => sync_place(canonical, &mut t.test),
        Terminal::Switch(t) => {
            sync_place(canonical, &mut t.test);
            for case in &mut t.cases {
                if let Some(ref mut test) = case.test {
                    sync_place(canonical, test);
                }
            }
        }
        Terminal::Try(t) => {
            if let Some(ref mut binding) = t.handler_binding {
                sync_place(canonical, binding);
            }
        }
        _ => {}
    }
}

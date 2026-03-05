/// Collect hoistable property loads.
///
/// Port of `HIR/CollectHoistablePropertyLoads.ts` from the React Compiler.
///
/// Uses control flow graph analysis to determine which identifiers and
/// property paths can be assumed to be non-null objects on a per-block basis.
/// This enables hoisting property loads out of conditionals when it's safe
/// to do so.
///
/// The key data structure is `PropertyPathRegistry`, which tracks full
/// dependency paths like `props.value.x` and deduplicates them across blocks.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::compiler_error::SourceLocation;
use crate::hir::environment::get_hook_kind_for_type;
use crate::hir::types::PropertyLiteral;

use super::hir_types::{
    BlockId, CallArg, DependencyPathEntry, HIRFunction, Identifier, IdentifierId, InstructionId,
    InstructionValue, JsxAttribute, LoweredFunction, ReactFunctionType, ReactiveParam,
    ReactiveScopeDependency, ScopeId, Terminal,
};

// =============================================================================
// PropertyPathNode / PropertyPathRegistry
// =============================================================================

/// Index into the `PropertyPathRegistry.nodes` arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PropertyPathNode(u32);

/// A single node in the property-path tree.
#[derive(Debug, Clone)]
struct PathNodeData {
    full_path: ReactiveScopeDependency,
    has_optional: bool,
    /// Non-optional children keyed by property name.
    properties: FxHashMap<PropertyLiteral, PropertyPathNode>,
    /// Optional children keyed by property name.
    optional_properties: FxHashMap<PropertyLiteral, PropertyPathNode>,
}

/// Registry that deduplicates property-access paths (e.g. `a.b.c`).
///
/// Port of the TS `PropertyPathRegistry` class.
pub struct PropertyPathRegistry {
    nodes: Vec<PathNodeData>,
    roots: FxHashMap<IdentifierId, PropertyPathNode>,
}

impl PropertyPathRegistry {
    fn new() -> Self {
        Self { nodes: Vec::new(), roots: FxHashMap::default() }
    }

    fn alloc(&mut self, data: PathNodeData) -> PropertyPathNode {
        let idx = self.nodes.len();
        self.nodes.push(data);
        PropertyPathNode(u32::try_from(idx).expect("property path index overflow"))
    }

    pub fn get_full_path(&self, node: PropertyPathNode) -> &ReactiveScopeDependency {
        &self.nodes[node.0 as usize].full_path
    }

    fn get_has_optional(&self, node: PropertyPathNode) -> bool {
        self.nodes[node.0 as usize].has_optional
    }

    fn get_or_create_identifier(
        &mut self,
        identifier: &Identifier,
        reactive: bool,
        loc: SourceLocation,
    ) -> PropertyPathNode {
        if let Some(&existing) = self.roots.get(&identifier.id) {
            return existing;
        }
        let node = self.alloc(PathNodeData {
            full_path: ReactiveScopeDependency {
                identifier: identifier.clone(),
                reactive,
                path: vec![],
                loc,
            },
            has_optional: false,
            properties: FxHashMap::default(),
            optional_properties: FxHashMap::default(),
        });
        self.roots.insert(identifier.id, node);
        node
    }

    fn get_or_create_property_entry(
        &mut self,
        parent: PropertyPathNode,
        entry: &DependencyPathEntry,
    ) -> PropertyPathNode {
        // Check if child already exists
        let parent_data = &self.nodes[parent.0 as usize];
        let map =
            if entry.optional { &parent_data.optional_properties } else { &parent_data.properties };
        if let Some(&existing) = map.get(&entry.property) {
            return existing;
        }

        // Build the full path for the new child
        let parent_data = &self.nodes[parent.0 as usize];
        let mut new_path = parent_data.full_path.path.clone();
        new_path.push(entry.clone());
        let child_full_path = ReactiveScopeDependency {
            identifier: parent_data.full_path.identifier.clone(),
            reactive: parent_data.full_path.reactive,
            path: new_path,
            loc: entry.loc,
        };
        let child_has_optional = parent_data.has_optional || entry.optional;

        let child = self.alloc(PathNodeData {
            full_path: child_full_path,
            has_optional: child_has_optional,
            properties: FxHashMap::default(),
            optional_properties: FxHashMap::default(),
        });

        // Insert into parent's map
        let parent_data = &mut self.nodes[parent.0 as usize];
        if entry.optional {
            parent_data.optional_properties.insert(entry.property.clone(), child);
        } else {
            parent_data.properties.insert(entry.property.clone(), child);
        }
        child
    }

    fn get_or_create_property(&mut self, dep: &ReactiveScopeDependency) -> PropertyPathNode {
        let mut curr = self.get_or_create_identifier(&dep.identifier, dep.reactive, dep.loc);
        for entry in &dep.path {
            curr = self.get_or_create_property_entry(curr, entry);
        }
        curr
    }
}

// =============================================================================
// BlockInfo — per-block non-null information
// =============================================================================

/// Per-block information about assumed non-null objects.
///
/// Port of the TS `BlockInfo` type.
pub struct BlockInfo {
    pub assumed_non_null_objects: FxHashSet<PropertyPathNode>,
}

// =============================================================================
// Context for the collection pass
// =============================================================================

struct CollectContext<'a> {
    temporaries: &'a FxHashMap<IdentifierId, ReactiveScopeDependency>,
    known_immutable_identifiers: FxHashSet<IdentifierId>,
    hoistable_from_optionals: &'a FxHashMap<BlockId, ReactiveScopeDependency>,
    registry: PropertyPathRegistry,
    nested_fn_immutable_context: Option<FxHashSet<IdentifierId>>,
    assumed_invoked_fns: FxHashSet<LoweredFunctionKey>,
}

/// Since `LoweredFunction` doesn't impl Hash/Eq, we use a thin wrapper that
/// keys on the pointer identity of the `Box<HIRFunction>` inside.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct LoweredFunctionKey(*const HIRFunction);

fn lowered_fn_key(lf: &LoweredFunction) -> LoweredFunctionKey {
    LoweredFunctionKey(&raw const *lf.func)
}

// =============================================================================
// Public API
// =============================================================================

/// Result of hoistable property load analysis.
pub struct HoistablePropertyLoads {
    pub block_infos: FxHashMap<BlockId, BlockInfo>,
    pub registry: PropertyPathRegistry,
}

/// Collect hoistable property loads for the given function.
///
/// Port of the TS `collectHoistablePropertyLoads`.
// Cannot generalize over BuildHasher: .clippy.toml disallows std::collections::HashMap
#[expect(clippy::implicit_hasher)]
pub fn collect_hoistable_property_loads(
    func: &HIRFunction,
    temporaries: &FxHashMap<IdentifierId, ReactiveScopeDependency>,
    hoistable_from_optionals: &FxHashMap<BlockId, ReactiveScopeDependency>,
) -> HoistablePropertyLoads {
    let registry = PropertyPathRegistry::new();

    let mut known_immutable_identifiers = FxHashSet::default();
    // Add all function params as known immutable identifiers.
    //
    // The TS reference only does this for Component/Hook types, but in the Rust HIR
    // params of *any* function type can acquire a wide mutable range due to how
    // InferMutationAliasingRanges works. Function parameters are semantically
    // read-only within the function body (they may be reassigned in JS, but they
    // are not co-mutated with the values they're passed), so it's safe to treat
    // them as always-immutable for the purpose of hoistable property load analysis.
    // This is the same fix the TS reference applied for Component/Hook:
    // "Due to current limitations of mutable range inference, there are edge cases
    // in which we infer known-immutable values (e.g. props or hook params) to have
    // a mutable range and scope."
    for p in &func.params {
        if let ReactiveParam::Place(place) = p {
            known_immutable_identifiers.insert(place.identifier.id);
        }
    }

    let assumed_invoked_fns = get_assumed_invoked_functions(func);

    let mut context = CollectContext {
        temporaries,
        known_immutable_identifiers,
        hoistable_from_optionals,
        registry,
        nested_fn_immutable_context: None,
        assumed_invoked_fns,
    };

    let block_infos = collect_hoistable_property_loads_impl(func, &mut context);
    HoistablePropertyLoads { block_infos, registry: context.registry }
}

/// Re-key a block-indexed map by scope ID, resolving PropertyPathNodes via registry.
///
/// Port of the TS `keyByScopeId`.
pub fn key_by_scope_id_with_registry(
    func: &HIRFunction,
    hoistable: &HoistablePropertyLoads,
) -> FxHashMap<ScopeId, Vec<ReactiveScopeDependency>> {
    let mut keyed = FxHashMap::default();
    for (_block_id, block) in &func.body.blocks {
        if let Terminal::Scope(t) = &block.terminal
            && let Some(info) = hoistable.block_infos.get(&t.block)
        {
            let paths: Vec<ReactiveScopeDependency> = info
                .assumed_non_null_objects
                .iter()
                .map(|&node| hoistable.registry.get_full_path(node).clone())
                .collect();
            keyed.insert(t.scope.id, paths);
        }
    }
    keyed
}

// =============================================================================
// Implementation
// =============================================================================

fn collect_hoistable_property_loads_impl(
    func: &HIRFunction,
    context: &mut CollectContext<'_>,
) -> FxHashMap<BlockId, BlockInfo> {
    let mut nodes = collect_non_nulls_in_blocks(func, context);
    propagate_non_null(func, &mut nodes, &mut context.registry);
    nodes
}

// =============================================================================
// getMaybeNonNullInInstruction
// =============================================================================

fn get_maybe_non_null_in_instruction(
    value: &InstructionValue,
    context: &mut CollectContext<'_>,
) -> Option<PropertyPathNode> {
    let path: Option<ReactiveScopeDependency> = match value {
        InstructionValue::PropertyLoad(v) => {
            let resolved = context.temporaries.get(&v.object.identifier.id);
            Some(match resolved {
                Some(dep) => dep.clone(),
                None => ReactiveScopeDependency {
                    identifier: v.object.identifier.clone(),
                    reactive: v.object.reactive,
                    path: vec![],
                    loc: v.loc,
                },
            })
        }
        InstructionValue::Destructure(v) => {
            context.temporaries.get(&v.value.identifier.id).cloned()
        }
        InstructionValue::ComputedLoad(v) => {
            context.temporaries.get(&v.object.identifier.id).cloned()
        }
        _ => None,
    };

    path.map(|p| context.registry.get_or_create_property(&p))
}

// =============================================================================
// isImmutableAtInstr
// =============================================================================

fn is_immutable_at_instr(
    identifier: &Identifier,
    instr: InstructionId,
    context: &CollectContext<'_>,
) -> bool {
    if let Some(nested_ctx) = &context.nested_fn_immutable_context {
        return nested_ctx.contains(&identifier.id);
    }

    let mutable_at_instr = identifier.mutable_range.end
        > InstructionId(identifier.mutable_range.start.0 + 1)
        && identifier.scope.is_some()
        && {
            let scope = identifier.scope.as_ref().unwrap();
            // inRange: id >= range.start && id < range.end
            instr >= scope.range.start && instr < scope.range.end
        };

    !mutable_at_instr || context.known_immutable_identifiers.contains(&identifier.id)
}

// =============================================================================
// collectNonNullsInBlocks
// =============================================================================

fn collect_non_nulls_in_blocks(
    func: &HIRFunction,
    context: &mut CollectContext<'_>,
) -> FxHashMap<BlockId, BlockInfo> {
    // Known non-null objects (e.g. component props)
    let mut known_non_null_identifiers = FxHashSet::default();
    if func.fn_type == ReactFunctionType::Component
        && !func.params.is_empty()
        && matches!(&func.params[0], ReactiveParam::Place(_))
        && let ReactiveParam::Place(place) = &func.params[0]
    {
        let node = context.registry.get_or_create_identifier(&place.identifier, true, place.loc);
        known_non_null_identifiers.insert(node);
    }

    let mut nodes = FxHashMap::default();

    // Iterate blocks in order by collecting block IDs first.
    // IMPORTANT: We must NOT clone the blocks, because LoweredFunctionKey uses
    // raw pointer identity for Box<HIRFunction>. Cloning would create new heap
    // allocations, breaking the pointer-based lookup in assumed_invoked_fns.
    let block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();

    for block_id in &block_ids {
        let block = &func.body.blocks[block_id];
        let mut assumed_non_null_objects: FxHashSet<PropertyPathNode> =
            known_non_null_identifiers.clone();

        // Add hoistable from optional chains
        if let Some(dep) = context.hoistable_from_optionals.get(block_id) {
            let node = context.registry.get_or_create_property(dep);
            assumed_non_null_objects.insert(node);
        }

        for instr in &block.instructions {
            let maybe_non_null = get_maybe_non_null_in_instruction(&instr.value, context);
            if let Some(node) = maybe_non_null {
                let full_path = context.registry.get_full_path(node);
                let ident = full_path.identifier.clone();
                if is_immutable_at_instr(&ident, instr.id, context) {
                    assumed_non_null_objects.insert(node);
                }
            }

            if let InstructionValue::FunctionExpression(fe) = &instr.value {
                let inner_fn_key = lowered_fn_key(&fe.lowered_func);
                if context.assumed_invoked_fns.contains(&inner_fn_key) {
                    // Build nested context for inner function.
                    //
                    // Match the TS reference (CollectHoistablePropertyLoads.ts):
                    // include all context variables that are immutable at the
                    // instruction point, including function parameters.
                    //
                    // Previously, we excluded outer function parameters to avoid
                    // false-positive "Differences in ref.current access" validation
                    // errors. That workaround is no longer needed because
                    // infer_types now correctly unifies ref-like identifiers (e.g.
                    // `maybeRef`) with BuiltInUseRefId, and visit_dependency
                    // truncates `.current` accesses on such identifiers.
                    let nested_fn_immutable_context =
                        context.nested_fn_immutable_context.clone().unwrap_or_else(|| {
                            fe.lowered_func
                                .func
                                .context
                                .iter()
                                .filter(|place| {
                                    is_immutable_at_instr(&place.identifier, instr.id, context)
                                })
                                .map(|place| place.identifier.id)
                                .collect()
                        });

                    let inner_assumed = get_assumed_invoked_functions(&fe.lowered_func.func);

                    let mut inner_context = CollectContext {
                        temporaries: context.temporaries,
                        known_immutable_identifiers: context.known_immutable_identifiers.clone(),
                        hoistable_from_optionals: context.hoistable_from_optionals,
                        registry: PropertyPathRegistry::new(),
                        nested_fn_immutable_context: Some(nested_fn_immutable_context),
                        assumed_invoked_fns: inner_assumed,
                    };
                    // Transfer the registry to inner context temporarily
                    std::mem::swap(&mut context.registry, &mut inner_context.registry);

                    let inner_hoistable_map = collect_hoistable_property_loads_impl(
                        &fe.lowered_func.func,
                        &mut inner_context,
                    );

                    // Transfer registry back
                    std::mem::swap(&mut context.registry, &mut inner_context.registry);

                    if let Some(inner_info) =
                        inner_hoistable_map.get(&fe.lowered_func.func.body.entry)
                    {
                        for &entry in &inner_info.assumed_non_null_objects {
                            assumed_non_null_objects.insert(entry);
                        }
                    }
                }
            } else if func.env.config.enable_preserve_existing_memoization_guarantees
                && let InstructionValue::StartMemoize(sm) = &instr.value
                && let Some(deps) = &sm.deps
            {
                for dep in deps {
                    if let super::hir_types::ManualMemoDependencyRoot::NamedLocal {
                        value, ..
                    } = &dep.root
                    {
                        if !is_immutable_at_instr(&value.identifier, instr.id, context) {
                            continue;
                        }
                        for i in 0..dep.path.len() {
                            let path_entry = &dep.path[i];
                            if path_entry.optional {
                                break;
                            }
                            let sub_dep = ReactiveScopeDependency {
                                identifier: value.identifier.clone(),
                                path: dep.path[..i].to_vec(),
                                reactive: value.reactive,
                                loc: dep.loc,
                            };
                            let dep_node = context.registry.get_or_create_property(&sub_dep);
                            assumed_non_null_objects.insert(dep_node);
                        }
                    }
                }
            }
        }

        nodes.insert(*block_id, BlockInfo { assumed_non_null_objects });
    }

    nodes
}

// =============================================================================
// propagateNonNull — bidirectional fixed-point propagation
// =============================================================================

fn propagate_non_null(
    func: &HIRFunction,
    nodes: &mut FxHashMap<BlockId, BlockInfo>,
    registry: &mut PropertyPathRegistry,
) {
    // Build successor map
    let mut block_successors: FxHashMap<BlockId, FxHashSet<BlockId>> = FxHashMap::default();

    for (&block_id, block) in &func.body.blocks {
        for &pred in &block.preds {
            block_successors.entry(pred).or_default().insert(block_id);
        }
    }

    let block_ids_forward: Vec<BlockId> = func.body.blocks.keys().copied().collect();
    let mut block_ids_backward: Vec<BlockId> = block_ids_forward.clone();
    block_ids_backward.reverse();

    let mut changed = true;
    let mut iteration = 0u32;
    while changed {
        assert!(
            iteration < 100,
            "[CollectHoistablePropertyLoads] fixed point iteration did not terminate after 100 loops"
        );
        iteration += 1;
        changed = false;

        // Forward pass
        let mut traversal_state: FxHashMap<BlockId, TraversalStatus> = FxHashMap::default();
        for &block_id in &block_ids_forward {
            let fwd_changed = recursively_propagate_non_null(
                block_id,
                Direction::Forward,
                &mut traversal_state,
                nodes,
                registry,
                func,
                &block_successors,
            );
            changed |= fwd_changed;
        }

        // Backward pass
        traversal_state.clear();
        for &block_id in &block_ids_backward {
            let bwd_changed = recursively_propagate_non_null(
                block_id,
                Direction::Backward,
                &mut traversal_state,
                nodes,
                registry,
                func,
                &block_successors,
            );
            changed |= bwd_changed;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TraversalStatus {
    Active,
    Done,
}

#[derive(Clone, Copy)]
enum Direction {
    Forward,
    Backward,
}

fn recursively_propagate_non_null(
    node_id: BlockId,
    direction: Direction,
    traversal_state: &mut FxHashMap<BlockId, TraversalStatus>,
    nodes: &mut FxHashMap<BlockId, BlockInfo>,
    registry: &mut PropertyPathRegistry,
    func: &HIRFunction,
    block_successors: &FxHashMap<BlockId, FxHashSet<BlockId>>,
) -> bool {
    if traversal_state.contains_key(&node_id) {
        return false;
    }
    traversal_state.insert(node_id, TraversalStatus::Active);

    let block = func.body.blocks.get(&node_id).expect("Bad node in propagateNonNull");

    let neighbors: Vec<BlockId> = match direction {
        Direction::Backward => {
            block_successors.get(&node_id).map_or_else(Vec::new, |s| s.iter().copied().collect())
        }
        Direction::Forward => block.preds.iter().copied().collect(),
    };

    let mut changed = false;
    for &pred in &neighbors {
        if !traversal_state.contains_key(&pred) {
            let neighbor_changed = recursively_propagate_non_null(
                pred,
                direction,
                traversal_state,
                nodes,
                registry,
                func,
                block_successors,
            );
            changed |= neighbor_changed;
        }
    }

    // Intersect non-null sets from done neighbors
    let done_neighbor_sets: Vec<FxHashSet<PropertyPathNode>> = neighbors
        .iter()
        .filter(|n| traversal_state.get(n) == Some(&TraversalStatus::Done))
        .filter_map(|n| nodes.get(n))
        .map(|info| info.assumed_non_null_objects.clone())
        .collect();

    let neighbor_accesses = set_intersect(&done_neighbor_sets);

    let prev_objects =
        nodes.get(&node_id).map(|info| info.assumed_non_null_objects.clone()).unwrap_or_default();

    let mut merged_objects = set_union(&prev_objects, &neighbor_accesses);
    reduce_maybe_optional_chains(&mut merged_objects, registry);

    let objects_changed = !set_equal(&prev_objects, &merged_objects);

    nodes
        .entry(node_id)
        .or_insert_with(|| BlockInfo { assumed_non_null_objects: FxHashSet::default() })
        .assumed_non_null_objects = merged_objects;

    traversal_state.insert(node_id, TraversalStatus::Done);
    changed |= objects_changed;
    changed
}

// =============================================================================
// reduceMaybeOptionalChains
// =============================================================================

fn reduce_maybe_optional_chains(
    nodes_set: &mut FxHashSet<PropertyPathNode>,
    registry: &mut PropertyPathRegistry,
) {
    let optional_chain_nodes: FxHashSet<PropertyPathNode> =
        nodes_set.iter().copied().filter(|&n| registry.get_has_optional(n)).collect();

    if optional_chain_nodes.is_empty() {
        return;
    }

    let mut optional_nodes = optional_chain_nodes;
    let mut outer_changed = true;
    while outer_changed {
        outer_changed = false;

        let current_optionals: Vec<PropertyPathNode> = optional_nodes.iter().copied().collect();
        for original in current_optionals {
            let full_path = registry.get_full_path(original).clone();
            let identifier = full_path.identifier.clone();
            let reactive = full_path.reactive;
            let orig_loc = full_path.loc;
            let orig_path = full_path.path.clone();

            let mut curr_node = registry.get_or_create_identifier(&identifier, reactive, orig_loc);

            for entry in &orig_path {
                // If the base is known to be non-null, replace with a non-optional load
                let next_entry = if entry.optional && nodes_set.contains(&curr_node) {
                    DependencyPathEntry {
                        property: entry.property.clone(),
                        optional: false,
                        loc: entry.loc,
                    }
                } else {
                    entry.clone()
                };
                curr_node = registry.get_or_create_property_entry(curr_node, &next_entry);
            }

            if curr_node != original {
                outer_changed = true;
                optional_nodes.remove(&original);
                optional_nodes.insert(curr_node);
                nodes_set.remove(&original);
                nodes_set.insert(curr_node);
            }
        }
    }
}

// =============================================================================
// getAssumedInvokedFunctions
// =============================================================================

struct FnTemporary {
    lowered_fn: LoweredFunctionKey,
    may_invoke: FxHashSet<LoweredFunctionKey>,
}

fn get_assumed_invoked_functions(func: &HIRFunction) -> FxHashSet<LoweredFunctionKey> {
    let mut temporaries: FxHashMap<IdentifierId, FnTemporary> = FxHashMap::default();
    get_assumed_invoked_functions_inner(func, &mut temporaries)
}

fn get_assumed_invoked_functions_inner(
    func: &HIRFunction,
    temporaries: &mut FxHashMap<IdentifierId, FnTemporary>,
) -> FxHashSet<LoweredFunctionKey> {
    let mut hoistable_functions = FxHashSet::default();

    // Step 1: Collect identifier -> function expression mappings
    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            match &instr.value {
                InstructionValue::FunctionExpression(fe) => {
                    let key = lowered_fn_key(&fe.lowered_func);
                    temporaries.insert(
                        instr.lvalue.identifier.id,
                        FnTemporary { lowered_fn: key, may_invoke: FxHashSet::default() },
                    );
                }
                InstructionValue::StoreLocal(sl) => {
                    let lvalue_id = sl.lvalue.place.identifier.id;
                    if let Some(existing) = temporaries.get(&sl.value.identifier.id) {
                        let cloned = FnTemporary {
                            lowered_fn: existing.lowered_fn,
                            may_invoke: existing.may_invoke.clone(),
                        };
                        temporaries.insert(lvalue_id, cloned);
                    }
                }
                InstructionValue::LoadLocal(ll) => {
                    if let Some(existing) = temporaries.get(&ll.place.identifier.id) {
                        let cloned = FnTemporary {
                            lowered_fn: existing.lowered_fn,
                            may_invoke: existing.may_invoke.clone(),
                        };
                        temporaries.insert(instr.lvalue.identifier.id, cloned);
                    }
                }
                InstructionValue::LoadContext(lc) => {
                    // In the Rust HIR, captured variables are loaded via LoadContext rather
                    // than LoadLocal (as in the TS reference). We need to propagate the
                    // temporaries mapping through LoadContext so that inner function bodies
                    // that call outer-declared lambdas (e.g. `log = () => { logA(); }` where
                    // `logA` is captured via context) can detect those calls as assumed-invoked.
                    if let Some(existing) = temporaries.get(&lc.place.identifier.id) {
                        let cloned = FnTemporary {
                            lowered_fn: existing.lowered_fn,
                            may_invoke: existing.may_invoke.clone(),
                        };
                        temporaries.insert(instr.lvalue.identifier.id, cloned);
                    }
                }
                _ => {}
            }
        }
    }

    // Step 2: Forward pass — analyze assumed function calls
    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            match &instr.value {
                InstructionValue::CallExpression(call) => {
                    let callee_id = call.callee.identifier.id;
                    let maybe_hook =
                        get_hook_kind_for_type(&func.env, &call.callee.identifier.type_);

                    if let Some(existing) = temporaries.get(&callee_id) {
                        // Direct calls
                        hoistable_functions.insert(existing.lowered_fn);
                    } else if maybe_hook.is_some() {
                        // Assume arguments to all hooks are safe to invoke
                        for arg in &call.args {
                            if let CallArg::Place(place) = arg
                                && let Some(existing) = temporaries.get(&place.identifier.id)
                            {
                                hoistable_functions.insert(existing.lowered_fn);
                            }
                        }
                    }
                }
                InstructionValue::JsxExpression(jsx) => {
                    // Assume JSX attributes and children are safe to invoke
                    for attr in &jsx.props {
                        match attr {
                            JsxAttribute::Spread { .. } => {}
                            JsxAttribute::Attribute { place, .. } => {
                                if let Some(existing) = temporaries.get(&place.identifier.id) {
                                    hoistable_functions.insert(existing.lowered_fn);
                                }
                            }
                        }
                    }
                    if let Some(children) = &jsx.children {
                        for child in children {
                            if let Some(existing) = temporaries.get(&child.identifier.id) {
                                hoistable_functions.insert(existing.lowered_fn);
                            }
                        }
                    }
                }
                InstructionValue::FunctionExpression(fe) => {
                    // Recursively traverse into other function expressions
                    let lambdas_called =
                        get_assumed_invoked_functions_inner(&fe.lowered_func.func, temporaries);
                    if let Some(existing) = temporaries.get_mut(&instr.lvalue.identifier.id) {
                        for called in lambdas_called {
                            existing.may_invoke.insert(called);
                        }
                    }
                }
                _ => {}
            }
        }

        // Return terminal: assume directly returned functions are safe to call
        if let Terminal::Return(ret) = &block.terminal
            && let Some(existing) = temporaries.get(&ret.value.identifier.id)
        {
            hoistable_functions.insert(existing.lowered_fn);
        }
    }

    // Propagate: if a function is hoistable, all functions it may invoke are too
    let entries: Vec<(LoweredFunctionKey, FxHashSet<LoweredFunctionKey>)> =
        temporaries.values().map(|t| (t.lowered_fn, t.may_invoke.clone())).collect();
    for (fn_key, may_invoke) in &entries {
        if hoistable_functions.contains(fn_key) {
            for called in may_invoke {
                hoistable_functions.insert(*called);
            }
        }
    }

    hoistable_functions
}

// =============================================================================
// Set utilities
// =============================================================================

fn set_intersect<T: Eq + std::hash::Hash + Copy>(sets: &[FxHashSet<T>]) -> FxHashSet<T> {
    if sets.is_empty() {
        return FxHashSet::default();
    }
    let mut result = sets[0].clone();
    for s in &sets[1..] {
        result.retain(|item| s.contains(item));
    }
    result
}

fn set_union<T: Eq + std::hash::Hash + Copy>(a: &FxHashSet<T>, b: &FxHashSet<T>) -> FxHashSet<T> {
    let mut result = a.clone();
    for item in b {
        result.insert(*item);
    }
    result
}

fn set_equal<T: Eq + std::hash::Hash>(a: &FxHashSet<T>, b: &FxHashSet<T>) -> bool {
    a.len() == b.len() && a.iter().all(|item| b.contains(item))
}

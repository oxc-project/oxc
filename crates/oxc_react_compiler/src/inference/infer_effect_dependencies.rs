/// Infer reactive dependencies captured by `useEffect` lambdas and emit a
/// matching deps array argument.
///
/// Port of `Inference/InferEffectDependencies.ts` (~710 LoC) from the React
/// Compiler reference. Given `useEffect(fn, AUTODEPS)` (or any configured
/// effect wrapper), the pass:
///
/// 1. Detects the `AUTODEPS` operand at the configured argument index by
///    checking that the operand's identifier carries the
///    `BUILT_IN_AUTODEPS_ID` shape.
/// 2. Walks the inner function to compute the dependencies it captures from
///    the outer scope, filtering refs / setStates / effect-event functions
///    that are known to be non-reactive.
/// 3. Emits LoadLocal + chain-of-PropertyLoad instructions to materialise
///    each dependency as a `Place`, builds an ArrayExpression from those
///    places, and replaces the `AUTODEPS` operand with the new array place.
///
/// # Scope reuse
///
/// When the function expression has been assigned its own reactive scope by
/// `infer_reactive_scope_variables`, the scope's already-computed
/// `dependencies` set is reused — this avoids duplicating
/// `propagate_scope_dependencies_hir`'s analysis. The reused set already
/// reflects truncation, optional-chain handling and minimal-dep derivation
/// from the main pass.
///
/// When the lambda has no scope (e.g. the entire function is being retried
/// with `no_inferred_memo`), the dependencies are recomputed here by
/// running a stripped-down variant of `propagate_scope_dependencies_hir`
/// over the inner function only.
///
/// # Limitations vs. TS reference
///
/// - The Rust port emits dependency instructions directly into the
///   surrounding block via a "splice" list, instead of building a value
///   sub-block with `HIRBuilder`. This keeps the pass entirely linear
///   without introducing new blocks, which is sufficient for non-optional
///   dependency chains and works for the typical `useEffect(fn, AUTODEPS)`
///   shape.
/// - Optional-chain dependency rendering (e.g. `a?.b.c`) preserves the
///   `optional` bit on the `DependencyPath` so downstream codegen renders
///   `a?.b.c` in the deps array. The runtime evaluation uses regular
///   PropertyLoad chains, matching what the TS pass would produce after
///   `deadCodeElimination` for ergonomic property reads.
/// - Bailout-retry is split across two layers rather than being implemented
///   inside this pass:
///
///   - This pass always emits the inferred deps array when the AUTODEPS
///     sentinel is detected (it never holds back work based on later
///     validation).
///   - When the resulting `Client` compilation fails, the test runner
///     (`tests/fixtures.rs`) re-attempts with `CompilerOutputMode::ClientNoMemo`.
///     In NoMemo mode several memoization-only pipeline phases (mutation-
///     aliasing inference's error propagation, reactive-scope dependency
///     recording, and the memo-only validators) are gated off in
///     `entrypoint/pipeline.rs` via `Environment::enable_memoization()`,
///     matching upstream `isInferredMemoEnabled` in `Pipeline.ts`.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::compiler_error::GENERATED_SOURCE;
use crate::hir::derive_minimal_dependencies_hir::DependencyTree;
use crate::hir::object_shape::{
    BUILT_IN_AUTODEPS_ID, BUILT_IN_EFFECT_EVENT_ID, BUILT_IN_SET_STATE_ID,
    BUILT_IN_USE_EFFECT_EVENT_ID, BUILT_IN_USE_REF_ID,
};
use crate::hir::types::{ObjectType, PropertyLiteral, Type};
use crate::hir::visitors::{each_instruction_operand, each_terminal_operand};
use crate::hir::{
    ArrayExpression, ArrayExpressionElement, BasicBlock, BlockId, CallArg, DependencyPathEntry,
    Effect, HIRFunction, Identifier, IdentifierId, Instruction, InstructionId, InstructionValue,
    LoadLocal, NonLocalBinding, Place, PropertyLoad, ReactiveScopeDependency, ScopeId, Terminal,
};

const DEFAULT_EXPORT: &str = "default";

/// A record of one `useEffect(fn, AUTODEPS)` rewrite to apply.
///
/// `instr_index` is the position of the call instruction in its containing
/// block at the time the rewrite was recorded. Rewrites are applied in
/// per-block sorted order so we only walk each block's original
/// instructions once.
struct Rewrite {
    block_id: BlockId,
    instr_index: usize,
    autodeps_arg_index: usize,
    deps: Vec<ReactiveScopeDependency>,
}

/// Run the `infer_effect_dependencies` pass on a HIR function.
///
/// Mutates the function in place: each `useEffect(fn, AUTODEPS)` call is
/// rewritten to `useEffect(fn, [d1, d2, ...])` and the supporting
/// LoadLocal / PropertyLoad instructions are inserted before the call.
///
/// Has no effect when `func.env.config().infer_effect_dependencies` is `None`.
pub fn infer_effect_dependencies(func: &mut HIRFunction) {
    let Some(targets) = func.env.config().infer_effect_dependencies.clone() else {
        return;
    };

    // module -> {import_specifier -> autodeps_index}
    let mut autodep_fn_configs: FxHashMap<String, FxHashMap<String, u32>> = FxHashMap::default();
    for entry in &targets {
        autodep_fn_configs
            .entry(entry.function.source.clone())
            .or_default()
            .insert(entry.function.import_specifier_name.clone(), entry.autodeps_index);
    }

    // Identifier IDs of `useEffect`-like callees (mapped to the expected
    // autodeps argument index).
    let mut autodep_fn_loads: FxHashMap<IdentifierId, u32> = FxHashMap::default();
    // Identifier IDs of namespace imports that contain autodeps-eligible
    // members (mapped to the configured `{import_specifier -> autodeps_index}`).
    let mut autodep_module_loads: FxHashMap<IdentifierId, FxHashMap<String, u32>> =
        FxHashMap::default();
    // Identifier IDs of LoadGlobal results (used to short-circuit dep
    // inference for global callees passed to useEffect).
    let mut load_globals: FxHashSet<IdentifierId> = FxHashSet::default();
    // Identifier IDs of FunctionExpression lvalues, used to look up the
    // inner function from `useEffect(fn, AUTODEPS)`'s first argument.
    let mut fn_expressions: FxHashMap<IdentifierId, (BlockId, usize)> = FxHashMap::default();
    // Scope id -> set of deps. Populated for scopes that contain ONLY a
    // function expression (the typical lambda-in-its-own-scope case).
    let mut scope_infos: FxHashMap<ScopeId, Vec<ReactiveScopeDependency>> = FxHashMap::default();

    // Identifier IDs that hold reactive values somewhere in the function.
    let reactive_ids = infer_reactive_identifiers(func);

    // ---- Phase 1: collect sidemaps + record AUTODEPS rewrite sites ----

    // We split rewriting into a separate phase so that we don't need to
    // mutably borrow `func.body.blocks` while reading from it.
    let mut rewrites: Vec<Rewrite> = Vec::new();

    // Collect ordered block IDs upfront to deterministically iterate.
    let block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();

    for &block_id in &block_ids {
        let Some(block) = func.body.blocks.get(&block_id) else { continue };

        // Record scope info when the scope contains exactly one instruction
        // (a function expression) and its body block is a goto to the
        // fallthrough. This mirrors the TS reference's `scopeInfos.set`
        // block at the top of the loop.
        if let Terminal::Scope(scope_term) = &block.terminal
            && let Some(scope_block) = func.body.blocks.get(&scope_term.block)
            && scope_block.instructions.len() == 1
            && matches!(&scope_block.terminal, Terminal::Goto(g) if g.block == scope_term.fallthrough)
        {
            let deps: Vec<ReactiveScopeDependency> =
                scope_term.scope.dependencies.iter().cloned().collect();
            scope_infos.insert(scope_term.scope.id, deps);
        }

        for (instr_idx, instr) in block.instructions.iter().enumerate() {
            match &instr.value {
                InstructionValue::FunctionExpression(_) => {
                    fn_expressions.insert(instr.lvalue.identifier.id, (block_id, instr_idx));
                }
                InstructionValue::PropertyLoad(v) => {
                    // `ns.useEffect` style: track namespace member loads
                    // whose property is a configured autodeps target.
                    if let PropertyLiteral::String(prop_name) = &v.property
                        && let Some(module_targets) =
                            autodep_module_loads.get(&v.object.identifier.id)
                        && let Some(&expected_idx) = module_targets.get(prop_name)
                    {
                        autodep_fn_loads.insert(instr.lvalue.identifier.id, expected_idx);
                    }
                }
                InstructionValue::LoadGlobal(v) => {
                    load_globals.insert(instr.lvalue.identifier.id);
                    match &v.binding {
                        NonLocalBinding::ImportNamespace { module, .. } => {
                            if let Some(targets) = autodep_fn_configs.get(module) {
                                autodep_module_loads
                                    .insert(instr.lvalue.identifier.id, targets.clone());
                            }
                        }
                        NonLocalBinding::ImportSpecifier { module, imported, .. } => {
                            if let Some(targets) = autodep_fn_configs.get(module)
                                && let Some(&expected_idx) = targets.get(imported)
                            {
                                autodep_fn_loads.insert(instr.lvalue.identifier.id, expected_idx);
                            }
                        }
                        NonLocalBinding::ImportDefault { module, .. } => {
                            if let Some(targets) = autodep_fn_configs.get(module)
                                && let Some(&expected_idx) = targets.get(DEFAULT_EXPORT)
                            {
                                autodep_fn_loads.insert(instr.lvalue.identifier.id, expected_idx);
                            }
                        }
                        _ => {}
                    }
                }
                InstructionValue::CallExpression(_) | InstructionValue::MethodCall(_) => {
                    let (callee_id, args) = match &instr.value {
                        InstructionValue::CallExpression(c) => {
                            (c.callee.identifier.id, c.args.as_slice())
                        }
                        InstructionValue::MethodCall(m) => {
                            (m.property.identifier.id, m.args.as_slice())
                        }
                        _ => unreachable!(),
                    };

                    let expected_idx = match autodep_fn_loads.get(&callee_id) {
                        Some(&i) => i as usize,
                        None => continue,
                    };

                    // Find which arg is the AUTODEPS sentinel.
                    let autodeps_arg_index = args.iter().position(|arg| {
                        matches!(arg,
                            CallArg::Place(p) if is_autodeps_type(&p.identifier)
                        )
                    });

                    if !args.is_empty()
                        && autodeps_arg_index == Some(expected_idx)
                        && let Some(first_arg) = args.first()
                        && let CallArg::Place(first_place) = first_arg
                    {
                        let first_arg_id = first_place.identifier.id;

                        if let Some(&(_fn_block_id, _fn_idx)) = fn_expressions.get(&first_arg_id) {
                            // Locate the function expression instruction and
                            // resolve its deps.
                            //
                            // 1) Try to reuse a precomputed scope's deps.
                            // 2) Otherwise, recompute via inner-fn analysis.
                            let scope_id = first_place.identifier.scope.as_deref().map(|s| s.id);
                            let scope_deps = scope_id.and_then(|id| scope_infos.get(&id).cloned());

                            let raw_deps: Vec<ReactiveScopeDependency> = if let Some(d) = scope_deps
                            {
                                d
                            } else {
                                // Re-fetch the function expression instruction
                                // and recompute deps.
                                find_fn_expression(func, first_arg_id)
                                    .map(infer_minimal_dependencies)
                                    .unwrap_or_default()
                            };

                            // Reorder deps to match the order in which their
                            // root identifiers are first referenced inside the
                            // inner lambda. This matches the TS reference
                            // (where `scope.dependencies` is a JS Set whose
                            // iteration order is insertion order, and items
                            // are inserted when first encountered).
                            let ordered_deps =
                                order_deps_by_first_use(&raw_deps, func, first_arg_id);

                            // Filter + truncate deps to the form we emit.
                            let mut deps: Vec<ReactiveScopeDependency> = Vec::new();
                            for maybe_dep in ordered_deps {
                                if dep_is_filtered_out(&maybe_dep, &reactive_ids) {
                                    continue;
                                }
                                let truncated = truncate_dep_at_current(maybe_dep);
                                deps.push(truncated);
                            }

                            rewrites.push(Rewrite {
                                block_id,
                                instr_index: instr_idx,
                                autodeps_arg_index: expected_idx,
                                deps,
                            });
                        } else if load_globals.contains(&first_arg_id) {
                            // Global functions have no reactive deps — emit
                            // an empty deps array.
                            rewrites.push(Rewrite {
                                block_id,
                                instr_index: instr_idx,
                                autodeps_arg_index: expected_idx,
                                deps: Vec::new(),
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if rewrites.is_empty() {
        return;
    }

    // ---- Phase 2: pre-allocate fresh identifier IDs for every synthetic
    // place we will emit. We do this upfront so that the actual block
    // mutation (which borrows `func.body.blocks` mutably) does not need
    // simultaneous access to `func.env`.

    let mut rewrites_by_block: FxHashMap<BlockId, Vec<PreparedRewrite>> = FxHashMap::default();
    let mut any_emitted = false;
    for rw in rewrites {
        // For each dep: 1 LoadLocal + path.len() PropertyLoads. Plus 1
        // ArrayExpression lvalue per rewrite.
        let mut dep_chains: Vec<DepChain> = Vec::with_capacity(rw.deps.len());
        for dep in &rw.deps {
            let root_id = func.env.next_identifier_id();
            let mut chain_ids = Vec::with_capacity(dep.path.len());
            for _ in &dep.path {
                chain_ids.push(func.env.next_identifier_id());
            }
            dep_chains.push(DepChain { root_id, chain_ids });
        }
        let array_id = func.env.next_identifier_id();
        rewrites_by_block.entry(rw.block_id).or_default().push(PreparedRewrite {
            instr_index: rw.instr_index,
            autodeps_arg_index: rw.autodeps_arg_index,
            deps: rw.deps,
            dep_chains,
            array_id,
        });
        any_emitted = true;
    }

    // ---- Phase 3: apply rewrites block-by-block ----
    for (block_id, mut block_rewrites) in rewrites_by_block {
        block_rewrites.sort_by_key(|r| r.instr_index);
        let Some(block) = func.body.blocks.get_mut(&block_id) else { continue };
        apply_block_rewrites(block, &mut block_rewrites);
    }

    if any_emitted {
        // Mirror the TS reference's post-pass cleanup:
        //   reversePostorderBlocks → markPredecessors → markInstructionIds →
        //   fixScopeAndIdentifierRanges → deadCodeElimination
        //
        // Re-establishing RPO + predecessors keeps later passes that depend
        // on a canonical block order happy. Marking instruction IDs
        // renumbers the newly-inserted instructions, after which the scope
        // ranges (which key off InstructionIds) need to be rebuilt.
        // Finally DCE removes the now-unreferenced `AUTODEPS` LoadGlobal
        // along with any other instructions that became dead.
        crate::hir::hir_builder::reverse_postorder_blocks(&mut func.body);
        crate::hir::hir_builder::mark_predecessors(&mut func.body);
        crate::hir::hir_builder::mark_instruction_ids(&mut func.body);
        crate::hir::build_reactive_scope_terminals_hir::fix_scope_and_identifier_ranges(
            &mut func.body,
        );
        crate::optimization::dead_code_elimination::dead_code_elimination(func);
        func.env.mark_has_inferred_effect();
    }
}

/// A dependency chain pre-allocated with fresh identifier IDs.
struct DepChain {
    /// LoadLocal lvalue identifier.
    root_id: IdentifierId,
    /// PropertyLoad lvalue identifiers (one per path entry).
    chain_ids: Vec<IdentifierId>,
}

/// Rewrite plan with pre-allocated identifier IDs, ready to apply.
struct PreparedRewrite {
    instr_index: usize,
    autodeps_arg_index: usize,
    deps: Vec<ReactiveScopeDependency>,
    dep_chains: Vec<DepChain>,
    /// ArrayExpression lvalue identifier.
    array_id: IdentifierId,
}

/// Apply all per-block rewrites in increasing instr_index order.
fn apply_block_rewrites(block: &mut BasicBlock, rewrites: &mut [PreparedRewrite]) {
    let original = std::mem::take(&mut block.instructions);
    let mut new_instrs: Vec<Instruction> = Vec::with_capacity(original.len() + rewrites.len() * 4);
    let mut rewrite_iter = rewrites.iter_mut().peekable();
    for (idx, mut instr) in original.into_iter().enumerate() {
        if let Some(rw) = rewrite_iter.peek()
            && rw.instr_index == idx
        {
            let rw = rewrite_iter.next().unwrap();
            // Materialise each dep as a Place using LoadLocal + chain
            // of PropertyLoad instructions.
            let mut dep_places: Vec<Place> = Vec::with_capacity(rw.deps.len());
            for (dep, chain) in rw.deps.iter().zip(rw.dep_chains.iter()) {
                let place = emit_dependency_instructions(dep, chain, &mut new_instrs);
                dep_places.push(place);
            }

            // Build the ArrayExpression instruction producing the deps array.
            let array_lvalue = make_temp_place(rw.array_id, Effect::Mutate);
            let array_value = InstructionValue::ArrayExpression(ArrayExpression {
                elements: dep_places.into_iter().map(ArrayExpressionElement::Place).collect(),
                loc: GENERATED_SOURCE,
            });
            new_instrs.push(Instruction {
                id: InstructionId(0),
                lvalue: array_lvalue.clone(),
                value: array_value,
                effects: None,
                loc: GENERATED_SOURCE,
            });

            // Substitute the AUTODEPS arg with the array Place.
            let mut deps_place = array_lvalue;
            deps_place.effect = Effect::Freeze;
            replace_call_arg(&mut instr.value, rw.autodeps_arg_index, deps_place);

            new_instrs.push(instr);
            continue;
        }
        new_instrs.push(instr);
    }
    block.instructions = new_instrs;
}

/// Build a temporary `Place` for a freshly-allocated identifier id with
/// the given effect. The identifier is named (`name = None`) so codegen
/// will inline it when used exactly once.
fn make_temp_place(id: IdentifierId, effect: Effect) -> Place {
    Place {
        identifier: Identifier {
            id,
            declaration_id: crate::hir::DeclarationId(id.0),
            name: None,
            mutable_range: crate::hir::MutableRange::default(),
            scope: None,
            type_: crate::hir::types::make_type(),
            loc: GENERATED_SOURCE,
        },
        effect,
        reactive: false,
        loc: GENERATED_SOURCE,
    }
}

/// Emit LoadLocal + chain of PropertyLoad instructions to materialise a
/// dependency as a Place. Returns the final Place produced by the last
/// instruction in the chain.
fn emit_dependency_instructions(
    dep: &ReactiveScopeDependency,
    chain: &DepChain,
    out: &mut Vec<Instruction>,
) -> Place {
    let loc = dep.identifier.loc;

    // LoadLocal of the root identifier.
    let mut root_lvalue = make_temp_place(chain.root_id, Effect::Mutate);
    root_lvalue.reactive = dep.reactive;
    out.push(Instruction {
        id: InstructionId(0),
        lvalue: root_lvalue.clone(),
        value: InstructionValue::LoadLocal(LoadLocal {
            place: Place {
                identifier: dep.identifier.clone(),
                effect: Effect::Freeze,
                reactive: dep.reactive,
                loc,
            },
            loc,
        }),
        effects: None,
        loc,
    });

    // Chain PropertyLoads for each path entry.
    //
    // The `optional` bit on each `DependencyPathEntry` is preserved on the
    // synthesized `PropertyLoad` so that codegen reproduces the original
    // `?.` segment when rendering the inferred deps array (e.g. emitting
    // `[obj.a?.b]` instead of `[obj.a.b]` for `useEffect(() =>
    // print(obj.a?.b), AUTODEPS)`).
    let mut current_id = root_lvalue.identifier;
    for (entry, &next_id) in dep.path.iter().zip(chain.chain_ids.iter()) {
        let mut next_lvalue = make_temp_place(next_id, Effect::Mutate);
        next_lvalue.reactive = dep.reactive;
        out.push(Instruction {
            id: InstructionId(0),
            lvalue: next_lvalue.clone(),
            value: InstructionValue::PropertyLoad(PropertyLoad {
                object: Place {
                    identifier: current_id.clone(),
                    effect: Effect::Freeze,
                    reactive: dep.reactive,
                    loc,
                },
                property: entry.property.clone(),
                optional: entry.optional,
                loc,
            }),
            effects: None,
            loc,
        });
        current_id = next_lvalue.identifier;
    }

    Place { identifier: current_id, effect: Effect::Freeze, reactive: dep.reactive, loc }
}

/// Replace the argument at `idx` (which must be a non-spread Place) in a
/// call expression with the given Place.
fn replace_call_arg(value: &mut InstructionValue, idx: usize, place: Place) {
    match value {
        InstructionValue::CallExpression(c) if idx < c.args.len() => {
            c.args[idx] = CallArg::Place(place);
        }
        InstructionValue::MethodCall(m) if idx < m.args.len() => {
            m.args[idx] = CallArg::Place(place);
        }
        _ => {}
    }
}

/// Returns true if a dependency should be filtered out of the inferred
/// deps array (refs/setStates/effect-event values that are statically
/// known to be non-reactive).
fn dep_is_filtered_out(
    dep: &ReactiveScopeDependency,
    reactive_ids: &FxHashSet<IdentifierId>,
) -> bool {
    let id = &dep.identifier;
    if (is_use_ref_type(id) || is_set_state_type(id)) && !reactive_ids.contains(&id.id) {
        return true;
    }
    if is_fire_function_type(id) {
        return true;
    }
    if is_effect_event_function_type(id) {
        return true;
    }
    false
}

/// Truncate a dependency path at the first `.current` segment.
///
/// `useRef` values are stable, so `ref.current.foo` reduces to `ref` (or in
/// practice the compiler will then filter the ref dependency entirely via
/// `dep_is_filtered_out`).
fn truncate_dep_at_current(dep: ReactiveScopeDependency) -> ReactiveScopeDependency {
    let idx = dep
        .path
        .iter()
        .position(|p| matches!(&p.property, PropertyLiteral::String(s) if s == "current"));
    match idx {
        None => dep,
        Some(i) => ReactiveScopeDependency {
            identifier: dep.identifier,
            reactive: dep.reactive,
            path: dep.path[..i].to_vec(),
            loc: dep.loc,
        },
    }
}

fn is_autodeps_type(identifier: &Identifier) -> bool {
    matches!(
        &identifier.type_,
        Type::Object(ObjectType { shape_id: Some(id) }) if id == BUILT_IN_AUTODEPS_ID
    )
}

fn is_use_ref_type(identifier: &Identifier) -> bool {
    matches!(
        &identifier.type_,
        Type::Object(ObjectType { shape_id: Some(id) }) if id == BUILT_IN_USE_REF_ID
    )
}

fn is_set_state_type(identifier: &Identifier) -> bool {
    matches!(
        &identifier.type_,
        Type::Function(f) if f.shape_id.as_deref() == Some(BUILT_IN_SET_STATE_ID)
    )
}

fn is_fire_function_type(identifier: &Identifier) -> bool {
    // Matches TS reference: checks shape_id == BuiltInFireFunctionId.
    // The Rust port wires this up once `enable_fire` lowers fire
    // bindings to the BUILT_IN_FIRE_FUNCTION_ID shape (see
    // `transform/transform_fire.rs`).
    matches!(
        &identifier.type_,
        Type::Function(f)
        if f.shape_id.as_deref() == Some(crate::hir::object_shape::BUILT_IN_FIRE_FUNCTION_ID)
    )
}

fn is_effect_event_function_type(identifier: &Identifier) -> bool {
    match &identifier.type_ {
        Type::Function(f) => match f.shape_id.as_deref() {
            Some(id) => id == BUILT_IN_EFFECT_EVENT_ID || id == BUILT_IN_USE_EFFECT_EVENT_ID,
            None => false,
        },
        _ => false,
    }
}

/// Collect the set of identifier IDs that appear with `place.reactive ==
/// true` anywhere in the outer function. Used to keep refs/setStates that
/// become reactive through scope pruning.
///
/// Matches the TS reference's `inferReactiveIdentifiers`: only the
/// top-level function body is scanned; we do NOT descend into nested
/// function expressions because their effects are captured via
/// LoweredFunction.dependencies during the outer walk.
fn infer_reactive_identifiers(func: &HIRFunction) -> FxHashSet<IdentifierId> {
    let mut reactive_ids: FxHashSet<IdentifierId> = FxHashSet::default();
    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            for place in each_instruction_operand(instr) {
                if place.reactive {
                    reactive_ids.insert(place.identifier.id);
                }
            }
        }
        for place in each_terminal_operand(&block.terminal) {
            if place.reactive {
                reactive_ids.insert(place.identifier.id);
            }
        }
    }
    reactive_ids
}

/// Reorder a list of dependencies so they match the order in which their
/// full dependency path is first materialised inside the lambda
/// associated with `fn_lvalue_id`.
///
/// Algorithm:
/// 1. Walk the inner function's body in instruction order, building a
///    sidemap from each temporary identifier id to its underlying
///    dependency path (LoadLocal → root, PropertyLoad → extend path).
/// 2. The first time a temporary that maps to a given (root_decl_id,
///    path-as-strings) shows up, assign it a rank.
/// 3. Sort the input deps by that rank, breaking ties by their original
///    index.
///
/// Sidemap entry used by `order_deps_by_first_use`: a temporary identifier
/// id is mapped to the declaration id of its root + the accumulated path.
#[derive(Clone)]
struct TempDep {
    root_decl: crate::hir::DeclarationId,
    path: Vec<DependencyPathEntry>,
}

/// Stable string representation of a dependency path. Used as a HashMap
/// key for first-use ranking.
fn path_key(path: &[DependencyPathEntry]) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    for entry in path {
        if entry.optional {
            s.push('?');
        }
        s.push('.');
        match &entry.property {
            PropertyLiteral::String(name) => s.push_str(name),
            PropertyLiteral::Number(n) => {
                let _ = write!(s, "{n}");
            }
        }
    }
    s
}

fn order_deps_by_first_use(
    deps: &[ReactiveScopeDependency],
    func: &HIRFunction,
    fn_lvalue_id: IdentifierId,
) -> Vec<ReactiveScopeDependency> {
    if deps.len() <= 1 {
        return deps.to_vec();
    }
    let Some(fn_instr) = find_fn_expression(func, fn_lvalue_id) else {
        return deps.to_vec();
    };
    let lowered = match &fn_instr.value {
        InstructionValue::FunctionExpression(v) => &v.lowered_func.func,
        _ => return deps.to_vec(),
    };
    let mut temps: FxHashMap<IdentifierId, TempDep> = FxHashMap::default();
    let mut rank: FxHashMap<(crate::hir::DeclarationId, String), u32> = FxHashMap::default();
    let mut next_rank: u32 = 0;

    let bump = |key: (crate::hir::DeclarationId, String),
                rank: &mut FxHashMap<(crate::hir::DeclarationId, String), u32>,
                next_rank: &mut u32| {
        rank.entry(key).or_insert_with(|| {
            let r = *next_rank;
            *next_rank += 1;
            r
        });
    };

    for block in lowered.body.blocks.values() {
        for instr in &block.instructions {
            match &instr.value {
                InstructionValue::LoadLocal(v) => {
                    let decl = v.place.identifier.declaration_id;
                    temps.insert(
                        instr.lvalue.identifier.id,
                        TempDep { root_decl: decl, path: Vec::new() },
                    );
                    bump((decl, String::new()), &mut rank, &mut next_rank);
                }
                InstructionValue::LoadContext(v) => {
                    let decl = v.place.identifier.declaration_id;
                    temps.insert(
                        instr.lvalue.identifier.id,
                        TempDep { root_decl: decl, path: Vec::new() },
                    );
                    bump((decl, String::new()), &mut rank, &mut next_rank);
                }
                InstructionValue::PropertyLoad(v) => {
                    if let Some(prev) = temps.get(&v.object.identifier.id).cloned() {
                        let mut new_path = prev.path.clone();
                        new_path.push(DependencyPathEntry {
                            property: v.property.clone(),
                            optional: false,
                            loc: v.loc,
                        });
                        let key = (prev.root_decl, path_key(&new_path));
                        bump(key, &mut rank, &mut next_rank);
                        temps.insert(
                            instr.lvalue.identifier.id,
                            TempDep { root_decl: prev.root_decl, path: new_path },
                        );
                    }
                }
                _ => {}
            }
            // Walk operands to also catch direct uses (e.g. `foo` appears as
            // a non-LoadLocal operand). This rank is for the bare identifier.
            for place in each_instruction_operand(instr) {
                let decl = place.identifier.declaration_id;
                bump((decl, String::new()), &mut rank, &mut next_rank);
            }
        }
    }

    let mut indexed: Vec<(usize, ReactiveScopeDependency)> =
        deps.iter().enumerate().map(|(i, d)| (i, d.clone())).collect();
    indexed.sort_by_key(|(i, d)| {
        let key = (d.identifier.declaration_id, path_key(&d.path));
        // If the exact-path isn't in `rank`, try just the root.
        let r = rank.get(&key).copied().unwrap_or_else(|| {
            rank.get(&(d.identifier.declaration_id, String::new())).copied().unwrap_or(u32::MAX)
        });
        (r, *i)
    });
    indexed.into_iter().map(|(_, d)| d).collect()
}

/// Look up a FunctionExpression instruction by its lvalue identifier id.
fn find_fn_expression(func: &HIRFunction, lvalue_id: IdentifierId) -> Option<&Instruction> {
    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            if matches!(&instr.value, InstructionValue::FunctionExpression(_))
                && instr.lvalue.identifier.id == lvalue_id
            {
                return Some(instr);
            }
        }
    }
    None
}

/// Compute the minimal set of reactive dependencies for a nested function
/// expression. Mirrors the TS `inferMinimalDependencies`.
///
/// This implementation is intentionally simpler than the TS reference: it
/// walks the inner function body, tracks identifiers declared inside the
/// inner function, and records any LoadLocal/LoadContext/PropertyLoad
/// chains that resolve to outer-context variables.
///
/// Optional-chain handling: the pass calls `collect_optional_chain_sidemap`
/// on the inner function to recover the full path (including `?.` segments)
/// of optional-chain expressions. The sidemap's `temporaries_read_in_optional`
/// maps the identifier produced by an optional chain (e.g. `arr[0]?.value`)
/// back to a `ReactiveScopeDependency` with the proper path. Without this,
/// `arr[0]?.value` would only contribute `arr` as a dependency because the
/// optional terminal lowering hides the path behind a phi.
fn infer_minimal_dependencies(fn_instr: &Instruction) -> Vec<ReactiveScopeDependency> {
    let lowered = match &fn_instr.value {
        InstructionValue::FunctionExpression(v) => &v.lowered_func.func,
        _ => return Vec::new(),
    };

    // Outer-context identifiers — known from `fn.context`. Anything else
    // is considered inner-defined by construction (SSA + scoping ensure
    // every other instruction lvalue is a fresh inner identifier).
    let outer_context: FxHashSet<IdentifierId> =
        lowered.context.iter().map(|c| c.identifier.id).collect();

    // Collect optional-chain sidemap so we can recover the full path of
    // `a?.b.c` reads — without this, the optional terminal lowering hides
    // the path behind a phi and only the root variable would be tracked.
    let opt_sidemap =
        crate::hir::collect_optional_chain_dependencies::collect_optional_chain_sidemap(lowered);

    // Temporaries map: lvalue id -> source dep root (Place + accumulated path).
    let mut temporaries: FxHashMap<IdentifierId, ReactiveScopeDependency> = FxHashMap::default();
    // Seed from the optional-chain sidemap. Optional-chain temporaries
    // already encode the full chain (with the `optional` bit per entry),
    // and downstream reads should use these instead of treating them as
    // fresh outer-context identifiers.
    for (&id, dep) in &opt_sidemap.temporaries_read_in_optional {
        temporaries.insert(id, dep.clone());
    }

    let mut raw_deps: Vec<ReactiveScopeDependency> = Vec::new();
    let record_place = |place: &Place,
                        temporaries: &FxHashMap<IdentifierId, ReactiveScopeDependency>,
                        raw_deps: &mut Vec<ReactiveScopeDependency>| {
        // Skip identifiers defined in the inner fn.
        if let Some(resolved) = temporaries.get(&place.identifier.id) {
            if outer_context.contains(&resolved.identifier.id) {
                raw_deps.push(resolved.clone());
            }
            return;
        }
        if outer_context.contains(&place.identifier.id) {
            raw_deps.push(ReactiveScopeDependency {
                identifier: place.identifier.clone(),
                reactive: place.reactive,
                path: Vec::new(),
                loc: place.loc,
            });
        }
    };

    for block in lowered.body.blocks.values() {
        // Visit phi operands first. A phi at a fallthrough block of an
        // optional chain merges the consequent's StoreLocal identifier with
        // the alternate's `undefined`. The consequent identifier is recorded
        // in `temporaries_read_in_optional` and resolves to the full
        // optional-chain dep, so visiting phi operands here captures the
        // dependency even when the phi result is consumed via a separate
        // operand later.
        for phi in &block.phis {
            for operand in phi.operands.values() {
                if let Some(resolved) =
                    opt_sidemap.temporaries_read_in_optional.get(&operand.identifier.id)
                    && outer_context.contains(&resolved.identifier.id)
                {
                    raw_deps.push(resolved.clone());
                }
            }
        }

        for instr in &block.instructions {
            // Defer instructions that are processed in an optional-chain
            // (their effects are folded into the optional-chain dep).
            let lvalue_is_optional_temp =
                opt_sidemap.temporaries_read_in_optional.contains_key(&instr.lvalue.identifier.id);
            let is_processed_in_optional =
                opt_sidemap.processed_instrs_in_optional.contains(&instr.id);
            if lvalue_is_optional_temp || is_processed_in_optional {
                continue;
            }

            // Mirror TS `isDeferredDependency`: an instruction whose lvalue
            // becomes a temporary (alias of an outer-context dep or a
            // property-chain of one) is skipped — its dependency is folded
            // into the temporary map and re-emitted only at the final use
            // site. Without this gate, walking the LoadLocal's operand
            // would push the bare root identifier (e.g. `arr`) as a dep,
            // which then merges with the longer optional-chain dep
            // (`arr[0]?.value`) at the tree's root and collapses the path.
            let mut defer_operand_walk = false;

            match &instr.value {
                // If reading an outer-context identifier (directly), register
                // a temporary mapping for the lvalue.
                InstructionValue::LoadLocal(v)
                    if outer_context.contains(&v.place.identifier.id) =>
                {
                    temporaries.insert(
                        instr.lvalue.identifier.id,
                        ReactiveScopeDependency {
                            identifier: v.place.identifier.clone(),
                            reactive: v.place.reactive,
                            path: Vec::new(),
                            loc: v.place.loc,
                        },
                    );
                    defer_operand_walk = true;
                }
                InstructionValue::LoadContext(v)
                    if outer_context.contains(&v.place.identifier.id) =>
                {
                    temporaries.insert(
                        instr.lvalue.identifier.id,
                        ReactiveScopeDependency {
                            identifier: v.place.identifier.clone(),
                            reactive: v.place.reactive,
                            path: Vec::new(),
                            loc: v.place.loc,
                        },
                    );
                    defer_operand_walk = true;
                }
                InstructionValue::PropertyLoad(v) => {
                    // If `obj.prop` reads a known dep prefix, extend it.
                    if let Some(resolved) = temporaries.get(&v.object.identifier.id) {
                        let mut path = resolved.path.clone();
                        path.push(DependencyPathEntry {
                            property: v.property.clone(),
                            optional: v.optional,
                            loc: v.loc,
                        });
                        temporaries.insert(
                            instr.lvalue.identifier.id,
                            ReactiveScopeDependency {
                                identifier: resolved.identifier.clone(),
                                reactive: resolved.reactive,
                                path,
                                loc: v.loc,
                            },
                        );
                        defer_operand_walk = true;
                    } else if outer_context.contains(&v.object.identifier.id) {
                        // First-level property access on outer context.
                        temporaries.insert(
                            instr.lvalue.identifier.id,
                            ReactiveScopeDependency {
                                identifier: v.object.identifier.clone(),
                                reactive: v.object.reactive,
                                path: vec![DependencyPathEntry {
                                    property: v.property.clone(),
                                    optional: v.optional,
                                    loc: v.loc,
                                }],
                                loc: v.loc,
                            },
                        );
                        defer_operand_walk = true;
                    }
                }
                _ => {}
            }

            if !defer_operand_walk {
                // Record real reads (those that "consume" a Place rather than
                // produce a temporary).
                for place in each_instruction_operand(instr) {
                    record_place(place, &temporaries, &mut raw_deps);
                }
            }
        }

        // Skip the terminal if its instruction id has been recorded as
        // part of an optional chain.
        let terminal_processed =
            opt_sidemap.processed_instrs_in_optional.contains(&block.terminal.id());
        if !terminal_processed {
            for place in each_terminal_operand(&block.terminal) {
                record_place(place, &temporaries, &mut raw_deps);
            }
        }
    }

    // Build the dependency tree seeded with the inner function's hoistable
    // objects. The `hoistable_objects` map records the safe non-null prefix
    // for each optional chain test block (e.g. for `arr[0]?.value` it
    // records `arr` because reading `arr[0]` is safe even when `arr[0]` may
    // be null). Without this seed, `add_dependency` truncates dependency
    // paths at the first optional segment, collapsing `arr[0]?.value` down
    // to just `arr`.
    let hoistable_deps: Vec<ReactiveScopeDependency> =
        opt_sidemap.hoistable_objects.values().cloned().collect();
    let mut tree = DependencyTree::new(hoistable_deps);
    for dep in &raw_deps {
        tree.add_dependency(dep);
    }
    tree.derive_minimal_dependencies()
}

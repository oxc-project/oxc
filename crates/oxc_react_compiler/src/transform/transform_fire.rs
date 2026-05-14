/// Rewrite `fire(callee(args))` inside `useEffect` lambdas into a stable
/// `useFire`-bound dispatch.
///
/// Port of `Transform/TransformFire.ts` (~740 LoC) from the React Compiler.
///
/// # Algorithm
///
/// The TS reference runs as a single recursive walk that maintains a
/// mutable `Context` and threads an `inUseEffectLambda` bit. It mutates
/// HIR instructions in place as it goes (deleting `fire(...)` calls,
/// mutating `LoadLocal.place`, splicing new instructions before the
/// `useEffect` LoadGlobal, etc.).
///
/// The Rust port follows the same three-stage shape used by other HIR
/// passes (see `inference/infer_effect_dependencies.rs`):
///
/// 1. **Snapshot** every function-tree position into a flat plan
///    (`TransformPlan`). This phase is fully read-only (`&HIRFunction`),
///    making the recursive walk easy to express.
/// 2. **Allocate** all fresh identifier ids used by emitted instructions
///    by drawing from `func.env.next_identifier_id()` up-front.
/// 3. **Apply** the plan in a single mutation pass over the HIR tree.
///
/// # Pipeline placement
///
/// Runs between `InferTypes` and `OptimizePropsMethodCalls`, matching
/// upstream `Pipeline.ts:215-218`. Gated by `Environment.config().enable_fire`.
///
/// # Limitations / TODOs (mirrors upstream)
/// - Only supports `fire(callExpression(args))` shape.
/// - Does not support `fire(this.method())` or `fire(obj.method())`.
/// - Does not support `React.useEffect(...)` (PropertyLoad form).
/// - Only `useEffect` is recognised — `useLayoutEffect`/`useInsertionEffect`
///   are intentionally NOT included (matches TS `isUseEffectHookType`).
use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};

/// Insertion-order-preserving map keyed by `IdentifierId`. Matches the
/// behaviour of JS's `Map`, which the TS reference relies on (iteration
/// order = insertion order). Using `FxHashMap` here causes the emitted
/// `useFire(...)` calls to come out in non-deterministic order.
type OrderedIdMap<V> = IndexMap<IdentifierId, V, FxBuildHasher>;

use crate::compiler_error::{
    CompilerError, CompilerErrorDetail, CompilerErrorDetailOptions, ErrorCategory,
    GENERATED_SOURCE, SourceLocation,
};
use crate::hir::object_shape::{
    BUILT_IN_DEFAULT_NONMUTATING_HOOK_ID, BUILT_IN_FIRE_FUNCTION_ID, BUILT_IN_FIRE_ID,
    BUILT_IN_USE_EFFECT_HOOK_ID,
};
use crate::hir::types::{FunctionType, Type};
use crate::hir::visitors::each_instruction_operand;
use crate::hir::{
    ArrayExpressionElement, BlockId, CallArg, CallExpression, DeclarationId, Effect, HIRFunction,
    Identifier, IdentifierId, IdentifierName, Instruction, InstructionId, InstructionKind,
    InstructionValue, LValue, LoadGlobal, LoadLocal, MutableRange, NonLocalBinding, Place,
    StoreLocal,
};

const CANNOT_COMPILE_FIRE: &str = "Cannot compile `fire`";
const USE_FIRE_FUNCTION_NAME: &str = "useFire";
const REACT_RUNTIME_MODULE: &str = "react/compiler-runtime";

/// Run the `transform_fire` pass on a HIR function.
///
/// # Errors
/// Returns `Err(CompilerError)` if any `fire(...)` call is malformed
/// or `fire` is referenced outside a useEffect lambda. Errors are
/// aggregated.
pub fn transform_fire(func: &mut HIRFunction) -> Result<(), CompilerError> {
    let mut errors = CompilerError::new();

    // Phase 1: gather sidemaps + an analysis plan from the read-only HIR.
    let analysis = analyse_function(func, &mut errors);

    // Phase 2: when no errors yet, also verify there are no untransformed
    // `fire` references left in the tree (matches TS `ensureNoMoreFireUses`).
    if !errors.has_any_errors() {
        // Skip the places of instructions we're going to delete (fire
        // CallExpressions and `fire` LoadGlobals): they're the
        // legitimately-transformed uses, and TS does this check post-
        // mutation, so they wouldn't show up there either.
        let mut deleted: FxHashSet<InstructionId> = FxHashSet::default();
        deleted.extend(analysis.fire_call_locs.iter().map(|l| l.instr_id));
        deleted.extend(analysis.fire_load_global_locs.iter().map(|l| l.instr_id));
        ensure_no_more_fire_uses(func, &deleted, &mut errors);
    }

    if errors.has_any_errors() {
        return Err(errors);
    }

    // Phase 3: nothing to do? Return early.
    if analysis.is_noop() {
        return Ok(());
    }

    // Phase 4: build the concrete instruction-mutation plan using fresh
    // identifier ids from `func.env`.
    let plan = build_plan(func, &analysis);

    // Phase 5: apply the plan to the HIR.
    apply_plan(func, &plan);

    Ok(())
}

// =====================================================================================
// Analysis types
// =====================================================================================

/// Path identifying a specific function in the HIR tree. `Root` is the
/// outer function `transform_fire` was invoked on; `Nested(id)` is a
/// FunctionExpression with that lvalue id (in its parent).
///
/// We assume lvalue identifier ids are globally unique within a single
/// HIR function tree, which is true by construction (`Environment::next_identifier_id`
/// allocates monotonically across the whole tree).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FnPath {
    Root,
    Nested(IdentifierId),
}

#[derive(Debug, Clone, Copy)]
struct InstrLoc {
    block_id: BlockId,
    instr_id: InstructionId,
}

/// A fully-scoped instruction location: which function (path), which
/// block, which instruction id.
#[derive(Debug, Clone, Copy)]
struct ScopedInstrLoc {
    fn_path: FnPath,
    block_id: BlockId,
    instr_id: InstructionId,
}

#[derive(Debug, Clone)]
struct CallSnapshot {
    callee_id: IdentifierId,
    /// Retained for future diagnostics + completeness. Currently the
    /// analysis works off `callee_id` + the LoadLocal sidemap only.
    #[expect(dead_code, reason = "retained for diagnostics / future use")]
    args: Vec<CallArg>,
    /// Retained for future error messages tied to the inner call site.
    #[expect(dead_code, reason = "retained for diagnostics / future use")]
    loc: SourceLocation,
}

#[derive(Debug, Clone)]
struct LoadLocalSnapshot {
    loc: InstrLoc,
    place: Place,
}

#[derive(Debug, Clone)]
struct ArrayExprSnapshot {
    elements: Vec<ArrayExpressionElement>,
}

/// Sidemap collected per function in the tree.
#[derive(Default, Debug, Clone)]
struct FunctionSidemap {
    call_expressions: FxHashMap<IdentifierId, CallSnapshot>,
    fn_expressions: FxHashMap<IdentifierId, InstrLoc>,
    load_locals: FxHashMap<IdentifierId, LoadLocalSnapshot>,
    array_expressions: FxHashMap<IdentifierId, ArrayExprSnapshot>,
    load_globals: FxHashMap<IdentifierId, InstrLoc>,
}

/// A single `fire(callee(...))` rewrite request, recorded during analysis.
#[derive(Debug, Clone)]
struct FireRewrite {
    /// Source callee identifier id (the `foo` in `fire(foo(...))`).
    source_callee_id: IdentifierId,
    /// Source callee's identifier (cloned for the load instruction).
    #[expect(dead_code, reason = "retained for diagnostics / future use")]
    source_callee_identifier: Identifier,
    /// LoadLocal instruction whose `place` field needs to be rewired
    /// to the fire-function binding. Scoped by fn_path because
    /// instruction-ids collide across functions.
    load_local_loc: ScopedInstrLoc,
    /// The `fire(...)` CallExpression instruction location to delete.
    fire_call_loc: ScopedInstrLoc,
    /// Source location of the `fire(...)` call (for diagnostics).
    #[expect(dead_code, reason = "retained for diagnostics / future use")]
    fire_loc: SourceLocation,
}

/// A single `useEffect(lambda[, deps])` site recorded during analysis.
#[derive(Debug, Clone)]
struct UseEffectSite {
    /// Outer-function block containing the useEffect CallExpression.
    block_id: BlockId,
    /// Lvalue id of the useEffect callee LoadGlobal (or PropertyLoad in
    /// future). Used to splice `useFire(...)` instructions just before.
    use_effect_callee_load_id: IdentifierId,
    /// The fire-callees captured inside the lambda.
    captured: Vec<CapturedCallee>,
    /// Optional: the explicit deps array snapshot for rewriting.
    deps_arg: Option<DepsArg>,
}

#[derive(Debug, Clone)]
struct DepsArg {
    /// Whether the second arg is an Identifier (the only supported form).
    /// If it's a spread or non-Identifier, an error has been pushed.
    array_lvalue_id: Option<IdentifierId>,
}

#[derive(Debug, Clone)]
struct CapturedCallee {
    /// Source callee identifier id.
    source_callee_id: IdentifierId,
    /// Source callee identifier (full clone — used to construct the
    /// LoadLocal of the source in the useFire emission).
    source_callee_identifier: Identifier,
    /// Source location of the fire(...) call that introduced this
    /// binding (used in diagnostic messages).
    #[expect(dead_code, reason = "retained for diagnostics / future use")]
    fire_loc: SourceLocation,
}

/// Result of `analyse_function`.
#[derive(Debug, Default)]
struct Analysis {
    /// All blocks of the outer function's instruction tree, sidemap-collected.
    /// Keyed by FnPath.
    sidemaps: FxHashMap<FnPath, FunctionSidemap>,

    /// Per-fire-call rewrite requests gathered during the walk.
    fire_rewrites: Vec<FireRewrite>,

    /// One entry per `useEffect(lambda)` call discovered.
    use_effect_sites: Vec<UseEffectSite>,

    /// All `fire` (the React global) LoadGlobal instruction locations
    /// inside useEffect lambdas. These should be deleted.
    fire_load_global_locs: Vec<ScopedInstrLoc>,

    /// All `fire(...)` CallExpression (fn_path, instr_id) pairs —
    /// collected so the `ensure_no_more_fire_uses` pass can skip them
    /// when looking for untransformed `fire` references.
    fire_call_locs: Vec<ScopedInstrLoc>,

    /// For each FunctionExpression lvalue id in the tree, the captured
    /// callees that bubbled up from its body. Used to rewrite the inner
    /// function's `context` list at apply time.
    captures_by_fn: FxHashMap<IdentifierId, OrderedIdMap<CapturedCallee>>,
}

impl Analysis {
    fn is_noop(&self) -> bool {
        self.fire_rewrites.is_empty()
            && self.use_effect_sites.iter().all(|s| s.captured.is_empty())
            && self.fire_load_global_locs.is_empty()
    }
}

// =====================================================================================
// Phase 1 — Analyse (read-only)
// =====================================================================================

fn analyse_function(func: &HIRFunction, errors: &mut CompilerError) -> Analysis {
    let mut analysis = Analysis::default();

    // Collect sidemaps for every function in the tree, keyed by FnPath.
    collect_all_sidemaps(func, &mut analysis);

    // Walk top-level instructions, recursing into useEffect lambdas.
    // We process the outer function's blocks looking for useEffect calls.
    process_outer_for_use_effects(func, &mut analysis, errors);

    analysis
}

fn collect_all_sidemaps(func: &HIRFunction, analysis: &mut Analysis) {
    let mut work: Vec<(FnPath, &HIRFunction)> = vec![(FnPath::Root, func)];
    while let Some((path, fn_ref)) = work.pop() {
        let mut sidemap = FunctionSidemap::default();
        for (block_id, block) in &fn_ref.body.blocks {
            for instr in &block.instructions {
                let lid = instr.lvalue.identifier.id;
                let loc = InstrLoc { block_id: *block_id, instr_id: instr.id };
                match &instr.value {
                    InstructionValue::CallExpression(c) => {
                        sidemap.call_expressions.insert(
                            lid,
                            CallSnapshot {
                                callee_id: c.callee.identifier.id,
                                args: c.args.clone(),
                                loc: c.loc,
                            },
                        );
                    }
                    InstructionValue::FunctionExpression(_) => {
                        sidemap.fn_expressions.insert(lid, loc);
                    }
                    InstructionValue::LoadLocal(v) => {
                        sidemap
                            .load_locals
                            .insert(lid, LoadLocalSnapshot { loc, place: v.place.clone() });
                    }
                    InstructionValue::LoadContext(v) => {
                        // Treat LoadContext like LoadLocal: in deeply-
                        // nested function expressions, captured outer
                        // identifiers are loaded via LoadContext instead
                        // of LoadLocal, but the fire-rewrite handling
                        // is the same.
                        sidemap
                            .load_locals
                            .insert(lid, LoadLocalSnapshot { loc, place: v.place.clone() });
                    }
                    InstructionValue::ArrayExpression(a) => {
                        sidemap
                            .array_expressions
                            .insert(lid, ArrayExprSnapshot { elements: a.elements.clone() });
                    }
                    InstructionValue::LoadGlobal(_) => {
                        sidemap.load_globals.insert(lid, loc);
                    }
                    _ => {}
                }
            }
        }
        analysis.sidemaps.insert(path, sidemap);

        // Enqueue nested functions.
        for block in fn_ref.body.blocks.values() {
            for instr in &block.instructions {
                if let InstructionValue::FunctionExpression(v) = &instr.value {
                    work.push((FnPath::Nested(instr.lvalue.identifier.id), &v.lowered_func.func));
                }
            }
        }
    }
}

fn process_outer_for_use_effects(
    func: &HIRFunction,
    analysis: &mut Analysis,
    errors: &mut CompilerError,
) {
    // We need a clone of the outer sidemap before recursing, since
    // recursion may mutate captures_by_fn (read-only path, no actual
    // sidemap mutation, but borrow checker still flags shared mut).
    let outer_sidemap = analysis.sidemaps.get(&FnPath::Root).cloned().unwrap_or_default();

    for (block_id, block) in &func.body.blocks {
        for instr in &block.instructions {
            let InstructionValue::CallExpression(call) = &instr.value else { continue };
            if !is_use_effect_hook_callee(&call.callee.identifier) {
                continue;
            }
            let first_arg_id = match call.args.first() {
                Some(CallArg::Place(p)) => p.identifier.id,
                _ => continue,
            };
            let Some(fn_loc) = outer_sidemap.fn_expressions.get(&first_arg_id) else { continue };
            let Some(fn_instr) = fetch_instruction(func, fn_loc.block_id, fn_loc.instr_id) else {
                continue;
            };
            let InstructionValue::FunctionExpression(fn_expr) = &fn_instr.value else {
                continue;
            };

            // Recurse into the lambda. Each fire(...) call inside is
            // recorded; captures bubble up.
            let captured = process_lambda_for_fire(
                &fn_expr.lowered_func.func,
                FnPath::Nested(first_arg_id),
                analysis,
                errors,
            );

            // Record captures for this top-level lambda too — apply
            // phase reads `captures_by_fn[lambda_id]` to substitute the
            // lambda's context list, swapping captured-callee context
            // items for their fire-binding places.
            analysis.captures_by_fn.insert(first_arg_id, captured.clone());

            // Verify ensure_no_remaining_callee_captures. We pass the
            // set of LoadLocal instruction ids that will be rewritten
            // — those references already go through fire() so we
            // exclude them from the "must use fire()" check.
            let rewritten_ll_ids: FxHashSet<InstructionId> =
                analysis.fire_rewrites.iter().map(|r| r.load_local_loc.instr_id).collect();
            ensure_no_remaining_callee_captures(
                &fn_expr.lowered_func.func,
                &captured,
                &rewritten_ll_ids,
                errors,
            );

            // Build the deps-arg entry if the call has more than one arg.
            let deps_arg = if call.args.len() > 1 {
                handle_use_effect_deps_arg(&call.args[1], &outer_sidemap, &captured, errors)
            } else {
                None
            };

            // Record the useEffect site.
            let captured_vec: Vec<_> = captured.values().cloned().collect();
            analysis.use_effect_sites.push(UseEffectSite {
                block_id: *block_id,
                use_effect_callee_load_id: call.callee.identifier.id,
                captured: captured_vec,
                deps_arg,
            });
        }
    }
}

/// Process a single lambda's body. Records:
/// - `fire(...)` rewrites
/// - Captured callees (returned)
/// - `fire` LoadGlobal instruction ids to delete
/// - Recursively, any nested FunctionExpressions
fn process_lambda_for_fire(
    func: &HIRFunction,
    fn_path: FnPath,
    analysis: &mut Analysis,
    errors: &mut CompilerError,
) -> OrderedIdMap<CapturedCallee> {
    let mut local_captures: OrderedIdMap<CapturedCallee> = OrderedIdMap::default();

    let sidemap = analysis.sidemaps.get(&fn_path).cloned().unwrap_or_default();

    // Walk this function's blocks.
    for (block_id, block) in &func.body.blocks {
        for instr in &block.instructions {
            match &instr.value {
                InstructionValue::CallExpression(call)
                    if is_fire_callee_type(&call.callee.identifier) =>
                {
                    let scoped =
                        ScopedInstrLoc { fn_path, block_id: *block_id, instr_id: instr.id };
                    analysis.fire_call_locs.push(scoped);
                    handle_fire_call(
                        scoped,
                        instr,
                        call,
                        &sidemap,
                        fn_path,
                        &mut local_captures,
                        analysis,
                        errors,
                    );
                }
                InstructionValue::LoadGlobal(v) if is_fire_import_binding(&v.binding) => {
                    analysis.fire_load_global_locs.push(ScopedInstrLoc {
                        fn_path,
                        block_id: *block_id,
                        instr_id: instr.id,
                    });
                }
                _ => {}
            }
        }
    }

    // Recurse into nested FunctionExpressions inside this lambda. Even
    // when the nested function is NOT a useEffect, the TS reference
    // treats it as syntactically inside the parent useEffect lambda, so
    // fire(...) calls inside it are still rewritten.
    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            if let InstructionValue::FunctionExpression(v) = &instr.value {
                let nested_path = FnPath::Nested(instr.lvalue.identifier.id);
                let nested_captures =
                    process_lambda_for_fire(&v.lowered_func.func, nested_path, analysis, errors);

                // Bubble captures upward.
                for (id, info) in &nested_captures {
                    local_captures.entry(*id).or_insert_with(|| info.clone());
                }
                analysis.captures_by_fn.insert(instr.lvalue.identifier.id, nested_captures);
            }
        }
    }

    local_captures
}

fn handle_fire_call(
    fire_call_loc: ScopedInstrLoc,
    instr: &Instruction,
    call: &CallExpression,
    sidemap: &FunctionSidemap,
    fn_path: FnPath,
    local_captures: &mut OrderedIdMap<CapturedCallee>,
    analysis: &mut Analysis,
    errors: &mut CompilerError,
) {
    let _ = instr;
    // Validate single-Place arg shape.
    if call.args.len() != 1 {
        let mut description =
            "fire() can only take in a single call expression as an argument".to_string();
        if call.args.is_empty() {
            description.push_str(" but received none");
        } else {
            description.push_str(" but received multiple arguments");
        }
        push_fire_error(errors, description, call.loc);
        return;
    }

    let arg_place = match call.args.first() {
        Some(CallArg::Place(p)) => p,
        Some(CallArg::Spread(s)) => {
            push_fire_error(
                errors,
                "fire() can only take in a single call expression as an argument \
                 but received a spread argument"
                    .to_string(),
                s.place.loc,
            );
            return;
        }
        None => return,
    };

    let Some(inner_call) = sidemap.call_expressions.get(&arg_place.identifier.id) else {
        push_fire_error(
            errors,
            "`fire()` can only receive a function call such as `fire(fn(a,b))`. \
             Method calls and other expressions are not allowed"
                .to_string(),
            call.loc,
        );
        return;
    };

    let callee_id = inner_call.callee_id;
    let Some(load_local) = sidemap.load_locals.get(&callee_id) else {
        // No LoadLocal / LoadContext for this callee in the current
        // function's sidemap. This typically means the HIR builder
        // emitted a LoadGlobal (e.g., because the captured local was
        // not threaded as a context-var into a deeply-nested function
        // declaration). The TS reference pushes a generic invariant
        // error in this case.
        errors.push_error_detail(CompilerErrorDetail::new(CompilerErrorDetailOptions {
            category: ErrorCategory::Fire,
            reason: CANNOT_COMPILE_FIRE.to_string(),
            description: None,
            loc: Some(call.loc),
            suggestions: None,
        }));
        return;
    };

    let source_callee_id = load_local.place.identifier.id;

    let rewrite = FireRewrite {
        source_callee_id,
        source_callee_identifier: load_local.place.identifier.clone(),
        load_local_loc: ScopedInstrLoc {
            fn_path,
            block_id: load_local.loc.block_id,
            instr_id: load_local.loc.instr_id,
        },
        fire_call_loc,
        fire_loc: call.loc,
    };
    analysis.fire_rewrites.push(rewrite);

    local_captures.entry(source_callee_id).or_insert(CapturedCallee {
        source_callee_id,
        source_callee_identifier: load_local.place.identifier.clone(),
        fire_loc: call.loc,
    });
}

fn handle_use_effect_deps_arg(
    deps_arg: &CallArg,
    outer_sidemap: &FunctionSidemap,
    captured: &OrderedIdMap<CapturedCallee>,
    errors: &mut CompilerError,
) -> Option<DepsArg> {
    match deps_arg {
        CallArg::Place(p) => {
            // The deps must be an ArrayExpression literal.
            let Some(array_snap) = outer_sidemap.array_expressions.get(&p.identifier.id) else {
                if !captured.is_empty() {
                    push_fire_error(
                        errors,
                        "You must use an array literal for an effect dependency array \
                         when that effect uses `fire()`"
                            .to_string(),
                        p.loc,
                    );
                }
                return None;
            };
            // We only emit a deps-arg entry if there's actually
            // something to rewrite OR if there's an array. Either way,
            // its presence is purely informational at this analysis
            // stage; rewriting LoadLocal of dependency happens in
            // `build_plan` using the snapshot.
            let _ = array_snap; // we already have the array elements in outer_sidemap
            Some(DepsArg { array_lvalue_id: Some(p.identifier.id) })
        }
        CallArg::Spread(s) => {
            if !captured.is_empty() {
                push_fire_error(
                    errors,
                    "You must use an array literal for an effect dependency array \
                     when that effect uses `fire()`"
                        .to_string(),
                    s.place.loc,
                );
            }
            None
        }
    }
}

// =====================================================================================
// ensure_no_remaining_callee_captures + ensure_no_more_fire_uses
// =====================================================================================

/// For each place reachable from this function (including nested fn
/// expressions), verify that any reference to a captured callee
/// identifier id goes through a fire(...) call. Otherwise, raise the
/// "all uses must be either with fire() or without" error.
///
/// The TS reference checks: every Place whose identifier id is a
/// captured callee must NOT appear directly outside of a fire(...)
/// rewrite. Since rewrites mutate the LoadLocal's `place`, post-
/// rewrite there should be no remaining references. At analysis time
/// the rewrite hasn't happened, so we check by walking and ignoring
/// the LoadLocal instructions whose ids match a recorded rewrite.
fn ensure_no_remaining_callee_captures(
    func: &HIRFunction,
    captured: &OrderedIdMap<CapturedCallee>,
    rewritten_load_local_ids: &FxHashSet<InstructionId>,
    errors: &mut CompilerError,
) {
    if captured.is_empty() {
        return;
    }
    // Walk every place in the function tree, but skip the LoadLocal
    // operands of instructions whose ids are in `rewritten_load_local_ids`
    // (those LoadLocals will be rewired to the fire-function binding at
    // apply time, matching TS where the place mutation has already
    // happened before this check runs).
    let mut places: Vec<&Place> = Vec::new();
    each_reachable_place_excluding(func, rewritten_load_local_ids, &mut places);
    for place in places {
        let pid = place.identifier.id;
        if let Some(info) = captured.get(&pid) {
            let name = match &info.source_callee_identifier.name {
                Some(IdentifierName::Named(n) | IdentifierName::Promoted(n)) => n.as_str(),
                None => "<unknown>",
            };
            push_fire_error(
                errors,
                format!(
                    "All uses of {name} must be either used with a fire() call in this \
                     effect or not used with a fire() call at all"
                ),
                place.loc,
            );
        }
    }
}

/// Like `each_reachable_place_inner`, but skips operands of instructions
/// whose ids are listed in `skip_instr_ids`. Used by
/// `ensure_no_remaining_callee_captures` to ignore the LoadLocals that
/// will be rewritten by `apply_plan`.
fn each_reachable_place_excluding<'a>(
    func: &'a HIRFunction,
    skip_instr_ids: &FxHashSet<InstructionId>,
    out: &mut Vec<&'a Place>,
) {
    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            match &instr.value {
                InstructionValue::FunctionExpression(v) => {
                    each_reachable_place_excluding(&v.lowered_func.func, skip_instr_ids, out);
                }
                InstructionValue::ObjectMethod(v) => {
                    each_reachable_place_excluding(&v.lowered_func.func, skip_instr_ids, out);
                }
                _ => {
                    if !skip_instr_ids.contains(&instr.id) {
                        out.extend(each_instruction_operand(instr));
                    }
                }
            }
        }
        for place in crate::hir::visitors::each_terminal_operand(&block.terminal) {
            out.push(place);
        }
    }
}

/// After all transformations, every remaining Place whose identifier
/// type is `BuiltInFireId` is an untransformed reference to `fire` —
/// emit a "Cannot use fire outside of useEffect" error.
///
/// We pass the set of instruction ids that WILL be deleted by the
/// plan (fire calls, fire LoadGlobals) so this validation reflects
/// the post-mutation state, matching TS where mutations happen
/// in-place during the walk and this check runs at the very end.
fn ensure_no_more_fire_uses(
    func: &HIRFunction,
    deleted_instr_ids: &FxHashSet<InstructionId>,
    errors: &mut CompilerError,
) {
    let mut places: Vec<&Place> = Vec::new();
    each_reachable_place_excluding(func, deleted_instr_ids, &mut places);
    for place in places {
        if is_fire_callee_type(&place.identifier) {
            push_fire_error(
                errors,
                "Cannot use `fire` outside of a useEffect function".to_string(),
                place.identifier.loc,
            );
        }
    }
}

fn push_fire_error(errors: &mut CompilerError, description: String, loc: SourceLocation) {
    errors.push_error_detail(CompilerErrorDetail::new(CompilerErrorDetailOptions {
        category: ErrorCategory::Fire,
        reason: CANNOT_COMPILE_FIRE.to_string(),
        description: Some(description),
        loc: Some(loc),
        suggestions: None,
    }));
}

// =====================================================================================
// Phase 4 — Build plan (allocate fresh ids)
// =====================================================================================

#[derive(Debug, Default)]
struct TransformPlan {
    /// Instruction (function-path, instruction-id) pairs to delete.
    /// Scoped by FnPath because InstructionId values are NOT globally
    /// unique across nested functions — `mark_instruction_ids` runs
    /// per-function, so two different functions can share an id.
    delete_instr_ids: FxHashSet<(FnPath, InstructionId)>,

    /// LoadLocal substitutions: each entry mutates the LoadLocal at
    /// `(fn_path, block_id, instr_id)` to have its `place` replaced.
    load_local_subs: Vec<LoadLocalSub>,

    /// Splices: insert `new_instrs` immediately before the instruction
    /// with `before_instr_id` in `(fn_path, block_id)`.
    splices: Vec<Splice>,

    /// FunctionExpression context substitutions, keyed by FunctionExpression
    /// lvalue id.
    fn_context_subs: FxHashMap<IdentifierId, Vec<PlaceSub>>,
}

#[derive(Debug)]
struct LoadLocalSub {
    fn_path: FnPath,
    block_id: BlockId,
    instr_id: InstructionId,
    new_place: Place,
}

#[derive(Debug)]
struct Splice {
    fn_path: FnPath,
    block_id: BlockId,
    before_instr_id: InstructionId,
    new_instrs: Vec<Instruction>,
}

#[derive(Debug, Clone)]
struct PlaceSub {
    from_id: IdentifierId,
    to_place: Place,
}

fn build_plan(func: &mut HIRFunction, analysis: &Analysis) -> TransformPlan {
    let mut plan = TransformPlan::default();

    // Step 1: allocate fire-function bindings — one per unique source
    // callee id. The binding is a fresh promoted temporary with the
    // BuiltInFireFunction type.
    let mut fire_bindings: FxHashMap<IdentifierId, Place> = FxHashMap::default();
    let mut all_captured: FxHashSet<IdentifierId> = FxHashSet::default();
    for rw in &analysis.fire_rewrites {
        all_captured.insert(rw.source_callee_id);
    }
    for site in &analysis.use_effect_sites {
        for c in &site.captured {
            all_captured.insert(c.source_callee_id);
        }
    }
    for fc_map in analysis.captures_by_fn.values() {
        for id in fc_map.keys() {
            all_captured.insert(*id);
        }
    }
    for &source_id in &all_captured {
        let new_id = func.env.next_identifier_id();
        let place = make_fire_binding_place(new_id);
        fire_bindings.insert(source_id, place);
    }

    // Step 2: for each fire rewrite, emit a LoadLocal substitution and a delete.
    for rw in &analysis.fire_rewrites {
        let binding = fire_bindings
            .get(&rw.source_callee_id)
            .expect("invariant: every fire rewrite must have an allocated binding");
        plan.load_local_subs.push(LoadLocalSub {
            fn_path: rw.load_local_loc.fn_path,
            block_id: rw.load_local_loc.block_id,
            instr_id: rw.load_local_loc.instr_id,
            new_place: binding.clone(),
        });
        plan.delete_instr_ids.insert((rw.fire_call_loc.fn_path, rw.fire_call_loc.instr_id));
    }

    // Step 3: delete the `fire` LoadGlobal imports.
    for loc in &analysis.fire_load_global_locs {
        plan.delete_instr_ids.insert((loc.fn_path, loc.instr_id));
    }

    // Step 4: for each useEffect site, emit a `useFire(callee)` block
    // and queue a splice before the useEffect callee's LoadGlobal.
    let mut already_emitted: FxHashSet<IdentifierId> = FxHashSet::default();
    let outer_sidemap = analysis.sidemaps.get(&FnPath::Root).cloned().unwrap_or_default();
    for site in &analysis.use_effect_sites {
        // Locate the LoadGlobal of useEffect.
        let Some(load_loc) = outer_sidemap.load_globals.get(&site.use_effect_callee_load_id) else {
            continue;
        };
        let mut splice_instrs: Vec<Instruction> = Vec::new();
        for captured in &site.captured {
            if !already_emitted.insert(captured.source_callee_id) {
                continue;
            }
            let binding = fire_bindings
                .get(&captured.source_callee_id)
                .expect("invariant: every captured callee must have an allocated binding")
                .clone();
            let emitted =
                emit_use_fire_block(&mut func.env, &captured.source_callee_identifier, binding);
            splice_instrs.extend(emitted);
        }
        if !splice_instrs.is_empty() {
            plan.splices.push(Splice {
                fn_path: FnPath::Root,
                block_id: site.block_id,
                before_instr_id: load_loc.instr_id,
                new_instrs: splice_instrs,
            });
        }

        // Step 5: if there's an explicit deps array, substitute each
        // captured callee's load-local source in the deps with the fire
        // binding place.
        if let Some(deps) = &site.deps_arg
            && let Some(array_id) = deps.array_lvalue_id
            && let Some(array_snap) = outer_sidemap.array_expressions.get(&array_id)
        {
            for element in &array_snap.elements {
                if let ArrayExpressionElement::Place(p) = element {
                    // The element is a LoadLocal lvalue (codegen flattening).
                    // Find the LoadLocal instr for this lvalue and check if
                    // its source is a captured callee.
                    if let Some(ll_snap) = outer_sidemap.load_locals.get(&p.identifier.id) {
                        let src_id = ll_snap.place.identifier.id;
                        if let Some(binding) = fire_bindings.get(&src_id) {
                            // Substitute the LoadLocal's place to point
                            // to the fire binding identifier instead.
                            plan.load_local_subs.push(LoadLocalSub {
                                fn_path: FnPath::Root,
                                block_id: ll_snap.loc.block_id,
                                instr_id: ll_snap.loc.instr_id,
                                new_place: binding.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    // Step 6: for each nested fn with captures, substitute its context list.
    for (fn_id, captures) in &analysis.captures_by_fn {
        for (cid, _info) in captures {
            if let Some(binding) = fire_bindings.get(cid) {
                plan.fn_context_subs
                    .entry(*fn_id)
                    .or_default()
                    .push(PlaceSub { from_id: *cid, to_place: binding.clone() });
            }
        }
    }

    plan
}

/// Build a Place for a fire-function binding. The identifier carries
/// the BuiltInFireFunction type so it can flow through subsequent
/// inference passes, and is named as a Promoted temporary so that
/// `promote_used_temporaries` doesn't touch it.
fn make_fire_binding_place(id: IdentifierId) -> Place {
    let identifier = Identifier {
        id,
        declaration_id: DeclarationId(id.0),
        name: Some(IdentifierName::Promoted(format!("#t{}", id.0))),
        mutable_range: MutableRange::default(),
        scope: None,
        type_: Type::Function(FunctionType {
            shape_id: Some(BUILT_IN_FIRE_FUNCTION_ID.to_string()),
            return_type: Box::new(Type::Poly),
            is_constructor: false,
        }),
        loc: GENERATED_SOURCE,
    };
    Place { identifier, effect: Effect::Unknown, reactive: false, loc: GENERATED_SOURCE }
}

/// Emit:
///   const _useFire = (LoadGlobal of useFire)
///   const _foo_local = (LoadLocal of foo)
///   const _useFire_call = _useFire(_foo_local)
///   const _fire_foo = _useFire_call   // StoreLocal of fire binding
fn emit_use_fire_block(
    env: &mut crate::hir::environment::Environment,
    source_callee_identifier: &Identifier,
    fire_binding: Place,
) -> Vec<Instruction> {
    // 1) LoadGlobal useFire — type as DefaultNonmutatingHook so it's
    //    treated as a hook call by downstream passes (matches TS).
    let use_fire_load_id = env.next_identifier_id();
    let mut use_fire_place = make_temp_place(use_fire_load_id, Effect::Read);
    use_fire_place.identifier.type_ = Type::Function(FunctionType {
        shape_id: Some(BUILT_IN_DEFAULT_NONMUTATING_HOOK_ID.to_string()),
        return_type: Box::new(Type::Poly),
        is_constructor: false,
    });
    let use_fire_load = Instruction {
        id: InstructionId(0),
        lvalue: use_fire_place.clone(),
        value: InstructionValue::LoadGlobal(LoadGlobal {
            binding: NonLocalBinding::ImportSpecifier {
                name: USE_FIRE_FUNCTION_NAME.to_string(),
                module: REACT_RUNTIME_MODULE.to_string(),
                imported: USE_FIRE_FUNCTION_NAME.to_string(),
            },
            loc: GENERATED_SOURCE,
        }),
        effects: None,
        loc: GENERATED_SOURCE,
    };

    // 2) LoadLocal of source callee
    let source_load_id = env.next_identifier_id();
    let mut source_lvalue = make_temp_place(source_load_id, Effect::Unknown);
    source_lvalue.identifier.type_ = source_callee_identifier.type_.clone();
    let source_load = Instruction {
        id: InstructionId(0),
        lvalue: source_lvalue.clone(),
        value: InstructionValue::LoadLocal(LoadLocal {
            place: Place {
                identifier: source_callee_identifier.clone(),
                effect: Effect::Unknown,
                reactive: false,
                loc: source_callee_identifier.loc,
            },
            loc: source_callee_identifier.loc,
        }),
        effects: None,
        loc: GENERATED_SOURCE,
    };

    // 3) CallExpression _useFire(_foo)
    let call_id = env.next_identifier_id();
    let call_lvalue = make_temp_place(call_id, Effect::Read);
    let call_instr = Instruction {
        id: InstructionId(0),
        lvalue: call_lvalue.clone(),
        value: InstructionValue::CallExpression(CallExpression {
            callee: use_fire_place,
            args: vec![CallArg::Place(source_lvalue)],
            loc: GENERATED_SOURCE,
        }),
        effects: None,
        loc: GENERATED_SOURCE,
    };

    // 4) StoreLocal const fire_binding = _useFire_call
    let store_lvalue_id = env.next_identifier_id();
    let store_lvalue = make_temp_place(store_lvalue_id, Effect::Unknown);
    let store_instr = Instruction {
        id: InstructionId(0),
        lvalue: store_lvalue,
        value: InstructionValue::StoreLocal(StoreLocal {
            lvalue: LValue { place: fire_binding, kind: InstructionKind::Const },
            value: call_lvalue,
            loc: GENERATED_SOURCE,
        }),
        effects: None,
        loc: GENERATED_SOURCE,
    };

    vec![use_fire_load, source_load, call_instr, store_instr]
}

fn make_temp_place(id: IdentifierId, effect: Effect) -> Place {
    Place {
        identifier: Identifier {
            id,
            declaration_id: DeclarationId(id.0),
            name: None,
            mutable_range: MutableRange::default(),
            scope: None,
            type_: crate::hir::types::make_type(),
            loc: GENERATED_SOURCE,
        },
        effect,
        reactive: false,
        loc: GENERATED_SOURCE,
    }
}

// =====================================================================================
// Phase 5 — Apply plan
// =====================================================================================

fn apply_plan(func: &mut HIRFunction, plan: &TransformPlan) {
    apply_plan_to_function(func, FnPath::Root, plan);

    // After mutation, rebuild instruction ids + scope ranges (matches TS
    // `markInstructionIds` at the end of `replaceFireFunctions`).
    crate::hir::hir_builder::mark_instruction_ids(&mut func.body);
}

/// Recursively apply the plan to a function and its nested
/// FunctionExpressions. The `fn_path` parameter identifies which entries
/// in the plan apply to this function.
fn apply_plan_to_function(func: &mut HIRFunction, fn_path: FnPath, plan: &TransformPlan) {
    let block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        // Find splices for this (fn_path, block).
        let mut splices_for_block: FxHashMap<InstructionId, Vec<Instruction>> =
            FxHashMap::default();
        for splice in &plan.splices {
            if splice.fn_path == fn_path && splice.block_id == block_id {
                splices_for_block
                    .entry(splice.before_instr_id)
                    .or_default()
                    .extend(splice.new_instrs.iter().cloned());
            }
        }

        let Some(block) = func.body.blocks.get_mut(&block_id) else { continue };
        let original = std::mem::take(&mut block.instructions);
        let mut new_instrs: Vec<Instruction> = Vec::with_capacity(original.len() + 4);

        for mut instr in original {
            // Apply load-local / load-context substitution scoped to
            // this function. We also mutate the lvalue's identifier so
            // its TYPE reflects the new place's identifier (the fire-
            // binding's BuiltInFireFunction type) — this is what makes
            // the subsequent CallExpression of this lvalue inherit the
            // fire-function type, so it later gets the fire shape
            // through type-inference.
            for sub in &plan.load_local_subs {
                if sub.fn_path == fn_path && sub.block_id == block_id && sub.instr_id == instr.id {
                    match &mut instr.value {
                        InstructionValue::LoadLocal(ll) => {
                            ll.place = sub.new_place.clone();
                        }
                        InstructionValue::LoadContext(lc) => {
                            lc.place = sub.new_place.clone();
                        }
                        _ => {}
                    }
                }
            }

            // Apply function-expression context substitutions.
            if let InstructionValue::FunctionExpression(fn_expr) = &mut instr.value {
                let fn_id = instr.lvalue.identifier.id;
                if let Some(subs) = plan.fn_context_subs.get(&fn_id) {
                    for context_item in &mut fn_expr.lowered_func.func.context {
                        for sub in subs {
                            if context_item.identifier.id == sub.from_id {
                                *context_item = sub.to_place.clone();
                            }
                        }
                    }
                }
            }

            // Splice new instructions BEFORE this one.
            if let Some(new_block) = splices_for_block.remove(&instr.id) {
                for new_instr in new_block {
                    new_instrs.push(new_instr);
                }
            }

            // Delete? Only if (fn_path, instr_id) matches.
            if plan.delete_instr_ids.contains(&(fn_path, instr.id)) {
                continue;
            }

            new_instrs.push(instr);
        }

        block.instructions = new_instrs;
    }

    // Recurse into nested FunctionExpressions.
    let block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        let nested_lvalue_ids: Vec<IdentifierId> = {
            let Some(block) = func.body.blocks.get(&block_id) else { continue };
            block
                .instructions
                .iter()
                .filter_map(|i| match &i.value {
                    InstructionValue::FunctionExpression(_) => Some(i.lvalue.identifier.id),
                    _ => None,
                })
                .collect()
        };
        for lvalue_id in nested_lvalue_ids {
            let Some(block) = func.body.blocks.get_mut(&block_id) else { continue };
            let Some(instr) =
                block.instructions.iter_mut().find(|i| i.lvalue.identifier.id == lvalue_id)
            else {
                continue;
            };
            if let InstructionValue::FunctionExpression(fn_expr) = &mut instr.value {
                apply_plan_to_function(
                    &mut fn_expr.lowered_func.func,
                    FnPath::Nested(lvalue_id),
                    plan,
                );
            }
        }
    }
}

// =====================================================================================
// Helpers
// =====================================================================================

fn is_use_effect_hook_callee(id: &Identifier) -> bool {
    matches!(
        &id.type_,
        Type::Function(FunctionType { shape_id: Some(s), .. })
        if s == BUILT_IN_USE_EFFECT_HOOK_ID
    )
}

fn is_fire_callee_type(id: &Identifier) -> bool {
    matches!(
        &id.type_,
        Type::Function(FunctionType { shape_id: Some(s), .. })
        if s == BUILT_IN_FIRE_ID
    )
}

fn is_fire_import_binding(binding: &NonLocalBinding) -> bool {
    matches!(binding,
        NonLocalBinding::ImportSpecifier { module, imported, .. }
        if (module == "react" || module == "React") && imported == "fire"
    )
}

/// Locate an instruction by (block_id, instr_id). Returns `None` if
/// either lookup fails.
fn fetch_instruction(
    func: &HIRFunction,
    block_id: BlockId,
    instr_id: InstructionId,
) -> Option<&Instruction> {
    func.body.blocks.get(&block_id).and_then(|b| b.instructions.iter().find(|i| i.id == instr_id))
}

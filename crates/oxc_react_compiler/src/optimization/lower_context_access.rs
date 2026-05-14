/// Lower `const {x, y} = useContext(MyContext)` patterns to a selector-based form.
///
/// Port of `Optimization/LowerContextAccess.ts` (~308 LoC) from the React
/// Compiler. Gated by `Environment.config().lower_context_access` (a non-null
/// `ExternalFunction` names the runtime module + export to import as the
/// lowered callee, canonically
/// `{ source: "react-compiler-runtime", importSpecifierName: "useContext_withSelector" }`).
///
/// # Algorithm (mirrors upstream)
///
/// 1. **Collect**. Walk every instruction in the function body:
///    * Record `CallExpression`s whose callee is typed
///      `BuiltInUseContextHook` (so the operand is the hook itself, not the
///      returned context object). The lvalue identifier id is the destructure
///      source.
///    * For each `Destructure` whose source identifier id matches a recorded
///      `useContext` lvalue, extract the key list (only `ObjectPattern` with
///      named, non-computed, non-shorthand-only string keys; anything else
///      bails out the entire pass via `return`, exactly like TS).
/// 2. **Rewrite**. If both maps are non-empty:
///    * Lazily register the import via `ProgramContext::add_import_specifier`
///      so the local alias used by every emitted `LoadGlobal` matches the
///      module-level import declaration. The TS reference performs the same
///      `programContext.addImportSpecifier(...)` call (line 96-99 upstream).
///    * For every block, for every `useContext(...)` `CallExpression` whose
///      lvalue id is in `contextKeys`:
///        - Emit a fresh `LoadGlobal` whose binding is the lowered callee.
///        - Emit a fresh `FunctionExpression` whose body is
///          `(t0) => [t0.k1, t0.k2, ...]` (arrow, no context, anonymous,
///          parameter promoted to `#t{n}`). This becomes the selector arg.
///        - Rewire `value.callee` to the LoadGlobal's lvalue and append the
///          selector function as the last argument.
/// 3. **Cleanup**. After rewriting, re-mark instruction ids and re-run
///    `infer_types` so subsequent passes see the new instructions correctly
///    typed. Matches upstream lines 129-130.
///
/// # Pipeline placement
///
/// Runs between `TransformFire` and `OptimizePropsMethodCalls`, mirroring
/// upstream `Pipeline.ts:220-222`:
///
/// ```text
/// if (env.config.lowerContextAccess) {
///   lowerContextAccess(hir, env.config.lowerContextAccess);
/// }
///
/// optimizePropsMethodCalls(hir);
/// ```
///
/// # Production-surface gating
///
/// The synthetic selector `FunctionExpression`s have:
/// * `context: vec![]` — closureless
/// * `id: None` — anonymous
///
/// so they are picked up later by `optimization::outline_functions`, becoming
/// top-level `_temp`, `_temp2`, ... helpers (the same `_temp` you see in the
/// upstream fixture outputs).
use rustc_hash::FxHashSet;

use crate::{
    compiler_error::GENERATED_SOURCE,
    entrypoint::imports::ProgramContext,
    hir::{
        ArrayExpression, ArrayExpressionElement, BasicBlock, BlockId, BlockKind, BlockMap, CallArg,
        Destructure, FunctionExpressionType, FunctionExpressionValue, HIRFunction, Hir,
        IdentifierId, IdentifierName, Instruction, InstructionId, InstructionValue, LoadGlobal,
        LoadLocal, LoweredFunction, NonLocalBinding, ObjectPattern, ObjectPatternProperty,
        ObjectPropertyKey, Pattern, Place, PropertyLoad, ReactFunctionType, ReactiveParam,
        ReturnTerminal, ReturnVariant, Terminal,
        environment::{Environment, ExternalFunction},
        hir_builder::{create_temporary_place, mark_instruction_ids, reverse_postorder_blocks},
        object_shape::BUILT_IN_USE_CONTEXT_HOOK_ID,
        types::PropertyLiteral,
    },
};

/// Run the `LowerContextAccess` optimization on a HIR function.
///
/// Mutates `func` in place. Registers an import via `program_context` only when
/// at least one rewrite is actually performed (matches TS — the
/// `importLoweredContextCallee` ??= pattern at line 96 upstream).
pub fn lower_context_access(
    func: &mut HIRFunction,
    lowered_callee: &ExternalFunction,
    program_context: &mut ProgramContext,
) {
    // ---- Phase 1: collect useContext calls and their destructure keys. ----
    //
    // The two maps below mirror TS `contextAccess` and `contextKeys` (lines
    // 40-41). `contextAccess` is keyed by `CallExpression` lvalue id and
    // holds nothing of value beyond presence (TS stores the whole call but
    // never reads it back — we only ever check `.has(id)`). Use a HashSet.
    let mut context_access: FxHashSet<IdentifierId> = FxHashSet::default();
    // `contextKeys` is keyed by the SAME id (destructured-source = useContext
    // lvalue) and holds the key list. We also bail on:
    //   * `ArrayPattern` (returns null upstream)
    //   * any non-string-keyed/spread/method/computed property
    //   * any duplicate destructure of the same useContext lvalue
    let mut context_keys: rustc_hash::FxHashMap<IdentifierId, Vec<String>> =
        rustc_hash::FxHashMap::default();

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            match &instr.value {
                InstructionValue::CallExpression(call) if is_use_context_callee(&call.callee) => {
                    context_access.insert(instr.lvalue.identifier.id);
                }
                InstructionValue::Destructure(destructure) => {
                    let source_id = destructure.value.identifier.id;
                    if !context_access.contains(&source_id) {
                        continue;
                    }
                    let Some(keys) = get_context_keys(destructure) else {
                        // Upstream: `if (keys === null) return;`
                        // Any unsupported pattern bails out the WHOLE pass for
                        // this function.
                        return;
                    };
                    if context_keys.contains_key(&source_id) {
                        // TODO(gsn) upstream: support accessing context over
                        // multiple statements.
                        return;
                    }
                    context_keys.insert(source_id, keys);
                }
                _ => {}
            }
        }
    }

    // ---- Phase 2: bail out cheaply when nothing to rewrite. ----
    if context_access.is_empty() || context_keys.is_empty() {
        return;
    }

    // ---- Phase 3: register the import lazily, then rewrite per-block. ----
    //
    // The import is registered exactly once for the function — TS does
    // `importLoweredContextCallee ??=` inside the loop. We hoist it here
    // because Rust borrow checking + the iteration shape make a single
    // up-front registration cleaner.
    let import_alias = program_context.add_import_specifier(lowered_callee);

    let mut any_rewritten = false;
    let block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        // Decide once whether this block contains any rewrite candidate.
        // If not, skip the take/rebuild churn entirely. (TS achieves the
        // same lazy behavior by deferring `nextInstructions = …`.)
        let Some(block_ref) = func.body.blocks.get(&block_id) else { continue };
        let has_rewrite = block_ref.instructions.iter().any(|instr| {
            matches!(
                &instr.value,
                InstructionValue::CallExpression(call)
                    if is_use_context_callee(&call.callee)
                        && context_keys.contains_key(&instr.lvalue.identifier.id)
            )
        });
        if !has_rewrite {
            continue;
        }

        // Now mutate. Drain the original instructions and build a new
        // vector with the LoadGlobal + selector + (rewired) CallExpression
        // spliced in.
        //
        // The `unwrap` is guarded by the `block_ref` lookup immediately
        // above: we only reach this branch when the block is present in
        // the map, and we hold the only `&mut` reference on the function
        // body.
        let Some(block) = func.body.blocks.get_mut(&block_id) else { continue };
        let original = std::mem::take(&mut block.instructions);
        let mut next_instructions: Vec<Instruction> = Vec::with_capacity(original.len() + 4);
        for instr in original {
            let needs_rewrite = matches!(
                &instr.value,
                InstructionValue::CallExpression(call)
                    if is_use_context_callee(&call.callee)
                        && context_keys.contains_key(&instr.lvalue.identifier.id)
            );

            if needs_rewrite {
                let keys = &context_keys[&instr.lvalue.identifier.id];
                let load_global_instr =
                    emit_load_lowered_context_callee(&mut func.env, lowered_callee, &import_alias);
                let lowered_callee_place = load_global_instr.lvalue.clone();
                next_instructions.push(load_global_instr);

                let selector_fn_instr = emit_selector_fn(&mut func.env, keys);
                let selector_place = selector_fn_instr.lvalue.clone();
                next_instructions.push(selector_fn_instr);

                // Rewire the call: callee = lowered LoadGlobal lvalue,
                // append selector arg. Reuse the existing `instr` to keep
                // its `lvalue` (the destructure source) stable.
                let mut new_instr = instr;
                if let InstructionValue::CallExpression(call) = &mut new_instr.value {
                    call.callee = lowered_callee_place;
                    call.args.push(CallArg::Place(selector_place));
                }
                next_instructions.push(new_instr);
                any_rewritten = true;
            } else {
                next_instructions.push(instr);
            }
        }
        block.instructions = next_instructions;
    }

    // ---- Phase 4: cleanup. ----
    if any_rewritten {
        mark_instruction_ids(&mut func.body);
        // Upstream calls `inferTypes(fn)` here (line 130). The Rust port has
        // an `infer_types` pass — re-run it so the newly-emitted
        // `LoadGlobal` and `FunctionExpression` lvalues receive the correct
        // types and so the `useContext_withSelector(...)` callee type is
        // refreshed (it is now a generic external function call, not the
        // built-in `BuiltInUseContextHook`).
        let _ = crate::type_inference::infer_types::infer_types(func);
    }
}

/// Returns true iff `callee.identifier.type_` is a `Function` typed
/// `BuiltInUseContextHook`. Mirrors upstream `isUseContextHookType(id)`
/// (`HIR/HIR.ts` lines 1979-1983).
fn is_use_context_callee(callee: &Place) -> bool {
    matches!(
        &callee.identifier.type_,
        crate::hir::types::Type::Function(f)
            if f.shape_id.as_deref() == Some(BUILT_IN_USE_CONTEXT_HOOK_ID)
    )
}

/// Extract the (string) keys from a destructuring pattern, or return `None`
/// if the pattern shape is unsupported. Mirrors upstream `getContextKeys`
/// (lines 153-178).
///
/// Supported:
///   `const {foo, bar} = ...` → `["foo", "bar"]`
/// Unsupported (returns `None`, bails out the whole pass):
///   * `ArrayPattern` (`const [foo] = ...`)
///   * spread (`const {...rest} = ...`)
///   * computed keys (`const {[expr]: foo} = ...`)
///   * non-named identifier keys (shouldn't occur in well-formed object
///     patterns, but guarded for parity)
fn get_context_keys(destructure: &Destructure) -> Option<Vec<String>> {
    let Pattern::Object(obj) = &destructure.lvalue.pattern else {
        // TS line 158-160: ArrayPattern returns null.
        return None;
    };
    extract_object_keys(obj)
}

fn extract_object_keys(obj: &ObjectPattern) -> Option<Vec<String>> {
    let mut keys: Vec<String> = Vec::with_capacity(obj.properties.len());
    for prop in &obj.properties {
        match prop {
            ObjectPatternProperty::Property(p) => {
                // Upstream guards (line 164-172):
                //   place.kind !== 'ObjectProperty' || place.type !== 'property'
                //   || place.key.kind !== 'identifier'
                //   || place.place.identifier.name === null
                //   || place.place.identifier.name.kind !== 'named'
                //
                // Mapping to Rust:
                //   * `ObjectPatternProperty::Property(_)` covers "kind ==
                //     ObjectProperty". `Spread(_)` is the other branch.
                //   * The Rust `ObjectProperty` doesn't model "type !=
                //     'property'" (methods don't appear in destructure
                //     patterns), but we still gate on
                //     `property_type == Property` for parity.
                //   * `ObjectPropertyKey::Identifier` is the identifier-key
                //     case; `String`/`Number`/`Computed` are unsupported.
                if p.property_type != crate::hir::ObjectPropertyType::Property {
                    return None;
                }
                let key_name = match &p.key {
                    ObjectPropertyKey::Identifier(name) => name.clone(),
                    _ => return None,
                };
                // Bind value must be a named identifier (mirrors
                // `place.identifier.name.kind !== 'named'` upstream).
                match &p.place.identifier.name {
                    Some(IdentifierName::Named(_)) => {}
                    _ => return None,
                }
                keys.push(key_name);
            }
            ObjectPatternProperty::Spread(_) => return None,
        }
    }
    Some(keys)
}

/// Emit `const _tmp = useContext_withSelector;` as a `LoadGlobal`
/// instruction. The `binding` carries the local alias produced by
/// `ProgramContext::add_import_specifier`, so codegen renders the correct
/// identifier (matching whatever import declaration the program-level
/// emitter inserts).
///
/// Mirrors upstream `emitLoadLoweredContextCallee` (lines 134-151). Upstream
/// stores the `NonLocalImportSpecifier` directly — in the Rust port we
/// reconstruct the equivalent `NonLocalBinding::ImportSpecifier` from the
/// alias + the user-configured external function.
fn emit_load_lowered_context_callee(
    env: &mut Environment,
    lowered_callee: &ExternalFunction,
    import_alias: &str,
) -> Instruction {
    let lvalue = create_temporary_place(env, GENERATED_SOURCE);
    Instruction {
        id: InstructionId(0),
        loc: GENERATED_SOURCE,
        lvalue,
        value: InstructionValue::LoadGlobal(LoadGlobal {
            binding: NonLocalBinding::ImportSpecifier {
                name: import_alias.to_string(),
                module: lowered_callee.source.clone(),
                imported: lowered_callee.import_specifier_name.clone(),
            },
            loc: GENERATED_SOURCE,
        }),
        effects: None,
    }
}

/// Emit an anonymous arrow selector function:
///
/// ```text
///   (t0) => {
///     return [t0.k1, t0.k2, ...]; // ArrayExpression
///   }
/// ```
///
/// The `t0` parameter has its temporary identifier promoted (`#t{n}`) so
/// codegen prints it as `t0` (or `t1`, ...) rather than inlining. The body
/// is a single block with:
///   * For each key: `LoadLocal t0` then `PropertyLoad object.key`.
///   * One `ArrayExpression` collecting the loaded places.
///   * `Terminal::Return` of the array lvalue.
///
/// The function has `context: []` and `id: None` so downstream
/// `optimization::outline_functions` will outline it as a top-level
/// `_temp`/`_temp2`/... helper, exactly like upstream's emitted output.
///
/// Mirrors upstream `emitSelectorFn` (lines 219-291). Upstream additionally
/// runs `reversePostorderBlocks` + `markInstructionIds` + `enterSSA` +
/// `inferTypes` on the synthesized function; the Rust port mirrors that
/// inline so the inner HIR is in a state subsequent passes expect.
fn emit_selector_fn(env: &mut Environment, keys: &[String]) -> Instruction {
    // Synthesize `(t0)` param. Promote it so codegen renders the temporary
    // as `t0` (a real binding) rather than inlining it.
    let mut obj = create_temporary_place(env, GENERATED_SOURCE);
    let decl_id = obj.identifier.declaration_id.0;
    obj.identifier.name = Some(IdentifierName::Promoted(format!("#t{decl_id}")));

    let mut instructions: Vec<Instruction> = Vec::with_capacity(keys.len() * 2 + 1);
    let mut element_places: Vec<Place> = Vec::with_capacity(keys.len());

    for key in keys {
        // LoadLocal `t0`
        let load_lvalue = create_temporary_place(env, GENERATED_SOURCE);
        let load_local_instr = Instruction {
            id: InstructionId(0),
            loc: GENERATED_SOURCE,
            lvalue: load_lvalue.clone(),
            value: InstructionValue::LoadLocal(LoadLocal {
                place: obj.clone(),
                loc: GENERATED_SOURCE,
            }),
            effects: None,
        };
        instructions.push(load_local_instr);

        // PropertyLoad `object.key`
        let prop_lvalue = create_temporary_place(env, GENERATED_SOURCE);
        let prop_instr = Instruction {
            id: InstructionId(0),
            loc: GENERATED_SOURCE,
            lvalue: prop_lvalue.clone(),
            value: InstructionValue::PropertyLoad(PropertyLoad {
                object: load_lvalue,
                property: PropertyLiteral::String(key.clone()),
                optional: false,
                loc: GENERATED_SOURCE,
            }),
            effects: None,
        };
        instructions.push(prop_instr);

        element_places.push(prop_lvalue);
    }

    // ArrayExpression `[...elements]`
    let array_lvalue = create_temporary_place(env, GENERATED_SOURCE);
    let array_instr = Instruction {
        id: InstructionId(0),
        loc: GENERATED_SOURCE,
        lvalue: array_lvalue.clone(),
        value: InstructionValue::ArrayExpression(ArrayExpression {
            elements: element_places.into_iter().map(ArrayExpressionElement::Place).collect(),
            loc: GENERATED_SOURCE,
        }),
        effects: None,
    };
    instructions.push(array_instr);

    let block_id = env.next_block_id();
    let block = BasicBlock {
        kind: BlockKind::Block,
        id: block_id,
        instructions,
        terminal: Terminal::Return(ReturnTerminal {
            id: InstructionId(0),
            value: array_lvalue,
            return_variant: ReturnVariant::Explicit,
            effects: None,
            loc: GENERATED_SOURCE,
        }),
        preds: FxHashSet::default(),
        phis: Vec::new(),
    };

    let mut blocks = BlockMap::default();
    blocks.insert(block_id, block);

    // Allocate the selector's return place from the OUTER env BEFORE the
    // clone into `inner.env`. If we allocated this from the outer env after
    // the clone (or from the inner env after the clone but before SSA),
    // `enter_ssa` would observe an `inner.env` whose counter has not been
    // advanced past `returns.identifier.id`, and the SSA builder's first
    // fresh allocation could collide with the return place. The post-SSA
    // IdentifierId uniqueness invariant must hold across params, returns,
    // body lvalues, and the outer function-expression lvalue.
    let returns_place = create_temporary_place(env, GENERATED_SOURCE);

    let mut inner = HIRFunction {
        loc: GENERATED_SOURCE,
        id: None,
        name_hint: None,
        fn_type: ReactFunctionType::Other,
        // Clone AFTER all pre-SSA places (param, body lvalues, returns) have
        // been allocated from the outer env so `inner.env` starts with a
        // counter strictly greater than every existing IdentifierId in the
        // synthesized function.
        env: env.clone(),
        params: vec![ReactiveParam::Place(obj)],
        returns: returns_place,
        context: Vec::new(),
        body: Hir { entry: block_id, blocks },
        generator: false,
        is_async: false,
        directives: Vec::new(),
        aliasing_effects: None,
    };

    // Mirror upstream lines 269-272: reversePostorderBlocks +
    // markInstructionIds + enterSSA + inferTypes. The inner HIR must be in
    // a state that subsequent compiler passes (in particular function
    // outlining + reactive scope building) expect — sequential
    // instruction ids and SSA-form locals.
    reverse_postorder_blocks(&mut inner.body);
    mark_instruction_ids(&mut inner.body);
    // Best-effort: errors here are surfaced via the same `recordError`
    // channel the rest of the pipeline uses. The synthesized function is
    // closed (no captures, no labels, no try/catch), so SSA is essentially
    // a no-op other than allocating fresh DeclarationIds.
    if let Err(err) = crate::ssa::enter_ssa::enter_ssa(&mut inner, env) {
        // Should never happen given the trivial structure, but record the
        // error rather than panic so the outer compile path can surface it.
        env.record_errors(Err(err));
    }
    let _ = crate::type_inference::infer_types::infer_types(&mut inner);

    // After SSA the inner function's environment counters have advanced.
    // Sync them back to the outer environment so subsequent allocations in
    // the OUTER function do not collide with SSA-allocated ids inside the
    // inner function. Matches the post-`enter_ssa` sync done in
    // `entrypoint/pipeline.rs` for the main function.
    env.advance_counters_past(&inner.env);

    // Debug-only invariant: after SSA, the IdentifierIds attached to the
    // selector's param, return, body lvalues, and the synthesized function
    // expression's lvalue must all be distinct. Catches counter-skew bugs
    // like the one this fixed (selector returns sharing an id with the
    // first SSA-allocated param) before they manifest as alias/effect
    // analysis confusions downstream.
    let fn_lvalue = create_temporary_place(env, GENERATED_SOURCE);
    debug_assert!(
        selector_identifier_ids_unique(&inner, &fn_lvalue),
        "emit_selector_fn produced colliding IdentifierIds after enter_ssa"
    );
    Instruction {
        id: InstructionId(0),
        loc: GENERATED_SOURCE,
        lvalue: fn_lvalue,
        value: InstructionValue::FunctionExpression(FunctionExpressionValue {
            name: None,
            name_hint: None,
            lowered_func: LoweredFunction { func: Box::new(inner) },
            expression_type: FunctionExpressionType::ArrowFunctionExpression,
            loc: GENERATED_SOURCE,
        }),
        effects: None,
    }
}

/// Debug-only invariant: collect every IdentifierId attached to the
/// synthesized selector function (params, returns, body instruction
/// lvalues, terminal return value) plus the outer function-expression
/// lvalue, and confirm they are pairwise distinct. After `enter_ssa` this
/// must hold; downstream alias/effect/scope analyses key on `IdentifierId`
/// and a collision between the selector's parameter and its array return
/// would conflate two semantically unrelated locations.
///
/// Only invoked under `debug_assert!`; release builds skip this entirely.
fn selector_identifier_ids_unique(inner: &HIRFunction, fn_lvalue: &Place) -> bool {
    let mut ids: FxHashSet<IdentifierId> = FxHashSet::default();

    for param in &inner.params {
        let pid = match param {
            ReactiveParam::Place(p) => p.identifier.id,
            ReactiveParam::Spread(s) => s.place.identifier.id,
        };
        if !ids.insert(pid) {
            return false;
        }
    }
    if !ids.insert(inner.returns.identifier.id) {
        return false;
    }
    for block in inner.body.blocks.values() {
        for instr in &block.instructions {
            if !ids.insert(instr.lvalue.identifier.id) {
                return false;
            }
        }
        // Terminal return value is a USE of an earlier lvalue, not a fresh
        // definition; intentionally not added to the uniqueness set.
    }
    if !ids.insert(fn_lvalue.identifier.id) {
        return false;
    }
    true
}

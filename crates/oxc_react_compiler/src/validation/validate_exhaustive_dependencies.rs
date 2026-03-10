/// Validate exhaustive dependencies for useMemo/useCallback/useEffect.
///
/// Port of `Validation/ValidateExhaustiveDependencies.ts` from the React Compiler.
///
/// Validates that memoization hooks (useMemo, useCallback) have correct
/// dependency arrays, and that effect hooks (useEffect, useLayoutEffect)
/// have exhaustive dependency arrays. This is the compiler's version of
/// the `react-hooks/exhaustive-deps` ESLint rule.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    compiler_error::{
        CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory,
        GENERATED_SOURCE, SourceLocation,
    },
    hir::{
        BlockId, CallArg, DependencyPath, DependencyPathEntry, Effect, HIRFunction, Identifier,
        IdentifierId, IdentifierName, InstructionKind, InstructionValue, ManualMemoDependency,
        ManualMemoDependencyRoot, Place, Terminal,
        environment::ExhaustiveEffectDepsMode,
        object_shape::{
            BUILT_IN_EFFECT_EVENT_ID, BUILT_IN_USE_EFFECT_HOOK_ID,
            BUILT_IN_USE_INSERTION_EFFECT_HOOK_ID, BUILT_IN_USE_LAYOUT_EFFECT_HOOK_ID,
        },
        types::{FunctionType, ObjectType, Type},
        visitors::{
            each_instruction_lvalue, each_instruction_value_lvalue, each_instruction_value_operand,
            each_terminal_operand,
        },
    },
};

/// Validate that dependencies are exhaustive and not extraneous.
///
/// # Errors
/// Returns a `CompilerError` if dependency arrays are incomplete or contain extraneous values.
pub fn validate_exhaustive_dependencies(func: &mut HIRFunction) -> Result<(), CompilerError> {
    let reactive = collect_reactive_identifiers(func);
    let mut temporaries: FxHashMap<IdentifierId, Temporary> = FxHashMap::default();

    // Initialize params as temporaries
    for param in &func.params {
        let place = get_param_place(param);
        temporaries.insert(
            place.identifier.id,
            Temporary::Local(LocalDep {
                identifier: place.identifier.clone(),
                path: Vec::new(),
                context: false,
                loc: place.loc,
            }),
        );
    }

    let mut error = CompilerError::new();
    let mut invalid_memo_ids: FxHashSet<u32> = FxHashSet::default();

    // Run the collection pass with memoization callbacks
    collect_dependencies_with_memos(
        func,
        &mut temporaries,
        &reactive,
        &mut error,
        &func.env,
        &mut invalid_memo_ids,
    );

    // Mark StartMemoize instructions that had invalid deps
    if !invalid_memo_ids.is_empty() {
        for block in func.body.blocks.values_mut() {
            for instr in &mut block.instructions {
                if let InstructionValue::StartMemoize(ref mut v) = instr.value
                    && invalid_memo_ids.contains(&v.manual_memo_id) {
                        v.has_invalid_deps = true;
                    }
            }
        }
    }

    error.into_result()
}

fn get_param_place(param: &crate::hir::ReactiveParam) -> &Place {
    match param {
        crate::hir::ReactiveParam::Place(p) => p,
        crate::hir::ReactiveParam::Spread(s) => &s.place,
    }
}

// =====================================================================================
// Types
// =====================================================================================

#[derive(Debug, Clone)]
struct LocalDep {
    identifier: Identifier,
    path: DependencyPath,
    context: bool,
    loc: SourceLocation,
}

#[derive(Debug, Clone)]
enum Temporary {
    Global { binding_name: String },
    Local(LocalDep),
    Aggregate { dependencies: Vec<Temporary>, loc: Option<SourceLocation> },
}

// =====================================================================================
// Core algorithm
// =====================================================================================

fn collect_reactive_identifiers(func: &HIRFunction) -> FxHashSet<IdentifierId> {
    let mut reactive = FxHashSet::default();
    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            for lvalue in each_instruction_lvalue(instr) {
                if lvalue.reactive {
                    reactive.insert(lvalue.identifier.id);
                }
            }
            for operand in each_instruction_value_operand(&instr.value) {
                if operand.reactive {
                    reactive.insert(operand.identifier.id);
                }
            }
        }
        for operand in each_terminal_operand(&block.terminal) {
            if operand.reactive {
                reactive.insert(operand.identifier.id);
            }
        }
    }
    reactive
}

fn is_use_ref_type(identifier: &Identifier) -> bool {
    matches!(
        &identifier.type_,
        Type::Object(ObjectType { shape_id: Some(id) }) if id == "BuiltInUseRefId"
    )
}

/// Returns `true` if the identifier is a useEffect / useLayoutEffect / useInsertionEffect hook.
fn is_effect_hook(identifier: &Identifier) -> bool {
    matches!(
        &identifier.type_,
        Type::Function(FunctionType { shape_id: Some(id), .. })
        if id == BUILT_IN_USE_EFFECT_HOOK_ID
            || id == BUILT_IN_USE_LAYOUT_EFFECT_HOOK_ID
            || id == BUILT_IN_USE_INSERTION_EFFECT_HOOK_ID
    )
}

/// Port of `findOptionalPlaces` from `ValidateExhaustiveDeps.ts`.
///
/// Builds a map from `IdentifierId` to `bool` indicating whether that identifier
/// is part of an optional chain (i.e. `?.`). This is used when constructing
/// dependency paths so that `x?.y` produces `{optional: true, property: "y"}`
/// instead of `{optional: false, property: "y"}`.
fn find_optional_places(func: &HIRFunction) -> FxHashMap<IdentifierId, bool> {
    let mut optionals: FxHashMap<IdentifierId, bool> = FxHashMap::default();
    let mut visited: FxHashSet<BlockId> = FxHashSet::default();

    for block in func.body.blocks.values() {
        if visited.contains(&block.id) {
            continue;
        }
        if let Terminal::Optional(optional_terminal) = &block.terminal {
            visited.insert(block.id);
            let fallthrough = optional_terminal.fallthrough;
            let mut test_block_id = optional_terminal.test;
            let mut queue: Vec<Option<bool>> = vec![Some(optional_terminal.optional)];

            loop {
                let test_block = &func.body.blocks[&test_block_id];
                visited.insert(test_block.id);

                match &test_block.terminal {
                    Terminal::Branch(branch) => {
                        // invariant: queue should have a value for each branch
                        let Some(is_optional) = queue.pop() else {
                            break;
                        };
                        if let Some(is_opt) = is_optional {
                            optionals.insert(branch.test.identifier.id, is_opt);
                        }
                        if branch.fallthrough == fallthrough {
                            // Found the end of the optional chain
                            let consequent_block = &func.body.blocks[&branch.consequent];
                            if let Some(last) = consequent_block.instructions.last()
                                && let InstructionValue::StoreLocal(store) = &last.value
                                && let Some(is_opt) = is_optional
                            {
                                optionals.insert(store.value.identifier.id, is_opt);
                            }
                            break;
                        }
                        test_block_id = branch.fallthrough;
                    }
                    Terminal::Optional(inner_opt) => {
                        queue.push(Some(inner_opt.optional));
                        test_block_id = inner_opt.test;
                    }
                    Terminal::Logical(logical) => {
                        queue.push(None);
                        test_block_id = logical.test;
                    }
                    Terminal::Ternary(ternary) => {
                        queue.push(None);
                        test_block_id = ternary.test;
                    }
                    Terminal::Sequence(seq) => {
                        // Don't push to queue - no corresponding branch terminal
                        test_block_id = seq.block;
                    }
                    Terminal::MaybeThrow(mt) => {
                        test_block_id = mt.continuation;
                    }
                    _ => {
                        break;
                    }
                }
            }
        }
    }

    optionals
}

fn add_dependency(
    dep: &Temporary,
    dependencies: &mut Vec<Temporary>,
    locals: &FxHashSet<IdentifierId>,
) {
    match dep {
        Temporary::Aggregate { dependencies: inner_deps, .. } => {
            for d in inner_deps {
                add_dependency(d, dependencies, locals);
            }
        }
        Temporary::Global { .. } => {
            dependencies.push(dep.clone());
        }
        Temporary::Local(local) => {
            if !locals.contains(&local.identifier.id) {
                dependencies.push(dep.clone());
            }
        }
    }
}

fn visit_candidate_dependency(
    place: &Place,
    temporaries: &FxHashMap<IdentifierId, Temporary>,
    dependencies: &mut Vec<Temporary>,
    locals: &FxHashSet<IdentifierId>,
) {
    if let Some(dep) = temporaries.get(&place.identifier.id) {
        add_dependency(dep, dependencies, locals);
    }
}

/// Collect dependencies and handle StartMemoize/FinishMemoize for top-level.
fn collect_dependencies_with_memos(
    func: &HIRFunction,
    temporaries: &mut FxHashMap<IdentifierId, Temporary>,
    reactive: &FxHashSet<IdentifierId>,
    errors: &mut CompilerError,
    env: &crate::hir::environment::Environment,
    invalid_memo_ids: &mut FxHashSet<u32>,
) {
    let optionals = find_optional_places(func);
    let mut locals: FxHashSet<IdentifierId> = FxHashSet::default();
    let mut dependencies: Vec<Temporary> = Vec::new();
    let mut start_memo: Option<crate::hir::StartMemoize> = None;

    for block in func.body.blocks.values() {
        // Phi nodes
        process_phi_nodes(block, temporaries);

        for instr in &block.instructions {
            let lvalue_id = instr.lvalue.identifier.id;

            process_instruction(
                &instr.value,
                lvalue_id,
                instr,
                temporaries,
                &mut locals,
                &mut dependencies,
                reactive,
                errors,
                &optionals,
            );

            // Handle effect hooks (useEffect, useLayoutEffect, useInsertionEffect)
            let effect_mode = env.config.validate_exhaustive_effect_dependencies;
            if effect_mode != ExhaustiveEffectDepsMode::Off {
                handle_effect_hook_call(&instr.value, temporaries, reactive, errors, effect_mode);
            }

            // Handle memoization callbacks
            if let InstructionValue::StartMemoize(v) = &instr.value {
                start_memo = Some(v.clone());
                dependencies.clear();
                locals.clear();
            }
            if let InstructionValue::FinishMemoize(v) = &instr.value {
                if env.config.validate_exhaustive_memoization_dependencies
                    && let Some(ref start) = start_memo
                {
                    visit_candidate_dependency(&v.decl, temporaries, &mut dependencies, &locals);

                    let manual = start.deps.as_deref().unwrap_or(&[]);
                    if let Some(diagnostic) = validate_dependencies(
                        dependencies.clone(),
                        manual,
                        reactive,
                        ErrorCategory::MemoDependencies,
                        ExhaustiveEffectDepsMode::All,
                    ) {
                        errors.push_diagnostic(diagnostic);
                        invalid_memo_ids.insert(start.manual_memo_id);
                    }
                }
                dependencies.clear();
                locals.clear();
                start_memo = None;
            }
        }

        for operand in each_terminal_operand(&block.terminal) {
            // Skip operands that are part of optional chains — they are
            // intermediate values tracked by findOptionalPlaces.
            if optionals.contains_key(&operand.identifier.id) {
                continue;
            }
            visit_candidate_dependency(operand, temporaries, &mut dependencies, &locals);
        }
    }
}

/// Detect effect hook calls and validate their dependency arrays.
///
/// For calls like `useEffect(fn, deps)`, we extract the function's inferred
/// dependencies and the manually specified dependencies, then validate them.
fn handle_effect_hook_call(
    value: &InstructionValue,
    temporaries: &FxHashMap<IdentifierId, Temporary>,
    reactive: &FxHashSet<IdentifierId>,
    errors: &mut CompilerError,
    effect_mode: ExhaustiveEffectDepsMode,
) {
    let (receiver, args) = match value {
        InstructionValue::CallExpression(call) => (&call.callee, &call.args),
        InstructionValue::MethodCall(method) => (&method.property, &method.args),
        _ => return,
    };

    if !is_effect_hook(&receiver.identifier) {
        return;
    }

    // Effect hooks expect (fn, deps) — we need both arguments
    if args.len() < 2 {
        return;
    }

    let fn_place = match &args[0] {
        CallArg::Place(p) => p,
        CallArg::Spread(_) => return,
    };
    let deps_place = match &args[1] {
        CallArg::Place(p) => p,
        CallArg::Spread(_) => return,
    };

    let fn_deps = temporaries.get(&fn_place.identifier.id);
    let manual_deps = temporaries.get(&deps_place.identifier.id);

    let (fn_deps, manual_deps) = match (fn_deps, manual_deps) {
        (
            Some(Temporary::Aggregate { dependencies: fn_deps, .. }),
            Some(Temporary::Aggregate { dependencies: manual_deps, loc: manual_loc }),
        ) => (fn_deps, (manual_deps, manual_loc)),
        _ => return,
    };

    // Convert manual deps (Temporary) to ManualMemoDependency format
    let manual_memo_deps: Vec<ManualMemoDependency> = manual_deps
        .0
        .iter()
        .filter_map(|dep| match dep {
            Temporary::Local(local) => Some(ManualMemoDependency {
                root: ManualMemoDependencyRoot::NamedLocal {
                    value: Place {
                        identifier: local.identifier.clone(),
                        effect: Effect::Read,
                        reactive: reactive.contains(&local.identifier.id),
                        loc: local.loc,
                    },
                    constant: false,
                },
                path: local.path.clone(),
                loc: local.loc,
            }),
            Temporary::Global { binding_name } => Some(ManualMemoDependency {
                root: ManualMemoDependencyRoot::Global { identifier_name: binding_name.clone() },
                path: Vec::new(),
                loc: GENERATED_SOURCE,
            }),
            Temporary::Aggregate { .. } => None,
        })
        .collect();

    if let Some(diagnostic) = validate_dependencies(
        fn_deps.clone(),
        &manual_memo_deps,
        reactive,
        ErrorCategory::EffectExhaustiveDependencies,
        effect_mode,
    ) {
        errors.push_diagnostic(diagnostic);
    }
}

fn process_phi_nodes(
    block: &crate::hir::BasicBlock,
    temporaries: &mut FxHashMap<IdentifierId, Temporary>,
) {
    for phi in &block.phis {
        let mut deps: Vec<Temporary> = Vec::new();
        for operand in phi.operands.values() {
            if let Some(dep) = temporaries.get(&operand.identifier.id) {
                match dep {
                    Temporary::Aggregate { dependencies: inner, .. } => {
                        deps.extend(inner.iter().cloned());
                    }
                    other => {
                        deps.push(other.clone());
                    }
                }
            }
        }
        if deps.is_empty() {
            continue;
        }
        if deps.len() == 1 {
            if let Some(first) = deps.into_iter().next() {
                temporaries.insert(phi.place.identifier.id, first);
            }
        } else {
            temporaries.insert(
                phi.place.identifier.id,
                Temporary::Aggregate { dependencies: deps, loc: None },
            );
        }
    }
}

fn process_instruction(
    value: &InstructionValue,
    lvalue_id: IdentifierId,
    instr: &crate::hir::Instruction,
    temporaries: &mut FxHashMap<IdentifierId, Temporary>,
    locals: &mut FxHashSet<IdentifierId>,
    dependencies: &mut Vec<Temporary>,
    reactive: &FxHashSet<IdentifierId>,
    errors: &mut CompilerError,
    optionals: &FxHashMap<IdentifierId, bool>,
) {
    match value {
        InstructionValue::LoadGlobal(v) => {
            temporaries.insert(
                lvalue_id,
                Temporary::Global { binding_name: v.binding.name().to_string() },
            );
        }
        InstructionValue::LoadContext(v) => {
            if let Some(temp) = temporaries.get(&v.place.identifier.id) {
                let temp = match temp {
                    Temporary::Local(local) => {
                        Temporary::Local(LocalDep { loc: v.place.loc, ..local.clone() })
                    }
                    other => other.clone(),
                };
                temporaries.insert(lvalue_id, temp);
                if locals.contains(&v.place.identifier.id) {
                    locals.insert(lvalue_id);
                }
            }
        }
        InstructionValue::LoadLocal(v) => {
            if let Some(temp) = temporaries.get(&v.place.identifier.id) {
                let temp = match temp {
                    Temporary::Local(local) => {
                        Temporary::Local(LocalDep { loc: v.place.loc, ..local.clone() })
                    }
                    other => other.clone(),
                };
                temporaries.insert(lvalue_id, temp);
                if locals.contains(&v.place.identifier.id) {
                    locals.insert(lvalue_id);
                }
            }
        }
        InstructionValue::DeclareLocal(v) => {
            let local = Temporary::Local(LocalDep {
                identifier: v.lvalue.place.identifier.clone(),
                path: Vec::new(),
                context: false,
                loc: v.lvalue.place.loc,
            });
            temporaries.insert(v.lvalue.place.identifier.id, local);
            locals.insert(v.lvalue.place.identifier.id);
        }
        InstructionValue::StoreLocal(v) => {
            if v.lvalue.place.identifier.name.is_none() {
                if let Some(temp) = temporaries.get(&v.value.identifier.id) {
                    let temp = temp.clone();
                    temporaries.insert(v.lvalue.place.identifier.id, temp);
                }
            } else {
                visit_candidate_dependency(&v.value, temporaries, dependencies, locals);
                if v.lvalue.kind != InstructionKind::Reassign {
                    let local = Temporary::Local(LocalDep {
                        identifier: v.lvalue.place.identifier.clone(),
                        path: Vec::new(),
                        context: false,
                        loc: v.lvalue.place.loc,
                    });
                    temporaries.insert(v.lvalue.place.identifier.id, local);
                    locals.insert(v.lvalue.place.identifier.id);
                }
            }
        }
        InstructionValue::DeclareContext(v) => {
            let local = Temporary::Local(LocalDep {
                identifier: v.lvalue_place.identifier.clone(),
                path: Vec::new(),
                context: true,
                loc: v.lvalue_place.loc,
            });
            temporaries.insert(v.lvalue_place.identifier.id, local);
        }
        InstructionValue::StoreContext(v) => {
            visit_candidate_dependency(&v.value, temporaries, dependencies, locals);
            if v.lvalue_kind != InstructionKind::Reassign {
                let local = Temporary::Local(LocalDep {
                    identifier: v.lvalue_place.identifier.clone(),
                    path: Vec::new(),
                    context: true,
                    loc: v.lvalue_place.loc,
                });
                temporaries.insert(v.lvalue_place.identifier.id, local);
                locals.insert(v.lvalue_place.identifier.id);
            }
        }
        InstructionValue::Destructure(v) => {
            visit_candidate_dependency(&v.value, temporaries, dependencies, locals);
            if v.lvalue.kind != InstructionKind::Reassign {
                for lvalue in each_instruction_value_lvalue(value) {
                    let local = Temporary::Local(LocalDep {
                        identifier: lvalue.identifier.clone(),
                        path: Vec::new(),
                        context: false,
                        loc: lvalue.loc,
                    });
                    temporaries.insert(lvalue.identifier.id, local);
                    locals.insert(lvalue.identifier.id);
                }
            }
        }
        InstructionValue::PropertyLoad(v) => {
            let is_numeric = matches!(&v.property, crate::hir::types::PropertyLiteral::Number(_));
            let is_ref_current = is_use_ref_type(&v.object.identifier)
                && matches!(&v.property, crate::hir::types::PropertyLiteral::String(s) if s == "current");
            if is_numeric || is_ref_current {
                visit_candidate_dependency(&v.object, temporaries, dependencies, locals);
            } else if let Some(Temporary::Local(local)) = temporaries.get(&v.object.identifier.id) {
                let optional = optionals.get(&v.object.identifier.id).copied().unwrap_or(false);
                let mut new_path = local.path.clone();
                new_path.push(DependencyPathEntry {
                    optional,
                    property: v.property.clone(),
                    loc: v.loc,
                });
                let new_local = Temporary::Local(LocalDep {
                    identifier: local.identifier.clone(),
                    path: new_path,
                    context: local.context,
                    loc: v.loc,
                });
                temporaries.insert(lvalue_id, new_local);
            }
        }
        InstructionValue::FunctionExpression(fe) => {
            let function_deps = collect_dependencies_inner(
                &fe.lowered_func.func,
                temporaries,
                reactive,
                errors,
                true,
            );
            temporaries.insert(lvalue_id, function_deps.clone());
            add_dependency(&function_deps, dependencies, locals);
        }
        InstructionValue::ObjectMethod(om) => {
            let function_deps = collect_dependencies_inner(
                &om.lowered_func.func,
                temporaries,
                reactive,
                errors,
                true,
            );
            temporaries.insert(lvalue_id, function_deps.clone());
            add_dependency(&function_deps, dependencies, locals);
        }
        InstructionValue::StartMemoize(_) | InstructionValue::FinishMemoize(_) => {
            // Handled by outer loop
        }
        InstructionValue::ArrayExpression(v) => {
            let mut array_deps: Vec<Temporary> = Vec::new();
            for item in &v.elements {
                let place = match item {
                    crate::hir::ArrayExpressionElement::Place(p) => Some(p),
                    crate::hir::ArrayExpressionElement::Spread(s) => Some(&s.place),
                    crate::hir::ArrayExpressionElement::Hole => None,
                };
                if let Some(place) = place {
                    if let Some(dep) = temporaries.get(&place.identifier.id) {
                        match dep {
                            Temporary::Aggregate { dependencies: inner, .. } => {
                                array_deps.extend(inner.iter().cloned());
                            }
                            other => {
                                array_deps.push(other.clone());
                            }
                        }
                    }
                    visit_candidate_dependency(place, temporaries, dependencies, locals);
                }
            }
            temporaries.insert(
                lvalue_id,
                Temporary::Aggregate { dependencies: array_deps, loc: Some(v.loc) },
            );
        }
        InstructionValue::CallExpression(_) | InstructionValue::MethodCall(_) => {
            for operand in each_instruction_value_operand(value) {
                if let InstructionValue::MethodCall(m) = value
                    && operand.identifier.id == m.property.identifier.id
                {
                    continue;
                }
                visit_candidate_dependency(operand, temporaries, dependencies, locals);
            }
        }
        _ => {
            for operand in each_instruction_value_operand(value) {
                visit_candidate_dependency(operand, temporaries, dependencies, locals);
            }
            for lvalue in each_instruction_lvalue(instr) {
                locals.insert(lvalue.identifier.id);
            }
        }
    }
}

/// Collect dependencies for function expressions (no memo callbacks).
fn collect_dependencies_inner(
    func: &HIRFunction,
    temporaries: &mut FxHashMap<IdentifierId, Temporary>,
    reactive: &FxHashSet<IdentifierId>,
    errors: &mut CompilerError,
    is_function_expression: bool,
) -> Temporary {
    let optionals = find_optional_places(func);
    let mut locals: FxHashSet<IdentifierId> = FxHashSet::default();

    if is_function_expression {
        for param in &func.params {
            let place = get_param_place(param);
            locals.insert(place.identifier.id);
        }
    }

    let mut dependencies: Vec<Temporary> = Vec::new();

    for block in func.body.blocks.values() {
        process_phi_nodes(block, temporaries);

        for instr in &block.instructions {
            let lvalue_id = instr.lvalue.identifier.id;
            process_instruction(
                &instr.value,
                lvalue_id,
                instr,
                temporaries,
                &mut locals,
                &mut dependencies,
                reactive,
                errors,
                &optionals,
            );
        }

        for operand in each_terminal_operand(&block.terminal) {
            // Skip operands that are part of optional chains.
            if optionals.contains_key(&operand.identifier.id) {
                continue;
            }
            visit_candidate_dependency(operand, temporaries, &mut dependencies, &locals);
        }
    }

    Temporary::Aggregate { dependencies, loc: None }
}

// =====================================================================================
// Dependency validation
// =====================================================================================

fn validate_dependencies(
    mut inferred: Vec<Temporary>,
    manual_dependencies: &[ManualMemoDependency],
    reactive: &FxHashSet<IdentifierId>,
    category: ErrorCategory,
    report_mode: ExhaustiveEffectDepsMode,
) -> Option<CompilerDiagnostic> {
    // Sort with full tie-breaking: name, path length, optionals, property names
    inferred.sort_by(|a, b| {
        let name_cmp = dep_name(a).cmp(&dep_name(b));
        if name_cmp != std::cmp::Ordering::Equal {
            return name_cmp;
        }
        let a_path = dep_path(a);
        let b_path = dep_path(b);
        // Shorter path first
        let len_cmp = a_path.len().cmp(&b_path.len());
        if len_cmp != std::cmp::Ordering::Equal {
            return len_cmp;
        }
        // Compare path elements: non-optionals before optionals, then property name
        for (ae, be) in a_path.iter().zip(b_path.iter()) {
            // Non-optional (false) before optional (true)
            let opt_cmp = ae.optional.cmp(&be.optional);
            if opt_cmp != std::cmp::Ordering::Equal {
                return opt_cmp;
            }
            let prop_cmp = ae.property.to_string().cmp(&be.property.to_string());
            if prop_cmp != std::cmp::Ordering::Equal {
                return prop_cmp;
            }
        }
        std::cmp::Ordering::Equal
    });

    // Deduplicate
    inferred.dedup_by(|a, b| is_equal_temporary(a, b));

    let mut matched: FxHashSet<usize> = FxHashSet::default();
    let mut global_matched: FxHashSet<usize> = FxHashSet::default();
    let mut missing: Vec<Temporary> = Vec::new();

    for inferred_dep in &inferred {
        match inferred_dep {
            Temporary::Global { binding_name } => {
                for (idx, manual_dep) in manual_dependencies.iter().enumerate() {
                    if let ManualMemoDependencyRoot::Global { identifier_name } = &manual_dep.root
                        && identifier_name == binding_name
                    {
                        matched.insert(idx);
                        global_matched.insert(idx);
                    }
                }
            }
            Temporary::Local(local) => {
                if is_effect_event_function_type(&local.identifier.type_) {
                    continue;
                }

                let mut has_match = false;
                for (idx, manual_dep) in manual_dependencies.iter().enumerate() {
                    if let ManualMemoDependencyRoot::NamedLocal { value, .. } = &manual_dep.root
                        && value.identifier.id == local.identifier.id
                        && (are_equal_paths(&manual_dep.path, &local.path)
                            || is_sub_path_ignoring_optionals(&manual_dep.path, &local.path))
                    {
                        has_match = true;
                        matched.insert(idx);
                    }
                }

                if has_match || is_optional_dependency(&local.identifier, reactive) {
                    continue;
                }

                missing.push(inferred_dep.clone());
            }
            Temporary::Aggregate { .. } => {}
        }
    }

    let mut extra: Vec<&ManualMemoDependency> = Vec::new();
    for (idx, dep) in manual_dependencies.iter().enumerate() {
        // Global deps that were matched are still "extra" (unnecessary) —
        // values declared outside a component don't need to be dependencies
        if matched.contains(&idx) && !global_matched.contains(&idx) {
            continue;
        }
        if let ManualMemoDependencyRoot::NamedLocal { constant: true, value } = &dep.root
            && !value.reactive
            && value.identifier.is_primitive_type()
        {
            continue;
        }
        extra.push(dep);
    }

    // Filter based on report mode
    let filtered_missing: Vec<Temporary> = match report_mode {
        ExhaustiveEffectDepsMode::ExtraOnly => Vec::new(),
        _ => missing,
    };
    let filtered_extra: Vec<&ManualMemoDependency> = match report_mode {
        ExhaustiveEffectDepsMode::MissingOnly => Vec::new(),
        _ => extra,
    };

    if filtered_missing.is_empty() && filtered_extra.is_empty() {
        return None;
    }

    let (reason, description) =
        create_diagnostic_message(category, &filtered_missing, &filtered_extra);

    let mut diagnostic = CompilerDiagnostic::create(category, reason, Some(description), None);

    for dep in &filtered_missing {
        diagnostic = diagnostic.with_detail(CompilerDiagnosticDetail::Error {
            loc: Some(dep_loc(dep)),
            message: Some(format!("Missing dependency `{}`", print_inferred_dependency(dep))),
        });
    }

    for dep in &filtered_extra {
        let dep_str = print_manual_memo_dependency(dep);
        let message = if let ManualMemoDependencyRoot::Global { .. } = &dep.root {
            format!(
                "Unnecessary dependency `{dep_str}`. Values declared outside of a \
                 component/hook should not be listed as dependencies as the component \
                 will not re-render if they change"
            )
        } else {
            // Check if the extra dep matches an inferred dep that is an effect event function
            let root = match &dep.root {
                ManualMemoDependencyRoot::NamedLocal { value, .. } => Some(value),
                ManualMemoDependencyRoot::Global { .. } => None,
            };
            let matching_inferred = root.and_then(|root_val| {
                inferred.iter().find(|inferred_dep| {
                    if let Temporary::Local(local) = inferred_dep {
                        local.identifier.id == root_val.identifier.id
                            && is_sub_path_ignoring_optionals(&local.path, &dep.path)
                    } else {
                        false
                    }
                })
            });

            if let Some(Temporary::Local(matched_local)) = matching_inferred {
                if is_effect_event_function_type(&matched_local.identifier.type_) {
                    format!(
                        "Functions returned from `useEffectEvent` must not be included \
                         in the dependency array. Remove `{dep_str}` from the dependencies."
                    )
                } else if !is_optional_dependency(&matched_local.identifier, reactive) {
                    format!(
                        "Overly precise dependency `{dep_str}`, use `{}` instead",
                        print_inferred_dependency(&Temporary::Local(matched_local.clone()))
                    )
                } else {
                    format!("Unnecessary dependency `{dep_str}`")
                }
            } else {
                format!("Unnecessary dependency `{dep_str}`")
            }
        };
        diagnostic = diagnostic.with_detail(CompilerDiagnosticDetail::Error {
            loc: Some(dep.loc),
            message: Some(message),
        });
    }

    // Add inferred dependencies hint
    let inferred_hint: Vec<String> = inferred
        .iter()
        .filter(|dep| {
            if let Temporary::Local(local) = dep {
                !is_optional_dependency(&local.identifier, reactive)
                    && !is_effect_event_function_type(&local.identifier.type_)
            } else {
                false
            }
        })
        .map(print_inferred_dependency)
        .collect();
    diagnostic = diagnostic.with_detail(CompilerDiagnosticDetail::Hint {
        message: format!("Inferred dependencies: `[{}]`", inferred_hint.join(", ")),
    });

    Some(diagnostic)
}

// =====================================================================================
// Helpers
// =====================================================================================

fn dep_name(dep: &Temporary) -> String {
    match dep {
        Temporary::Global { binding_name } => binding_name.clone(),
        Temporary::Local(local) => {
            if let Some(IdentifierName::Named(name)) = &local.identifier.name {
                name.clone()
            } else {
                format!("${}", local.identifier.id.0)
            }
        }
        Temporary::Aggregate { .. } => String::new(),
    }
}

fn dep_path(dep: &Temporary) -> &[DependencyPathEntry] {
    match dep {
        Temporary::Local(local) => &local.path,
        _ => &[],
    }
}

fn dep_loc(dep: &Temporary) -> SourceLocation {
    match dep {
        Temporary::Local(local) => local.loc,
        _ => crate::compiler_error::GENERATED_SOURCE,
    }
}

fn is_equal_temporary(a: &Temporary, b: &Temporary) -> bool {
    match (a, b) {
        (
            Temporary::Global { binding_name: a_name },
            Temporary::Global { binding_name: b_name },
        ) => a_name == b_name,
        (Temporary::Local(a_local), Temporary::Local(b_local)) => {
            a_local.identifier.id == b_local.identifier.id
                && are_equal_paths(&a_local.path, &b_local.path)
        }
        _ => false,
    }
}

fn are_equal_paths(a: &[DependencyPathEntry], b: &[DependencyPathEntry]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).all(|(ae, be)| ae.property == be.property && ae.optional == be.optional)
}

fn is_sub_path_ignoring_optionals(
    parent: &[DependencyPathEntry],
    child: &[DependencyPathEntry],
) -> bool {
    if parent.len() > child.len() {
        return false;
    }
    parent.iter().zip(child.iter()).all(|(pe, ce)| pe.property == ce.property)
}

fn is_effect_event_function_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Function(FunctionType { shape_id: Some(id), .. })
        if id == BUILT_IN_EFFECT_EVENT_ID
    )
}

fn is_optional_dependency(identifier: &Identifier, reactive: &FxHashSet<IdentifierId>) -> bool {
    !reactive.contains(&identifier.id)
        && (identifier.is_primitive_type() || is_stable_type(&identifier.type_))
}

fn is_stable_type(ty: &Type) -> bool {
    match ty {
        Type::Function(FunctionType { shape_id: Some(id), .. }) => matches!(
            id.as_str(),
            "BuiltInSetState"
                | "BuiltInSetActionState"
                | "BuiltInDispatch"
                | "BuiltInUseReducerDispatch"
                | "BuiltInStartTransition"
                | "BuiltInSetOptimistic"
        ),
        Type::Object(ObjectType { shape_id: Some(id) }) => id == "BuiltInUseRefId",
        _ => false,
    }
}

fn print_inferred_dependency(dep: &Temporary) -> String {
    match dep {
        Temporary::Global { binding_name } => binding_name.clone(),
        Temporary::Local(local) => {
            use std::fmt::Write;
            let name = if let Some(IdentifierName::Named(n)) = &local.identifier.name {
                n.clone()
            } else {
                format!("${}", local.identifier.id.0)
            };
            let mut path_str = name;
            for p in &local.path {
                let sep = if p.optional { "?." } else { "." };
                write!(path_str, "{sep}{}", p.property).unwrap();
            }
            path_str
        }
        Temporary::Aggregate { .. } => "[aggregate]".to_string(),
    }
}

fn print_manual_memo_dependency(dep: &ManualMemoDependency) -> String {
    use std::fmt::Write;
    let name = match &dep.root {
        ManualMemoDependencyRoot::Global { identifier_name } => identifier_name.clone(),
        ManualMemoDependencyRoot::NamedLocal { value, .. } => {
            if let Some(IdentifierName::Named(n)) = &value.identifier.name {
                n.clone()
            } else {
                format!("${}", value.identifier.id.0)
            }
        }
    };
    let mut path_str = name;
    for p in &dep.path {
        let sep = if p.optional { "?." } else { "." };
        write!(path_str, "{sep}{}", p.property).unwrap();
    }
    path_str
}

fn create_diagnostic_message(
    category: ErrorCategory,
    missing: &[Temporary],
    extra: &[&ManualMemoDependency],
) -> (String, String) {
    let has_missing = !missing.is_empty();
    let has_extra = !extra.is_empty();

    let kind = if has_missing && has_extra {
        "missing/extra"
    } else if has_missing {
        "missing"
    } else {
        "extra"
    };

    match category {
        ErrorCategory::MemoDependencies => {
            let reason = format!("Found {kind} memoization dependencies");
            let mut desc_parts = Vec::new();
            if has_missing {
                desc_parts.push(
                    "Missing dependencies can cause a value to update less often than it \
                     should, resulting in stale UI",
                );
            }
            if has_extra {
                desc_parts.push(
                    "Extra dependencies can cause a value to update more often than it \
                     should, resulting in performance problems such as excessive renders \
                     or effects firing too often",
                );
            }
            (reason, desc_parts.join(". "))
        }
        ErrorCategory::EffectExhaustiveDependencies => {
            let reason = format!("Found {kind} effect dependencies");
            let mut desc_parts = Vec::new();
            if has_missing {
                desc_parts.push(
                    "Missing dependencies can cause an effect to fire less often than it should",
                );
            }
            if has_extra {
                desc_parts.push(
                    "Extra dependencies can cause an effect to fire more often than it \
                     should, resulting in performance problems such as excessive renders \
                     and side effects",
                );
            }
            (reason, desc_parts.join(". "))
        }
        _ => (format!("Found {kind} dependencies"), String::new()),
    }
}

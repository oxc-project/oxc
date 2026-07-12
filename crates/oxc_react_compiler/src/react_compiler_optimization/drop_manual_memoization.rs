// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Removes manual memoization using `useMemo` and `useCallback` APIs.
//!
//! For useMemo: replaces `Call useMemo(fn, deps)` with `Call fn()`
//! For useCallback: replaces `Call useCallback(fn, deps)` with `LoadLocal fn`
//!
//! When validation flags are set, inserts `StartMemoize`/`FinishMemoize` markers.
//!
//! Analogous to TS `Inference/DropManualMemoization.ts`.

use std::mem::discriminant;

use cow_utils::CowUtils;
use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;

use oxc_diagnostics::OxcDiagnostic;

use crate::diagnostics::ErrorCategory;
use crate::react_compiler_hir::ArrayElement;
use crate::react_compiler_hir::DependencyPathEntry;
use crate::react_compiler_hir::Effect;
use crate::react_compiler_hir::EvaluationOrder;
use crate::react_compiler_hir::HirFunction;
use crate::react_compiler_hir::IdentifierId;
use crate::react_compiler_hir::IdentifierName;
use crate::react_compiler_hir::Instruction;
use crate::react_compiler_hir::InstructionId;
use crate::react_compiler_hir::InstructionValue;
use crate::react_compiler_hir::ManualMemoDependency;
use crate::react_compiler_hir::ManualMemoDependencyRoot;
use crate::react_compiler_hir::NonLocalBinding;
use crate::react_compiler_hir::Place;
use crate::react_compiler_hir::PlaceOrSpread;
use crate::react_compiler_hir::PropertyLiteral;
use crate::react_compiler_hir::Span;
use crate::react_compiler_hir::Terminal;
use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_lowering::create_temporary_place;
use crate::react_compiler_lowering::mark_instruction_ids;

// =============================================================================
// Types
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ManualMemoKind {
    UseMemo,
    UseCallback,
}

#[derive(Debug, Clone)]
struct ManualMemoCallee {
    kind: ManualMemoKind,
    /// InstructionId of the LoadGlobal or PropertyLoad that loaded the callee.
    load_instr_id: InstructionId,
}

struct IdentifierSidemap<'a> {
    /// Maps identifier id -> InstructionId of FunctionExpression instructions
    functions: FxHashSet<IdentifierId>,
    /// Maps identifier id -> ManualMemoCallee for useMemo/useCallback callees
    manual_memos: FxHashMap<IdentifierId, ManualMemoCallee>,
    /// Set of identifier ids that loaded 'React' global
    react: FxHashSet<IdentifierId>,
    /// Maps identifier id -> deps list info for array expressions
    maybe_deps_lists: FxHashMap<IdentifierId, MaybeDepsListInfo>,
    /// Maps identifier id -> ManualMemoDependency for dependency tracking
    maybe_deps: FxHashMap<IdentifierId, ManualMemoDependency<'a>>,
    /// Set of identifier ids that are results of optional chains
    optionals: FxHashSet<IdentifierId>,
}

#[derive(Debug, Clone)]
struct MaybeDepsListInfo {
    span: Option<Span>,
    deps: Vec<Place>,
}

struct ExtractedMemoArgs<'a> {
    fn_place: Place,
    deps_list: Option<Vec<ManualMemoDependency<'a>>>,
    deps_span: Option<Span>,
}

// =============================================================================
// Main pass
// =============================================================================

/// Drop manual memoization (useMemo/useCallback calls), replacing them
/// with direct invocations/references.
pub fn drop_manual_memoization<'a>(
    func: &mut HirFunction<'a>,
    env: &mut Environment<'a>,
) -> Result<(), OxcDiagnostic> {
    let is_validation_enabled = env.validate_preserve_existing_memoization_guarantees
        || env.validate_no_set_state_in_render
        || env.enable_preserve_existing_memoization_guarantees;

    let optionals = find_optional_places(func)?;
    let mut sidemap = IdentifierSidemap {
        functions: FxHashSet::default(),
        manual_memos: FxHashMap::default(),
        react: FxHashSet::default(),
        maybe_deps: FxHashMap::default(),
        maybe_deps_lists: FxHashMap::default(),
        optionals,
    };
    let mut next_manual_memo_id: u32 = 0;

    // Phase 1:
    // - Overwrite manual memoization CallExpression/MethodCall
    // - (if validation is enabled) collect manual memoization markers
    //
    // queued_inserts maps InstructionId -> new Instruction to insert after that instruction
    let mut queued_inserts: FxHashMap<InstructionId, Instruction<'a>> = FxHashMap::default();

    // Collect all block instruction lists up front to avoid borrowing func immutably
    // while needing to mutate it
    let all_block_instructions: Vec<Vec<InstructionId>> =
        func.body.blocks.values().map(|block| block.instructions.clone()).collect();

    for block_instructions in &all_block_instructions {
        for &instr_id in block_instructions {
            let instr = &func.instructions[instr_id.0 as usize];

            // Extract the identifier we need to look up, and whether it's a call/method
            let lookup_id = match &instr.value {
                InstructionValue::CallExpression { callee, .. } => Some(callee.identifier),
                InstructionValue::MethodCall { property, .. } => Some(property.identifier),
                _ => None,
            };

            let manual_memo = lookup_id.and_then(|id| sidemap.manual_memos.get(&id).cloned());

            if let Some(manual_memo) = manual_memo {
                process_manual_memo_call(
                    func,
                    env,
                    instr_id,
                    &manual_memo,
                    &mut sidemap,
                    is_validation_enabled,
                    &mut next_manual_memo_id,
                    &mut queued_inserts,
                );
            } else {
                collect_temporaries(func, env, instr_id, &mut sidemap);
            }
        }
    }

    // Phase 2: Insert manual memoization markers as needed
    if !queued_inserts.is_empty() {
        let mut has_changes = false;
        for block in func.body.blocks.values_mut() {
            let mut next_instructions: Option<Vec<InstructionId>> = None;
            for i in 0..block.instructions.len() {
                let instr_id = block.instructions[i];
                if let Some(insert_instr) = queued_inserts.remove(&instr_id) {
                    if next_instructions.is_none() {
                        next_instructions = Some(block.instructions[..i].to_vec());
                    }
                    let ni = next_instructions.as_mut().unwrap();
                    ni.push(instr_id);
                    // Add the new instruction to the flat table and get its InstructionId
                    let new_instr_id = InstructionId(func.instructions.len() as u32);
                    func.instructions.push(insert_instr);
                    ni.push(new_instr_id);
                } else if let Some(ni) = next_instructions.as_mut() {
                    ni.push(instr_id);
                }
            }
            if let Some(ni) = next_instructions {
                block.instructions = ni;
                has_changes = true;
            }
        }

        if has_changes {
            mark_instruction_ids(&mut func.body, &mut func.instructions);
        }
    }

    Ok(())
}

// =============================================================================
// Phase 1 helpers
// =============================================================================

#[allow(clippy::too_many_arguments)]
fn process_manual_memo_call<'a>(
    func: &mut HirFunction<'a>,
    env: &mut Environment<'a>,
    instr_id: InstructionId,
    manual_memo: &ManualMemoCallee,
    sidemap: &mut IdentifierSidemap<'a>,
    is_validation_enabled: bool,
    next_manual_memo_id: &mut u32,
    queued_inserts: &mut FxHashMap<InstructionId, Instruction<'a>>,
) {
    let instr = &func.instructions[instr_id.0 as usize];

    let memo_details = extract_manual_memoization_args(instr, manual_memo.kind, sidemap, env);

    let Some(memo_details) = memo_details else {
        return;
    };

    let ExtractedMemoArgs { fn_place, deps_list, deps_span } = memo_details;

    let span = func.instructions[instr_id.0 as usize].value.span().cloned();

    // Replace the instruction value with the memoization replacement
    let replacement = get_manual_memoization_replacement(&fn_place, span, manual_memo.kind);
    func.instructions[instr_id.0 as usize].value = replacement;

    if is_validation_enabled {
        // Bail out when we encounter manual memoization without inline function expressions
        if !sidemap.functions.contains(&fn_place.identifier) {
            let diag = ErrorCategory::UseMemo
                .diagnostic("Expected the first argument to be an inline function expression")
                .with_help(
                    "Expected the first argument to be an inline function expression".to_string(),
                )
                .with_labels(fn_place.span.map(|s| {
                    s.label(
                        "Expected the first argument to be an inline function expression"
                            .to_string(),
                    )
                }));
            env.record_diagnostic(diag);
            return;
        }

        let memo_decl: Place = if manual_memo.kind == ManualMemoKind::UseMemo {
            func.instructions[instr_id.0 as usize].lvalue.clone()
        } else {
            Place {
                identifier: fn_place.identifier,
                effect: Effect::Unknown,
                reactive: false,
                span: fn_place.span,
            }
        };

        let manual_memo_id = *next_manual_memo_id;
        *next_manual_memo_id += 1;

        let (start_marker, finish_marker) = make_manual_memoization_markers(
            &fn_place,
            env,
            deps_list,
            deps_span,
            &memo_decl,
            manual_memo_id,
        );

        queued_inserts.insert(manual_memo.load_instr_id, start_marker);
        queued_inserts.insert(instr_id, finish_marker);
    }
}

fn collect_temporaries<'a>(
    func: &HirFunction<'a>,
    env: &Environment<'a>,
    instr_id: InstructionId,
    sidemap: &mut IdentifierSidemap<'a>,
) {
    let instr = &func.instructions[instr_id.0 as usize];
    let lvalue_id = instr.lvalue.identifier;

    match &instr.value {
        InstructionValue::FunctionExpression { .. } => {
            sidemap.functions.insert(lvalue_id);
        }
        InstructionValue::LoadGlobal { binding, .. } => {
            let hook_name = get_hook_detection_name(binding);
            let mut detected = false;
            if let Some(name) = hook_name {
                if name == "useMemo" {
                    sidemap.manual_memos.insert(
                        lvalue_id,
                        ManualMemoCallee { kind: ManualMemoKind::UseMemo, load_instr_id: instr_id },
                    );
                    detected = true;
                } else if name == "useCallback" {
                    sidemap.manual_memos.insert(
                        lvalue_id,
                        ManualMemoCallee {
                            kind: ManualMemoKind::UseCallback,
                            load_instr_id: instr_id,
                        },
                    );
                    detected = true;
                }
            }
            if !detected && binding.name() == "React" {
                sidemap.react.insert(lvalue_id);
            }
        }
        InstructionValue::PropertyLoad { object, property, .. } => {
            if sidemap.react.contains(&object.identifier) {
                if let PropertyLiteral::String(prop_name) = property {
                    if prop_name == "useMemo" {
                        sidemap.manual_memos.insert(
                            lvalue_id,
                            ManualMemoCallee {
                                kind: ManualMemoKind::UseMemo,
                                load_instr_id: instr_id,
                            },
                        );
                    } else if prop_name == "useCallback" {
                        sidemap.manual_memos.insert(
                            lvalue_id,
                            ManualMemoCallee {
                                kind: ManualMemoKind::UseCallback,
                                load_instr_id: instr_id,
                            },
                        );
                    }
                }
            }
        }
        InstructionValue::ArrayExpression { elements, .. } => {
            // Check if all elements are Identifier (Place) - no spreads or holes
            let all_places: Option<Vec<Place>> = elements
                .iter()
                .map(|e| match e {
                    ArrayElement::Place(p) => Some(p.clone()),
                    _ => None,
                })
                .collect();

            if let Some(deps) = all_places {
                sidemap.maybe_deps_lists.insert(
                    lvalue_id,
                    MaybeDepsListInfo { span: instr.value.span().cloned(), deps },
                );
            }
        }
        _ => {}
    }

    let is_optional = sidemap.optionals.contains(&lvalue_id);
    let maybe_dep =
        collect_maybe_memo_dependencies(&instr.value, &sidemap.maybe_deps, is_optional, env);
    if let Some(dep) = maybe_dep {
        // For StoreLocal, also insert under the StoreLocal's lvalue place identifier,
        // matching the TS behavior where collectMaybeMemoDependencies inserts into
        // maybeDeps directly for StoreLocal's target variable.
        if let InstructionValue::StoreLocal { lvalue, .. } = &instr.value {
            sidemap.maybe_deps.insert(lvalue.place.identifier, dep.clone());
        }
        sidemap.maybe_deps.insert(lvalue_id, dep);
    }
}

// =============================================================================
// collectMaybeMemoDependencies
// =============================================================================

/// Collect loads from named variables and property reads into `maybe_deps`.
/// Returns the variable + property reads represented by the instruction value.
pub fn collect_maybe_memo_dependencies<'a>(
    value: &InstructionValue<'a>,
    maybe_deps: &FxHashMap<IdentifierId, ManualMemoDependency<'a>>,
    optional: bool,
    env: &Environment<'a>,
) -> Option<ManualMemoDependency<'a>> {
    match value {
        InstructionValue::LoadGlobal { binding, span, .. } => Some(ManualMemoDependency {
            root: ManualMemoDependencyRoot::Global { identifier_name: binding.name() },
            path: vec![],
            span: *span,
        }),
        InstructionValue::PropertyLoad { object, property, span, .. } => {
            maybe_deps.get(&object.identifier).map(|object_dep| ManualMemoDependency {
                root: object_dep.root.clone(),
                path: {
                    let mut path = object_dep.path.clone();
                    path.push(DependencyPathEntry { property: *property, optional, span: *span });
                    path
                },
                span: *span,
            })
        }
        InstructionValue::LoadLocal { place, .. } | InstructionValue::LoadContext { place, .. } => {
            if let Some(source) = maybe_deps.get(&place.identifier) {
                Some(source.clone())
            } else if matches!(
                &env.identifiers[place.identifier.0 as usize].name,
                Some(IdentifierName::Named(_))
            ) {
                Some(ManualMemoDependency {
                    root: ManualMemoDependencyRoot::NamedLocal {
                        value: place.clone(),
                        constant: false,
                    },
                    path: vec![],
                    span: place.span,
                })
            } else {
                None
            }
        }
        InstructionValue::StoreLocal { lvalue, value: val, .. } => {
            // Value blocks rely on StoreLocal to populate their return value.
            // We need to track these as optional property chains are valid in
            // source depslists
            let lvalue_id = lvalue.place.identifier;
            let rvalue_id = val.identifier;
            if let Some(aliased) = maybe_deps.get(&rvalue_id) {
                let lvalue_name = &env.identifiers[lvalue_id.0 as usize].name;
                if !matches!(lvalue_name, Some(IdentifierName::Named(_))) {
                    // Note: we can't insert into maybe_deps here since we only have
                    // a shared reference. The caller handles insertion.
                    return Some(aliased.clone());
                }
            }
            None
        }
        _ => None,
    }
}

// =============================================================================
// Replacement helpers
// =============================================================================

fn get_manual_memoization_replacement<'a>(
    fn_place: &Place,
    span: Option<Span>,
    kind: ManualMemoKind,
) -> InstructionValue<'a> {
    if kind == ManualMemoKind::UseMemo {
        // Replace with Call fn() - invoke the memo function directly
        InstructionValue::CallExpression { callee: fn_place.clone(), args: vec![], span }
    } else {
        // Replace with LoadLocal fn - just reference the function
        InstructionValue::LoadLocal {
            place: Place {
                identifier: fn_place.identifier,
                effect: Effect::Unknown,
                reactive: false,
                span,
            },
            span,
        }
    }
}

fn make_manual_memoization_markers<'a>(
    fn_expr: &Place,
    env: &mut Environment<'a>,
    deps_list: Option<Vec<ManualMemoDependency<'a>>>,
    deps_span: Option<Span>,
    memo_decl: &Place,
    manual_memo_id: u32,
) -> (Instruction<'a>, Instruction<'a>) {
    let start = Instruction {
        id: EvaluationOrder(0),
        lvalue: create_temporary_place(env, fn_expr.span),
        value: InstructionValue::StartMemoize {
            manual_memo_id,
            deps: deps_list,
            deps_span: Some(deps_span),
            has_invalid_deps: false,
            span: fn_expr.span,
        },
        span: fn_expr.span,
        effects: None,
    };
    let finish = Instruction {
        id: EvaluationOrder(0),
        lvalue: create_temporary_place(env, fn_expr.span),
        value: InstructionValue::FinishMemoize {
            manual_memo_id,
            decl: memo_decl.clone(),
            pruned: false,
            span: fn_expr.span,
        },
        span: fn_expr.span,
        effects: None,
    };
    (start, finish)
}

fn extract_manual_memoization_args<'a>(
    instr: &Instruction,
    kind: ManualMemoKind,
    sidemap: &IdentifierSidemap<'a>,
    env: &mut Environment,
) -> Option<ExtractedMemoArgs<'a>> {
    let args: &[PlaceOrSpread] = match &instr.value {
        InstructionValue::CallExpression { args, .. } => args,
        InstructionValue::MethodCall { args, .. } => args,
        _ => return None,
    };

    let kind_name = match kind {
        ManualMemoKind::UseMemo => "useMemo",
        ManualMemoKind::UseCallback => "useCallback",
    };

    // Get the first arg (fn)
    let fn_place = match args.first() {
        Some(PlaceOrSpread::Place(p)) => p.clone(),
        _ => {
            let span = instr.value.span().cloned();
            env.record_diagnostic(
                ErrorCategory::UseMemo
                    .diagnostic(format!("Expected a callback function to be passed to {kind_name}"))
                    .with_help(if kind == ManualMemoKind::UseCallback {
                        "The first argument to useCallback() must be a function to cache".to_string()
                    } else {
                        "The first argument to useMemo() must be a function that calculates a result to cache".to_string()
                    })
                    .with_labels(span.map(|s| {
                        s.label(if kind == ManualMemoKind::UseCallback {
                            "Expected a callback function".to_string()
                        } else {
                            "Expected a memoization function".to_string()
                        })
                    })),
            );
            return None;
        }
    };

    // Get the second arg (deps list), if present
    let deps_list_place = args.get(1);
    if deps_list_place.is_none() {
        return Some(ExtractedMemoArgs { fn_place, deps_list: None, deps_span: None });
    }

    let deps_list_id = match deps_list_place {
        Some(PlaceOrSpread::Place(p)) => Some(p.identifier),
        _ => None,
    };

    let maybe_deps_list = deps_list_id.and_then(|id| sidemap.maybe_deps_lists.get(&id));

    if maybe_deps_list.is_none() {
        let span = match deps_list_place {
            Some(PlaceOrSpread::Place(p)) => p.span,
            _ => instr.span,
        };
        env.record_diagnostic(
            ErrorCategory::UseMemo
                .diagnostic(format!(
                    "Expected the dependency list for {kind_name} to be an array literal"
                ))
                .with_help(format!(
                    "Expected the dependency list for {kind_name} to be an array literal"
                ))
                .with_labels(span.map(|s| {
                    s.label(format!(
                        "Expected the dependency list for {kind_name} to be an array literal"
                    ))
                })),
        );
        return None;
    }

    let deps_info = maybe_deps_list.unwrap();
    let mut deps_list: Vec<ManualMemoDependency<'a>> = Vec::new();
    for dep in &deps_info.deps {
        let maybe_dep = sidemap.maybe_deps.get(&dep.identifier);
        if let Some(d) = maybe_dep {
            deps_list.push(d.clone());
        } else {
            env.record_diagnostic(
                ErrorCategory::UseMemo
                    .diagnostic("Expected the dependency list to be an array of simple expressions (e.g. `x`, `x.y.z`, `x?.y?.z`)")
                    .with_help("Expected the dependency list to be an array of simple expressions (e.g. `x`, `x.y.z`, `x?.y?.z`)".to_string())
                    .with_labels(dep.span.map(|s| {
                        s.label("Expected the dependency list to be an array of simple expressions (e.g. `x`, `x.y.z`, `x?.y?.z`)".to_string())
                    })),
            );
        }
    }

    Some(ExtractedMemoArgs { fn_place, deps_list: Some(deps_list), deps_span: deps_info.span })
}

// =============================================================================
// findOptionalPlaces
// =============================================================================

fn find_optional_places(func: &HirFunction) -> Result<FxHashSet<IdentifierId>, OxcDiagnostic> {
    let mut optionals = FxHashSet::default();
    for block in func.body.blocks.values() {
        if let Terminal::Optional { optional: true, test, fallthrough, .. } = &block.terminal {
            let optional_fallthrough = *fallthrough;
            let mut test_block_id = *test;
            loop {
                let test_block = &func.body.blocks[&test_block_id];
                match &test_block.terminal {
                    Terminal::Branch { consequent, fallthrough, .. } => {
                        if *fallthrough == optional_fallthrough {
                            // Found it
                            let consequent_block = &func.body.blocks[consequent];
                            if let Some(&last_instr_id) = consequent_block.instructions.last() {
                                let last_instr = &func.instructions[last_instr_id.0 as usize];
                                if let InstructionValue::StoreLocal { value, .. } =
                                    &last_instr.value
                                {
                                    optionals.insert(value.identifier);
                                }
                            }
                            break;
                        } else {
                            test_block_id = *fallthrough;
                        }
                    }
                    Terminal::Optional { fallthrough, .. }
                    | Terminal::Logical { fallthrough, .. }
                    | Terminal::Sequence { fallthrough, .. }
                    | Terminal::Ternary { fallthrough, .. } => {
                        test_block_id = *fallthrough;
                    }
                    Terminal::MaybeThrow { continuation, .. } => {
                        test_block_id = *continuation;
                    }
                    other => {
                        // Invariant: unexpected terminal in optional
                        // In TS this throws CompilerError.invariant
                        return Err(ErrorCategory::Invariant.diagnostic(format!(
                            "Unexpected terminal kind in optional: {:?}",
                            discriminant(other)
                        )));
                    }
                }
            }
        }
    }
    Ok(optionals)
}

fn is_known_react_module(module: &str) -> bool {
    let lower = module.cow_to_lowercase();
    lower == "react" || lower == "react-dom"
}

/// Returns the name to use for useMemo/useCallback detection, matching the TS
/// behavior of `getGlobalDeclaration` + `getHookKindForType`.
///
/// - `Global`: use the binding name (matches globals.get(name) in TS)
/// - `ImportSpecifier` from known React module: use the `imported` name
/// - `ImportSpecifier` from unknown module: return None (TS returns a generic
///   custom hook type with hookKind 'Custom', not 'useMemo'/'useCallback')
/// - `ModuleLocal`: return None (same reason as above)
/// - `ImportDefault`/`ImportNamespace` from known React module: use the local name
/// - `ImportDefault`/`ImportNamespace` from unknown module: return None
fn get_hook_detection_name<'a>(binding: &NonLocalBinding<'a>) -> Option<&'a str> {
    match binding {
        NonLocalBinding::Global { name } => Some(name.as_str()),
        NonLocalBinding::ImportSpecifier { imported, module, .. } => {
            if is_known_react_module(module) { Some(imported.as_str()) } else { None }
        }
        NonLocalBinding::ImportDefault { name, module }
        | NonLocalBinding::ImportNamespace { name, module } => {
            if is_known_react_module(module) {
                Some(name.as_str())
            } else {
                None
            }
        }
        NonLocalBinding::ModuleLocal { .. } => None,
    }
}

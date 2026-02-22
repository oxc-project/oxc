/// Drop manual memoization (useMemo/useCallback) and replace with compiler markers.
///
/// Port of `Inference/DropManualMemoization.ts` from the React Compiler.
///
/// This pass identifies useMemo/useCallback calls, validates their usage,
/// and replaces them with StartMemoize/FinishMemoize marker instructions
/// that the compiler can reason about.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    compiler_error::{
        CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory, SourceLocation,
    },
    hir::{
        CallExpression as CallExpressionValue, Effect, HIRFunction, IdentifierId, IdentifierName,
        Instruction, InstructionId, InstructionValue, ManualMemoDependency,
        ManualMemoDependencyRoot, Place,
        environment::Environment,
        hir_builder::{create_temporary_place, mark_instruction_ids},
    },
};

/// Collect potential manual memo dependencies from an instruction value.
///
/// Returns the dependency represented by the instruction, if any.
pub(crate) fn collect_maybe_memo_dependencies(
    value: &InstructionValue,
    maybe_deps: &mut FxHashMap<IdentifierId, ManualMemoDependency>,
    optional: bool,
) -> Option<ManualMemoDependency> {
    match value {
        InstructionValue::LoadGlobal(v) => {
            let name = v.binding.name().to_string();
            Some(ManualMemoDependency {
                root: ManualMemoDependencyRoot::Global { identifier_name: name },
                path: Vec::new(),
                loc: v.loc,
            })
        }
        InstructionValue::PropertyLoad(v) => {
            let object_dep = maybe_deps.get(&v.object.identifier.id)?;
            Some(ManualMemoDependency {
                root: object_dep.root.clone(),
                path: {
                    let mut path = object_dep.path.clone();
                    path.push(crate::hir::DependencyPathEntry {
                        property: v.property.clone(),
                        optional,
                        loc: v.loc,
                    });
                    path
                },
                loc: v.loc,
            })
        }
        InstructionValue::LoadLocal(v) => {
            if let Some(source) = maybe_deps.get(&v.place.identifier.id) {
                return Some(source.clone());
            }
            if let Some(IdentifierName::Named(_name)) = &v.place.identifier.name {
                Some(ManualMemoDependency {
                    root: ManualMemoDependencyRoot::NamedLocal {
                        value: v.place.clone(),
                        constant: false,
                    },
                    path: Vec::new(),
                    loc: v.place.loc,
                })
            } else {
                None
            }
        }
        InstructionValue::LoadContext(v) => {
            if let Some(source) = maybe_deps.get(&v.place.identifier.id) {
                return Some(source.clone());
            }
            if let Some(IdentifierName::Named(_name)) = &v.place.identifier.name {
                Some(ManualMemoDependency {
                    root: ManualMemoDependencyRoot::NamedLocal {
                        value: v.place.clone(),
                        constant: false,
                    },
                    path: Vec::new(),
                    loc: v.place.loc,
                })
            } else {
                None
            }
        }
        InstructionValue::StoreLocal(v) => {
            let rvalue_id = v.value.identifier.id;
            if let Some(aliased) = maybe_deps.get(&rvalue_id) {
                let lvalue = &v.lvalue.place.identifier;
                if !matches!(&lvalue.name, Some(IdentifierName::Named(_))) {
                    let dep = aliased.clone();
                    maybe_deps.insert(lvalue.id, dep.clone());
                    return Some(dep);
                }
            }
            None
        }
        _ => None,
    }
}

/// Track identifiers that are useMemo/useCallback callees.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManualMemoKind {
    UseMemo,
    UseCallback,
}

/// Information about a manual memoization callee.
#[derive(Debug, Clone, Copy)]
struct ManualMemoCallee {
    kind: ManualMemoKind,
    /// The `InstructionId` of the `LoadGlobal`/`PropertyLoad` that loaded the hook.
    load_instr_id: InstructionId,
}

/// Sidemap for tracking identifiers during the pass.
struct IdentifierSidemap {
    functions: FxHashSet<IdentifierId>,
    manual_memos: FxHashMap<IdentifierId, ManualMemoCallee>,
    react_ids: FxHashSet<IdentifierId>,
    maybe_deps: FxHashMap<IdentifierId, ManualMemoDependency>,
    maybe_deps_lists: FxHashMap<IdentifierId, (SourceLocation, Vec<Place>)>,
    optionals: FxHashSet<IdentifierId>,
}

/// Result of extracting manual memoization arguments.
struct ManualMemoArgs {
    fn_place: Place,
    deps_list: Option<Vec<ManualMemoDependency>>,
    deps_loc: Option<SourceLocation>,
}

/// Drop manual memoization and replace with compiler markers.
///
/// # Errors
/// Returns a `CompilerError` if validation of manual memoization fails.
pub fn drop_manual_memoization(func: &mut HIRFunction) -> Result<(), CompilerError> {
    let is_validation_enabled = func.env.config.validate_preserve_existing_memoization_guarantees
        || func.env.config.validate_no_set_state_in_render
        || func.env.config.enable_preserve_existing_memoization_guarantees;

    let optionals = find_optional_places(func);
    let mut sidemap = IdentifierSidemap {
        functions: FxHashSet::default(),
        manual_memos: FxHashMap::default(),
        react_ids: FxHashSet::default(),
        maybe_deps: FxHashMap::default(),
        maybe_deps_lists: FxHashMap::default(),
        optionals,
    };
    let mut next_manual_memo_id: u32 = 0;
    let mut errors = CompilerError::new();

    // Phase 1: Overwrite manual memoization calls and collect markers.
    //
    // For each useMemo/useCallback call:
    //   - Validate arguments (callback + optional deps)
    //   - Replace CallExpression with either an IIFE (useMemo) or LoadLocal (useCallback)
    //   - If validation is enabled, queue StartMemoize/FinishMemoize markers
    let mut queued_inserts: FxHashMap<InstructionId, Instruction> = FxHashMap::default();

    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    // Destructure to allow independent borrows of env and body
    let HIRFunction { ref mut env, ref mut body, .. } = *func;
    for block_id in &block_ids {
        let Some(block) = body.blocks.get_mut(block_id) else {
            continue;
        };

        for instr in &mut block.instructions {
            // Check if this instruction is a useMemo/useCallback call
            let manual_memo = match &instr.value {
                InstructionValue::CallExpression(v) => {
                    sidemap.manual_memos.get(&v.callee.identifier.id).copied()
                }
                InstructionValue::MethodCall(v) => {
                    sidemap.manual_memos.get(&v.property.identifier.id).copied()
                }
                _ => None,
            };

            if let Some(callee_info) = manual_memo {
                let memo_details =
                    extract_manual_memoization_args(instr, callee_info.kind, &sidemap, &mut errors);

                let Some(ManualMemoArgs { fn_place, deps_list, deps_loc }) = memo_details else {
                    continue;
                };

                // Replace the instruction value
                instr.value = get_manual_memoization_replacement(
                    &fn_place,
                    instr.value.loc(),
                    callee_info.kind,
                );

                if is_validation_enabled {
                    // Bail out when the memo function is not inline -- our validation
                    // assumes inline function expressions due to the exhaustive-deps lint.
                    if !sidemap.functions.contains(&fn_place.identifier.id) {
                        errors.push_diagnostic(
                            CompilerDiagnostic::create(
                                ErrorCategory::UseMemo,
                                "Expected the first argument to be an inline function expression"
                                    .to_string(),
                                Some(
                                    "Expected the first argument to be an inline function expression"
                                        .to_string(),
                                ),
                                Some(Vec::new()),
                            )
                            .with_detail(CompilerDiagnosticDetail::Error {
                                loc: Some(fn_place.loc),
                                message: Some(
                                    "Expected the first argument to be an inline function expression"
                                        .to_string(),
                                ),
                            }),
                        );
                        continue;
                    }

                    let memo_decl: Place = if callee_info.kind == ManualMemoKind::UseMemo {
                        instr.lvalue.clone()
                    } else {
                        Place {
                            identifier: fn_place.identifier.clone(),
                            effect: Effect::Unknown,
                            reactive: false,
                            loc: fn_place.loc,
                        }
                    };

                    let memo_id = next_manual_memo_id;
                    next_manual_memo_id += 1;

                    let (start_marker, finish_marker) = make_manual_memoization_markers(
                        env, &fn_place, deps_list, deps_loc, memo_decl, memo_id,
                    );

                    // Insert StartMarker right after the hook load instruction
                    queued_inserts.insert(callee_info.load_instr_id, start_marker);
                    // Insert FinishMarker right after the current call instruction
                    queued_inserts.insert(instr.id, finish_marker);
                }
            } else {
                collect_temporaries(instr, &mut sidemap);
            }
        }
    }

    // Phase 2: Insert manual memoization markers at the correct positions.
    //
    // Walk the instruction stream again and splice in queued markers
    // immediately after the instruction they are keyed to.
    if !queued_inserts.is_empty() {
        let mut has_changes = false;
        for block_id in &block_ids {
            let Some(block) = func.body.blocks.get_mut(block_id) else {
                continue;
            };

            let mut next_instructions: Option<Vec<Instruction>> = None;
            for i in 0..block.instructions.len() {
                let instr_id = block.instructions[i].id;
                let insert_instr = queued_inserts.remove(&instr_id);
                if let Some(marker) = insert_instr {
                    let next =
                        next_instructions.get_or_insert_with(|| block.instructions[..i].to_vec());
                    next.push(block.instructions[i].clone());
                    next.push(marker);
                } else if let Some(ref mut next) = next_instructions {
                    next.push(block.instructions[i].clone());
                }
            }
            if let Some(next) = next_instructions {
                block.instructions = next;
                has_changes = true;
            }
        }

        if has_changes {
            mark_instruction_ids(&mut func.body);
        }
    }

    errors.into_result()
}

/// Extract and validate the arguments to a useMemo/useCallback call.
///
/// Returns the function place and optional deps list, or `None` if the arguments
/// are invalid (e.g., first arg is a spread, or deps is not an array literal).
fn extract_manual_memoization_args(
    instr: &Instruction,
    kind: ManualMemoKind,
    sidemap: &IdentifierSidemap,
    errors: &mut CompilerError,
) -> Option<ManualMemoArgs> {
    let args = match &instr.value {
        InstructionValue::CallExpression(v) => &v.args,
        InstructionValue::MethodCall(v) => &v.args,
        _ => return None,
    };

    let kind_name = match kind {
        ManualMemoKind::UseMemo => "useMemo",
        ManualMemoKind::UseCallback => "useCallback",
    };

    // First arg must be an identifier (the callback function)
    let Some(crate::hir::CallArg::Place(fn_place)) = args.first() else {
        let (reason, description, message) = if kind == ManualMemoKind::UseCallback {
            (
                format!("Expected a callback function to be passed to {kind_name}"),
                "The first argument to useCallback() must be a function to cache".to_string(),
                "Expected a callback function".to_string(),
            )
        } else {
            (
                format!("Expected a callback function to be passed to {kind_name}"),
                "The first argument to useMemo() must be a function that calculates a result to cache".to_string(),
                "Expected a memoization function".to_string(),
            )
        };
        errors.push_diagnostic(
            CompilerDiagnostic::create(ErrorCategory::UseMemo, reason, Some(description), None)
                .with_detail(CompilerDiagnosticDetail::Error {
                    loc: Some(instr.value.loc()),
                    message: Some(message),
                }),
        );
        return None;
    };

    // Optional second arg is the deps array
    let deps_list_place = match args.get(1) {
        Some(crate::hir::CallArg::Place(p)) => Some(p),
        Some(crate::hir::CallArg::Spread(_)) => {
            errors.push_diagnostic(
                CompilerDiagnostic::create(
                    ErrorCategory::UseMemo,
                    format!("Expected the dependency list for {kind_name} to be an array literal"),
                    Some(format!(
                        "Expected the dependency list for {kind_name} to be an array literal"
                    )),
                    None,
                )
                .with_detail(CompilerDiagnosticDetail::Error {
                    loc: Some(instr.loc),
                    message: Some(format!(
                        "Expected the dependency list for {kind_name} to be an array literal"
                    )),
                }),
            );
            return None;
        }
        None => None,
    };

    let Some(deps_list_place) = deps_list_place else {
        return Some(ManualMemoArgs {
            fn_place: fn_place.clone(),
            deps_list: None,
            deps_loc: None,
        });
    };

    let maybe_deps_list = sidemap.maybe_deps_lists.get(&deps_list_place.identifier.id);

    let Some((loc, places)) = maybe_deps_list else {
        // deps argument is not an array literal
        let err_loc = deps_list_place.loc;
        errors.push_diagnostic(
            CompilerDiagnostic::create(
                ErrorCategory::UseMemo,
                format!("Expected the dependency list for {kind_name} to be an array literal"),
                Some(format!(
                    "Expected the dependency list for {kind_name} to be an array literal"
                )),
                None,
            )
            .with_detail(CompilerDiagnosticDetail::Error {
                loc: Some(err_loc),
                message: Some(format!(
                    "Expected the dependency list for {kind_name} to be an array literal"
                )),
            }),
        );
        return None;
    };

    let mut deps_list: Vec<ManualMemoDependency> = Vec::new();
    for dep in places {
        if let Some(dep_val) = sidemap.maybe_deps.get(&dep.identifier.id) {
            deps_list.push(dep_val.clone());
        } else {
            errors.push_diagnostic(
                CompilerDiagnostic::create(
                    ErrorCategory::UseMemo,
                    "Expected the dependency list to be an array of simple expressions (e.g. `x`, `x.y.z`, `x?.y?.z`)".to_string(),
                    Some("Expected the dependency list to be an array of simple expressions (e.g. `x`, `x.y.z`, `x?.y?.z`)".to_string()),
                    None,
                )
                .with_detail(CompilerDiagnosticDetail::Error {
                    loc: Some(dep.loc),
                    message: Some("Expected the dependency list to be an array of simple expressions (e.g. `x`, `x.y.z`, `x?.y?.z`)".to_string()),
                }),
            );
        }
    }

    Some(ManualMemoArgs {
        fn_place: fn_place.clone(),
        deps_list: Some(deps_list),
        deps_loc: Some(*loc),
    })
}

/// Get the replacement instruction value for a useMemo/useCallback call.
///
/// For `useMemo`: Replace with `CallExpression { callee: fnPlace, args: [] }` -- making
/// it an IIFE that `InlineIIFE` will later inline.
///
/// For `useCallback`: Replace with `LoadLocal { place: fnPlace }` -- aliasing the
/// function directly.
fn get_manual_memoization_replacement(
    fn_place: &Place,
    loc: SourceLocation,
    kind: ManualMemoKind,
) -> InstructionValue {
    match kind {
        ManualMemoKind::UseMemo => {
            // Replace the hook call with a call to the memo function itself (no args).
            // A later pass (InlineIIFE) will inline this.
            InstructionValue::CallExpression(CallExpressionValue {
                callee: fn_place.clone(),
                args: Vec::new(),
                loc,
            })
        }
        ManualMemoKind::UseCallback => {
            // Instead of calling useCallback, just alias the function directly.
            InstructionValue::LoadLocal(crate::hir::LoadLocal {
                place: Place {
                    identifier: fn_place.identifier.clone(),
                    effect: Effect::Unknown,
                    reactive: false,
                    loc,
                },
                loc,
            })
        }
    }
}

/// Create `StartMemoize` and `FinishMemoize` marker instructions.
///
/// These markers let downstream passes (validation, codegen) reason about
/// the boundaries of manual memoization.
fn make_manual_memoization_markers(
    env: &mut Environment,
    fn_expr: &Place,
    deps_list: Option<Vec<ManualMemoDependency>>,
    deps_loc: Option<SourceLocation>,
    memo_decl: Place,
    manual_memo_id: u32,
) -> (Instruction, Instruction) {
    let start = Instruction {
        id: InstructionId::ZERO,
        lvalue: create_temporary_place(env, fn_expr.loc),
        value: InstructionValue::StartMemoize(crate::hir::StartMemoize {
            manual_memo_id,
            deps: deps_list,
            deps_loc,
            loc: fn_expr.loc,
        }),
        effects: None,
        loc: fn_expr.loc,
    };
    let finish = Instruction {
        id: InstructionId::ZERO,
        lvalue: create_temporary_place(env, fn_expr.loc),
        value: InstructionValue::FinishMemoize(crate::hir::FinishMemoize {
            manual_memo_id,
            decl: memo_decl,
            pruned: false,
            loc: fn_expr.loc,
        }),
        effects: None,
        loc: fn_expr.loc,
    };
    (start, finish)
}

/// Collect temporaries -- track function expressions, manual memo callees,
/// React namespace identifiers, deps lists, and memo dependencies.
fn collect_temporaries(instr: &Instruction, sidemap: &mut IdentifierSidemap) {
    let lvalue_id = instr.lvalue.identifier.id;
    match &instr.value {
        InstructionValue::FunctionExpression(_) => {
            sidemap.functions.insert(lvalue_id);
        }
        InstructionValue::LoadGlobal(v) => {
            let name = v.binding.name();
            match name {
                "useMemo" => {
                    sidemap.manual_memos.insert(
                        lvalue_id,
                        ManualMemoCallee { kind: ManualMemoKind::UseMemo, load_instr_id: instr.id },
                    );
                }
                "useCallback" => {
                    sidemap.manual_memos.insert(
                        lvalue_id,
                        ManualMemoCallee {
                            kind: ManualMemoKind::UseCallback,
                            load_instr_id: instr.id,
                        },
                    );
                }
                "React" => {
                    sidemap.react_ids.insert(lvalue_id);
                }
                _ => {}
            }
        }
        InstructionValue::PropertyLoad(v) => {
            if sidemap.react_ids.contains(&v.object.identifier.id) {
                let prop = v.property.to_string();
                match prop.as_str() {
                    "useMemo" => {
                        sidemap.manual_memos.insert(
                            lvalue_id,
                            ManualMemoCallee {
                                kind: ManualMemoKind::UseMemo,
                                load_instr_id: instr.id,
                            },
                        );
                    }
                    "useCallback" => {
                        sidemap.manual_memos.insert(
                            lvalue_id,
                            ManualMemoCallee {
                                kind: ManualMemoKind::UseCallback,
                                load_instr_id: instr.id,
                            },
                        );
                    }
                    _ => {}
                }
            }
        }
        InstructionValue::ArrayExpression(v) => {
            // Track array literals that might be deps lists
            let all_places: Vec<Place> = v
                .elements
                .iter()
                .filter_map(|e| match e {
                    crate::hir::ArrayExpressionElement::Place(p) => Some(p.clone()),
                    _ => None,
                })
                .collect();
            if all_places.len() == v.elements.len() {
                sidemap.maybe_deps_lists.insert(lvalue_id, (v.loc, all_places));
            }
        }
        _ => {}
    }

    // Collect maybe-memo dependencies
    let is_optional = sidemap.optionals.contains(&lvalue_id);
    let dep = collect_maybe_memo_dependencies(&instr.value, &mut sidemap.maybe_deps, is_optional);
    if let Some(dep) = dep {
        sidemap.maybe_deps.insert(lvalue_id, dep);
    }
}

/// Find all places that are the result of optional chaining.
///
/// This identifies identifiers that are the result of optional property accesses
/// (e.g., `a?.b?.c`) so that dependencies can be tracked correctly.
fn find_optional_places(func: &HIRFunction) -> FxHashSet<IdentifierId> {
    let mut optionals = FxHashSet::default();
    for block in func.body.blocks.values() {
        if let crate::hir::Terminal::Optional(opt_terminal) = &block.terminal {
            if !opt_terminal.optional {
                continue;
            }
            let Some(mut test_block) = func.body.blocks.get(&opt_terminal.test) else {
                continue;
            };
            loop {
                match &test_block.terminal {
                    crate::hir::Terminal::Branch(branch) => {
                        if branch.fallthrough == opt_terminal.fallthrough {
                            // Found the branch that corresponds to this optional
                            let Some(consequent) = func.body.blocks.get(&branch.consequent) else {
                                break;
                            };
                            if let Some(last) = consequent.instructions.last()
                                && let InstructionValue::StoreLocal(store) = &last.value
                            {
                                optionals.insert(store.value.identifier.id);
                            }
                            break;
                        }
                        let Some(next) = func.body.blocks.get(&branch.fallthrough) else {
                            break;
                        };
                        test_block = next;
                    }
                    crate::hir::Terminal::Optional(t) => {
                        let Some(next) = func.body.blocks.get(&t.fallthrough) else {
                            break;
                        };
                        test_block = next;
                    }
                    crate::hir::Terminal::Logical(t) => {
                        let Some(next) = func.body.blocks.get(&t.fallthrough) else {
                            break;
                        };
                        test_block = next;
                    }
                    crate::hir::Terminal::Sequence(t) => {
                        let Some(next) = func.body.blocks.get(&t.fallthrough) else {
                            break;
                        };
                        test_block = next;
                    }
                    crate::hir::Terminal::Ternary(t) => {
                        let Some(next) = func.body.blocks.get(&t.fallthrough) else {
                            break;
                        };
                        test_block = next;
                    }
                    crate::hir::Terminal::MaybeThrow(t) => {
                        let Some(next) = func.body.blocks.get(&t.continuation) else {
                            break;
                        };
                        test_block = next;
                    }
                    _ => {
                        // Unexpected terminal kind in optional chain
                        break;
                    }
                }
            }
        }
    }
    optionals
}

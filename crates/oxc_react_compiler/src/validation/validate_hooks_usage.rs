/// Validate hooks usage according to the Rules of Hooks.
///
/// Port of `Validation/ValidateHooksUsage.ts` from the React Compiler.
///
/// Validates that hooks are:
/// - Called unconditionally (not inside conditions, loops)
/// - Not used as first-class values (only called)
/// - The same function on every render (not dynamically reassigned)
/// - Called at the top level (not inside nested function expressions)
use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHashMap};

use crate::{
    compiler_error::{
        CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory, SourceLocation,
    },
    hir::{
        HIRFunction, IdentifierId, InstructionValue, Place, ReactiveParam,
        compute_unconditional_blocks::compute_unconditional_blocks,
        environment::{get_hook_kind_for_type, is_hook_name},
        types::PropertyLiteral,
        visitors::{each_instruction_lvalue, each_instruction_operand, each_terminal_operand},
    },
};

/// Possible kinds of value during abstract interpretation.
/// The kinds form a lattice, with earlier items taking precedence
/// over later items (see `join_kinds`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// A potential/known hook which was already used in an invalid way
    Error,
    /// A known hook (e.g. from LoadGlobal with hook type, or property of Global/KnownHook)
    KnownHook,
    /// A potential hook (e.g. hook-named lvalue, property of PotentialHook/Local)
    PotentialHook,
    /// LoadGlobal values whose type was not inferred as a hook
    Global,
    /// All other values (local variables)
    Local,
}

fn join_kinds(a: Kind, b: Kind) -> Kind {
    if a == Kind::Error || b == Kind::Error {
        Kind::Error
    } else if a == Kind::KnownHook || b == Kind::KnownHook {
        Kind::KnownHook
    } else if a == Kind::PotentialHook || b == Kind::PotentialHook {
        Kind::PotentialHook
    } else if a == Kind::Global || b == Kind::Global {
        Kind::Global
    } else {
        Kind::Local
    }
}

/// Validate hooks usage in the given function.
///
/// # Errors
/// Returns a `CompilerError` if hooks usage violations are found.
pub fn validate_hooks_usage(func: &HIRFunction) -> Result<(), CompilerError> {
    let unconditional_blocks = compute_unconditional_blocks(func);
    let mut errors = CompilerError::new();
    let mut errors_by_place: IndexMap<SourceLocation, CompilerDiagnostic, FxBuildHasher> =
        IndexMap::default();

    let mut value_kinds: FxHashMap<IdentifierId, Kind> = FxHashMap::default();

    let get_kind_for_place = |value_kinds: &FxHashMap<IdentifierId, Kind>, place: &Place| -> Kind {
        let known_kind = value_kinds.get(&place.identifier.id).copied();
        if let Some(name) = &place.identifier.name {
            if is_hook_name(name.value()) {
                return join_kinds(known_kind.unwrap_or(Kind::Local), Kind::PotentialHook);
            }
        }
        known_kind.unwrap_or(Kind::Local)
    };

    let set_kind = |value_kinds: &mut FxHashMap<IdentifierId, Kind>, place: &Place, kind: Kind| {
        value_kinds.insert(place.identifier.id, kind);
    };

    // Initialize params
    for param in &func.params {
        let place = match param {
            ReactiveParam::Place(p) => p,
            ReactiveParam::Spread(s) => &s.place,
        };
        let kind = get_kind_for_place(&value_kinds, place);
        set_kind(&mut value_kinds, place, kind);
    }

    for (&block_id, block) in &func.body.blocks {
        // Process phi nodes
        for phi in &block.phis {
            let mut kind: Kind =
                if phi.place.identifier.name.as_ref().is_some_and(|n| is_hook_name(n.value())) {
                    Kind::PotentialHook
                } else {
                    Kind::Local
                };
            for (_, operand) in &phi.operands {
                let operand_kind = value_kinds.get(&operand.identifier.id).copied();
                // NOTE: we currently skip operands whose value is unknown
                // (which can only occur for functions with loops). We may
                // miss invalid code in some cases.
                if let Some(ok) = operand_kind {
                    kind = join_kinds(kind, ok);
                }
            }
            value_kinds.insert(phi.place.identifier.id, kind);
        }

        for instr in &block.instructions {
            match &instr.value {
                InstructionValue::LoadGlobal(_) => {
                    // Globals are the one source of known hooks: they are either
                    // directly a hook, or infer a Global kind from which known hooks
                    // can be derived later via property access
                    if get_hook_kind_for_type(&func.env, &instr.lvalue.identifier.type_).is_some() {
                        set_kind(&mut value_kinds, &instr.lvalue, Kind::KnownHook);
                    } else {
                        set_kind(&mut value_kinds, &instr.lvalue, Kind::Global);
                    }
                }
                InstructionValue::LoadLocal(v) => {
                    visit_place(&value_kinds, &v.place, &mut errors, &mut errors_by_place);
                    let kind = get_kind_for_place(&value_kinds, &v.place);
                    set_kind(&mut value_kinds, &instr.lvalue, kind);
                }
                InstructionValue::LoadContext(v) => {
                    visit_place(&value_kinds, &v.place, &mut errors, &mut errors_by_place);
                    let kind = get_kind_for_place(&value_kinds, &v.place);
                    set_kind(&mut value_kinds, &instr.lvalue, kind);
                }
                InstructionValue::StoreLocal(v) => {
                    visit_place(&value_kinds, &v.value, &mut errors, &mut errors_by_place);
                    let kind = join_kinds(
                        get_kind_for_place(&value_kinds, &v.value),
                        get_kind_for_place(&value_kinds, &v.lvalue.place),
                    );
                    set_kind(&mut value_kinds, &v.lvalue.place, kind);
                    set_kind(&mut value_kinds, &instr.lvalue, kind);
                }
                InstructionValue::StoreContext(v) => {
                    visit_place(&value_kinds, &v.value, &mut errors, &mut errors_by_place);
                    let kind = join_kinds(
                        get_kind_for_place(&value_kinds, &v.value),
                        get_kind_for_place(&value_kinds, &v.lvalue_place),
                    );
                    set_kind(&mut value_kinds, &v.lvalue_place, kind);
                    set_kind(&mut value_kinds, &instr.lvalue, kind);
                }
                InstructionValue::ComputedLoad(v) => {
                    visit_place(&value_kinds, &v.object, &mut errors, &mut errors_by_place);
                    let kind = get_kind_for_place(&value_kinds, &v.object);
                    let lvalue_kind = get_kind_for_place(&value_kinds, &instr.lvalue);
                    set_kind(&mut value_kinds, &instr.lvalue, join_kinds(lvalue_kind, kind));
                }
                InstructionValue::PropertyLoad(v) => {
                    let object_kind = get_kind_for_place(&value_kinds, &v.object);
                    let is_hook_property =
                        matches!(&v.property, PropertyLiteral::String(s) if is_hook_name(s));
                    let kind = match object_kind {
                        Kind::Error => Kind::Error,
                        Kind::KnownHook => {
                            if is_hook_property {
                                Kind::KnownHook
                            } else {
                                Kind::Local
                            }
                        }
                        Kind::PotentialHook => Kind::PotentialHook,
                        Kind::Global => {
                            if is_hook_property {
                                Kind::KnownHook
                            } else {
                                Kind::Global
                            }
                        }
                        Kind::Local => {
                            if is_hook_property {
                                Kind::PotentialHook
                            } else {
                                Kind::Local
                            }
                        }
                    };
                    set_kind(&mut value_kinds, &instr.lvalue, kind);
                }
                InstructionValue::CallExpression(v) => {
                    let callee_kind = get_kind_for_place(&value_kinds, &v.callee);
                    let is_hook_callee =
                        callee_kind == Kind::KnownHook || callee_kind == Kind::PotentialHook;
                    if is_hook_callee && !unconditional_blocks.contains(&block_id) {
                        record_conditional_hook_error(
                            &mut value_kinds,
                            &v.callee,
                            &mut errors,
                            &mut errors_by_place,
                        );
                    } else if callee_kind == Kind::PotentialHook {
                        record_dynamic_hook_usage_error(
                            &v.callee,
                            &mut errors,
                            &mut errors_by_place,
                        );
                    }
                    // Check usages of operands, but skip the callee (validated above)
                    for operand in each_instruction_operand(instr) {
                        if std::ptr::eq(operand, &v.callee) {
                            continue;
                        }
                        visit_place(&value_kinds, operand, &mut errors, &mut errors_by_place);
                    }
                }
                InstructionValue::MethodCall(v) => {
                    let callee_kind = get_kind_for_place(&value_kinds, &v.property);
                    let is_hook_callee =
                        callee_kind == Kind::KnownHook || callee_kind == Kind::PotentialHook;
                    if is_hook_callee && !unconditional_blocks.contains(&block_id) {
                        record_conditional_hook_error(
                            &mut value_kinds,
                            &v.property,
                            &mut errors,
                            &mut errors_by_place,
                        );
                    } else if callee_kind == Kind::PotentialHook {
                        record_dynamic_hook_usage_error(
                            &v.property,
                            &mut errors,
                            &mut errors_by_place,
                        );
                    }
                    // Check usages of operands, but skip the property (validated above)
                    for operand in each_instruction_operand(instr) {
                        if std::ptr::eq(operand, &v.property) {
                            continue;
                        }
                        visit_place(&value_kinds, operand, &mut errors, &mut errors_by_place);
                    }
                }
                InstructionValue::Destructure(v) => {
                    visit_place(&value_kinds, &v.value, &mut errors, &mut errors_by_place);
                    let object_kind = get_kind_for_place(&value_kinds, &v.value);
                    for lvalue in each_instruction_lvalue(instr) {
                        // Skip the instruction's own lvalue (first element), only process pattern lvalues
                        if std::ptr::eq(lvalue, &instr.lvalue) {
                            continue;
                        }
                        let is_hook_property = lvalue
                            .identifier
                            .name
                            .as_ref()
                            .is_some_and(|n| is_hook_name(n.value()));
                        let kind = match object_kind {
                            Kind::Error => Kind::Error,
                            Kind::KnownHook => Kind::KnownHook,
                            Kind::PotentialHook => Kind::PotentialHook,
                            Kind::Global => {
                                if is_hook_property {
                                    Kind::KnownHook
                                } else {
                                    Kind::Global
                                }
                            }
                            Kind::Local => {
                                if is_hook_property {
                                    Kind::PotentialHook
                                } else {
                                    Kind::Local
                                }
                            }
                        };
                        set_kind(&mut value_kinds, lvalue, kind);
                    }
                }
                InstructionValue::FunctionExpression(v) => {
                    visit_function_expression(&mut errors, &v.lowered_func.func);
                }
                InstructionValue::ObjectMethod(v) => {
                    visit_function_expression(&mut errors, &v.lowered_func.func);
                }
                _ => {
                    // Check usages of operands, but do *not* flow properties
                    // from operands into the lvalues.
                    for operand in each_instruction_operand(instr) {
                        visit_place(&value_kinds, operand, &mut errors, &mut errors_by_place);
                    }
                    for lvalue in each_instruction_lvalue(instr) {
                        let kind = get_kind_for_place(&value_kinds, lvalue);
                        set_kind(&mut value_kinds, lvalue, kind);
                    }
                }
            }
        }

        // Check terminal operands
        for operand in each_terminal_operand(&block.terminal) {
            visit_place(&value_kinds, operand, &mut errors, &mut errors_by_place);
        }
    }

    for (_, error) in errors_by_place {
        errors.push_diagnostic(error);
    }

    errors.into_result()
}

/// Check if a place holds a KnownHook being used as a value (non-call usage).
fn visit_place(
    value_kinds: &FxHashMap<IdentifierId, Kind>,
    place: &Place,
    errors: &mut CompilerError,
    errors_by_place: &mut IndexMap<SourceLocation, CompilerDiagnostic, FxBuildHasher>,
) {
    let kind = value_kinds.get(&place.identifier.id).copied();
    if kind == Some(Kind::KnownHook) {
        record_invalid_hook_usage_error(place, errors, errors_by_place);
    }
}

fn record_conditional_hook_error(
    value_kinds: &mut FxHashMap<IdentifierId, Kind>,
    place: &Place,
    errors: &mut CompilerError,
    errors_by_place: &mut IndexMap<SourceLocation, CompilerDiagnostic, FxBuildHasher>,
) {
    // Once a particular hook has a conditional call error, don't report any further issues
    value_kinds.insert(place.identifier.id, Kind::Error);

    let reason = "Hooks must always be called in a consistent order, and may not be called conditionally. See the Rules of Hooks (https://react.dev/warnings/invalid-hook-call-warning)";

    let previous_error = match &place.loc {
        SourceLocation::Generated => None,
        loc => errors_by_place.get(loc),
    };

    // In some circumstances such as optional calls, we may first encounter a
    // "hook may not be referenced as normal values" error. If that same place is
    // also used as a conditional call, upgrade the error to a conditional hook error.
    if previous_error.is_none() || previous_error.is_some_and(|e| e.options.reason != reason) {
        record_error(
            place.loc,
            CompilerDiagnostic::create(ErrorCategory::Hooks, reason.to_string(), None, None)
                .with_detail(CompilerDiagnosticDetail::Error {
                    loc: Some(place.loc),
                    message: None,
                }),
            errors,
            errors_by_place,
        );
    }
}

fn record_invalid_hook_usage_error(
    place: &Place,
    errors: &mut CompilerError,
    errors_by_place: &mut IndexMap<SourceLocation, CompilerDiagnostic, FxBuildHasher>,
) {
    let previous_error = match &place.loc {
        SourceLocation::Generated => None,
        loc => errors_by_place.get(loc),
    };
    if previous_error.is_none() {
        record_error(
            place.loc,
            CompilerDiagnostic::create(
                ErrorCategory::Hooks,
                "Hooks may not be referenced as normal values, they must be called. See https://react.dev/reference/rules/react-calls-components-and-hooks#never-pass-around-hooks-as-regular-values".to_string(),
                None,
                None,
            )
            .with_detail(CompilerDiagnosticDetail::Error {
                loc: Some(place.loc),
                message: None,
            }),
            errors,
            errors_by_place,
        );
    }
}

fn record_dynamic_hook_usage_error(
    place: &Place,
    errors: &mut CompilerError,
    errors_by_place: &mut IndexMap<SourceLocation, CompilerDiagnostic, FxBuildHasher>,
) {
    let previous_error = match &place.loc {
        SourceLocation::Generated => None,
        loc => errors_by_place.get(loc),
    };
    if previous_error.is_none() {
        record_error(
            place.loc,
            CompilerDiagnostic::create(
                ErrorCategory::Hooks,
                "Hooks must be the same function on every render, but this value may change over time to a different function. See https://react.dev/reference/rules/react-calls-components-and-hooks#dont-dynamically-use-hooks".to_string(),
                None,
                None,
            )
            .with_detail(CompilerDiagnosticDetail::Error {
                loc: Some(place.loc),
                message: None,
            }),
            errors,
            errors_by_place,
        );
    }
}

fn record_error(
    loc: SourceLocation,
    diagnostic: CompilerDiagnostic,
    errors: &mut CompilerError,
    errors_by_place: &mut IndexMap<SourceLocation, CompilerDiagnostic, FxBuildHasher>,
) {
    match loc {
        SourceLocation::Generated => {
            errors.push_diagnostic(diagnostic);
        }
        loc => {
            errors_by_place.insert(loc, diagnostic);
        }
    }
}

/// Recursively validate function expressions for hooks called within nested functions.
fn visit_function_expression(errors: &mut CompilerError, func: &HIRFunction) {
    for (_, block) in &func.body.blocks {
        for instr in &block.instructions {
            match &instr.value {
                InstructionValue::FunctionExpression(v) => {
                    visit_function_expression(errors, &v.lowered_func.func);
                }
                InstructionValue::ObjectMethod(v) => {
                    visit_function_expression(errors, &v.lowered_func.func);
                }
                InstructionValue::CallExpression(v) => {
                    let hook_kind = get_hook_kind_for_type(&func.env, &v.callee.identifier.type_);
                    if let Some(hk) = hook_kind {
                        let description = format!(
                            "Cannot call {} within a function expression",
                            hook_kind_to_string(hk)
                        );
                        errors.push_diagnostic(
                            CompilerDiagnostic::create(
                                ErrorCategory::Hooks,
                                "Hooks must be called at the top level in the body of a function component or custom hook, and may not be called within function expressions. See the Rules of Hooks (https://react.dev/warnings/invalid-hook-call-warning)".to_string(),
                                Some(description),
                                None,
                            )
                            .with_detail(CompilerDiagnosticDetail::Error {
                                loc: Some(v.callee.loc),
                                message: None,
                            }),
                        );
                    }
                }
                InstructionValue::MethodCall(v) => {
                    let hook_kind = get_hook_kind_for_type(&func.env, &v.property.identifier.type_);
                    if let Some(hk) = hook_kind {
                        let description = format!(
                            "Cannot call {} within a function expression",
                            hook_kind_to_string(hk)
                        );
                        errors.push_diagnostic(
                            CompilerDiagnostic::create(
                                ErrorCategory::Hooks,
                                "Hooks must be called at the top level in the body of a function component or custom hook, and may not be called within function expressions. See the Rules of Hooks (https://react.dev/warnings/invalid-hook-call-warning)".to_string(),
                                Some(description),
                                None,
                            )
                            .with_detail(CompilerDiagnosticDetail::Error {
                                loc: Some(v.property.loc),
                                message: None,
                            }),
                        );
                    }
                }
                _ => {}
            }
        }
    }
}

/// Convert a HookKind to the string used in error descriptions.
/// In the TS reference: `hookKind === 'Custom' ? 'hook' : hookKind`
fn hook_kind_to_string(kind: crate::hir::object_shape::HookKind) -> &'static str {
    use crate::hir::object_shape::HookKind;
    match kind {
        HookKind::Custom => "hook",
        HookKind::UseContext => "useContext",
        HookKind::UseState => "useState",
        HookKind::UseActionState => "useActionState",
        HookKind::UseReducer => "useReducer",
        HookKind::UseRef => "useRef",
        HookKind::UseEffect => "useEffect",
        HookKind::UseLayoutEffect => "useLayoutEffect",
        HookKind::UseInsertionEffect => "useInsertionEffect",
        HookKind::UseMemo => "useMemo",
        HookKind::UseCallback => "useCallback",
        HookKind::UseTransition => "useTransition",
        HookKind::UseImperativeHandle => "useImperativeHandle",
        HookKind::UseEffectEvent => "useEffectEvent",
        HookKind::UseOptimistic => "useOptimistic",
    }
}

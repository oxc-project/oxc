/// Validate no setState in effects.
///
/// Port of `Validation/ValidateNoSetStateInEffects.ts` from the React Compiler.
///
/// Validates against calling setState in the body of an effect (useEffect and friends),
/// while allowing calling setState in callbacks scheduled by the effect.
///
/// Calling setState during execution of a useEffect triggers a re-render, which is
/// often bad for performance and frequently has more efficient and straightforward
/// alternatives. See https://react.dev/learn/you-might-not-need-an-effect for examples.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    compiler_error::{CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory},
    hir::{
        CallArg, Effect, HIRFunction, Identifier, IdentifierId, InstructionValue, Place,
        object_shape::{
            BUILT_IN_REF_VALUE_ID, BUILT_IN_SET_STATE_ID, BUILT_IN_USE_EFFECT_EVENT_ID,
            BUILT_IN_USE_EFFECT_HOOK_ID, BUILT_IN_USE_INSERTION_EFFECT_HOOK_ID,
            BUILT_IN_USE_LAYOUT_EFFECT_HOOK_ID, BUILT_IN_USE_REF_ID,
        },
        types::{FunctionType, ObjectType, PropertyLiteral, Type},
        visitors::{each_instruction_lvalue, each_instruction_value_operand},
    },
    inference::control_dominators::create_control_dominators,
    reactive_scopes::infer_reactive_scope_variables::is_mutable,
};

/// Validate no setState in effects.
pub fn validate_no_set_state_in_effects(func: &HIRFunction) -> CompilerError {
    let mut set_state_functions: FxHashMap<IdentifierId, Place> = FxHashMap::default();
    let mut errors = CompilerError::new();

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            match &instr.value {
                InstructionValue::LoadLocal(v) => {
                    if set_state_functions.contains_key(&v.place.identifier.id) {
                        set_state_functions.insert(instr.lvalue.identifier.id, v.place.clone());
                    }
                }
                InstructionValue::StoreLocal(v) => {
                    if set_state_functions.contains_key(&v.value.identifier.id) {
                        set_state_functions.insert(v.lvalue.place.identifier.id, v.value.clone());
                        set_state_functions.insert(instr.lvalue.identifier.id, v.value.clone());
                    }
                }
                InstructionValue::FunctionExpression(v) => {
                    // Check if the function expression captures a setState
                    let has_set_state_operand =
                        each_instruction_value_operand(&instr.value).iter().any(|operand| {
                            is_set_state_type(&operand.identifier.type_)
                                || set_state_functions.contains_key(&operand.identifier.id)
                        });

                    if has_set_state_operand {
                        let callee =
                            get_set_state_call(&v.lowered_func.func, &mut set_state_functions);
                        if let Some(callee) = callee {
                            set_state_functions.insert(instr.lvalue.identifier.id, callee);
                        }
                    }
                }
                InstructionValue::CallExpression(_) | InstructionValue::MethodCall(_) => {
                    let callee = match &instr.value {
                        InstructionValue::MethodCall(m) => &m.property,
                        InstructionValue::CallExpression(c) => &c.callee,
                        _ => continue,
                    };
                    let args = match &instr.value {
                        InstructionValue::MethodCall(m) => &m.args,
                        InstructionValue::CallExpression(c) => &c.args,
                        _ => continue,
                    };

                    if is_use_effect_event_type(&callee.identifier.type_) {
                        // useEffectEvent wraps a function
                        if let Some(CallArg::Place(arg)) = args.first()
                            && let Some(set_state) = set_state_functions.get(&arg.identifier.id)
                        {
                            set_state_functions
                                .insert(instr.lvalue.identifier.id, set_state.clone());
                        }
                    } else if (is_use_effect_hook_type(&callee.identifier.type_)
                        || is_use_layout_effect_hook_type(&callee.identifier.type_)
                        || is_use_insertion_effect_hook_type(&callee.identifier.type_))
                        && let Some(CallArg::Place(arg)) = args.first()
                        && let Some(set_state) = set_state_functions.get(&arg.identifier.id)
                    {
                        let enable_verbose = func.env.config.enable_verbose_no_set_state_in_effect;
                        let description = if enable_verbose {
                            "Effects are intended to synchronize state between \
                             React and external systems. Calling setState \
                             synchronously causes cascading renders that hurt \
                             performance.\n\n\
                             This pattern may indicate one of several issues:\n\n\
                             **1. Non-local derived data**: If the value being \
                             set could be computed from props/state but requires \
                             data from a parent component, consider restructuring \
                             state ownership so the derivation can happen during \
                             render in the component that owns the relevant \
                             state.\n\n\
                             **2. Derived event pattern**: If you're detecting \
                             when a prop changes (e.g., `isPlaying` transitioning \
                             from false to true), this often indicates the parent \
                             should provide an event callback (like `onPlay`) \
                             instead of just the current state. Request access to \
                             the original event.\n\n\
                             **3. Force update / external sync**: If you're \
                             forcing a re-render to sync with an external data \
                             source (mutable values outside React), use \
                             `useSyncExternalStore` to properly subscribe to \
                             external state changes.\n\n\
                             See: https://react.dev/learn/you-might-not-need-an-effect"
                                .to_string()
                        } else {
                            "Effects are intended to synchronize state between \
                             React and external systems such as manually updating \
                             the DOM, state management libraries, or other \
                             platform APIs. In general, the body of an effect \
                             should do one or both of the following:\n\
                             * Update external systems with the latest state from \
                             React.\n\
                             * Subscribe for updates from some external system, \
                             calling setState in a callback function when external \
                             state changes.\n\n\
                             Calling setState synchronously within an effect body \
                             causes cascading renders that can hurt performance, \
                             and is not recommended. \
                             (https://react.dev/learn/you-might-not-need-an-effect)"
                                .to_string()
                        };
                        errors.push_diagnostic(
                            CompilerDiagnostic::create(
                                ErrorCategory::EffectSetState,
                                "Calling setState synchronously within an effect can \
                                 trigger cascading renders"
                                    .to_string(),
                                Some(description),
                                None,
                            )
                            .with_detail(
                                CompilerDiagnosticDetail::Error {
                                    loc: Some(set_state.loc),
                                    message: Some(
                                        "Avoid calling setState() directly within an effect"
                                            .to_string(),
                                    ),
                                },
                            ),
                        );
                    }
                }
                _ => {}
            }
        }
    }

    errors
}

/// Check if an identifier is a useRef type.
fn is_use_ref_type(identifier: &Identifier) -> bool {
    matches!(
        &identifier.type_,
        Type::Object(ObjectType { shape_id: Some(id) }) if id == BUILT_IN_USE_REF_ID
    )
}

/// Check if an identifier is a ref value type.
fn is_ref_value_type(identifier: &Identifier) -> bool {
    matches!(
        &identifier.type_,
        Type::Object(ObjectType { shape_id: Some(id) }) if id == BUILT_IN_REF_VALUE_ID
    )
}

/// Check if a place is derived from a ref.
fn is_derived_from_ref(place: &Place, ref_derived_values: &FxHashSet<IdentifierId>) -> bool {
    ref_derived_values.contains(&place.identifier.id)
        || is_use_ref_type(&place.identifier)
        || is_ref_value_type(&place.identifier)
}

/// Check if a function expression unconditionally calls setState, returning
/// the callee Place if found.
///
/// When `enableAllowSetStateFromRefsInEffects` is enabled, tracks ref-derived
/// values through the function and exempts setState calls where:
/// 1. The first argument is derived from a ref
/// 2. The block is controlled by a ref-derived conditional
fn get_set_state_call(
    func: &HIRFunction,
    set_state_functions: &mut FxHashMap<IdentifierId, Place>,
) -> Option<Place> {
    let enable_allow_set_state_from_refs_in_effects =
        func.env.config.enable_allow_set_state_from_refs_in_effects;

    let mut ref_derived_values: FxHashSet<IdentifierId> = FxHashSet::default();

    // We need to capture ref_derived_values by reference in the closure passed
    // to create_control_dominators. However, ref_derived_values is mutated later.
    // The TS version works because JS closures capture by reference naturally.
    // In Rust, we use a raw pointer to share access safely within this function scope.
    let ref_derived_ptr = &raw const ref_derived_values;
    let is_derived_from_ref_for_control = move |place: &Place| -> bool {
        // SAFETY: ref_derived_ptr points to ref_derived_values which lives
        // for the entire duration of get_set_state_call. The closure is only
        // called within this function's scope, so the reference is always valid.
        let ref_derived = unsafe { &*ref_derived_ptr };
        ref_derived.contains(&place.identifier.id)
            || is_use_ref_type(&place.identifier)
            || is_ref_value_type(&place.identifier)
    };

    let is_ref_controlled_block: Box<dyn Fn(crate::hir::BlockId) -> bool + '_> =
        if enable_allow_set_state_from_refs_in_effects {
            Box::new(create_control_dominators(func, &is_derived_from_ref_for_control))
        } else {
            Box::new(|_: crate::hir::BlockId| -> bool { false })
        };

    for block in func.body.blocks.values() {
        if enable_allow_set_state_from_refs_in_effects {
            for phi in &block.phis {
                if is_derived_from_ref(&phi.place, &ref_derived_values) {
                    continue;
                }
                let mut is_phi_derived_from_ref = false;
                for operand in phi.operands.values() {
                    if is_derived_from_ref(operand, &ref_derived_values) {
                        is_phi_derived_from_ref = true;
                        break;
                    }
                }
                if is_phi_derived_from_ref {
                    ref_derived_values.insert(phi.place.identifier.id);
                } else {
                    for &pred in phi.operands.keys() {
                        if is_ref_controlled_block(pred) {
                            ref_derived_values.insert(phi.place.identifier.id);
                            break;
                        }
                    }
                }
            }
        }

        for instr in &block.instructions {
            if enable_allow_set_state_from_refs_in_effects {
                let has_ref_operand = each_instruction_value_operand(&instr.value)
                    .iter()
                    .any(|operand| is_derived_from_ref(operand, &ref_derived_values));

                if has_ref_operand {
                    for lvalue in each_instruction_lvalue(instr) {
                        ref_derived_values.insert(lvalue.identifier.id);
                    }
                    // Ref-derived values can also propagate through mutation
                    for operand in each_instruction_value_operand(&instr.value) {
                        match operand.effect {
                            Effect::Capture
                            | Effect::Store
                            | Effect::ConditionallyMutate
                            | Effect::ConditionallyMutateIterator
                            | Effect::Mutate => {
                                if is_mutable(&operand.identifier, instr.id) {
                                    ref_derived_values.insert(operand.identifier.id);
                                }
                            }
                            // Effect::Freeze, Effect::Read: no-op
                            // Effect::Unknown: TS throws an invariant here, but
                            // we silently skip
                            _ => {}
                        }
                    }
                }

                if let InstructionValue::PropertyLoad(v) = &instr.value
                    && matches!(&v.property, PropertyLiteral::String(s) if s == "current")
                    && (is_use_ref_type(&v.object.identifier)
                        || is_ref_value_type(&v.object.identifier))
                {
                    ref_derived_values.insert(instr.lvalue.identifier.id);
                }
            }

            match &instr.value {
                InstructionValue::LoadLocal(v) => {
                    if set_state_functions.contains_key(&v.place.identifier.id) {
                        set_state_functions.insert(instr.lvalue.identifier.id, v.place.clone());
                    }
                }
                InstructionValue::StoreLocal(v) => {
                    if set_state_functions.contains_key(&v.value.identifier.id) {
                        set_state_functions.insert(v.lvalue.place.identifier.id, v.value.clone());
                        set_state_functions.insert(instr.lvalue.identifier.id, v.value.clone());
                    }
                }
                InstructionValue::CallExpression(v) => {
                    if is_set_state_type(&v.callee.identifier.type_)
                        || set_state_functions.contains_key(&v.callee.identifier.id)
                    {
                        if enable_allow_set_state_from_refs_in_effects {
                            if let Some(CallArg::Place(arg)) = v.args.first()
                                && ref_derived_values.contains(&arg.identifier.id)
                            {
                                // The one special case where we allow setStates in effects
                                // is in the very specific scenario where the value being
                                // set is derived from a ref. For example this may be needed
                                // when initial layout measurements from refs need to be
                                // stored in state.
                                return None;
                            }
                            if is_ref_controlled_block(block.id) {
                                continue;
                            }
                        }
                        // TODO: once we support multiple locations per error, we should
                        // link to the original Place in the case that
                        // setStateFunction.has(callee)
                        return Some(v.callee.clone());
                    }
                }
                _ => {}
            }
        }
    }
    None
}

fn is_set_state_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Function(FunctionType { shape_id: Some(id), .. })
        if id == BUILT_IN_SET_STATE_ID
    )
}

fn is_use_effect_hook_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Function(FunctionType { shape_id: Some(id), .. })
        if id == BUILT_IN_USE_EFFECT_HOOK_ID
    )
}

fn is_use_layout_effect_hook_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Function(FunctionType { shape_id: Some(id), .. })
        if id == BUILT_IN_USE_LAYOUT_EFFECT_HOOK_ID
    )
}

fn is_use_insertion_effect_hook_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Function(FunctionType { shape_id: Some(id), .. })
        if id == BUILT_IN_USE_INSERTION_EFFECT_HOOK_ID
    )
}

fn is_use_effect_event_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Function(FunctionType { shape_id: Some(id), .. })
        if id == BUILT_IN_USE_EFFECT_EVENT_ID
    )
}

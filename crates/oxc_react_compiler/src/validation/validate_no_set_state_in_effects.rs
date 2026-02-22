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
use rustc_hash::FxHashMap;

use crate::{
    compiler_error::{CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory},
    hir::{
        CallArg, HIRFunction, IdentifierId, InstructionValue, Place,
        object_shape::{
            BUILT_IN_SET_STATE_ID, BUILT_IN_USE_EFFECT_EVENT_ID, BUILT_IN_USE_EFFECT_HOOK_ID,
            BUILT_IN_USE_INSERTION_EFFECT_HOOK_ID, BUILT_IN_USE_LAYOUT_EFFECT_HOOK_ID,
        },
        types::{FunctionType, Type},
        visitors::each_instruction_value_operand,
    },
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
                        errors.push_diagnostic(
                            CompilerDiagnostic::create(
                                ErrorCategory::EffectSetState,
                                "Calling setState synchronously within an effect can \
                                         trigger cascading renders"
                                    .to_string(),
                                Some(
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
                                        .to_string(),
                                ),
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

/// Check if a function expression unconditionally calls setState, returning
/// the callee Place if found.
fn get_set_state_call(
    func: &HIRFunction,
    set_state_functions: &mut FxHashMap<IdentifierId, Place>,
) -> Option<Place> {
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
                InstructionValue::CallExpression(v) => {
                    if is_set_state_type(&v.callee.identifier.type_)
                        || set_state_functions.contains_key(&v.callee.identifier.id)
                    {
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

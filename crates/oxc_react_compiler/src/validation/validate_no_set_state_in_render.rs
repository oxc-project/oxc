/// Validate no setState calls during render.
///
/// Port of `Validation/ValidateNoSetStateInRender.ts` from the React Compiler.
///
/// Validates that the given function does not have an infinite update loop
/// caused by unconditionally calling setState during render. This validation
/// is conservative and cannot catch all cases of unconditional setState in
/// render, but avoids false positives.
use rustc_hash::FxHashSet;

use crate::{
    compiler_error::{CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory},
    hir::{
        HIRFunction, IdentifierId, InstructionValue,
        compute_unconditional_blocks::compute_unconditional_blocks,
        object_shape::BUILT_IN_SET_STATE_ID,
        types::{FunctionType, Type},
        visitors::each_instruction_value_operand,
    },
};

/// Validate no setState in render.
///
/// # Errors
/// Returns a `CompilerError` if unconditional setState calls are found during render.
pub fn validate_no_set_state_in_render(func: &HIRFunction) -> Result<(), CompilerError> {
    let mut unconditional_set_state_functions: FxHashSet<IdentifierId> = FxHashSet::default();
    validate_impl(func, &mut unconditional_set_state_functions)
}

fn validate_impl(
    func: &HIRFunction,
    unconditional_set_state_fns: &mut FxHashSet<IdentifierId>,
) -> Result<(), CompilerError> {
    let unconditional_blocks = compute_unconditional_blocks(func);
    let mut errors = CompilerError::new();
    let mut active_manual_memo_id: Option<u32> = None;

    for block in func.body.blocks.values() {
        let is_unconditional = unconditional_blocks.contains(&block.id);

        for instr in &block.instructions {
            match &instr.value {
                // Propagate setState identity through LoadLocal
                InstructionValue::LoadLocal(v) => {
                    if unconditional_set_state_fns.contains(&v.place.identifier.id) {
                        unconditional_set_state_fns.insert(instr.lvalue.identifier.id);
                    }
                }
                // Propagate setState identity through StoreLocal
                InstructionValue::StoreLocal(v) => {
                    if unconditional_set_state_fns.contains(&v.value.identifier.id) {
                        unconditional_set_state_fns.insert(v.lvalue.place.identifier.id);
                        unconditional_set_state_fns.insert(instr.lvalue.identifier.id);
                    }
                }
                // Track function expressions that unconditionally call setState
                InstructionValue::FunctionExpression(v) => {
                    let has_set_state_operand =
                        each_instruction_value_operand(&instr.value).iter().any(|operand| {
                            is_set_state_type(&operand.identifier.type_)
                                || unconditional_set_state_fns.contains(&operand.identifier.id)
                        });
                    if has_set_state_operand {
                        let inner_result =
                            validate_impl(&v.lowered_func.func, unconditional_set_state_fns);
                        if inner_result.is_err() {
                            unconditional_set_state_fns.insert(instr.lvalue.identifier.id);
                        }
                    }
                }
                InstructionValue::ObjectMethod(v) => {
                    let has_set_state_operand =
                        each_instruction_value_operand(&instr.value).iter().any(|operand| {
                            is_set_state_type(&operand.identifier.type_)
                                || unconditional_set_state_fns.contains(&operand.identifier.id)
                        });
                    if has_set_state_operand {
                        let inner_result =
                            validate_impl(&v.lowered_func.func, unconditional_set_state_fns);
                        if inner_result.is_err() {
                            unconditional_set_state_fns.insert(instr.lvalue.identifier.id);
                        }
                    }
                }
                // Track manual memoization boundaries
                InstructionValue::StartMemoize(v) => {
                    active_manual_memo_id = Some(v.manual_memo_id);
                }
                InstructionValue::FinishMemoize(v) => {
                    if active_manual_memo_id == Some(v.manual_memo_id) {
                        active_manual_memo_id = None;
                    }
                }
                InstructionValue::CallExpression(v) => {
                    let callee_is_set_state = is_set_state_type(&v.callee.identifier.type_)
                        || unconditional_set_state_fns.contains(&v.callee.identifier.id);

                    if callee_is_set_state {
                        if active_manual_memo_id.is_some() {
                            // setState inside useMemo
                            errors.push_diagnostic(
                                CompilerDiagnostic::create(
                                    ErrorCategory::RenderSetState,
                                    "Calling setState from useMemo may trigger an infinite loop".to_string(),
                                    Some(
                                        "Each time the memo callback is evaluated it will change state. \
                                         This can cause a memoization dependency to change, running the \
                                         memo function again and causing an infinite loop. Instead of \
                                         setting state in useMemo(), prefer deriving the value during \
                                         render. (https://react.dev/reference/react/useState)"
                                            .to_string(),
                                    ),
                                    None,
                                )
                                .with_detail(CompilerDiagnosticDetail::Error {
                                    loc: Some(v.callee.loc),
                                    message: Some(
                                        "Found setState() within useMemo()".to_string(),
                                    ),
                                }),
                            );
                        } else if is_unconditional {
                            errors.push_diagnostic(
                                CompilerDiagnostic::create(
                                    ErrorCategory::RenderSetState,
                                    "Cannot call setState during render".to_string(),
                                    Some(
                                        "Calling setState during render may trigger an infinite loop.\n\
                                         * To reset state when other state/props change, store the \
                                         previous value in state and update conditionally: \
                                         https://react.dev/reference/react/useState\
                                         #storing-information-from-previous-renders\n\
                                         * To derive data from other state/props, compute the derived \
                                         data during render without using state"
                                            .to_string(),
                                    ),
                                    None,
                                )
                                .with_detail(CompilerDiagnosticDetail::Error {
                                    loc: Some(v.callee.loc),
                                    message: Some(
                                        "Found setState() in render".to_string(),
                                    ),
                                }),
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }

    errors.into_result()
}

fn is_set_state_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Function(FunctionType { shape_id: Some(id), .. })
        if id == BUILT_IN_SET_STATE_ID
    )
}

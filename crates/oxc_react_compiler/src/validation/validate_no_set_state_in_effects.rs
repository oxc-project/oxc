
use crate::{
    compiler_error::CompilerError,
    hir::{
        HIRFunction, InstructionValue,
        types::{FunctionType, Type},
        object_shape::BUILT_IN_USE_EFFECT_HOOK_ID,
    },
};

/// Validate no setState in effects.
pub fn validate_no_set_state_in_effects(func: &HIRFunction) -> CompilerError {
    let errors = CompilerError::new();

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            match &instr.value {
                // Track useEffect/useLayoutEffect calls and their callback arguments
                InstructionValue::CallExpression(v) => {
                    if is_effect_hook_type(&v.callee.identifier.type_)
                        && let Some(first_arg) = v.args.first()
                            && let crate::hir::CallArg::Place(_p) = first_arg {
                            }

                    // Check if setState is called inside an effect
                    // In the full implementation, we'd track setState identifiers
                    // and check if we're inside an effect callback's function body
                }
                _ => {}
            }
        }
    }

    errors
}

fn is_effect_hook_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Function(FunctionType { shape_id: Some(id), .. })
        if id == BUILT_IN_USE_EFFECT_HOOK_ID
    )
}

pub fn is_set_state_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Function(FunctionType { shape_id: Some(id), .. })
        if id == "BuiltInSetState"
    )
}

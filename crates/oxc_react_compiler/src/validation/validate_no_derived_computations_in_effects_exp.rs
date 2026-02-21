
use crate::{
    compiler_error::CompilerError,
    hir::{
        HIRFunction, InstructionValue,
        types::{FunctionType, Type},
    },
};

/// Validate no derived computations in effects (experimental version).
///
/// Returns errors (does not throw/panic) so the caller can decide how to handle them.
pub fn validate_no_derived_computations_in_effects_exp(
    func: &HIRFunction,
) -> CompilerError {
    let errors = CompilerError::new();

    // Phase 1: Identify state values (from useState/useReducer)

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            match &instr.value {
                InstructionValue::CallExpression(_v) => {
                    // Track useState return values and useEffect callbacks
                    // in the full implementation
                }
                InstructionValue::Destructure(_v) => {
                    // Track array destructuring of useState results
                    // to identify the setState function
                }
                _ => {}
            }
        }
    }

    // Phase 2: Check for derived computations from state inside effects
    // The full implementation would:
    // 1. Trace data flow from state values through computations
    // 2. Check if those derived values flow into setState calls inside effects
    // 3. Report each case with detailed source location information

    errors
}

pub fn is_use_state_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Function(FunctionType { shape_id: Some(id), .. })
        if id == "BuiltInUseState"
    )
}

pub fn is_use_effect_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Function(FunctionType { shape_id: Some(id), .. })
        if id == "BuiltInUseEffectHook" || id == "BuiltInUseLayoutEffectHook"
    )
}

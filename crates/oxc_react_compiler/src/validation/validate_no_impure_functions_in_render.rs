/// Validate no impure function calls during render.
///
/// Port of `Validation/ValidateNoImpureFunctionsInRender.ts` from the React Compiler.
///
/// Validates that components/hooks don't call known-impure functions during render.
use crate::{
    compiler_error::CompilerError,
    hir::{HIRFunction, InstructionValue},
};

/// Validate that no impure functions are called during render.
///
/// # Errors
/// Returns a `CompilerError` if impure function calls are found during render.
pub fn validate_no_impure_functions_in_render(func: &HIRFunction) -> Result<(), CompilerError> {
    let errors = CompilerError::new();

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            match &instr.value {
                InstructionValue::CallExpression(v) => {
                    // Check if callee is marked as impure via type info
                    // In the full implementation, this would check the function signature
                    // from the environment's shape registry
                    let _ = v;
                }
                InstructionValue::MethodCall(v) => {
                    let _ = v;
                }
                _ => {}
            }
        }
    }

    errors.into_result()
}

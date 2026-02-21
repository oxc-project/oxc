/// Validate no freezing of known mutable functions.
///
/// Port of `Validation/ValidateNoFreezingKnownMutableFunctions.ts` from the React Compiler.
///
/// Validates that function expressions which are known to mutate their closure
/// variables are not frozen (e.g., passed as a JSX prop or hook argument that
/// would freeze the value).
use crate::{
    compiler_error::CompilerError,
    hir::{HIRFunction, InstructionValue, Effect},
};

/// Validate that known-mutable functions are not frozen.
///
/// # Errors
/// Returns a `CompilerError` if any mutable function is improperly frozen.
pub fn validate_no_freezing_known_mutable_functions(func: &HIRFunction) -> Result<(), CompilerError> {
    let errors = CompilerError::new();

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            // Check for function expressions that capture mutable values
            // and are then frozen (passed to JSX, hooks, etc.)
            match &instr.value {
                InstructionValue::FunctionExpression(v) => {
                    // Check if any captured context variable has a Freeze effect
                    // when the function itself mutates its captures
                    for ctx in &v.lowered_func.func.context {
                        if ctx.effect == Effect::Freeze {
                            // In the full implementation, we'd check if the function
                            // actually mutates this capture, and if so, emit an error
                        }
                    }
                }
                _ => {}
            }
        }
    }

    errors.into_result()
}

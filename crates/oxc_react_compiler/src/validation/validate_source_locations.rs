/// Validate source locations in the compiled output.
///
/// Port of `Validation/ValidateSourceLocations.ts` from the React Compiler.
///
/// Validates that the compiled output preserves source locations from the
/// original code, which is important for source maps and debugging.
use crate::compiler_error::{
    CompilerError, CompilerErrorDetail, CompilerErrorDetailOptions, ErrorCategory,
};
use crate::reactive_scopes::codegen_reactive_function::CodegenOutput;

/// Validate that source locations are preserved in the output.
///
/// # Errors
/// Returns a `CompilerError` with a Todo diagnostic when source location
/// validation is enabled, since the Rust port does not yet fully implement
/// source-location tracking in the generated AST.
pub fn validate_source_locations(_codegen: &CodegenOutput<'_>) -> Result<(), CompilerError> {
    let mut errors = CompilerError::new();
    errors.push_error_detail(CompilerErrorDetail::new(CompilerErrorDetailOptions {
        category: ErrorCategory::Todo,
        reason: "Important source location missing in generated code".to_string(),
        description: Some(
            "Source location validation is not yet implemented in the Rust port. \
             This can cause coverage instrumentation to fail to track this code properly, \
             resulting in inaccurate coverage reports."
                .to_string(),
        ),
        loc: None,
        suggestions: None,
    }));
    Err(errors)
}

/// Validate static components.
///
/// Port of `Validation/ValidateStaticComponents.ts` from the React Compiler.
///
/// Validates that components are static — not recreated every render.
/// Components that are recreated dynamically can reset state and trigger
/// excessive re-rendering.
use crate::{
    compiler_error::{CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory},
    hir::{HIRFunction, InstructionValue, JsxTag},
};

/// Validate that components referenced in JSX are static.
pub fn validate_static_components(func: &HIRFunction) -> CompilerError {
    let mut errors = CompilerError::new();

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            if let InstructionValue::JsxExpression(jsx) = &instr.value {
                // Check if the tag is a dynamically-created component
                if let JsxTag::Place(place) = &jsx.tag {
                    // In the full implementation, we'd check if the place
                    // is a function expression created within the current
                    // render scope (not a stable reference)
                    let is_reactive = place.reactive;
                    if is_reactive {
                        // A reactive tag means the component might be recreated
                        // each render, which would reset its state
                        errors.push_diagnostic(
                            CompilerDiagnostic::create(
                                ErrorCategory::StaticComponents,
                                "Components should be static".to_string(),
                                Some("This component reference may change between renders, which would reset its state".to_string()),
                                None,
                            )
                            .with_detail(CompilerDiagnosticDetail::Error {
                                loc: Some(jsx.opening_loc),
                                message: Some("dynamic component reference".to_string()),
                            }),
                        );
                    }
                }
            }
        }
    }

    errors
}

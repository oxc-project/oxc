/// Validate no impure function calls during render.
///
/// Port of `Validation/ValidateNoImpureFunctionsInRender.ts` from the React Compiler.
///
/// Checks that known-impure functions are not called during render. Examples of
/// invalid functions to call during render are `Math.random()` and `Date.now()`.
/// Users may extend this set of impure functions via a module type provider
/// and specifying functions with `impure: true`.
use crate::{
    compiler_error::{
        CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory, SourceLocation,
    },
    hir::{HIRFunction, InstructionValue, object_shape::FunctionSignature, types::Type},
};

/// Validate that no impure functions are called during render.
///
/// # Errors
/// Returns a `CompilerError` if impure function calls are found during render.
pub fn validate_no_impure_functions_in_render(func: &HIRFunction) -> Result<(), CompilerError> {
    let mut errors = CompilerError::new();

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            match &instr.value {
                InstructionValue::CallExpression(v) => {
                    if let Some(sig) =
                        get_function_call_signature(&func.env.shapes, &v.callee.identifier.type_)
                        && sig.impure
                    {
                        emit_impure_error(&mut errors, v.callee.loc, sig);
                    }
                }
                InstructionValue::MethodCall(v) => {
                    if let Some(sig) =
                        get_function_call_signature(&func.env.shapes, &v.property.identifier.type_)
                        && sig.impure
                    {
                        emit_impure_error(&mut errors, v.property.loc, sig);
                    }
                }
                _ => {}
            }
        }
    }

    errors.into_result()
}

fn emit_impure_error(errors: &mut CompilerError, loc: SourceLocation, sig: &FunctionSignature) {
    let description = if let Some(ref name) = sig.canonical_name {
        format!(
            "`{name}` is an impure function. \
             Calling an impure function can produce unstable results that update \
             unpredictably when the component happens to re-render. \
             (https://react.dev/reference/rules/components-and-hooks-must-be-pure\
             #components-and-hooks-must-be-idempotent)"
        )
    } else {
        "Calling an impure function can produce unstable results that update \
         unpredictably when the component happens to re-render. \
         (https://react.dev/reference/rules/components-and-hooks-must-be-pure\
         #components-and-hooks-must-be-idempotent)"
            .to_string()
    };
    errors.push_diagnostic(
        CompilerDiagnostic::create(
            ErrorCategory::Purity,
            "Cannot call impure function during render".to_string(),
            Some(description),
            None,
        )
        .with_detail(CompilerDiagnosticDetail::Error {
            loc: Some(loc),
            message: Some("Cannot call impure function".to_string()),
        }),
    );
}

/// Look up a function call signature from the shape registry by type.
fn get_function_call_signature<'a>(
    shapes: &'a crate::hir::object_shape::ShapeRegistry,
    ty: &Type,
) -> Option<&'a FunctionSignature> {
    let shape_id = ty.shape_id()?;
    let shape = shapes.get(shape_id)?;
    shape.function_type.as_ref()
}

/// Validate no impure function calls during render.
///
/// Port of `Validation/ValidateNoImpureFunctionsInRender.ts` from the React Compiler.
///
/// Checks that known-impure functions are not called during render. Examples of
/// invalid functions to call during render are `Math.random()` and `Date.now()`.
/// Users may extend this set of impure functions via a module type provider
/// and specifying functions with `impure: true`.
///
/// NOTE: In the TS reference, this file exists but is NOT called in Pipeline.ts.
/// The primary mechanism is the `Impure` effect emitted during inference in
/// `computeEffectsForLegacySignature` (InferMutationAliasingEffects.ts line 2332),
/// which propagates through function expression boundaries via effect bubbling.
/// The Rust port now also emits `Impure` effects during inference (in
/// `effects_from_signature`), making this separate pass a redundant safety net
/// for top-level impure calls only.
use crate::{
    compiler_error::{
        CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory, SourceLocation,
    },
    hir::{
        HIRFunction, Instruction, InstructionValue, environment::Environment,
        object_shape::FunctionSignature, types::Type,
    },
    validation::dispatcher::InstructionVisitor,
};

/// Validate that no impure functions are called during render.
///
/// # Errors
/// Returns a `CompilerError` if impure function calls are found during render.
pub fn validate_no_impure_functions_in_render(func: &HIRFunction) -> Result<(), CompilerError> {
    crate::validation::dispatcher::dispatch_instruction_visitors(
        func,
        vec![Box::new(ValidateNoImpureFunctionsInRender::default())],
    )
}

/// `InstructionVisitor` impl for `validate_no_impure_functions_in_render`.
///
/// Stateless: only inspects each `CallExpression` / `MethodCall`'s callee shape.
#[derive(Default)]
pub struct ValidateNoImpureFunctionsInRender {
    errors: CompilerError,
}

impl InstructionVisitor for ValidateNoImpureFunctionsInRender {
    fn visit_instruction(&mut self, env: &Environment, instr: &Instruction) {
        match &instr.value {
            InstructionValue::CallExpression(v) => {
                if let Some(sig) =
                    get_function_call_signature(env.shapes(), &v.callee.identifier.type_)
                    && sig.impure
                {
                    emit_impure_error(&mut self.errors, v.callee.loc, sig);
                }
            }
            InstructionValue::MethodCall(v) => {
                if let Some(sig) =
                    get_function_call_signature(env.shapes(), &v.property.identifier.type_)
                    && sig.impure
                {
                    emit_impure_error(&mut self.errors, v.property.loc, sig);
                }
            }
            _ => {}
        }
    }

    fn finish(self: Box<Self>, _env: &Environment) -> Result<(), CompilerError> {
        self.errors.into_result()
    }
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

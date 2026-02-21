/// Validate no setState calls during render.
///
/// Port of `Validation/ValidateNoSetStateInRender.ts` from the React Compiler.
///
/// Validates that the given function does not have an infinite update loop
/// caused by unconditionally calling setState during render.
use rustc_hash::FxHashSet;

use crate::{
    compiler_error::{
        CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory,
    },
    hir::{
        HIRFunction, IdentifierId, InstructionValue,
        compute_unconditional_blocks::compute_unconditional_blocks,
        types::Type,
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
    let set_state_ids: FxHashSet<IdentifierId> = FxHashSet::default();

    for (&block_id, block) in &func.body.blocks {
        let is_unconditional = unconditional_blocks.contains(&block_id);

        for instr in &block.instructions {
            match &instr.value {
                // Track identifiers that are setState functions
                InstructionValue::Destructure(_) | InstructionValue::StoreLocal(_) => {
                    // In a full implementation, we'd track setState bindings from
                    // useState destructuring. For now, we rely on type info.
                }
                InstructionValue::CallExpression(v) => {
                    let callee_is_set_state = is_set_state_type(&v.callee.identifier.type_)
                        || set_state_ids.contains(&v.callee.identifier.id)
                        || unconditional_set_state_fns.contains(&v.callee.identifier.id);

                    if callee_is_set_state && is_unconditional {
                        errors.push_diagnostic(
                            CompilerDiagnostic::create(
                                ErrorCategory::RenderSetState,
                                "Unexpected setState during render".to_string(),
                                Some(
                                    "Calling setState during render can cause an infinite loop"
                                        .to_string(),
                                ),
                                None,
                            )
                            .with_detail(CompilerDiagnosticDetail::Error {
                                loc: Some(v.loc),
                                message: Some("setState called during render".to_string()),
                            }),
                        );
                    }
                }
                // Track function expressions that unconditionally call setState
                InstructionValue::FunctionExpression(v) => {
                    let inner_result =
                        validate_impl(&v.lowered_func.func, unconditional_set_state_fns);
                    if inner_result.is_err() {
                        unconditional_set_state_fns.insert(instr.lvalue.identifier.id);
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
        Type::Function(crate::hir::types::FunctionType {
            shape_id: Some(id),
            ..
        }) if id == "BuiltInSetState"
    )
}

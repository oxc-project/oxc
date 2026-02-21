/// Validate no ref access during render.
///
/// Port of `Validation/ValidateNoRefAccessInRender.ts` from the React Compiler.
///
/// Validates that ref values (.current) are not read during render,
/// as refs are mutable and reading them during render breaks React's
/// rendering model.
use rustc_hash::FxHashSet;

use crate::{
    compiler_error::{
        CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory,
    },
    hir::{
        HIRFunction, IdentifierId, InstructionValue,
        types::{ObjectType, Type},
        object_shape::BUILT_IN_USE_REF_ID,
    },
};

/// Validate no ref access during render.
///
/// # Errors
/// Returns a `CompilerError` if ref values are accessed during render.
pub fn validate_no_ref_access_in_render(func: &HIRFunction) -> Result<(), CompilerError> {
    let mut errors = CompilerError::new();
    let mut ref_ids: FxHashSet<IdentifierId> = FxHashSet::default();
    let mut ref_value_ids: FxHashSet<IdentifierId> = FxHashSet::default();

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            // Track useRef return values
            if is_use_ref_type(&instr.lvalue.identifier.type_) {
                ref_ids.insert(instr.lvalue.identifier.id);
            }

            // Track .current access on refs
            if let InstructionValue::PropertyLoad(v) = &instr.value {
                if ref_ids.contains(&v.object.identifier.id) {
                    let prop = v.property.to_string();
                    if prop == "current" {
                        ref_value_ids.insert(instr.lvalue.identifier.id);
                    }
                }
            }

            // Check for reads of ref.current values during render
            match &instr.value {
                InstructionValue::LoadLocal(v) => {
                    if ref_value_ids.contains(&v.place.identifier.id) {
                        errors.push_diagnostic(
                            CompilerDiagnostic::create(
                                ErrorCategory::Refs,
                                "Ref values (the `current` property) may not be accessed during render"
                                    .to_string(),
                                Some("Reading a ref during render breaks React's rendering model".to_string()),
                                None,
                            )
                            .with_detail(CompilerDiagnosticDetail::Error {
                                loc: Some(v.loc),
                                message: Some("ref.current accessed during render".to_string()),
                            }),
                        );
                    }
                }
                _ => {}
            }
        }
    }

    errors.into_result()
}

fn is_use_ref_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Object(ObjectType { shape_id: Some(id) }) if id == BUILT_IN_USE_REF_ID
    )
}

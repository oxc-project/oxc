/// Validate static components.
///
/// Port of `Validation/ValidateStaticComponents.ts` from the React Compiler.
///
/// Validates against components that are created dynamically and whose identity
/// is not guaranteed to be stable (which would cause the component to reset on
/// each re-render).
use rustc_hash::FxHashMap;

use crate::{
    compiler_error::{
        CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory, SourceLocation,
    },
    hir::{HIRFunction, IdentifierId, InstructionValue, JsxTag},
};

/// Validate that components referenced in JSX are static.
///
/// Tracks dynamic values (function expressions, call results, new expressions,
/// method calls) through loads and stores, then reports an error when such a
/// value is used as a JSX component tag.
pub fn validate_static_components(func: &HIRFunction) -> CompilerError {
    let mut errors = CompilerError::new();
    let mut known_dynamic_components: FxHashMap<IdentifierId, SourceLocation> =
        FxHashMap::default();

    for block in func.body.blocks.values() {
        // Propagate dynamic status through phi nodes
        'phis: for phi in &block.phis {
            for operand in phi.operands.values() {
                if let Some(&loc) = known_dynamic_components.get(&operand.identifier.id) {
                    known_dynamic_components.insert(phi.place.identifier.id, loc);
                    continue 'phis;
                }
            }
        }

        for instr in &block.instructions {
            let lvalue_id = instr.lvalue.identifier.id;
            match &instr.value {
                // Function expressions, new expressions, method calls, and call expressions
                // produce dynamically-created values
                InstructionValue::FunctionExpression(v) => {
                    known_dynamic_components.insert(lvalue_id, v.loc);
                }
                InstructionValue::NewExpression(v) => {
                    known_dynamic_components.insert(lvalue_id, v.loc);
                }
                InstructionValue::MethodCall(v) => {
                    known_dynamic_components.insert(lvalue_id, v.loc);
                }
                InstructionValue::CallExpression(v) => {
                    known_dynamic_components.insert(lvalue_id, v.loc);
                }
                // Propagate dynamic status through loads
                InstructionValue::LoadLocal(v) => {
                    if let Some(&loc) = known_dynamic_components.get(&v.place.identifier.id) {
                        known_dynamic_components.insert(lvalue_id, loc);
                    }
                }
                // Propagate dynamic status through stores
                InstructionValue::StoreLocal(v) => {
                    if let Some(&loc) = known_dynamic_components.get(&v.value.identifier.id) {
                        known_dynamic_components.insert(lvalue_id, loc);
                        known_dynamic_components.insert(v.lvalue.place.identifier.id, loc);
                    }
                }
                // Check JSX usage
                InstructionValue::JsxExpression(jsx) => {
                    if let JsxTag::Place(tag_place) = &jsx.tag
                        && let Some(&location) =
                            known_dynamic_components.get(&tag_place.identifier.id)
                    {
                        errors.push_diagnostic(
                            CompilerDiagnostic::create(
                                ErrorCategory::StaticComponents,
                                "Cannot create components during render".to_string(),
                                Some(
                                    "Components created during render will reset their state \
                                         each time they are created. Declare components outside \
                                         of render"
                                        .to_string(),
                                ),
                                None,
                            )
                            .with_detail(CompilerDiagnosticDetail::Error {
                                loc: Some(tag_place.loc),
                                message: Some(
                                    "This component is created during render".to_string(),
                                ),
                            })
                            .with_detail(
                                CompilerDiagnosticDetail::Error {
                                    loc: Some(location),
                                    message: Some(
                                        "The component is created during render here".to_string(),
                                    ),
                                },
                            ),
                        );
                    }
                }
                _ => {}
            }
        }
    }

    errors
}

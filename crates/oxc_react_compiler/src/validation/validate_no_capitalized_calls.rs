/// Validate no capitalized function calls (they should be JSX components).
///
/// Port of `Validation/ValidateNoCapitalizedCalls.ts` from the React Compiler.
///
/// Capitalized functions are reserved for components, which must be invoked with JSX.
use rustc_hash::FxHashMap;

use crate::{
    compiler_error::{
        CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory,
    },
    hir::{HIRFunction, IdentifierId, InstructionValue},
};

/// Validate that no capitalized functions are called directly.
///
/// # Errors
/// Returns a `CompilerError` if capitalized call violations are found.
pub fn validate_no_capitalized_calls(func: &HIRFunction) -> Result<(), CompilerError> {
    let mut errors = CompilerError::new();
    let mut capital_load_globals: FxHashMap<IdentifierId, String> = FxHashMap::default();
    let mut capitalized_properties: FxHashMap<IdentifierId, String> = FxHashMap::default();

    let reason = "Capitalized functions are reserved for components, which must be invoked with JSX";

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            match &instr.value {
                InstructionValue::LoadGlobal(v) => {
                    let name = v.binding.name().to_string();
                    if !name.is_empty()
                        && name.starts_with(|c: char| c.is_ascii_uppercase())
                        && name != name.to_ascii_uppercase()
                    {
                        capital_load_globals.insert(instr.lvalue.identifier.id, name);
                    }
                }
                InstructionValue::CallExpression(v) => {
                    if let Some(callee_name) =
                        capital_load_globals.get(&v.callee.identifier.id)
                    {
                        errors.push_diagnostic(
                            CompilerDiagnostic::create(
                                ErrorCategory::CapitalizedCalls,
                                reason.to_string(),
                                Some(format!("{callee_name} may be a component")),
                                None,
                            )
                            .with_detail(CompilerDiagnosticDetail::Error {
                                loc: Some(v.loc),
                                message: Some(format!("{callee_name} called as function")),
                            }),
                        );
                    }
                }
                InstructionValue::PropertyLoad(v) => {
                    let prop_str = v.property.to_string();
                    if prop_str.starts_with(|c: char| c.is_ascii_uppercase()) {
                        capitalized_properties
                            .insert(instr.lvalue.identifier.id, prop_str);
                    }
                }
                InstructionValue::MethodCall(v) => {
                    if let Some(prop_name) =
                        capitalized_properties.get(&v.property.identifier.id)
                    {
                        errors.push_diagnostic(
                            CompilerDiagnostic::create(
                                ErrorCategory::CapitalizedCalls,
                                reason.to_string(),
                                Some(format!("{prop_name} may be a component")),
                                None,
                            )
                            .with_detail(CompilerDiagnosticDetail::Error {
                                loc: Some(v.loc),
                                message: Some(format!("{prop_name} called as method")),
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

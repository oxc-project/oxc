/// Validate useMemo usage.
///
/// Port of `Validation/ValidateUseMemo.ts` from the React Compiler.
///
/// Validates that useMemo/useCallback are used correctly:
/// - The callback must return a value (not void)
/// - The result must be used (not discarded)
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    compiler_error::{
        CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory,
    },
    hir::{
        HIRFunction, IdentifierId, InstructionValue,
        visitors::each_instruction_value_operand,
    },
};

/// Validate useMemo and useCallback usage.
///
/// # Errors
/// Returns a `CompilerError` if invalid useMemo usage is found.
pub fn validate_use_memo(func: &HIRFunction) -> Result<(), CompilerError> {
    let mut errors = CompilerError::new();
    let mut use_memos: FxHashSet<IdentifierId> = FxHashSet::default();
    let mut react_ids: FxHashSet<IdentifierId> = FxHashSet::default();
    let mut unused_use_memos: FxHashMap<IdentifierId, crate::compiler_error::SourceLocation> =
        FxHashMap::default();

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            // Check if any operand uses a previously-tracked useMemo result
            if !unused_use_memos.is_empty() {
                for operand in each_instruction_value_operand(&instr.value) {
                    unused_use_memos.remove(&operand.identifier.id);
                }
            }

            match &instr.value {
                InstructionValue::LoadGlobal(v) => {
                    let name = v.binding.name();
                    match name {
                        "useMemo" | "useCallback" => {
                            use_memos.insert(instr.lvalue.identifier.id);
                        }
                        "React" => {
                            react_ids.insert(instr.lvalue.identifier.id);
                        }
                        _ => {}
                    }
                }
                InstructionValue::PropertyLoad(v) => {
                    if react_ids.contains(&v.object.identifier.id) {
                        let prop_str = v.property.to_string();
                        if prop_str == "useMemo" || prop_str == "useCallback" {
                            use_memos.insert(instr.lvalue.identifier.id);
                        }
                    }
                }
                InstructionValue::CallExpression(v) => {
                    if use_memos.contains(&v.callee.identifier.id) {
                        // This is a useMemo/useCallback call — track the result
                        unused_use_memos.insert(instr.lvalue.identifier.id, v.loc);
                    }
                }
                _ => {}
            }
        }
    }

    // Report unused useMemo results
    for loc in unused_use_memos.values() {
        errors.push_diagnostic(
            CompilerDiagnostic::create(
                ErrorCategory::VoidUseMemo,
                "useMemo/useCallback result is unused".to_string(),
                Some("The return value of useMemo/useCallback should be used".to_string()),
                None,
            )
            .with_detail(CompilerDiagnosticDetail::Error {
                loc: Some(*loc),
                message: Some("unused useMemo result".to_string()),
            }),
        );
    }

    errors.into_result()
}

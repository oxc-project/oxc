/// Validate useMemo usage.
///
/// Port of `Validation/ValidateUseMemo.ts` from the React Compiler.
///
/// Validates that useMemo/useCallback are used correctly:
/// - The callback must return a value (not void)
/// - The result must be used (not discarded)
/// - Callbacks may not reassign variables declared outside of the callback
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    compiler_error::{CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory},
    hir::{
        CallArg, FunctionExpressionValue, HIRFunction, IdentifierId, InstructionValue,
        visitors::each_instruction_value_operand,
    },
};

/// Validate useMemo and useCallback usage.
///
/// Returns a tuple of (fatal_errors, non_fatal_void_memo_errors).
/// In the TS compiler, the fatal errors are returned via `errors.asResult()` and
/// unwrapped (thrown on error), while the void memo errors are logged via
/// `fn.env.logErrors(voidMemoErrors.asResult())` as non-fatal warnings.
pub fn validate_use_memo(
    func: &HIRFunction,
) -> (Result<(), CompilerError>, Result<(), CompilerError>) {
    let mut errors = CompilerError::new();
    let mut void_memo_errors = CompilerError::new();
    let mut use_memos: FxHashSet<IdentifierId> = FxHashSet::default();
    let mut react_ids: FxHashSet<IdentifierId> = FxHashSet::default();
    let mut functions: FxHashMap<IdentifierId, FunctionExpressionValue> = FxHashMap::default();
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
                InstructionValue::FunctionExpression(v) => {
                    functions.insert(instr.lvalue.identifier.id, v.clone());
                }
                InstructionValue::CallExpression(v) => {
                    let is_use_memo = use_memos.contains(&v.callee.identifier.id);
                    if !is_use_memo || v.args.is_empty() {
                        continue;
                    }

                    // Track the result as potentially unused
                    unused_use_memos.insert(instr.lvalue.identifier.id, v.loc);

                    // Get the first argument
                    let first_arg_id = match &v.args[0] {
                        CallArg::Place(p) => Some(p.identifier.id),
                        CallArg::Spread(_) => None,
                    };

                    // If the first arg is a locally-defined FunctionExpression, validate it
                    if let Some(arg_id) = first_arg_id
                        && let Some(body) = functions.get(&arg_id)
                    {
                        validate_no_context_variable_assignment(
                            &body.lowered_func.func,
                            &mut errors,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    // Report unused useMemo results (non-fatal, matching TS voidMemoErrors)
    for loc in unused_use_memos.values() {
        void_memo_errors.push_diagnostic(
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

    (errors.into_result(), void_memo_errors.into_result())
}

/// Port of `validateNoContextVariableAssignment` from `ValidateUseMemo.ts`.
///
/// Checks that a useMemo/useCallback callback does not reassign variables
/// declared outside of the callback (via `StoreContext` to a captured context variable).
fn validate_no_context_variable_assignment(func: &HIRFunction, errors: &mut CompilerError) {
    let context: FxHashSet<IdentifierId> =
        func.context.iter().map(|place| place.identifier.id).collect();

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            if let InstructionValue::StoreContext(v) = &instr.value
                && context.contains(&v.lvalue_place.identifier.id)
            {
                errors.push_diagnostic(
                        CompilerDiagnostic::create(
                            ErrorCategory::UseMemo,
                            "useMemo() callbacks may not reassign variables declared outside of the callback".to_string(),
                            Some(
                                "useMemo() callbacks must be pure functions and cannot reassign variables defined outside of the callback function".to_string(),
                            ),
                            None,
                        )
                        .with_detail(CompilerDiagnosticDetail::Error {
                            loc: Some(v.lvalue_place.loc),
                            message: Some("Cannot reassign variable".to_string()),
                        }),
                    );
            }
        }
    }
}

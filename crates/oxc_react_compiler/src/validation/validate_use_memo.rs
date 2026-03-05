/// Validate useMemo usage.
///
/// Port of `Validation/ValidateUseMemo.ts` from the React Compiler.
///
/// Validates that useMemo/useCallback are used correctly:
/// - Callbacks may not accept parameters
/// - Callbacks may not be async or generator functions
/// - Callbacks may not reassign variables declared outside of the callback
/// - (when `validateNoVoidUseMemo` is enabled) The callback must return a value (not void)
/// - (when `validateNoVoidUseMemo` is enabled) The result must be used (not discarded)
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    compiler_error::{
        CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory, SourceLocation,
    },
    hir::{
        CallArg, FunctionExpressionValue, HIRFunction, IdentifierId, InstructionValue,
        ReactiveParam, ReturnVariant, Terminal,
        visitors::{each_instruction_value_operand, each_terminal_operand},
    },
};

/// Validate useMemo and useCallback usage.
///
/// Returns a tuple of (fatal_errors, non_fatal_void_memo_errors).
/// In the TS compiler, the parameter/async/context-variable errors are recorded
/// via `env.recordError()` and the void-memo errors are logged via
/// `fn.env.logErrors(voidMemoErrors.asResult())` as non-fatal warnings.
pub fn validate_use_memo(
    func: &HIRFunction,
) -> (Result<(), CompilerError>, Result<(), CompilerError>) {
    let validate_no_void = func.env.config.validate_no_void_use_memo;
    let mut errors = CompilerError::new();
    let mut void_memo_errors = CompilerError::new();
    let mut use_memos: FxHashSet<IdentifierId> = FxHashSet::default();
    let mut react_ids: FxHashSet<IdentifierId> = FxHashSet::default();
    let mut functions: FxHashMap<IdentifierId, FunctionExpressionValue> = FxHashMap::default();
    let mut unused_use_memos: FxHashMap<IdentifierId, SourceLocation> = FxHashMap::default();

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
                        "useMemo" => {
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
                        if prop_str == "useMemo" {
                            use_memos.insert(instr.lvalue.identifier.id);
                        }
                    }
                }
                InstructionValue::FunctionExpression(v) => {
                    functions.insert(instr.lvalue.identifier.id, v.clone());
                }
                InstructionValue::CallExpression(v) => {
                    let callee_id = v.callee.identifier.id;
                    let is_use_memo = use_memos.contains(&callee_id);
                    if !is_use_memo || v.args.is_empty() {
                        continue;
                    }
                    validate_use_memo_call(
                        &v.args,
                        &functions,
                        &mut errors,
                        &mut void_memo_errors,
                        &mut unused_use_memos,
                        instr.lvalue.identifier.id,
                        v.callee.loc,
                        validate_no_void,
                    );
                }
                InstructionValue::MethodCall(v) => {
                    let callee_id = v.property.identifier.id;
                    let is_use_memo = use_memos.contains(&callee_id);
                    if !is_use_memo || v.args.is_empty() {
                        continue;
                    }
                    validate_use_memo_call(
                        &v.args,
                        &functions,
                        &mut errors,
                        &mut void_memo_errors,
                        &mut unused_use_memos,
                        instr.lvalue.identifier.id,
                        v.property.loc,
                        validate_no_void,
                    );
                }
                _ => {}
            }
        }
        if !unused_use_memos.is_empty() {
            for operand in each_terminal_operand(&block.terminal) {
                unused_use_memos.remove(&operand.identifier.id);
            }
        }
    }

    // Report unused useMemo results (non-fatal, matching TS voidMemoErrors)
    if !unused_use_memos.is_empty() {
        for loc in unused_use_memos.values() {
            void_memo_errors.push_diagnostic(
                CompilerDiagnostic::create(
                    ErrorCategory::VoidUseMemo,
                    "useMemo() result is unused".to_string(),
                    Some("This useMemo() value is unused. useMemo() is for computing and caching values, not for arbitrary side effects".to_string()),
                    None,
                )
                .with_detail(CompilerDiagnosticDetail::Error {
                    loc: Some(*loc),
                    message: Some("useMemo() result is unused".to_string()),
                }),
            );
        }
    }

    (errors.into_result(), void_memo_errors.into_result())
}

/// Validate a single useMemo/useCallback call.
///
/// Shared logic for both CallExpression and MethodCall variants.
#[expect(clippy::too_many_arguments)]
fn validate_use_memo_call(
    args: &[CallArg],
    functions: &FxHashMap<IdentifierId, FunctionExpressionValue>,
    errors: &mut CompilerError,
    void_memo_errors: &mut CompilerError,
    unused_use_memos: &mut FxHashMap<IdentifierId, SourceLocation>,
    lvalue_id: IdentifierId,
    callee_loc: SourceLocation,
    validate_no_void: bool,
) {
    // Get the first argument
    let first_arg_id = match &args[0] {
        CallArg::Place(p) => Some(p.identifier.id),
        CallArg::Spread(_) => None,
    };

    // If the first arg is a locally-defined FunctionExpression, validate it
    if let Some(arg_id) = first_arg_id
        && let Some(body) = functions.get(&arg_id)
    {
        // Validate: callbacks may not accept parameters
        if !body.lowered_func.func.params.is_empty() {
            let first_param = &body.lowered_func.func.params[0];
            let loc = match first_param {
                ReactiveParam::Place(p) => p.loc,
                ReactiveParam::Spread(p) => p.place.loc,
            };
            errors.push_diagnostic(
                CompilerDiagnostic::create(
                    ErrorCategory::UseMemo,
                    "useMemo() callbacks may not accept parameters".to_string(),
                    Some("useMemo() callbacks are called by React to cache calculations across re-renders. They should not take parameters. Instead, directly reference the props, state, or local variables needed for the computation".to_string()),
                    None,
                )
                .with_detail(CompilerDiagnosticDetail::Error {
                    loc: Some(loc),
                    message: Some("Callbacks with parameters are not supported".to_string()),
                }),
            );
        }

        // Validate: callbacks may not be async or generator functions
        if body.lowered_func.func.is_async || body.lowered_func.func.generator {
            errors.push_diagnostic(
                CompilerDiagnostic::create(
                    ErrorCategory::UseMemo,
                    "useMemo() callbacks may not be async or generator functions".to_string(),
                    Some(
                        "useMemo() callbacks are called once and must synchronously return a value"
                            .to_string(),
                    ),
                    None,
                )
                .with_detail(CompilerDiagnosticDetail::Error {
                    loc: Some(body.loc),
                    message: Some("Async and generator functions are not supported".to_string()),
                }),
            );
        }

        validate_no_context_variable_assignment(&body.lowered_func.func, errors);

        if validate_no_void {
            if has_non_void_return(&body.lowered_func.func) {
                unused_use_memos.insert(lvalue_id, callee_loc);
            } else {
                void_memo_errors.push_diagnostic(
                    CompilerDiagnostic::create(
                        ErrorCategory::VoidUseMemo,
                        "useMemo() callbacks must return a value".to_string(),
                        Some("This useMemo() callback doesn't return a value. useMemo() is for computing and caching values, not for arbitrary side effects".to_string()),
                        None,
                    )
                    .with_detail(CompilerDiagnosticDetail::Error {
                        loc: Some(body.loc),
                        message: Some("useMemo() callbacks must return a value".to_string()),
                    }),
                );
            }
        }
    }
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

/// Port of `hasNonVoidReturn` from `ValidateUseMemo.ts`.
///
/// Returns true if the function has at least one non-void return statement
/// (either explicit `return value` or implicit arrow return).
fn has_non_void_return(func: &HIRFunction) -> bool {
    for block in func.body.blocks.values() {
        if let Terminal::Return(ret) = &block.terminal
            && matches!(ret.return_variant, ReturnVariant::Explicit | ReturnVariant::Implicit)
        {
            return true;
        }
    }
    false
}

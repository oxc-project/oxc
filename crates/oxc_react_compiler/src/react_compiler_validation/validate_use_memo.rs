use rustc_hash::{FxHashMap, FxHashSet};

use oxc_index::IndexSlice;

use oxc_diagnostics::Diagnostics;

use crate::diagnostics;
use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::visitors::{
    each_instruction_value_operand_with_functions, each_terminal_operand,
};
use crate::react_compiler_hir::{
    FunctionId, HirFunction, Identifier, IdentifierId, InstructionValue, ParamPattern, Place,
    PlaceOrSpread, PropertyLiteral, ReturnVariant, Terminal,
};
use oxc_span::Span;

/// Validates useMemo() usage patterns.
///
/// Port of ValidateUseMemo.ts.
/// Returns VoidUseMemo errors separately (for logging via logErrors, not as compile errors).
pub fn validate_use_memo(func: &HirFunction, env: &mut Environment) -> Diagnostics {
    validate_use_memo_impl(
        func,
        &env.functions,
        &env.identifiers,
        &mut env.errors,
        env.config.validate_no_void_use_memo,
    )
}

/// Information about a FunctionExpression needed for validation.
struct FuncExprInfo {
    func_id: FunctionId,
    span: Option<Span>,
}

fn validate_use_memo_impl(
    func: &HirFunction,
    functions: &IndexSlice<FunctionId, [HirFunction]>,
    identifiers: &IndexSlice<IdentifierId, [Identifier<'_>]>,
    errors: &mut Diagnostics,
    validate_no_void_use_memo: bool,
) -> Diagnostics {
    let mut void_memo_errors = Diagnostics::new();
    let mut use_memos: FxHashSet<IdentifierId> = FxHashSet::default();
    let mut react: FxHashSet<IdentifierId> = FxHashSet::default();
    let mut func_exprs: FxHashMap<IdentifierId, FuncExprInfo> = FxHashMap::default();
    let mut unused_use_memos: FxHashMap<IdentifierId, (Span, Option<String>)> =
        FxHashMap::default();

    for (_block_id, block) in &func.body.blocks {
        for &instr_id in &block.instructions {
            let instr = &func.instructions[instr_id.index()];
            let lvalue = &instr.lvalue;
            let value = &instr.value;

            // Remove used operands from unused_use_memos
            if !unused_use_memos.is_empty() {
                for operand_id in each_instruction_value_operand_ids(value, functions) {
                    unused_use_memos.remove(&operand_id);
                }
            }

            match value {
                InstructionValue::LoadGlobal { binding, .. } => {
                    let name = binding.name();
                    if name == "useMemo" {
                        use_memos.insert(lvalue.identifier);
                    } else if name == "React" {
                        react.insert(lvalue.identifier);
                    }
                }
                InstructionValue::PropertyLoad { object, property, .. } => {
                    if react.contains(&object.identifier)
                        && let PropertyLiteral::String(prop_name) = property
                        && prop_name == "useMemo"
                    {
                        use_memos.insert(lvalue.identifier);
                    }
                }
                InstructionValue::FunctionExpression { lowered_func, span, .. } => {
                    func_exprs.insert(
                        lvalue.identifier,
                        FuncExprInfo { func_id: lowered_func.func, span: *span },
                    );
                }
                InstructionValue::CallExpression { callee, args, .. } => {
                    handle_possible_use_memo_call(
                        functions,
                        identifiers,
                        errors,
                        &mut void_memo_errors,
                        &use_memos,
                        &func_exprs,
                        &mut unused_use_memos,
                        callee,
                        args,
                        lvalue,
                        validate_no_void_use_memo,
                    );
                }
                InstructionValue::MethodCall { property, args, .. } => {
                    handle_possible_use_memo_call(
                        functions,
                        identifiers,
                        errors,
                        &mut void_memo_errors,
                        &use_memos,
                        &func_exprs,
                        &mut unused_use_memos,
                        property,
                        args,
                        lvalue,
                        validate_no_void_use_memo,
                    );
                }
                _ => {}
            }
        }

        // Check terminal operands for unused_use_memos
        if !unused_use_memos.is_empty() {
            for operand_id in each_terminal_operand_ids(&block.terminal) {
                unused_use_memos.remove(&operand_id);
            }
        }
    }

    // Report unused useMemo results
    if !unused_use_memos.is_empty() {
        for (span, _) in unused_use_memos.values() {
            void_memo_errors.push(diagnostics::unused_use_memo(*span));
        }
    }

    void_memo_errors
}

#[allow(clippy::too_many_arguments)]
fn handle_possible_use_memo_call(
    functions: &IndexSlice<FunctionId, [HirFunction]>,
    identifiers: &IndexSlice<IdentifierId, [Identifier<'_>]>,
    errors: &mut Diagnostics,
    void_memo_errors: &mut Diagnostics,
    use_memos: &FxHashSet<IdentifierId>,
    func_exprs: &FxHashMap<IdentifierId, FuncExprInfo>,
    unused_use_memos: &mut FxHashMap<IdentifierId, (Span, Option<String>)>,
    callee: &Place,
    args: &[PlaceOrSpread],
    lvalue: &Place,
    validate_no_void_use_memo: bool,
) {
    let is_use_memo = use_memos.contains(&callee.identifier);
    if !is_use_memo || args.is_empty() {
        return;
    }

    let first_arg = match &args[0] {
        PlaceOrSpread::Place(place) => place,
        PlaceOrSpread::Spread(_) => return,
    };

    let body_info = match func_exprs.get(&first_arg.identifier) {
        Some(info) => info,
        None => return,
    };

    let body_func = &functions[body_info.func_id];

    // Validate no parameters
    if !body_func.params.is_empty() {
        let first_param = &body_func.params[0];
        let span = match first_param {
            ParamPattern::Place(place) => place.span,
            ParamPattern::Spread(spread) => spread.place.span,
        };
        errors.push(diagnostics::use_memo_callback_parameters(span));
    }

    // Validate not async or generator
    if body_func.is_async || body_func.generator {
        errors.push(diagnostics::async_or_generator_use_memo(
            body_func.diagnostic_span().or(body_info.span),
        ));
    }

    // Validate no context variable assignment
    validate_no_context_variable_assignment(body_func, identifiers, errors);

    if validate_no_void_use_memo && !has_non_void_return(body_func) {
        void_memo_errors
            .push(diagnostics::use_memo_no_return(body_func.diagnostic_span().or(body_info.span)));
    } else if validate_no_void_use_memo && let Some(callee_span) = callee.span {
        // The callee is always useMemo/React.useMemo since we checked is_use_memo above.
        // The identifierName in Babel's AST Span is "useMemo".
        unused_use_memos.insert(lvalue.identifier, (callee_span, Some("useMemo".to_string())));
    }
}

fn validate_no_context_variable_assignment(
    func: &HirFunction,
    identifiers: &IndexSlice<IdentifierId, [Identifier<'_>]>,
    errors: &mut Diagnostics,
) {
    let context: FxHashMap<IdentifierId, Option<Span>> = func
        .context
        .iter()
        .map(|place| (place.identifier, identifiers[place.identifier].span.or(place.span)))
        .collect();

    for (_block_id, block) in &func.body.blocks {
        for &instr_id in &block.instructions {
            let instr = &func.instructions[instr_id.index()];
            if let InstructionValue::StoreContext { lvalue, .. } = &instr.value
                && let Some(origin_span) = context.get(&lvalue.place.identifier)
            {
                errors.push(diagnostics::use_memo_reassigns_outer_variable(
                    lvalue.place.span,
                    *origin_span,
                ));
            }
        }
    }
}

fn has_non_void_return(func: &HirFunction) -> bool {
    for (_block_id, block) in &func.body.blocks {
        if let Terminal::Return { return_variant, .. } = &block.terminal
            && matches!(return_variant, ReturnVariant::Explicit | ReturnVariant::Implicit)
        {
            return true;
        }
    }
    false
}

/// Collect all operand IdentifierIds from an InstructionValue.
/// Thin wrapper around canonical `each_instruction_value_operand_with_functions` that maps to ids.
fn each_instruction_value_operand_ids(
    value: &InstructionValue,
    functions: &IndexSlice<FunctionId, [HirFunction]>,
) -> Vec<IdentifierId> {
    each_instruction_value_operand_with_functions(value, functions)
        .into_iter()
        .map(|p| p.identifier)
        .collect()
}

/// Collect all operand IdentifierIds from a Terminal.
/// Thin wrapper around canonical `each_terminal_operand` that maps to ids.
fn each_terminal_operand_ids(terminal: &Terminal) -> Vec<IdentifierId> {
    each_terminal_operand(terminal).into_iter().map(|p| p.identifier).collect()
}

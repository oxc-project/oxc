/// Validate locals not reassigned after render.
///
/// Port of `Validation/ValidateLocalsNotReassignedAfterRender.ts` from the React Compiler.
///
/// Validates that local variables cannot be reassigned after render.
/// This prevents a category of bugs in which a closure captures a
/// binding from one render but does not update.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    compiler_error::{CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory},
    hir::{
        Effect, HIRFunction, IdentifierId, InstructionValue, Place,
        object_shape::ShapeRegistry,
        types::Type,
        visitors::{
            each_instruction_lvalue, each_instruction_value_operand, each_terminal_operand,
        },
    },
};

/// Validate that locals are not reassigned after render.
///
/// # Errors
///
/// Returns a `CompilerError` if a context variable is reassigned
/// after render, or if an async function reassigns a context variable.
pub fn validate_locals_not_reassigned_after_render(
    func: &HIRFunction,
) -> Result<(), CompilerError> {
    let mut context_variables: FxHashSet<IdentifierId> = FxHashSet::default();
    let reassignment =
        get_context_reassignment(func, &func.env.shapes, &mut context_variables, false, false);
    if let Some(reassignment) = reassignment {
        let mut errors = CompilerError::new();
        let variable = get_variable_name(&reassignment);
        errors.push_diagnostic(
            CompilerDiagnostic::create(
                ErrorCategory::Immutability,
                "Cannot reassign variable after render completes".to_string(),
                Some(format!(
                    "Reassigning {variable} after render has completed can cause inconsistent \
                     behavior on subsequent renders. Consider using state instead"
                )),
                None,
            )
            .with_detail(CompilerDiagnosticDetail::Error {
                loc: Some(reassignment.loc),
                message: Some(format!("Cannot reassign {variable} after render completes")),
            }),
        );
        return Err(errors);
    }
    Ok(())
}

fn get_variable_name(place: &Place) -> String {
    if let Some(ref name) = place.identifier.name
        && let crate::hir::IdentifierName::Named(val) = name
    {
        return format!("`{val}`");
    }
    "variable".to_string()
}

/// Check whether a type has a function signature with `no_alias=true`.
///
/// This mirrors `getFunctionCallSignature` + `signature?.noAlias` from the TypeScript port.
fn has_no_alias(shapes: &ShapeRegistry, ty: &Type) -> bool {
    let Some(shape_id) = ty.shape_id() else {
        return false;
    };
    let Some(shape) = shapes.get(shape_id) else {
        return false;
    };
    let Some(sig) = &shape.function_type else {
        return false;
    };
    sig.no_alias
}

fn get_context_reassignment(
    func: &HIRFunction,
    shapes: &ShapeRegistry,
    context_variables: &mut FxHashSet<IdentifierId>,
    is_function_expression: bool,
    is_async: bool,
) -> Option<Place> {
    let mut reassigning_functions: FxHashMap<IdentifierId, Place> = FxHashMap::default();

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            let lvalue_id = instr.lvalue.identifier.id;
            match &instr.value {
                InstructionValue::FunctionExpression(v) => {
                    let mut reassignment = get_context_reassignment(
                        &v.lowered_func.func,
                        shapes,
                        context_variables,
                        true,
                        is_async || v.lowered_func.func.is_async,
                    );
                    if reassignment.is_none() {
                        // Check if any operand is a reassigning function
                        for operand in each_instruction_value_operand(&instr.value) {
                            if let Some(r) = reassigning_functions.get(&operand.identifier.id) {
                                reassignment = Some(r.clone());
                                break;
                            }
                        }
                    }
                    if let Some(ref r) = reassignment {
                        if is_async || v.lowered_func.func.is_async {
                            // Async function reassignment is an immediate error
                            let variable = get_variable_name(r);
                            let mut errors = CompilerError::new();
                            errors.push_diagnostic(
                                CompilerDiagnostic::create(
                                    ErrorCategory::Immutability,
                                    "Cannot reassign variable in async function".to_string(),
                                    Some(
                                        "Reassigning a variable in an async function can cause \
                                         inconsistent behavior on subsequent renders. Consider \
                                         using state instead"
                                            .to_string(),
                                    ),
                                    None,
                                )
                                .with_detail(
                                    CompilerDiagnosticDetail::Error {
                                        loc: Some(r.loc),
                                        message: Some(format!("Cannot reassign {variable}")),
                                    },
                                ),
                            );
                            // In TS this throws; we just return the reassignment
                            return Some(r.clone());
                        }
                        reassigning_functions.insert(lvalue_id, r.clone());
                    }
                }
                InstructionValue::ObjectMethod(v) => {
                    let mut reassignment = get_context_reassignment(
                        &v.lowered_func.func,
                        shapes,
                        context_variables,
                        true,
                        is_async || v.lowered_func.func.is_async,
                    );
                    if reassignment.is_none() {
                        for operand in each_instruction_value_operand(&instr.value) {
                            if let Some(r) = reassigning_functions.get(&operand.identifier.id) {
                                reassignment = Some(r.clone());
                                break;
                            }
                        }
                    }
                    if let Some(ref r) = reassignment {
                        if is_async || v.lowered_func.func.is_async {
                            return Some(r.clone());
                        }
                        reassigning_functions.insert(lvalue_id, r.clone());
                    }
                }
                InstructionValue::StoreLocal(v) => {
                    if let Some(r) = reassigning_functions.get(&v.value.identifier.id) {
                        let r = r.clone();
                        reassigning_functions.insert(v.lvalue.place.identifier.id, r.clone());
                        reassigning_functions.insert(lvalue_id, r);
                    }
                }
                InstructionValue::LoadLocal(v) => {
                    if let Some(r) = reassigning_functions.get(&v.place.identifier.id) {
                        reassigning_functions.insert(lvalue_id, r.clone());
                    }
                }
                InstructionValue::DeclareContext(v) => {
                    if !is_function_expression {
                        context_variables.insert(v.lvalue_place.identifier.id);
                    }
                }
                InstructionValue::StoreContext(v) => {
                    if is_function_expression {
                        if context_variables.contains(&v.lvalue_place.identifier.id) {
                            return Some(v.lvalue_place.clone());
                        }
                    } else {
                        // Track reassignments of variables defined in the outer
                        // component or hook
                        context_variables.insert(v.lvalue_place.identifier.id);
                    }
                    if let Some(r) = reassigning_functions.get(&v.value.identifier.id) {
                        let r = r.clone();
                        reassigning_functions.insert(v.lvalue_place.identifier.id, r.clone());
                        reassigning_functions.insert(lvalue_id, r);
                    }
                }
                _ => {
                    // If we're calling a function that doesn't let its arguments escape
                    // (noAlias=true), only examine the callee/receiver/property — not the
                    // callback arguments. This prevents a callback that mutates a local
                    // variable during render (e.g. inside `x.map(cb)`) from being treated
                    // as a post-render reassignment just because it's passed to a call.
                    let all_operands = each_instruction_value_operand(&instr.value);
                    let operands: Vec<&Place> = match &instr.value {
                        InstructionValue::CallExpression(v) => {
                            if has_no_alias(shapes, &v.callee.identifier.type_) {
                                vec![&v.callee]
                            } else {
                                all_operands
                            }
                        }
                        InstructionValue::MethodCall(v) => {
                            if has_no_alias(shapes, &v.property.identifier.type_) {
                                vec![&v.receiver, &v.property]
                            } else {
                                all_operands
                            }
                        }
                        InstructionValue::TaggedTemplateExpression(v) => {
                            if has_no_alias(shapes, &v.tag.identifier.type_) {
                                vec![&v.tag]
                            } else {
                                all_operands
                            }
                        }
                        _ => all_operands,
                    };
                    for operand in &operands {
                        if let Some(r) = reassigning_functions.get(&operand.identifier.id) {
                            // Functions that reassign local variables are inherently mutable
                            // and are unsafe to pass to a place that expects a frozen value
                            if operand.effect == Effect::Freeze {
                                return Some(r.clone());
                            }
                            // If the operand is not frozen but it does reassign,
                            // then the lvalues of the instruction could also be reassigning
                            let r_cloned = r.clone();
                            for lval in each_instruction_lvalue(instr) {
                                reassigning_functions.insert(lval.identifier.id, r_cloned.clone());
                            }
                        }
                    }
                }
            }
        }
        // Check terminal operands
        for operand in each_terminal_operand(&block.terminal) {
            if let Some(r) = reassigning_functions.get(&operand.identifier.id) {
                return Some(r.clone());
            }
        }
    }
    None
}

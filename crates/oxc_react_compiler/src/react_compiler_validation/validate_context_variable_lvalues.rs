use rustc_hash::FxHashMap;
use std::fmt::Display;
use std::fmt::Formatter;

use oxc_index::IndexSlice;

use oxc_diagnostics::{Diagnostics, OxcDiagnostic};

use crate::diagnostics::ErrorCategory;
use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::visitors::{each_instruction_value_lvalue, each_pattern_operand};
use crate::react_compiler_hir::{
    FunctionId, HirFunction, Identifier, IdentifierId, InstructionValue, Place,
};

/// Variable reference kind: local, context, or destructure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VarRefKind {
    Local,
    Context,
    Destructure,
}

impl Display for VarRefKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VarRefKind::Local => write!(f, "local"),
            VarRefKind::Context => write!(f, "context"),
            VarRefKind::Destructure => write!(f, "destructure"),
        }
    }
}

type IdentifierKinds = FxHashMap<IdentifierId, (Place, VarRefKind)>;

/// Validates that context variable lvalues are used consistently.
///
/// Port of ValidateContextVariableLValues.ts
pub fn validate_context_variable_lvalues(
    func: &HirFunction,
    env: &mut Environment,
) -> Result<(), OxcDiagnostic> {
    validate_context_variable_lvalues_with_errors(
        func,
        &env.functions,
        &env.identifiers,
        &mut env.errors,
    )
}

/// Like [`validate_context_variable_lvalues`], but writes diagnostics into the
/// provided `errors` instead of `env.errors`. Useful when the caller wants to
/// discard the diagnostics (e.g. when lowering is incomplete).
pub fn validate_context_variable_lvalues_with_errors(
    func: &HirFunction,
    functions: &IndexSlice<FunctionId, [HirFunction]>,
    identifiers: &IndexSlice<IdentifierId, [Identifier]>,
    errors: &mut Diagnostics,
) -> Result<(), OxcDiagnostic> {
    let mut identifier_kinds: IdentifierKinds = FxHashMap::default();
    validate_context_variable_lvalues_impl(
        func,
        &mut identifier_kinds,
        functions,
        identifiers,
        errors,
    )
}

fn validate_context_variable_lvalues_impl(
    func: &HirFunction,
    identifier_kinds: &mut IdentifierKinds,
    functions: &IndexSlice<FunctionId, [HirFunction]>,
    identifiers: &IndexSlice<IdentifierId, [Identifier]>,
    errors: &mut Diagnostics,
) -> Result<(), OxcDiagnostic> {
    let mut inner_function_ids: Vec<FunctionId> = Vec::new();

    for (_block_id, block) in &func.body.blocks {
        for &instr_id in &block.instructions {
            let instr = &func.instructions[instr_id.index()];
            let value = &instr.value;

            match value {
                InstructionValue::DeclareContext { lvalue, .. }
                | InstructionValue::StoreContext { lvalue, .. } => {
                    visit(
                        identifier_kinds,
                        &lvalue.place,
                        VarRefKind::Context,
                        identifiers,
                        errors,
                    )?;
                }
                InstructionValue::LoadContext { place, .. } => {
                    visit(identifier_kinds, place, VarRefKind::Context, identifiers, errors)?;
                }
                InstructionValue::StoreLocal { lvalue, .. }
                | InstructionValue::DeclareLocal { lvalue, .. } => {
                    visit(identifier_kinds, &lvalue.place, VarRefKind::Local, identifiers, errors)?;
                }
                InstructionValue::LoadLocal { place, .. } => {
                    visit(identifier_kinds, place, VarRefKind::Local, identifiers, errors)?;
                }
                InstructionValue::PostfixUpdate { lvalue, .. }
                | InstructionValue::PrefixUpdate { lvalue, .. } => {
                    visit(identifier_kinds, lvalue, VarRefKind::Local, identifiers, errors)?;
                }
                InstructionValue::Destructure { lvalue, .. } => {
                    for place in each_pattern_operand(&lvalue.pattern) {
                        visit(
                            identifier_kinds,
                            &place,
                            VarRefKind::Destructure,
                            identifiers,
                            errors,
                        )?;
                    }
                }
                InstructionValue::FunctionExpression { lowered_func, .. }
                | InstructionValue::ObjectMethod { lowered_func, .. } => {
                    inner_function_ids.push(lowered_func.func);
                }
                _ => {
                    for _ in each_instruction_value_lvalue(value) {
                        errors.push(
                            ErrorCategory::Todo
                                .diagnostic(
                                    "ValidateContextVariableLValues: unhandled instruction variant",
                                )
                                .with_labels(value.span().copied()),
                        );
                    }
                }
            }
        }
    }

    // Process inner functions after the block loop to avoid borrow conflicts
    for func_id in inner_function_ids {
        let inner_func = &functions[func_id];
        validate_context_variable_lvalues_impl(
            inner_func,
            identifier_kinds,
            functions,
            identifiers,
            errors,
        )?;
    }

    Ok(())
}

/// Format a place like TS `printPlace()`: `<effect> <name>$<id>`
fn format_place(place: &Place, identifiers: &IndexSlice<IdentifierId, [Identifier]>) -> String {
    let id = place.identifier;
    let ident = &identifiers[id];
    let name = ident.name.as_ref().map_or("", |name| name.value());
    format!("{} {}${}", place.effect, name, id.index())
}

fn visit(
    identifiers: &mut IdentifierKinds,
    place: &Place,
    kind: VarRefKind,
    env_identifiers: &IndexSlice<IdentifierId, [Identifier]>,
    errors: &mut Diagnostics,
) -> Result<(), OxcDiagnostic> {
    if let Some((prev_place, prev_kind)) = identifiers.get(&place.identifier) {
        let was_context = *prev_kind == VarRefKind::Context;
        let is_context = kind == VarRefKind::Context;
        if was_context != is_context {
            if *prev_kind == VarRefKind::Destructure || kind == VarRefKind::Destructure {
                let span =
                    if kind == VarRefKind::Destructure { place.span } else { prev_place.span };
                errors.push(
                    ErrorCategory::Todo
                        .diagnostic("Support destructuring of context variables")
                        .with_labels(span),
                );
                return Ok(());
            }
            let place_str = format_place(place, env_identifiers);
            return Err(ErrorCategory::Invariant
                .diagnostic(
                    "Expected all references to a variable to be consistently local or context references",
                )
                .with_help(format!(
                    "Identifier {} is referenced as a {} variable, but was previously referenced as a {} variable",
                    place_str, kind, prev_kind
                ))
                .with_labels(place.span.map(|s| s.label(format!("this is {}", prev_kind)))));
        }
    }
    identifiers.insert(place.identifier, (*place, kind));
    Ok(())
}

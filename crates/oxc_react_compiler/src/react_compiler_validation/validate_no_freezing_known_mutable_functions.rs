/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use rustc_hash::{FxHashMap, FxHashSet};

use oxc_diagnostics::OxcDiagnostic;
use oxc_index::IndexSlice;

use crate::diagnostics::ErrorCategory;
use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::visitors::{each_instruction_value_operand, each_terminal_operand};
use crate::react_compiler_hir::{
    AliasingEffect, Effect, FunctionId, HirFunction, Identifier, IdentifierId, IdentifierName,
    InstructionValue, Place, Type, TypeId,
};
use oxc_span::Span;

/// Information about a known mutation effect: which identifier is mutated, and
/// the source location of the mutation.
#[derive(Debug, Clone)]
struct MutationInfo {
    value_identifier: IdentifierId,
    value_span: Option<Span>,
}

/// Validates that functions with known mutations (ie due to types) cannot be passed
/// where a frozen value is expected.
///
/// Because a function that mutates a captured variable is equivalent to a mutable value,
/// and the receiver has no way to avoid calling the function, this pass detects functions
/// with *known* mutations (Mutate or MutateTransitive, not conditional) that are passed
/// where a frozen value is expected and reports an error.
pub fn validate_no_freezing_known_mutable_functions(func: &HirFunction, env: &mut Environment) {
    let diagnostics = check_no_freezing_known_mutable_functions(
        func,
        &env.identifiers,
        &env.types,
        &env.functions,
        env,
    );
    for diagnostic in diagnostics {
        env.record_diagnostic(diagnostic);
    }
}

fn check_no_freezing_known_mutable_functions(
    func: &HirFunction,
    identifiers: &IndexSlice<IdentifierId, [Identifier]>,
    types: &IndexSlice<TypeId, [Type]>,
    functions: &IndexSlice<FunctionId, [HirFunction]>,
    env: &Environment,
) -> Vec<OxcDiagnostic> {
    // Maps an identifier to the mutation effect that makes it "known mutable"
    let mut context_mutation_effects: FxHashMap<IdentifierId, MutationInfo> = FxHashMap::default();
    let mut diagnostics: Vec<OxcDiagnostic> = Vec::new();

    for (_block_id, block) in &func.body.blocks {
        for &instruction_id in &block.instructions {
            let instr = &func.instructions[instruction_id.index()];

            match &instr.value {
                InstructionValue::LoadLocal { place, .. } => {
                    // Propagate known mutation from the loaded place to the lvalue
                    if let Some(mutation_info) = context_mutation_effects.get(&place.identifier) {
                        context_mutation_effects
                            .insert(instr.lvalue.identifier, mutation_info.clone());
                    }
                }

                InstructionValue::StoreLocal { lvalue, value, .. } => {
                    // Propagate known mutation from the stored value to both the
                    // instruction lvalue and the StoreLocal's target lvalue
                    if let Some(mutation_info) = context_mutation_effects.get(&value.identifier) {
                        let mutation_info = mutation_info.clone();
                        context_mutation_effects
                            .insert(instr.lvalue.identifier, mutation_info.clone());
                        context_mutation_effects.insert(lvalue.place.identifier, mutation_info);
                    }
                }

                InstructionValue::FunctionExpression { lowered_func, .. } => {
                    let inner_function = &functions[lowered_func.func];
                    if let Some(ref aliasing_effects) = inner_function.aliasing_effects {
                        let context_ids: FxHashSet<IdentifierId> =
                            inner_function.context.iter().map(|place| place.identifier).collect();

                        'effects: for effect in aliasing_effects {
                            match effect {
                                AliasingEffect::Mutate { value, .. }
                                | AliasingEffect::MutateTransitive { value, .. } => {
                                    // If the mutated value is already known-mutable, propagate
                                    if let Some(known_mutation) =
                                        context_mutation_effects.get(&value.identifier)
                                    {
                                        context_mutation_effects.insert(
                                            instr.lvalue.identifier,
                                            known_mutation.clone(),
                                        );
                                    } else if context_ids.contains(&value.identifier)
                                        && !is_ref_or_ref_like_mutable_type(
                                            value.identifier,
                                            identifiers,
                                            types,
                                        )
                                    {
                                        // New known mutation of a context variable
                                        context_mutation_effects.insert(
                                            instr.lvalue.identifier,
                                            MutationInfo {
                                                value_identifier: value.identifier,
                                                value_span: value.span,
                                            },
                                        );
                                        break 'effects;
                                    }
                                }

                                AliasingEffect::MutateConditionally { value, .. }
                                | AliasingEffect::MutateTransitiveConditionally { value, .. } => {
                                    // Only propagate existing known mutations for conditional effects
                                    if let Some(known_mutation) =
                                        context_mutation_effects.get(&value.identifier)
                                    {
                                        context_mutation_effects.insert(
                                            instr.lvalue.identifier,
                                            known_mutation.clone(),
                                        );
                                    }
                                }

                                _ => {}
                            }
                        }
                    }
                }

                _ => {
                    // For all other instruction kinds, check operands for freeze violations
                    for operand in each_instruction_value_operand(&instr.value, env) {
                        check_operand_for_freeze_violation(
                            &operand,
                            &context_mutation_effects,
                            identifiers,
                            &mut diagnostics,
                        );
                    }
                }
            }
        }

        // Also check terminal operands
        for operand in each_terminal_operand(&block.terminal) {
            check_operand_for_freeze_violation(
                &operand,
                &context_mutation_effects,
                identifiers,
                &mut diagnostics,
            );
        }
    }

    diagnostics
}

/// If an operand with Effect::Freeze is a known-mutable function, emit a diagnostic.
fn check_operand_for_freeze_violation(
    operand: &Place,
    context_mutation_effects: &FxHashMap<IdentifierId, MutationInfo>,
    identifiers: &IndexSlice<IdentifierId, [Identifier]>,
    diagnostics: &mut Vec<OxcDiagnostic>,
) {
    if operand.effect == Effect::Freeze {
        if let Some(mutation_info) = context_mutation_effects.get(&operand.identifier) {
            let identifier = &identifiers[mutation_info.value_identifier];
            let variable_name = match &identifier.name {
                Some(IdentifierName::Named(name)) => format!("`{}`", name),
                _ => "a local variable".to_string(),
            };

            diagnostics.push(
                ErrorCategory::Immutability
                    .diagnostic("Cannot modify local variables after render completes")
                    .with_help(format!(
                        "This argument is a function which may reassign or mutate {} after render, \
                         which can cause inconsistent behavior on subsequent renders. \
                         Consider using state instead",
                        variable_name
                    ))
                    .with_labels(operand.span.map(|s| {
                        s.label(format!(
                            "This function may (indirectly) reassign or modify {} after render",
                            variable_name
                        ))
                    }))
                    .and_labels(
                        mutation_info
                            .value_span
                            .map(|s| s.label(format!("This modifies {}", variable_name))),
                    ),
            );
        }
    }
}

/// Check if an identifier's type is a ref or ref-like mutable type.
fn is_ref_or_ref_like_mutable_type(
    identifier_id: IdentifierId,
    identifiers: &IndexSlice<IdentifierId, [Identifier]>,
    types: &IndexSlice<TypeId, [Type]>,
) -> bool {
    let identifier = &identifiers[identifier_id];
    crate::react_compiler_hir::is_ref_or_ref_like_mutable_type(&types[identifier.type_])
}

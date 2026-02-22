/// Validate no freezing of known mutable functions.
///
/// Port of `Validation/ValidateNoFreezingKnownMutableFunctions.ts` from the React Compiler.
///
/// Validates that functions with known mutations (due to types) cannot be passed
/// where a frozen value is expected. For example, if a function mutates a local
/// variable when called, that function cannot be passed as a JSX prop or hook
/// argument that would freeze the value.
use rustc_hash::FxHashMap;

use crate::{
    compiler_error::{CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory},
    hir::{
        Effect, HIRFunction, IdentifierId, InstructionValue, Place,
        visitors::{each_instruction_value_operand, each_terminal_operand},
    },
    inference::aliasing_effects::AliasingEffect,
};

/// Validate that known-mutable functions are not frozen.
///
/// # Errors
/// Returns a `CompilerError` if any mutable function is improperly frozen.
pub fn validate_no_freezing_known_mutable_functions(
    func: &HIRFunction,
) -> Result<(), CompilerError> {
    fn visit_operand(
        operand: &Place,
        context_mutation_effects: &FxHashMap<IdentifierId, MutationInfo>,
        errors: &mut CompilerError,
    ) {
        if operand.effect == Effect::Freeze
            && let Some(effect) = context_mutation_effects.get(&operand.identifier.id)
        {
            let variable = get_variable_name(&effect.mutated_place);
            errors.push_diagnostic(
                CompilerDiagnostic::create(
                    ErrorCategory::Immutability,
                    "Cannot modify local variables after render completes".to_string(),
                    Some(format!(
                        "This argument is a function which may reassign or mutate \
                             {variable} after render, which can cause inconsistent behavior \
                             on subsequent renders. Consider using state instead"
                    )),
                    None,
                )
                .with_detail(CompilerDiagnosticDetail::Error {
                    loc: Some(operand.loc),
                    message: Some(format!(
                        "This function may (indirectly) reassign or modify {variable} \
                             after render"
                    )),
                })
                .with_detail(CompilerDiagnosticDetail::Error {
                    loc: Some(effect.mutated_place.loc),
                    message: Some(format!("This modifies {variable}")),
                }),
            );
        }
    }

    let mut errors = CompilerError::new();
    // Maps identifier IDs to mutation effects (Mutate or MutateTransitive)
    // that the function performs on its context captures
    let mut context_mutation_effects: FxHashMap<IdentifierId, MutationInfo> = FxHashMap::default();

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            let lvalue_id = instr.lvalue.identifier.id;
            match &instr.value {
                InstructionValue::LoadLocal(v) => {
                    if let Some(effect) = context_mutation_effects.get(&v.place.identifier.id) {
                        context_mutation_effects.insert(lvalue_id, effect.clone());
                    }
                }
                InstructionValue::StoreLocal(v) => {
                    if let Some(effect) = context_mutation_effects.get(&v.value.identifier.id) {
                        let effect = effect.clone();
                        context_mutation_effects
                            .insert(v.lvalue.place.identifier.id, effect.clone());
                        context_mutation_effects.insert(lvalue_id, effect);
                    }
                }
                InstructionValue::FunctionExpression(v) => {
                    if let Some(ref aliasing_effects) = v.lowered_func.func.aliasing_effects {
                        let context: rustc_hash::FxHashSet<IdentifierId> =
                            v.lowered_func.func.context.iter().map(|p| p.identifier.id).collect();

                        for effect in aliasing_effects {
                            match effect {
                                AliasingEffect::Mutate { value, .. }
                                | AliasingEffect::MutateTransitive { value, .. } => {
                                    // Check if there's already a known mutation for this value
                                    if let Some(known) =
                                        context_mutation_effects.get(&value.identifier.id)
                                    {
                                        context_mutation_effects.insert(lvalue_id, known.clone());
                                    } else if context.contains(&value.identifier.id)
                                        && !is_ref_or_ref_like_mutable_type(&value.identifier.type_)
                                    {
                                        context_mutation_effects.insert(
                                            lvalue_id,
                                            MutationInfo { mutated_place: value.clone() },
                                        );
                                        // Break out of the effects loop (labeled break
                                        // from TS: `break effects`)
                                        break;
                                    }
                                }
                                AliasingEffect::MutateConditionally { value }
                                | AliasingEffect::MutateTransitiveConditionally { value } => {
                                    if let Some(known) =
                                        context_mutation_effects.get(&value.identifier.id)
                                    {
                                        context_mutation_effects.insert(lvalue_id, known.clone());
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {
                    for operand in each_instruction_value_operand(&instr.value) {
                        visit_operand(operand, &context_mutation_effects, &mut errors);
                    }
                }
            }
        }
        for operand in each_terminal_operand(&block.terminal) {
            visit_operand(operand, &context_mutation_effects, &mut errors);
        }
    }

    errors.into_result()
}

#[derive(Debug, Clone)]
struct MutationInfo {
    mutated_place: Place,
}

fn get_variable_name(place: &Place) -> String {
    if let Some(ref name) = place.identifier.name
        && let crate::hir::IdentifierName::Named(val) = name
    {
        return format!("`{val}`");
    }
    "a local variable".to_string()
}

fn is_ref_or_ref_like_mutable_type(ty: &crate::hir::types::Type) -> bool {
    match ty {
        crate::hir::types::Type::Object(obj) => {
            matches!(
                obj.shape_id.as_deref(),
                Some(
                    crate::hir::object_shape::BUILT_IN_REF_VALUE_ID
                        | crate::hir::object_shape::BUILT_IN_USE_REF_ID
                )
            )
        }
        _ => false,
    }
}

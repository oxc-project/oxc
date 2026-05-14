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
        Effect, HIRFunction, IdentifierId, Instruction, InstructionValue, Place, Terminal,
        environment::Environment,
        visitors::{each_instruction_value_operand, each_terminal_operand},
    },
    inference::aliasing_effects::AliasingEffect,
    validation::dispatcher::InstructionVisitor,
};

/// Validate that known-mutable functions are not frozen.
///
/// # Errors
/// Returns a `CompilerError` if any mutable function is improperly frozen.
pub fn validate_no_freezing_known_mutable_functions(
    func: &HIRFunction,
) -> Result<(), CompilerError> {
    crate::validation::dispatcher::dispatch_instruction_visitors(
        func,
        vec![Box::new(ValidateNoFreezingKnownMutableFunctions::default())],
    )
}

/// `InstructionVisitor` impl for `validate_no_freezing_known_mutable_functions`.
///
/// Maintains a function-local `context_mutation_effects` map that records which
/// identifier ids are known to wrap functions that mutate captured context.
/// The map is monotonically built up across all blocks (no cross-block fixpoint,
/// just sequential accumulation) and inspected at every operand-bearing
/// instruction and terminal.
#[derive(Default)]
pub struct ValidateNoFreezingKnownMutableFunctions {
    errors: CompilerError,
    context_mutation_effects: FxHashMap<IdentifierId, MutationInfo>,
}

impl InstructionVisitor for ValidateNoFreezingKnownMutableFunctions {
    fn visit_instruction(&mut self, _env: &Environment, instr: &Instruction) {
        let lvalue_id = instr.lvalue.identifier.id;
        match &instr.value {
            InstructionValue::LoadLocal(v) => {
                if let Some(effect) = self.context_mutation_effects.get(&v.place.identifier.id) {
                    self.context_mutation_effects.insert(lvalue_id, effect.clone());
                }
            }
            InstructionValue::StoreLocal(v) => {
                if let Some(effect) = self.context_mutation_effects.get(&v.value.identifier.id) {
                    let effect = effect.clone();
                    self.context_mutation_effects
                        .insert(v.lvalue.place.identifier.id, effect.clone());
                    self.context_mutation_effects.insert(lvalue_id, effect);
                }
            }
            InstructionValue::FunctionExpression(v) => {
                if let Some(ref aliasing_effects) = v.lowered_func.func.aliasing_effects {
                    // Only include context variables that are actively captured
                    // (Effect::Capture). Variables demoted to Effect::Read by
                    // inferMutationAliasingEffects (because they are global/frozen
                    // in the outer scope) should not be considered mutable captures.
                    // NOTE: The TS does NOT filter by effect here (line 98), but
                    // in the TS the inner function's context effects are computed
                    // differently. We filter to avoid false positives where
                    // global/frozen variables are marked as mutable captures.
                    let context: rustc_hash::FxHashSet<IdentifierId> = v
                        .lowered_func
                        .func
                        .context
                        .iter()
                        .filter(|p| p.effect == Effect::Capture)
                        .map(|p| p.identifier.id)
                        .collect();

                    for effect in aliasing_effects {
                        match effect {
                            AliasingEffect::Mutate { value, .. }
                            | AliasingEffect::MutateTransitive { value, .. } => {
                                // Check if there's already a known mutation for this value
                                if let Some(known) =
                                    self.context_mutation_effects.get(&value.identifier.id)
                                {
                                    self.context_mutation_effects.insert(lvalue_id, known.clone());
                                } else if context.contains(&value.identifier.id)
                                    && !is_ref_or_ref_like_mutable_type(&value.identifier.type_)
                                {
                                    self.context_mutation_effects.insert(
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
                                    self.context_mutation_effects.get(&value.identifier.id)
                                {
                                    self.context_mutation_effects.insert(lvalue_id, known.clone());
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {
                for operand in each_instruction_value_operand(&instr.value) {
                    visit_operand(operand, &self.context_mutation_effects, &mut self.errors);
                }
            }
        }
    }

    fn visit_terminal(&mut self, _env: &Environment, terminal: &Terminal) {
        for operand in each_terminal_operand(terminal) {
            visit_operand(operand, &self.context_mutation_effects, &mut self.errors);
        }
    }

    fn finish(self: Box<Self>, _env: &Environment) -> Result<(), CompilerError> {
        self.errors.into_result()
    }
}

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
                    crate::hir::object_shape::BUILT_IN_USE_REF_ID
                        | crate::hir::object_shape::REANIMATED_SHARED_VALUE_ID
                )
            )
        }
        _ => false,
    }
}

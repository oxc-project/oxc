/// Validate no derived computations in effects (experimental version).
///
/// Port of `Validation/ValidateNoDerivedComputationsInEffects_exp.ts` from the
/// React Compiler.
///
/// Uses a fixpoint iteration approach to track derivations from props and state
/// through the function body, then validates that effects don't set state with
/// values that could be computed during render instead.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    compiler_error::{CompilerDiagnostic, CompilerDiagnosticDetail, CompilerError, ErrorCategory},
    hir::{
        CallArg, FunctionExpressionValue, HIRFunction, IdentifierId, InstructionValue,
        ReactFunctionType,
        object_shape::{
            BUILT_IN_SET_STATE_ID, BUILT_IN_USE_EFFECT_HOOK_ID, BUILT_IN_USE_LAYOUT_EFFECT_HOOK_ID,
            BUILT_IN_USE_STATE_ID,
        },
        types::{FunctionType, Type},
        visitors::each_instruction_operand,
    },
};

const MAX_FIXPOINT_ITERATIONS: u32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeOfValue {
    Ignored,
    FromProps,
    FromState,
    FromPropsAndState,
}

#[derive(Debug, Clone)]
struct DerivationMetadata {
    type_of_value: TypeOfValue,
    is_state_source: bool,
    sources_ids: FxHashSet<IdentifierId>,
}

/// Validate no derived computations in effects (experimental version).
///
/// Returns errors (does not throw/panic) so the caller can decide how to handle them.
pub fn validate_no_derived_computations_in_effects_exp(func: &HIRFunction) -> CompilerError {
    let mut derivation_cache: FxHashMap<IdentifierId, DerivationMetadata> = FxHashMap::default();
    let mut functions: FxHashMap<IdentifierId, &FunctionExpressionValue> = FxHashMap::default();
    let mut candidate_deps: FxHashMap<IdentifierId, Vec<IdentifierId>> = FxHashMap::default();
    let mut errors = CompilerError::new();
    let mut effects_cache: FxHashSet<IdentifierId> = FxHashSet::default();
    let mut set_state_loads: FxHashMap<IdentifierId, Option<IdentifierId>> = FxHashMap::default();

    // Initialize derivation cache from params
    if func.fn_type == ReactFunctionType::Hook {
        for param in &func.params {
            if let crate::hir::ReactiveParam::Place(p) = param {
                derivation_cache.insert(
                    p.identifier.id,
                    DerivationMetadata {
                        type_of_value: TypeOfValue::FromProps,
                        is_state_source: true,
                        sources_ids: FxHashSet::default(),
                    },
                );
            }
        }
    } else if func.fn_type == ReactFunctionType::Component
        && let Some(crate::hir::ReactiveParam::Place(p)) = func.params.first()
    {
        derivation_cache.insert(
            p.identifier.id,
            DerivationMetadata {
                type_of_value: TypeOfValue::FromProps,
                is_state_source: true,
                sources_ids: FxHashSet::default(),
            },
        );
    }

    // Fixpoint iteration
    let mut is_first_pass = true;
    let mut iteration_count = 0u32;

    loop {
        let snapshot_size = derivation_cache.len();
        let snapshot: FxHashMap<IdentifierId, (TypeOfValue, usize)> = derivation_cache
            .iter()
            .map(|(k, v)| (*k, (v.type_of_value, v.sources_ids.len())))
            .collect();

        for block in func.body.blocks.values() {
            // Record phi derivations
            for phi in &block.phis {
                let mut type_of_value = TypeOfValue::Ignored;
                let mut sources_ids: FxHashSet<IdentifierId> = FxHashSet::default();
                for operand in phi.operands.values() {
                    if let Some(metadata) = derivation_cache.get(&operand.identifier.id) {
                        type_of_value = join_value(type_of_value, metadata.type_of_value);
                        sources_ids.insert(operand.identifier.id);
                    }
                }
                if type_of_value != TypeOfValue::Ignored {
                    add_derivation_entry(
                        &mut derivation_cache,
                        phi.place.identifier.id,
                        sources_ids,
                        type_of_value,
                        false,
                    );
                }
            }

            for instr in &block.instructions {
                let lvalue_id = instr.lvalue.identifier.id;

                // Record setState loads
                if is_set_state_type(&instr.lvalue.identifier.type_) {
                    set_state_loads.insert(lvalue_id, None);
                }
                if let InstructionValue::LoadLocal(v) = &instr.value
                    && set_state_loads.contains_key(&v.place.identifier.id)
                {
                    set_state_loads.insert(lvalue_id, Some(v.place.identifier.id));
                }

                // Track functions and dependencies
                if let InstructionValue::FunctionExpression(v) = &instr.value {
                    functions.insert(lvalue_id, v);
                    // Recursively process function body
                    for fn_block in v.lowered_func.func.body.blocks.values() {
                        for fn_instr in &fn_block.instructions {
                            let fn_lvalue_id = fn_instr.lvalue.identifier.id;
                            let mut type_of_value = TypeOfValue::Ignored;
                            let mut sources: FxHashSet<IdentifierId> = FxHashSet::default();

                            for operand in each_instruction_operand(fn_instr) {
                                if let Some(metadata) = derivation_cache.get(&operand.identifier.id)
                                {
                                    type_of_value =
                                        join_value(type_of_value, metadata.type_of_value);
                                    sources.insert(operand.identifier.id);
                                }
                            }

                            if type_of_value != TypeOfValue::Ignored {
                                add_derivation_entry(
                                    &mut derivation_cache,
                                    fn_lvalue_id,
                                    sources,
                                    type_of_value,
                                    false,
                                );
                            }
                        }
                    }
                } else if let InstructionValue::ArrayExpression(v) = &instr.value {
                    let element_ids: Vec<IdentifierId> = v
                        .elements
                        .iter()
                        .filter_map(|elem| match elem {
                            crate::hir::ArrayExpressionElement::Place(p) => Some(p.identifier.id),
                            _ => None,
                        })
                        .collect();
                    candidate_deps.insert(lvalue_id, element_ids);
                } else if let InstructionValue::CallExpression(v) = &instr.value {
                    let callee_type = &v.callee.identifier.type_;
                    if is_use_effect_hook_type(callee_type) && v.args.len() == 2 {
                        if let (Some(CallArg::Place(fn_arg)), Some(CallArg::Place(deps_arg))) =
                            (v.args.first(), v.args.get(1))
                            && functions.contains_key(&fn_arg.identifier.id)
                            && candidate_deps.contains_key(&deps_arg.identifier.id)
                        {
                            effects_cache.insert(fn_arg.identifier.id);
                        }
                    } else if is_use_state_type(&instr.lvalue.identifier.type_) {
                        add_derivation_entry(
                            &mut derivation_cache,
                            lvalue_id,
                            FxHashSet::default(),
                            TypeOfValue::FromState,
                            true,
                        );
                        continue;
                    }
                }

                // General operand tracking
                let mut type_of_value = TypeOfValue::Ignored;
                let mut sources: FxHashSet<IdentifierId> = FxHashSet::default();
                for operand in each_instruction_operand(instr) {
                    if let Some(metadata) = derivation_cache.get(&operand.identifier.id) {
                        type_of_value = join_value(type_of_value, metadata.type_of_value);
                        sources.insert(operand.identifier.id);
                    }
                }

                if type_of_value != TypeOfValue::Ignored {
                    add_derivation_entry(
                        &mut derivation_cache,
                        lvalue_id,
                        sources,
                        type_of_value,
                        false,
                    );
                }
            }
        }

        // Check for convergence
        let has_changes = derivation_cache.len() != snapshot_size
            || derivation_cache.iter().any(|(k, v)| {
                snapshot
                    .get(k)
                    .is_none_or(|(tv, sz)| *tv != v.type_of_value || *sz != v.sources_ids.len())
            });

        if !has_changes && !is_first_pass {
            break;
        }

        is_first_pass = false;
        iteration_count += 1;
        if iteration_count >= MAX_FIXPOINT_ITERATIONS {
            break;
        }
    }

    // Validate effects
    for fn_id in &effects_cache {
        if let Some(effect_fn) = functions.get(fn_id) {
            // Look for setState calls with derived values inside the effect
            for block in effect_fn.lowered_func.func.body.blocks.values() {
                for instr in &block.instructions {
                    if let InstructionValue::CallExpression(call) = &instr.value
                        && is_set_state_type(&call.callee.identifier.type_)
                        && call.args.len() == 1
                        && let Some(CallArg::Place(arg)) = call.args.first()
                    {
                        // Check if the setState callee derives from state
                        let callee_metadata = derivation_cache.get(&call.callee.identifier.id);
                        if let Some(cm) = callee_metadata
                            && cm.type_of_value != TypeOfValue::FromState
                        {
                            continue;
                        }
                        // Check if the argument derives from props/state
                        if let Some(arg_metadata) = derivation_cache.get(&arg.identifier.id)
                            && arg_metadata.type_of_value != TypeOfValue::Ignored
                        {
                            errors.push_diagnostic(
                                            CompilerDiagnostic::create(
                                                ErrorCategory::EffectDerivationsOfState,
                                                "You might not need an effect. Derive values in render, not effects."
                                                    .to_string(),
                                                Some(
                                                    "Using an effect triggers an additional render which can \
                                                     hurt performance and user experience, potentially briefly \
                                                     showing stale values to the user. \
                                                     See: https://react.dev/learn/you-might-not-need-an-effect\
                                                     #updating-state-based-on-props-or-state"
                                                        .to_string(),
                                                ),
                                                None,
                                            )
                                            .with_detail(CompilerDiagnosticDetail::Error {
                                                loc: Some(call.callee.loc),
                                                message: Some(
                                                    "This should be computed during render, not in an effect"
                                                        .to_string(),
                                                ),
                                            }),
                                        );
                        }
                    }
                }
            }
        }
    }

    errors
}

fn join_value(lvalue_type: TypeOfValue, value_type: TypeOfValue) -> TypeOfValue {
    if lvalue_type == TypeOfValue::Ignored {
        return value_type;
    }
    if value_type == TypeOfValue::Ignored {
        return lvalue_type;
    }
    if lvalue_type == value_type {
        return lvalue_type;
    }
    TypeOfValue::FromPropsAndState
}

fn add_derivation_entry(
    cache: &mut FxHashMap<IdentifierId, DerivationMetadata>,
    id: IdentifierId,
    sources_ids: FxHashSet<IdentifierId>,
    type_of_value: TypeOfValue,
    is_state_source: bool,
) {
    let mut final_is_source = is_state_source;
    if !final_is_source {
        for source_id in &sources_ids {
            if let Some(source_metadata) = cache.get(source_id)
                && source_metadata.is_state_source
            {
                final_is_source = true;
                break;
            }
        }
    }

    cache.insert(
        id,
        DerivationMetadata { type_of_value, is_state_source: final_is_source, sources_ids },
    );
}

fn is_set_state_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Function(FunctionType { shape_id: Some(id), .. })
        if id == BUILT_IN_SET_STATE_ID
    )
}

fn is_use_state_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Function(FunctionType { shape_id: Some(id), .. })
        if id == BUILT_IN_USE_STATE_ID
    )
}

fn is_use_effect_hook_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Function(FunctionType { shape_id: Some(id), .. })
        if id == BUILT_IN_USE_EFFECT_HOOK_ID || id == BUILT_IN_USE_LAYOUT_EFFECT_HOOK_ID
    )
}

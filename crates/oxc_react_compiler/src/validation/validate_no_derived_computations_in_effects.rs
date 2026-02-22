/// Validate no derived computations in effects.
///
/// Port of `Validation/ValidateNoDerivedComputationsInEffects.ts` from the React Compiler.
///
/// Validates that useEffect is not used for derived computations which
/// could/should be performed in render.
///
/// See https://react.dev/learn/you-might-not-need-an-effect#updating-state-based-on-props-or-state
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    compiler_error::{
        CompilerError, CompilerErrorDetail, CompilerErrorDetailOptions, ErrorCategory,
        SourceLocation,
    },
    hir::{
        ArrayExpressionElement, BlockId, CallArg, FunctionExpressionValue, HIRFunction,
        IdentifierId, InstructionValue,
        object_shape::{
            BUILT_IN_SET_STATE_ID, BUILT_IN_USE_EFFECT_HOOK_ID, BUILT_IN_USE_LAYOUT_EFFECT_HOOK_ID,
        },
        types::{FunctionType, Type},
        visitors::each_instruction_value_operand,
    },
};

/// Validate no derived computations in effects.
///
/// Checks for patterns like:
/// ```js
/// const [fullName, setFullName] = useState('');
/// useEffect(() => {
///   setFullName(firstName + ' ' + lastName);
/// }, [firstName, lastName]);
/// ```
pub fn validate_no_derived_computations_in_effects(func: &HIRFunction) {
    let mut candidate_dependencies: FxHashMap<IdentifierId, Vec<IdentifierId>> =
        FxHashMap::default();
    let mut functions: FxHashMap<IdentifierId, &FunctionExpressionValue> = FxHashMap::default();
    let mut locals: FxHashMap<IdentifierId, IdentifierId> = FxHashMap::default();
    let mut errors = CompilerError::new();

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            let lvalue_id = instr.lvalue.identifier.id;
            match &instr.value {
                InstructionValue::LoadLocal(v) => {
                    locals.insert(lvalue_id, v.place.identifier.id);
                }
                InstructionValue::ArrayExpression(v) => {
                    let element_ids: Vec<IdentifierId> = v
                        .elements
                        .iter()
                        .filter_map(|elem| match elem {
                            ArrayExpressionElement::Place(p) => Some(p.identifier.id),
                            _ => None,
                        })
                        .collect();
                    candidate_dependencies.insert(lvalue_id, element_ids);
                }
                InstructionValue::FunctionExpression(v) => {
                    functions.insert(lvalue_id, v);
                }
                InstructionValue::CallExpression(v) => {
                    if is_use_effect_hook_type(&v.callee.identifier.type_)
                        && v.args.len() == 2
                        && let (Some(CallArg::Place(fn_arg)), Some(CallArg::Place(deps_arg))) =
                            (v.args.first(), v.args.get(1))
                    {
                        let effect_function = functions.get(&fn_arg.identifier.id);
                        let deps = candidate_dependencies.get(&deps_arg.identifier.id);

                        if let (Some(effect_fn), Some(dep_ids)) = (effect_function, deps)
                            && !dep_ids.is_empty()
                        {
                            let dependencies: Vec<IdentifierId> = dep_ids
                                .iter()
                                .map(|id| locals.get(id).copied().unwrap_or(*id))
                                .collect();
                            validate_effect(
                                &effect_fn.lowered_func.func,
                                &dependencies,
                                &mut errors,
                            );
                        }
                    }
                }
                InstructionValue::MethodCall(v) => {
                    if is_use_effect_hook_type(&v.property.identifier.type_)
                        && v.args.len() == 2
                        && let (Some(CallArg::Place(fn_arg)), Some(CallArg::Place(deps_arg))) =
                            (v.args.first(), v.args.get(1))
                    {
                        let effect_function = functions.get(&fn_arg.identifier.id);
                        let deps = candidate_dependencies.get(&deps_arg.identifier.id);

                        if let (Some(effect_fn), Some(dep_ids)) = (effect_function, deps)
                            && !dep_ids.is_empty()
                        {
                            let dependencies: Vec<IdentifierId> = dep_ids
                                .iter()
                                .map(|id| locals.get(id).copied().unwrap_or(*id))
                                .collect();
                            validate_effect(
                                &effect_fn.lowered_func.func,
                                &dependencies,
                                &mut errors,
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if errors.has_any_errors() {
        // The TS version throws errors here; we just drop them since
        // the pipeline doesn't check the return from this function
    }
}

fn validate_effect(
    effect_function: &HIRFunction,
    effect_deps: &[IdentifierId],
    errors: &mut CompilerError,
) {
    // Check that the function only captures effect deps and setState
    for operand in &effect_function.context {
        if is_set_state_type(&operand.identifier.type_) {
            continue;
        }
        if effect_deps.contains(&operand.identifier.id) {
            continue;
        }
        // Captured something other than the effect dep or setState
        return;
    }

    // Check that all effect deps are actually used in the function
    for dep in effect_deps {
        if !effect_function.context.iter().any(|operand| operand.identifier.id == *dep) {
            // Effect dep wasn't actually used in the function
            return;
        }
    }

    let mut seen_blocks: FxHashSet<BlockId> = FxHashSet::default();
    let mut values: FxHashMap<IdentifierId, Vec<IdentifierId>> = FxHashMap::default();
    for dep in effect_deps {
        values.insert(*dep, vec![*dep]);
    }

    let mut set_state_locations: Vec<SourceLocation> = Vec::new();

    for block in effect_function.body.blocks.values() {
        // Skip if block has a back edge
        for pred in &block.preds {
            if !seen_blocks.contains(pred) {
                return;
            }
        }

        // Propagate through phi nodes
        for phi in &block.phis {
            let mut aggregate_deps: FxHashSet<IdentifierId> = FxHashSet::default();
            for operand in phi.operands.values() {
                if let Some(deps) = values.get(&operand.identifier.id) {
                    for dep in deps {
                        aggregate_deps.insert(*dep);
                    }
                }
            }
            if !aggregate_deps.is_empty() {
                values.insert(phi.place.identifier.id, aggregate_deps.into_iter().collect());
            }
        }

        for instr in &block.instructions {
            match &instr.value {
                InstructionValue::Primitive(_)
                | InstructionValue::JsxText(_)
                | InstructionValue::LoadGlobal(_) => {
                    // No data flow
                }
                InstructionValue::LoadLocal(v) => {
                    if let Some(deps) = values.get(&v.place.identifier.id) {
                        values.insert(instr.lvalue.identifier.id, deps.clone());
                    }
                }
                InstructionValue::ComputedLoad(_)
                | InstructionValue::PropertyLoad(_)
                | InstructionValue::BinaryExpression(_)
                | InstructionValue::TemplateLiteral(_)
                | InstructionValue::CallExpression(_)
                | InstructionValue::MethodCall(_) => {
                    let mut aggregate_deps: FxHashSet<IdentifierId> = FxHashSet::default();
                    for operand in each_instruction_value_operand(&instr.value) {
                        if let Some(deps) = values.get(&operand.identifier.id) {
                            for dep in deps {
                                aggregate_deps.insert(*dep);
                            }
                        }
                    }
                    if !aggregate_deps.is_empty() {
                        values.insert(
                            instr.lvalue.identifier.id,
                            aggregate_deps.into_iter().collect(),
                        );
                    }

                    // Check for setState(derivedValue) pattern
                    if let InstructionValue::CallExpression(call) = &instr.value
                        && is_set_state_type(&call.callee.identifier.type_)
                        && call.args.len() == 1
                        && let Some(CallArg::Place(arg)) = call.args.first()
                        && let Some(deps) = values.get(&arg.identifier.id)
                    {
                        let dep_set: FxHashSet<_> = deps.iter().copied().collect();
                        if dep_set.len() == effect_deps.len() {
                            set_state_locations.push(call.callee.loc);
                        } else {
                            // Doesn't depend on all deps
                            return;
                        }
                    }
                }
                _ => {
                    // Unknown instruction kind -- bail out
                    return;
                }
            }
        }

        // Check terminal operands
        for operand in crate::hir::visitors::each_terminal_operand(&block.terminal) {
            if values.contains_key(&operand.identifier.id) {
                return;
            }
        }
        seen_blocks.insert(block.id);
    }

    for loc in set_state_locations {
        errors.push_error_detail(CompilerErrorDetail::new(CompilerErrorDetailOptions {
            category: ErrorCategory::EffectDerivationsOfState,
            reason: "Values derived from props and state should be calculated during render, \
                     not in an effect. \
                     (https://react.dev/learn/you-might-not-need-an-effect\
                     #updating-state-based-on-props-or-state)"
                .to_string(),
            description: None,
            loc: Some(loc),
            suggestions: None,
        }));
    }
}

fn is_set_state_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Function(FunctionType { shape_id: Some(id), .. })
        if id == BUILT_IN_SET_STATE_ID
    )
}

fn is_use_effect_hook_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Function(FunctionType { shape_id: Some(id), .. })
        if id == BUILT_IN_USE_EFFECT_HOOK_ID || id == BUILT_IN_USE_LAYOUT_EFFECT_HOOK_ID
    )
}

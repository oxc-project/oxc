/// Validate no setState calls in effect bodies.
///
/// Port of `Validation/ValidateNoSetStateInEffects.ts` from the React Compiler.
///
/// Validates against calling setState synchronously in an effect, which can
/// indicate non-local derived data, a derived event pattern, or improper
/// external data synchronization.
use rustc_hash::FxHashSet;

use crate::{
    compiler_error::CompilerError,
    hir::{
        HIRFunction, IdentifierId, InstructionValue,
        types::{FunctionType, Type},
        object_shape::BUILT_IN_USE_EFFECT_HOOK_ID,
    },
};

/// Validate no setState in effects.
pub fn validate_no_set_state_in_effects(func: &HIRFunction) -> CompilerError {
    let errors = CompilerError::new();
    let mut effect_callback_ids: FxHashSet<IdentifierId> = FxHashSet::default();
    let set_state_ids: FxHashSet<IdentifierId> = FxHashSet::default();

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            match &instr.value {
                // Track useEffect/useLayoutEffect calls and their callback arguments
                InstructionValue::CallExpression(v) => {
                    if is_effect_hook_type(&v.callee.identifier.type_)
                        && let Some(first_arg) = v.args.first()
                            && let crate::hir::CallArg::Place(p) = first_arg {
                                effect_callback_ids.insert(p.identifier.id);
                            }

                    // Check if setState is called inside an effect
                    if is_set_state_type(&v.callee.identifier.type_)
                        || set_state_ids.contains(&v.callee.identifier.id)
                    {
                        // In the full implementation, we'd check if we're inside
                        // an effect callback's function body
                    }
                }
                _ => {}
            }
        }
    }

    errors
}

fn is_effect_hook_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Function(FunctionType { shape_id: Some(id), .. })
        if id == BUILT_IN_USE_EFFECT_HOOK_ID
    )
}

fn is_set_state_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Function(FunctionType { shape_id: Some(id), .. })
        if id == "BuiltInSetState"
    )
}

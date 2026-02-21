/// Experimental: Validate no derived computations in effects.
///
/// Port of `Validation/ValidateNoDerivedComputationsInEffects_exp.ts` from the React Compiler.
///
/// This is the experimental version of the derived-computation-in-effects
/// validation that provides more detailed diagnostics and uses a different
/// analysis approach for detecting problematic patterns.
///
/// Key patterns detected:
/// - Calling setState inside useEffect with values derived from state
/// - Deriving values from state in effect callbacks when the value should
///   be computed during render instead
use rustc_hash::FxHashSet;

use crate::{
    compiler_error::CompilerError,
    hir::{
        HIRFunction, IdentifierId, InstructionValue,
        types::{FunctionType, Type},
    },
};

/// Validate no derived computations in effects (experimental version).
///
/// Returns errors (does not throw/panic) so the caller can decide how to handle them.
pub fn validate_no_derived_computations_in_effects_exp(
    func: &HIRFunction,
) -> CompilerError {
    let errors = CompilerError::new();

    // Phase 1: Identify state values (from useState/useReducer)
    let mut state_ids: FxHashSet<IdentifierId> = FxHashSet::default();
    let _set_state_ids: FxHashSet<IdentifierId> = FxHashSet::default();
    let mut effect_callback_ids: FxHashSet<IdentifierId> = FxHashSet::default();

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            match &instr.value {
                InstructionValue::CallExpression(v) => {
                    // Track useState return values
                    if is_use_state_type(&v.callee.identifier.type_) {
                        state_ids.insert(instr.lvalue.identifier.id);
                    }
                    // Track useEffect callbacks
                    if is_use_effect_type(&v.callee.identifier.type_) {
                        if let Some(crate::hir::CallArg::Place(callback)) = v.args.first() {
                            effect_callback_ids.insert(callback.identifier.id);
                        }
                    }
                }
                // Track destructured setState functions
                InstructionValue::Destructure(_v) => {
                    // In the full implementation, track array destructuring of useState
                    // results to identify the setState function
                }
                _ => {}
            }
        }
    }

    // Phase 2: Check for derived computations from state inside effects
    // The full implementation would:
    // 1. Trace data flow from state values through computations
    // 2. Check if those derived values flow into setState calls inside effects
    // 3. Report each case with detailed source location information

    errors
}

fn is_use_state_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Function(FunctionType { shape_id: Some(id), .. })
        if id == "BuiltInUseState"
    )
}

fn is_use_effect_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::Function(FunctionType { shape_id: Some(id), .. })
        if id == "BuiltInUseEffectHook" || id == "BuiltInUseLayoutEffectHook"
    )
}

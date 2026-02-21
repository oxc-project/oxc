/// Validate no derived computations in effects.
///
/// Port of `Validation/ValidateNoDerivedComputationsInEffects.ts` from the React Compiler.
///
/// Validates against deriving values from state in an effect, which can
/// lead to unnecessary re-renders and inconsistent state.
use crate::hir::HIRFunction;

/// Validate no derived computations in effects.
pub fn validate_no_derived_computations_in_effects(_func: &HIRFunction) {
    // This validation checks that values derived from state are not
    // computed inside useEffect/useLayoutEffect callbacks.
    //
    // The full implementation uses data flow analysis to track which
    // values derive from state (via useState) and then checks if those
    // derivations happen inside effect callbacks.
    //
    // For now, this is a structural placeholder.
}

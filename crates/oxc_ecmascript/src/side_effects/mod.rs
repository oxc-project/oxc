mod context;
mod may_have_side_effects;

pub use context::{MayHaveSideEffectsContext, PropertyReadSideEffects};
pub use may_have_side_effects::{MayHaveSideEffects, is_side_effect_free_unbound_identifier_ref};

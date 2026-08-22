pub mod align_method_call_scopes;
pub mod align_object_method_scopes;
pub mod align_reactive_scopes_to_block_scopes_hir;
pub mod analyse_functions;
pub mod build_reactive_scope_terminals_hir;
pub mod flatten_reactive_loops_hir;
pub mod flatten_scopes_with_hooks_or_use_hir;
pub mod infer_mutation_aliasing_effects;
pub mod infer_mutation_aliasing_ranges;
pub mod infer_reactive_places;
pub mod infer_reactive_scope_variables;
pub mod memoize_fbt_and_macro_operands_in_same_scope;
pub mod merge_overlapping_reactive_scopes_hir;
pub mod propagate_scope_dependencies_hir;

pub use align_method_call_scopes::align_method_call_scopes;
pub use align_object_method_scopes::align_object_method_scopes;
pub use align_reactive_scopes_to_block_scopes_hir::align_reactive_scopes_to_block_scopes_hir;
pub use analyse_functions::analyse_functions;
pub use build_reactive_scope_terminals_hir::build_reactive_scope_terminals_hir;
pub use flatten_reactive_loops_hir::flatten_reactive_loops_hir;
pub use flatten_scopes_with_hooks_or_use_hir::flatten_scopes_with_hooks_or_use_hir;
pub use infer_mutation_aliasing_effects::infer_mutation_aliasing_effects;
pub use infer_mutation_aliasing_ranges::infer_mutation_aliasing_ranges;
pub use infer_reactive_places::infer_reactive_places;
pub use infer_reactive_scope_variables::infer_reactive_scope_variables;
pub use memoize_fbt_and_macro_operands_in_same_scope::memoize_fbt_and_macro_operands_in_same_scope;
pub use merge_overlapping_reactive_scopes_hir::merge_overlapping_reactive_scopes_hir;
pub use propagate_scope_dependencies_hir::propagate_scope_dependencies_hir;

use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::type_config::ValueKind;
use crate::react_compiler_hir::{Effect, Place};

/// Whether a tagged-template call has enough configured effect information to be memoized.
///
/// A tagged template passes the frozen template object as its first argument, followed by its
/// substitutions. Inferred result types are not evidence here: a consumer can constrain an opaque
/// result to `Primitive`. Only the tag's function signature can prove the call and result safe.
fn is_known_pure_tagged_template(
    env: &Environment,
    tag: &Place,
    substitution_count: usize,
) -> bool {
    let tag_type = &env.types[env.identifiers[tag.identifier].type_];
    let Some(signature) = env.get_function_signature(tag_type).ok().flatten() else {
        return false;
    };

    if signature.hook_kind.is_some()
        || signature.impure
        || signature.known_incompatible.is_some()
        || signature.aliasing.is_some()
        || signature.callee_effect != Effect::Read
        || !matches!(
            signature.return_value_kind,
            ValueKind::Primitive | ValueKind::Frozen | ValueKind::Mutable
        )
    {
        return false;
    }

    // Include the implicit template object at parameter index zero.
    (0..=substitution_count).all(|index| {
        signature.positional_params.get(index).copied().or(signature.rest_param)
            == Some(Effect::Read)
    })
}

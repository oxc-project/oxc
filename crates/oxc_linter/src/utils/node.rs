use oxc_ast::ast::{AssignmentTarget, IdentifierReference, MemberExpression};
use oxc_semantic::IsGlobalReference;
use oxc_str::static_ident;

use crate::{config::GlobalValue, context::LintContext};

/// Returns whether `ident` is a reference to the global CommonJS `exports` binding.
pub fn is_global_exports_reference(ident: &IdentifierReference, ctx: &LintContext) -> bool {
    ident.name == static_ident!("exports") && ctx.is_reference_to_global_variable(ident)
}

/// Returns whether `ident` is a reference to the global CommonJS `module` binding.
pub fn is_global_module_reference(ident: &IdentifierReference, ctx: &LintContext) -> bool {
    ident.name == static_ident!("module") && ctx.is_reference_to_global_variable(ident)
}

/// Returns whether `node` assigns to the global CommonJS `exports` binding.
///
/// This respects an explicit `globals.exports = off` override, matching rules that should ignore
/// CommonJS globals when users have disabled them.
pub fn is_global_exports_assignment_target(node: &AssignmentTarget, ctx: &LintContext) -> bool {
    let AssignmentTarget::AssignmentTargetIdentifier(id) = node else {
        return false;
    };
    id.is_global_reference_name(static_ident!("exports"), ctx.scoping())
        && !ctx.globals().get("exports").is_some_and(|value| *value == GlobalValue::Off)
}

/// Returns whether `member_expr` is a `module.exports` access on the global CommonJS `module`.
pub fn is_global_module_exports(member_expr: &MemberExpression, ctx: &LintContext) -> bool {
    member_expr.static_property_name().is_some_and(|name| name == "exports")
        && member_expr
            .object()
            .get_identifier_reference()
            .is_some_and(|ident| is_global_module_reference(ident, ctx))
}

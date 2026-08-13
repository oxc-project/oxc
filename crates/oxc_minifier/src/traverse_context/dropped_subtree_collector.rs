use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_semantic::Scoping;

use crate::state::PassChanges;

/// Returns the callee `IdentifierReference` if `call` is a direct
/// `eval(...)` call. Shared by the [`DroppedSubtreeCollector`] producer and the
/// `LiveDirectEvalCollector` consumer — the two must agree on what counts as a
/// direct eval call for the incremental refresh to be sound.
pub fn as_direct_eval_call<'a, 'b>(
    call: &'b CallExpression<'a>,
) -> Option<&'b IdentifierReference<'a>> {
    if call.optional {
        return None;
    }
    let ident = call.callee.get_identifier_reference()?;
    (ident.name == "eval").then_some(ident)
}

/// Walks AST subtrees being dropped or replaced, collecting
/// `IdentifierReference`s and direct `eval(...)` calls into the per-pass
/// [`PassChanges`] accumulator. Use via the `Visit` entry point matching the
/// dropped node (`visit_expression`, `visit_variable_declarator`, ...).
///
/// Mark-only semantics: every reference found in a dropped subtree is added
/// to `pass_changes.removed_references`; every direct eval call sets
/// `pass_changes.direct_eval_dropped = true`. Marks for unresolved references
/// are inert: the flush only filters per-symbol resolved-reference lists,
/// which never contain unresolved ids (and `root_unresolved_references` is
/// deliberately not pruned — no in-loop optimization consumes it and callers
/// rebuild scoping).
///
/// There is deliberately no "resurrect" walk over replacement values: a
/// `ReferenceId` marked removed can never reappear in a replacement. Subtrees
/// moved out of the old slot into the new value leave id-less `TakeIn` dummies
/// behind, so the removed-subtree walk never sees their ids; and replacements
/// are built with fresh references, never cloned ids (see
/// `substitute_is_object_and_not_null`).
pub struct DroppedSubtreeCollector<'a, 's> {
    pass_changes: &'s mut PassChanges<'a>,
    scoping: &'s Scoping,
}

impl<'a, 's> DroppedSubtreeCollector<'a, 's> {
    pub(crate) fn new(pass_changes: &'s mut PassChanges<'a>, scoping: &'s Scoping) -> Self {
        Self { pass_changes, scoping }
    }
}

impl<'a> Visit<'a> for DroppedSubtreeCollector<'a, '_> {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        // Freshly built `IdentifierReference` nodes (e.g. created via
        // `ast.identifier_reference(...)` or as a `TakeIn` dummy left in place
        // by `take_in`) have no `reference_id` yet. Such nodes carry no
        // semantic state to mark removed, so skip them.
        let Some(reference_id) = it.reference_id.get() else { return };
        self.pass_changes.remove_reference(reference_id, self.scoping);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if as_direct_eval_call(it).is_some() {
            self.pass_changes.direct_eval_dropped = true;
        }
        // Recurse — eval may be nested inside another call's arguments.
        walk_call_expression(self, it);
    }
}

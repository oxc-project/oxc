use crate::generated::visit::{Visit, walk::walk_call_expression};
use oxc_ast::ast::*;

use crate::state::PassDirty;

/// Returns the callee `IdentifierReference` if `call` is a direct
/// `eval(...)` call. Shared by the [`DropDiff`] producer and the
/// `LiveDirectEvalCollector` consumer — the two must agree on what counts
/// as a direct eval call for the incremental refresh to be sound.
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
/// `PassDirty` accumulator. Use via the `Visit` entry point matching the
/// dropped node (`visit_expression`, `visit_variable_declarator`, ...).
///
/// Mark-only semantics: every reference found in a dropped subtree is ADDED
/// to `dirty.dead_refs`; every direct eval call sets
/// `dirty.eval_dropped = true`. Marks for unresolved references are inert:
/// the flush only filters per-symbol resolved-reference lists, which never
/// contain unresolved ids (and `root_unresolved_references` is deliberately
/// not pruned — no in-loop optimization consumes it and callers rebuild
/// scoping).
///
/// There is deliberately no "resurrect" walk over replacement values: a
/// `ReferenceId` marked dead can never reappear in a replacement. Subtrees
/// moved out of the old slot into the new value leave id-less `TakeIn`
/// dummies behind, so the dead-walk never sees their ids; and replacements
/// are built with fresh references, never cloned ids (see
/// `substitute_is_object_and_not_null`).
pub struct DropDiff<'a, 's> {
    dirty: &'s mut PassDirty<'a>,
}

impl<'a, 's> DropDiff<'a, 's> {
    pub(crate) fn new(dirty: &'s mut PassDirty<'a>) -> Self {
        Self { dirty }
    }
}

impl<'a> Visit<'a> for DropDiff<'a, '_> {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        // Freshly built `IdentifierReference` nodes (e.g. created via
        // `ast.identifier_reference(...)` or as a `TakeIn` dummy left in place
        // by `take_in`) have no `reference_id` yet. Such nodes carry no
        // semantic state to mark dead, so skip them.
        let Some(reference_id) = it.reference_id.get() else { return };
        let idx = reference_id.index();
        // References minted mid-pass have indices beyond the bitset's
        // capacity and are treated as live — see `PassDirty::dead_refs`.
        // This is a legal flow (a fresh ident minted by one optimization can
        // be dropped later in the same pass by another), so no debug_assert.
        if idx < self.dirty.dead_refs.capacity() {
            self.dirty.dead_refs.set_bit(idx);
        }
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if as_direct_eval_call(it).is_some() {
            self.dirty.eval_dropped = true;
        }
        // Recurse — eval may be nested inside another call's arguments.
        walk_call_expression(self, it);
    }
}

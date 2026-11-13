use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_semantic::Scoping;
use oxc_syntax::symbol::SymbolId;

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

/// The redeclared symbols a binding identifier of `symbol_id` declares: the
/// symbol itself and, for a `var` redeclaring a catch parameter, the enclosing
/// var binding (see `Scoping::catch_var_binding`). Only these need to be kept in
/// sync with `Scoping::symbol_redeclarations`.
pub fn redeclared_symbols(
    symbol_id: SymbolId,
    scoping: &Scoping,
) -> impl Iterator<Item = SymbolId> + '_ {
    std::iter::once(symbol_id)
        .chain(scoping.catch_var_binding(symbol_id))
        .filter(|&symbol_id| !scoping.symbol_redeclarations(symbol_id).is_empty())
}

/// Walks AST subtrees being dropped or replaced, collecting
/// `IdentifierReference`s, direct `eval(...)` calls and declarations of
/// redeclared symbols into the per-pass
/// [`PassChanges`] accumulator. Use via the `Visit` entry point matching the
/// dropped node (`visit_expression`, `visit_variable_declarator`, ...).
///
/// Mark-only semantics: every reference found in a dropped subtree is added
/// to `pass_changes.removed_references`; every direct eval call sets
/// `pass_changes.direct_eval_dropped = true`; every binding of a redeclared
/// symbol is added to `pass_changes.removed_declarations`. Marks for unresolved references
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
/// `substitute_is_object_and_not_null`). Bindings are the exception: `KeepVar`
/// re-emits dropped `var`s with their symbol and span (see
/// `PassChanges::reemitted_declarations`).
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
        let idx = reference_id.index();
        // References minted mid-pass have indices beyond the bitset's capacity
        // and are treated as live — see `PassChanges::removed_references`.
        // This is a legal flow (a fresh ident minted by one optimization can
        // be dropped later in the same pass by another), so no debug_assert.
        if idx < self.pass_changes.removed_references.capacity() {
            self.pass_changes.removed_references.set_bit(idx);
        }
    }

    fn visit_binding_identifier(&mut self, it: &BindingIdentifier<'a>) {
        let Some(symbol_id) = it.symbol_id.get() else { return };
        for symbol_id in redeclared_symbols(symbol_id, self.scoping) {
            self.pass_changes.removed_declarations.insert((symbol_id, it.span));
        }
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if as_direct_eval_call(it).is_some() {
            self.pass_changes.direct_eval_dropped = true;
        }
        // Recurse — eval may be nested inside another call's arguments.
        walk_call_expression(self, it);
    }
}

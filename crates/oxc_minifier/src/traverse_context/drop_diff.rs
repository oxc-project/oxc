use oxc_ast::ast::*;
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_semantic::Scoping;

use crate::state::PassDirty;

/// Walks AST subtrees collecting `IdentifierReference`s and direct `eval(...)`
/// calls, updating the per-pass `PassDirty` accumulator.
///
/// Two distinct walk modes:
///
/// - `walk_old_*` — invoked on a subtree being dropped or replaced. Every
///   resolved reference found is ADDED to `dirty.dead_refs`. Every direct eval
///   call sets `dirty.eval_dropped = true`. Unresolved references are not
///   tracked (see `visit_identifier_reference`).
///
/// - `resurrect_from_expression` — invoked on the replacement value during a
///   `replace_expression` call. Every resolved reference found is REMOVED
///   from `dirty.dead_refs`. Load-bearing only for the single id-preserving
///   site (`substitute_is_object_and_not_null`); other slot kinds cannot alias
///   a `ReferenceId`, so they have no `resurrect_*`.
pub struct DropDiff<'a, 's> {
    dirty: &'s mut PassDirty<'a>,
    scoping: &'s Scoping,
    mode: DropDiffMode,
    /// Set `true` when a `walk_old_*` (MarkDead) walk marks at least one
    /// resolved reference dead. Read by [`Self::resurrect_is_noop`] to skip the
    /// replacement-value walk when there is nothing for it to un-mark.
    marked: bool,
}

#[derive(Clone, Copy)]
enum DropDiffMode {
    /// Add visited refs to the dirty set.
    MarkDead,
    /// Remove visited refs from the dirty set.
    Resurrect,
}

impl<'a, 's> DropDiff<'a, 's> {
    pub(crate) fn new(dirty: &'s mut PassDirty<'a>, scoping: &'s Scoping) -> Self {
        Self { dirty, scoping, mode: DropDiffMode::MarkDead, marked: false }
    }

    pub(crate) fn walk_old_expression(mut self, expr: &Expression<'a>) -> Self {
        self.mode = DropDiffMode::MarkDead;
        self.visit_expression(expr);
        self
    }

    pub(crate) fn walk_old_statement(mut self, stmt: &Statement<'a>) -> Self {
        self.mode = DropDiffMode::MarkDead;
        self.visit_statement(stmt);
        self
    }

    pub(crate) fn walk_old_assignment_target_property(
        mut self,
        prop: &AssignmentTargetProperty<'a>,
    ) -> Self {
        self.mode = DropDiffMode::MarkDead;
        self.visit_assignment_target_property(prop);
        self
    }

    pub(crate) fn walk_old_property_key(mut self, key: &PropertyKey<'a>) -> Self {
        self.mode = DropDiffMode::MarkDead;
        self.visit_property_key(key);
        self
    }

    pub(crate) fn walk_old_for_statement_left(mut self, lhs: &ForStatementLeft<'a>) -> Self {
        self.mode = DropDiffMode::MarkDead;
        self.visit_for_statement_left(lhs);
        self
    }

    pub(crate) fn walk_old_class_element(mut self, element: &ClassElement<'a>) -> Self {
        self.mode = DropDiffMode::MarkDead;
        self.visit_class_element(element);
        self
    }

    /// Whether the `resurrect_*` walk over the replacement value is a provable
    /// no-op and can be skipped.
    ///
    /// `resurrect_*` exists to UN-mark resolved references that `walk_old_*`
    /// marked dead but which actually survive inside the replacement value. The
    /// only way a `ReferenceId` can be in both the old subtree and the new value
    /// is an id-preserving copy. There are exactly two such sites in the
    /// minifier, both in `substitute_alternate_syntax.rs` and both landing within
    /// a single `replace_expression` call: `clone_in_with_semantic_ids` (the
    /// `typeof` operand clone) and `expression_identifier_with_reference_id`
    /// (reusing the null operand's `ReferenceId`).
    /// If `walk_old_*` marked nothing this call (`!marked`), there is no bit for
    /// this call's resurrect to clear, so walking the (often large, moved-in)
    /// replacement subtree only re-confirms already-clear bits.
    ///
    /// Safety: this relies on the same no-cross-call-aliasing invariant the
    /// whole incremental refresh already depends on — a `ReferenceId` is never
    /// simultaneously in a subtree dropped by one helper call and the
    /// replacement value of a *different* call. Both id-preserving sites above
    /// drop the original and install the copy within a single `replace_*` call,
    /// so the survivor is always un-marked by THIS call's resurrect (where
    /// `marked` is `true`). If a future pass ever splits an aliased id across two
    /// helper calls, this skip would leave a live reference pruned — verified
    /// absent by `cargo coverage -- minifier` (no output diff) and the debug
    /// over-prune assertion (`debug_assert_no_over_prune`).
    #[inline]
    fn resurrect_is_noop(&self) -> bool {
        !self.marked
    }

    /// Un-mark resolved references that `walk_old_expression` marked dead but
    /// which survive inside the replacement expression. This is load-bearing
    /// for exactly one site — `substitute_is_object_and_not_null`
    /// (`substitute_alternate_syntax.rs`) — where `clone_in_with_semantic_ids`
    /// and `expression_identifier_with_reference_id` reuse a `ReferenceId` so
    /// the same id lands in both the dropped slot and the replacement.
    ///
    /// Only `Expression` needs this: aliasing can only produce an expression
    /// (there is no statement/property-key/etc. clone), so the other slot kinds
    /// have no `resurrect_*` — their `replace_*` helpers just `walk_old_*`.
    pub(crate) fn resurrect_from_expression(mut self, expr: &Expression<'a>) -> Self {
        if self.resurrect_is_noop() {
            return self;
        }
        self.mode = DropDiffMode::Resurrect;
        self.visit_expression(expr);
        self
    }
}

impl<'a> Visit<'a> for DropDiff<'a, '_> {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        // Freshly built `IdentifierReference` nodes (e.g. created via
        // `ast.identifier_reference(...)` or as a `TakeIn` dummy left in place
        // by `take_in`) have no `reference_id` yet. Such nodes carry no
        // semantic state to mark dead or resurrect, so skip them.
        let Some(reference_id) = it.reference_id.get() else { return };
        let resolved = self.scoping.get_reference(reference_id).symbol_id().is_some();

        let idx = reference_id.index();
        match (self.mode, resolved) {
            (DropDiffMode::MarkDead, true) => {
                self.marked = true;
                // Refs minted MID-pass (via `create_reference` / `clone_in_with_semantic_ids`)
                // would have indices beyond the bitset's capacity (sized at
                // `enter_program`). A `debug_assert!` probe confirmed this case
                // is unreachable in both the test corpus (506 tests) and the
                // size-test corpus (`just minsize`); we rely on that invariant
                // to avoid a per-visit bounds check on the hot path. If the
                // invariant is ever broken in production, this would
                // out-of-bounds panic — caught immediately rather than silently
                // leaking refs.
                debug_assert!(idx < self.dirty.dead_refs.capacity());
                self.dirty.dead_refs.set_bit(idx);
            }
            (DropDiffMode::Resurrect, true) => {
                debug_assert!(idx < self.dirty.dead_refs.capacity());
                self.dirty.dead_refs.unset_bit(idx);
            }
            // Unresolved references (no `symbol_id`) are intentionally untracked:
            // `root_unresolved_references` is not consumed by any in-loop
            // optimization and the compressor's `Scoping` is never read back by a
            // consumer (callers rebuild it), so pruning it would be dead work.
            (_, false) => {}
        }
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if matches!(self.mode, DropDiffMode::MarkDead)
            && !it.optional
            && let Some(ident) = it.callee.get_identifier_reference()
            && ident.name == "eval"
        {
            self.dirty.eval_dropped = true;
        }
        // Recurse — eval may be nested inside another call's arguments.
        walk_call_expression(self, it);
    }
}

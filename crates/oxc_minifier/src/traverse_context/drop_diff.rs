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
///   reference found is ADDED to `dirty.dead_refs` (resolved) or
///   `dirty.dead_unresolved` (unresolved by name). Every direct eval call
///   sets `dirty.eval_dropped = true`.
///
/// - `resurrect_from_*` — invoked on the replacement value during a
///   `replace_*` helper call. Every resolved reference found is REMOVED
///   from `dirty.dead_refs`. Handles within-call and cross-call
///   `ReferenceId` preservation via `clone_in_with_semantic_ids`. Unresolved
///   references are not aggressively un-marked because pruning the
///   unresolved set is name-keyed and a name with many refs survives if any
///   one occurrence does — the `exit_program` prune walk handles this
///   correctly via per-name confirmation.
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
    /// marked dead but which actually survive inside the replacement value
    /// (the only way a reference can be in both the old subtree and the new
    /// value is `clone_in_with_semantic_ids`, which preserves the
    /// `ReferenceId` — see `substitute_alternate_syntax.rs`). If `walk_old_*`
    /// marked nothing this call (`!marked`), there is no bit for this call's
    /// resurrect to clear, so walking the (often large, moved-in) replacement
    /// subtree only re-confirms already-clear bits.
    ///
    /// Safety: this relies on the same no-cross-call-aliasing invariant the
    /// whole incremental refresh already depends on — a `ReferenceId` is never
    /// simultaneously in a subtree dropped by one helper call and the
    /// replacement value of a *different* call. Cloning (the only id-aliasing
    /// site) drops the original and installs the clone within a single
    /// `replace_*` call, so the surviving clone is always un-marked by THIS
    /// call's resurrect (where `marked` is `true`). If a future pass ever splits
    /// an aliased id across two helper calls, this skip would leave a live
    /// reference pruned — verified absent by `cargo coverage -- minifier` (no
    /// output diff) and a debug over-prune assertion during review.
    #[inline]
    fn resurrect_is_noop(&self) -> bool {
        !self.marked
    }

    pub(crate) fn resurrect_from_expression(mut self, expr: &Expression<'a>) -> Self {
        if self.resurrect_is_noop() {
            return self;
        }
        self.mode = DropDiffMode::Resurrect;
        self.visit_expression(expr);
        self
    }

    pub(crate) fn resurrect_from_statement(mut self, stmt: &Statement<'a>) -> Self {
        if self.resurrect_is_noop() {
            return self;
        }
        self.mode = DropDiffMode::Resurrect;
        self.visit_statement(stmt);
        self
    }

    pub(crate) fn resurrect_from_assignment_target_property(
        mut self,
        prop: &AssignmentTargetProperty<'a>,
    ) -> Self {
        if self.resurrect_is_noop() {
            return self;
        }
        self.mode = DropDiffMode::Resurrect;
        self.visit_assignment_target_property(prop);
        self
    }

    pub(crate) fn resurrect_from_property_key(mut self, key: &PropertyKey<'a>) -> Self {
        if self.resurrect_is_noop() {
            return self;
        }
        self.mode = DropDiffMode::Resurrect;
        self.visit_property_key(key);
        self
    }

    pub(crate) fn resurrect_from_for_statement_left(mut self, lhs: &ForStatementLeft<'a>) -> Self {
        if self.resurrect_is_noop() {
            return self;
        }
        self.mode = DropDiffMode::Resurrect;
        self.visit_for_statement_left(lhs);
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
            (DropDiffMode::MarkDead, false) => {
                self.dirty.dead_unresolved.insert(it.name);
            }
            (DropDiffMode::Resurrect, true) => {
                debug_assert!(idx < self.dirty.dead_refs.capacity());
                self.dirty.dead_refs.unset_bit(idx);
            }
            (DropDiffMode::Resurrect, false) => {
                // Intentionally no-op — see struct doc comment for rationale.
            }
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

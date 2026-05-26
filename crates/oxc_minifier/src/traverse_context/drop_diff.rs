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
        Self { dirty, scoping, mode: DropDiffMode::MarkDead }
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

    pub(crate) fn resurrect_from_expression(mut self, expr: &Expression<'a>) -> Self {
        self.mode = DropDiffMode::Resurrect;
        self.visit_expression(expr);
        self
    }

    pub(crate) fn resurrect_from_statement(mut self, stmt: &Statement<'a>) -> Self {
        self.mode = DropDiffMode::Resurrect;
        self.visit_statement(stmt);
        self
    }

    pub(crate) fn resurrect_from_assignment_target_property(
        mut self,
        prop: &AssignmentTargetProperty<'a>,
    ) -> Self {
        self.mode = DropDiffMode::Resurrect;
        self.visit_assignment_target_property(prop);
        self
    }

    pub(crate) fn resurrect_from_property_key(mut self, key: &PropertyKey<'a>) -> Self {
        self.mode = DropDiffMode::Resurrect;
        self.visit_property_key(key);
        self
    }

    pub(crate) fn resurrect_from_for_statement_left(mut self, lhs: &ForStatementLeft<'a>) -> Self {
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

        match (self.mode, resolved) {
            (DropDiffMode::MarkDead, true) => {
                self.dirty.dead_refs.insert(reference_id);
            }
            (DropDiffMode::MarkDead, false) => {
                self.dirty.dead_unresolved.insert(it.name);
            }
            (DropDiffMode::Resurrect, true) => {
                self.dirty.dead_refs.remove(&reference_id);
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

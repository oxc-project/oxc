/// Assert that all break/continue targets reference existent labels.
///
/// Port of `ReactiveScopes/AssertWellFormedBreakTargets.ts` from the React Compiler.
///
/// This is a validation pass that walks the reactive function tree and ensures
/// every break/continue terminal references a label that exists in the enclosing
/// scope chain.
use rustc_hash::FxHashSet;

use crate::{
    compiler_error::{CompilerError, GENERATED_SOURCE},
    hir::{BlockId, ReactiveFunction, ReactiveTerminal, ReactiveTerminalStatement},
    reactive_scopes::visitors::{ReactiveVisitor, visit_reactive_function},
};

/// Assert that all break/continue targets reference existent labels.
///
/// # Errors
/// Returns a `CompilerError` if a break/continue targets a non-existent label.
pub fn assert_well_formed_break_targets(func: &ReactiveFunction) -> Result<(), CompilerError> {
    let mut visitor = Visitor { error: None, seen_labels: FxHashSet::default() };
    visit_reactive_function(func, &mut visitor);
    if let Some(err) = visitor.error {
        return Err(err);
    }
    Ok(())
}

struct Visitor {
    error: Option<CompilerError>,
    seen_labels: FxHashSet<BlockId>,
}

impl ReactiveVisitor for Visitor {
    fn visit_terminal(&mut self, stmt: &ReactiveTerminalStatement) {
        if let Some(ref label) = stmt.label {
            self.seen_labels.insert(label.id);
        }
        match &stmt.terminal {
            ReactiveTerminal::Break(t) => {
                if !self.seen_labels.contains(&t.target) {
                    self.error = Some(CompilerError::invariant(
                        "Unexpected break to invalid label",
                        None,
                        GENERATED_SOURCE,
                    ));
                }
            }
            ReactiveTerminal::Continue(t) => {
                if !self.seen_labels.contains(&t.target) {
                    self.error = Some(CompilerError::invariant(
                        "Unexpected continue to invalid label",
                        None,
                        GENERATED_SOURCE,
                    ));
                }
            }
            _ => {}
        }
    }
}

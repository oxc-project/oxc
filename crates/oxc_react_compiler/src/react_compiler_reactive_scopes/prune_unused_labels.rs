// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Flattens labeled terminals where the label is not reachable, and
//! nulls out labels for other terminals where the label is unused.
//!
//! Corresponds to `src/ReactiveScopes/PruneUnusedLabels.ts`.

use rustc_hash::FxHashSet;

use oxc_diagnostics::OxcDiagnostic;

use crate::react_compiler_hir::{
    BlockId, ReactiveFunction, ReactiveStatement, ReactiveTerminal, ReactiveTerminalStatement,
    ReactiveTerminalTargetKind, environment::Environment,
};

use crate::react_compiler_reactive_scopes::visitors::{
    ReactiveFunctionTransform, Transformed, transform_reactive_function,
};

/// Prune unused labels from a reactive function.
pub fn prune_unused_labels<'a>(
    func: &mut ReactiveFunction<'a>,
    env: &Environment<'a>,
) -> Result<(), OxcDiagnostic> {
    let mut transform = Transform { env };
    let mut labels: FxHashSet<BlockId> = FxHashSet::default();
    transform_reactive_function(func, &mut transform, &mut labels)
}

struct Transform<'a, 'e> {
    env: &'e Environment<'a>,
}

impl<'a, 'e> ReactiveFunctionTransform<'a> for Transform<'a, 'e> {
    type State = FxHashSet<BlockId>;

    fn env(&self) -> &Environment<'a> {
        self.env
    }

    fn transform_terminal(
        &mut self,
        stmt: &mut ReactiveTerminalStatement<'a>,
        state: &mut FxHashSet<BlockId>,
    ) -> Result<Transformed<ReactiveStatement<'a>>, OxcDiagnostic> {
        // Traverse children first
        self.traverse_terminal(stmt, state)?;

        // Collect labeled break/continue targets
        match &stmt.terminal {
            ReactiveTerminal::Break {
                target,
                target_kind: ReactiveTerminalTargetKind::Labeled,
                ..
            }
            | ReactiveTerminal::Continue {
                target,
                target_kind: ReactiveTerminalTargetKind::Labeled,
                ..
            } => {
                state.insert(*target);
            }
            _ => {}
        }

        // Is this terminal reachable via a break/continue to its label?
        let is_reachable_label = stmt.label.as_ref().is_some_and(|label| state.contains(&label.id));

        if let ReactiveTerminal::Label { block, .. } = &mut stmt.terminal
            && !is_reachable_label
        {
            // Flatten labeled terminals where the label isn't necessary.
            // Note: In TS, there's a check for `last.terminal.target === null`
            // to pop a trailing break, but since target is always a BlockId (number),
            // that check is always false, so the trailing break is never removed.
            // ReplaceMany keeps a std `Vec`; drain the arena block into one.
            let flattened = block.drain(..).collect::<Vec<_>>();
            return Ok(Transformed::ReplaceMany(flattened));
        }

        if !is_reachable_label && let Some(label) = &mut stmt.label {
            label.implicit = true;
        }

        Ok(Transformed::Keep)
    }
}

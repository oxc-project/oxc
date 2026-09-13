// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Assert that program blocks and reactive scopes form a properly nested tree.
//!
//! Corresponds to `src/HIR/AssertValidBlockNesting.ts`.

use std::cmp::Ordering;

use oxc_diagnostics::OxcDiagnostic;
use rustc_hash::FxHashSet;

use crate::diagnostics;

use super::environment::Environment;
use super::visitors::{
    each_instruction_lvalue_ids, each_instruction_operand_ids, each_terminal_operand_ids,
    terminal_fallthrough,
};
use super::{EvaluationOrder, HirFunction, IdentifierId, ScopeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BlockRange {
    start: EvaluationOrder,
    end: EvaluationOrder,
}

/// Collect all unique scopes from places in the function that have non-empty ranges.
/// Corresponds to TS `getScopes(fn)`.
pub(crate) fn get_scopes(func: &HirFunction<'_>, env: &Environment<'_>) -> Vec<ScopeId> {
    let mut scope_ids: FxHashSet<ScopeId> = FxHashSet::default();

    let mut visit_place = |identifier_id: IdentifierId| {
        if let Some(scope_id) = env.identifiers[identifier_id].scope {
            let range = &env.scopes[scope_id].range;
            if range.start != range.end {
                scope_ids.insert(scope_id);
            }
        }
    };

    for block in func.body.blocks.values() {
        for &instr_id in &block.instructions {
            let instr = &func.instructions[instr_id.index()];
            for id in each_instruction_lvalue_ids(instr) {
                visit_place(id);
            }
            for id in each_instruction_operand_ids(instr, env) {
                visit_place(id);
            }
        }
        for id in each_terminal_operand_ids(&block.terminal) {
            visit_place(id);
        }
    }

    scope_ids.into_iter().collect()
}

/// Assert that program-block subtrees and reactive scopes are disjoint or nested.
pub fn assert_valid_block_nesting(
    func: &HirFunction<'_>,
    env: &Environment<'_>,
) -> Result<(), OxcDiagnostic> {
    let mut ranges: Vec<BlockRange> = get_scopes(func, env)
        .into_iter()
        .map(|scope_id| {
            let range = env.scopes[scope_id].range;
            BlockRange { start: range.start, end: range.end }
        })
        .collect();

    for block in func.body.blocks.values() {
        let Some(fallthrough_id) = terminal_fallthrough(&block.terminal) else {
            continue;
        };
        let Some(fallthrough) = func.body.blocks.get(&fallthrough_id) else {
            return Err(diagnostics::terminal_successor_references_unknown_block(
                fallthrough_id.index(),
                &block.terminal,
                block.terminal.span().copied(),
            ));
        };
        let end = fallthrough.instructions.first().map_or_else(
            || fallthrough.terminal.evaluation_order(),
            |instr_id| func.instructions[instr_id.index()].id,
        );
        ranges.push(BlockRange { start: block.terminal.evaluation_order(), end });
    }

    assert_valid_ranges(ranges)
}

/// Sort ranges into tree pre-order, then reject partial overlaps.
fn assert_valid_ranges(ranges: Vec<BlockRange>) -> Result<(), OxcDiagnostic> {
    let mut context = ();
    recursively_traverse_items(
        ranges,
        |range, ()| (range.start, range.end),
        &mut context,
        |_, ()| {},
        |_, ()| {},
    )
}

/// Traverse nested ranges in tree pre-order, calling `enter` and `exit` at range boundaries.
/// Corresponds to TS `recursivelyTraverseItems`.
pub(crate) fn recursively_traverse_items<T: Copy, Context>(
    mut items: Vec<T>,
    get_range: impl Fn(&T, &Context) -> (EvaluationOrder, EvaluationOrder),
    context: &mut Context,
    mut enter: impl FnMut(T, &mut Context),
    mut exit: impl FnMut(T, &mut Context),
) -> Result<(), OxcDiagnostic> {
    items.sort_unstable_by(|a, b| {
        let (a_start, a_end) = get_range(a, context);
        let (b_start, b_end) = get_range(b, context);
        let start_order = a_start.cmp(&b_start);
        if start_order != Ordering::Equal {
            return start_order;
        }
        b_end.cmp(&a_end)
    });
    let ranges: Vec<(EvaluationOrder, EvaluationOrder)> =
        items.iter().map(|item| get_range(item, context)).collect();

    let mut active_items: Vec<(T, EvaluationOrder, EvaluationOrder)> = Vec::new();
    for (current, (current_start, current_end)) in items.into_iter().zip(ranges) {
        while let Some(&(_, parent_start, parent_end)) = active_items.last() {
            let disjoint = current_start >= parent_end;
            let nested = current_end <= parent_end;
            if !disjoint && !nested {
                return Err(diagnostics::invalid_block_nesting(
                    parent_start.index(),
                    parent_end.index(),
                    current_start.index(),
                    current_end.index(),
                ));
            }
            if !disjoint {
                break;
            }
            let (parent, _, _) = active_items.pop().unwrap();
            exit(parent, context);
        }

        enter(current, context);
        active_items.push((current, current_start, current_end));
    }

    while let Some((item, _, _)) = active_items.pop() {
        exit(item, context);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn range(start: usize, end: usize) -> BlockRange {
        BlockRange {
            start: EvaluationOrder::from_usize(start),
            end: EvaluationOrder::from_usize(end),
        }
    }

    #[test]
    fn accepts_nested_ranges() {
        assert!(
            assert_valid_ranges(vec![range(2, 4), range(1, 8), range(5, 8), range(1, 5)]).is_ok()
        );
    }

    #[test]
    fn accepts_disjoint_ranges() {
        assert!(assert_valid_ranges(vec![range(8, 9), range(1, 3), range(3, 5)]).is_ok());
    }

    #[test]
    fn rejects_partially_overlapping_ranges() {
        let diagnostic = assert_valid_ranges(vec![range(1, 5), range(3, 7)]).unwrap_err();

        assert_eq!(diagnostic.message, "Invalid nesting in program blocks or scopes");
        assert_eq!(diagnostic.help.as_deref(), Some("Items overlap but are not nested: 1:5(3:7)"));
    }
}

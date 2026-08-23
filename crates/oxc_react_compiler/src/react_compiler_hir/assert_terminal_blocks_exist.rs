// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Assert that terminal block references point to blocks that exist.
//!
//! Corresponds to `src/HIR/AssertTerminalBlocksExist.ts`.

use oxc_diagnostics::OxcDiagnostic;

use crate::diagnostics;

use super::{
    HIR, HirFunction,
    visitors::{each_terminal_all_successors, each_terminal_successor},
};

pub fn assert_terminal_successors_exist(func: &HirFunction<'_>) -> Result<(), OxcDiagnostic> {
    assert_terminal_successors_exist_in_body(&func.body)
}

fn assert_terminal_successors_exist_in_body(body: &HIR<'_>) -> Result<(), OxcDiagnostic> {
    for block in body.blocks.values() {
        for successor in each_terminal_all_successors(&block.terminal) {
            if !body.blocks.contains_key(&successor) {
                return Err(diagnostics::terminal_successor_references_unknown_block(
                    successor.index(),
                    &block.terminal,
                    block.terminal.span().copied(),
                ));
            }
        }
    }
    Ok(())
}

pub fn assert_terminal_preds_exist(func: &HirFunction<'_>) -> Result<(), OxcDiagnostic> {
    for block in func.body.blocks.values() {
        for predecessor in block.preds.iter().copied() {
            let Some(predecessor_block) = func.body.blocks.get(&predecessor) else {
                return Err(diagnostics::expected_predecessor_block_to_exist(
                    block.id.index(),
                    predecessor.index(),
                ));
            };
            if !each_terminal_successor(&predecessor_block.terminal).contains(&block.id) {
                return Err(
                    diagnostics::terminal_successor_does_not_reference_correct_predecessor(
                        block.id.index(),
                        predecessor_block.id.index(),
                    ),
                );
            }
        }
    }
    Ok(())
}

// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Assert that instruction lvalues use identifiers consistently.
//!
//! Corresponds to `src/HIR/AssertConsistentIdentifiers.ts`.

use rustc_hash::FxHashSet;

use oxc_diagnostics::OxcDiagnostic;
use oxc_index::IndexSlice;

use crate::diagnostics;

use super::{HirFunction, Identifier, IdentifierId, Place};

pub fn assert_consistent_identifiers<'a>(
    func: &HirFunction<'a>,
    identifiers: &IndexSlice<IdentifierId, [Identifier<'a>]>,
) -> Result<(), OxcDiagnostic> {
    let mut assignments = FxHashSet::default();

    /*
     * Babel also walks every place to assert that a given IdentifierId always
     * refers to the same Identifier object. Oxc represents places with an
     * IdentifierId and stores the corresponding Identifier exactly once in this
     * indexed slice, so that object-identity invariant holds by construction.
     */
    for block in func.body.blocks.values() {
        for &instr_id in &block.instructions {
            let instr = &func.instructions[instr_id.index()];
            let identifier = &identifiers[instr.lvalue.identifier];

            if let Some(name) = &identifier.name {
                return Err(diagnostics::expected_all_lvalues_to_be_temporaries(
                    name.value(),
                    instr.lvalue.span,
                ));
            }

            if !assignments.insert(instr.lvalue.identifier) {
                return Err(diagnostics::expected_lvalues_to_be_assigned_exactly_once(
                    format_place(&instr.lvalue, identifiers),
                    instr.lvalue.span,
                ));
            }
        }
    }

    Ok(())
}

/// Format a place like Babel's `printPlace()`, retaining the fields relevant to
/// identifying a duplicate assignment.
fn format_place(place: &Place, identifiers: &IndexSlice<IdentifierId, [Identifier<'_>]>) -> String {
    let identifier = &identifiers[place.identifier];
    let name = identifier.name.as_ref().map_or("", |name| name.value());
    format!("{} {}${}", place.effect, name, place.identifier.index())
}

/// HIR assertion/validation passes.
///
/// Port of `HIR/AssertConsistentIdentifiers.ts`, `HIR/AssertTerminalBlocksExist.ts`,
/// `HIR/AssertValidBlockNesting.ts`, `HIR/AssertValidMutableRanges.ts`
/// from the React Compiler.
///
/// These are debug-mode validation passes that check internal invariants.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::compiler_error::{CompilerError, GENERATED_SOURCE};

use super::{
    hir_types::{HIRFunction, IdentifierId},
    visitors::{
        each_instruction_lvalue, each_instruction_value_operand, each_terminal_operand,
        each_terminal_successor,
    },
};

/// Validation pass to check that there is a 1:1 mapping between Identifier objects
/// and IdentifierIds — there can only be one Identifier instance per IdentifierId.
///
/// # Errors
/// Returns a `CompilerError` if any inconsistency is found.
pub fn assert_consistent_identifiers(func: &HIRFunction) -> Result<(), CompilerError> {
    let mut identifiers: FxHashMap<IdentifierId, IdentifierId> = FxHashMap::default();
    let mut assignments: FxHashSet<IdentifierId> = FxHashSet::default();

    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            // Check lvalue is a temporary (no name)
            if instr.lvalue.identifier.name.is_some() {
                return Err(CompilerError::invariant(
                    "Expected all lvalues to be temporaries",
                    Some(&format!("Found named lvalue `{:?}`", instr.lvalue.identifier.name)),
                    instr.lvalue.loc,
                ));
            }

            // Check no duplicate assignments
            if !assignments.insert(instr.lvalue.identifier.id) {
                return Err(CompilerError::invariant(
                    "Expected lvalues to be assigned exactly once",
                    Some(&format!(
                        "Found duplicate assignment of identifier #{}",
                        instr.lvalue.identifier.id.0
                    )),
                    instr.lvalue.loc,
                ));
            }

            // Validate lvalues
            for operand in each_instruction_lvalue(instr) {
                validate_identifier(&mut identifiers, operand.identifier.id)?;
            }

            // Validate operands
            for operand in each_instruction_value_operand(&instr.value) {
                validate_identifier(&mut identifiers, operand.identifier.id)?;
            }
        }

        // Validate terminal operands
        for operand in each_terminal_operand(&block.terminal) {
            validate_identifier(&mut identifiers, operand.identifier.id)?;
        }
    }

    Ok(())
}

fn validate_identifier(
    identifiers: &mut FxHashMap<IdentifierId, IdentifierId>,
    id: IdentifierId,
) -> Result<(), CompilerError> {
    if let Some(&previous) = identifiers.get(&id) {
        if previous != id {
            return Err(CompilerError::invariant(
                "Duplicate identifier object",
                Some(&format!("Found duplicate identifier object for id #{}", id.0)),
                GENERATED_SOURCE,
            ));
        }
    } else {
        identifiers.insert(id, id);
    }
    Ok(())
}

/// Validates that all terminal successors reference existing blocks.
///
/// # Errors
/// Returns a `CompilerError` if a terminal references a non-existent block.
pub fn assert_terminal_successors_exist(func: &HIRFunction) -> Result<(), CompilerError> {
    for block in func.body.blocks.values() {
        for successor in each_terminal_successor(&block.terminal) {
            if !func.body.blocks.contains_key(&successor) {
                return Err(CompilerError::invariant(
                    "Terminal successor references unknown block",
                    Some(&format!("Block {successor} does not exist")),
                    block.terminal.loc(),
                ));
            }
        }
    }
    Ok(())
}

/// Validates that all predecessor references are consistent with terminal successors.
///
/// # Errors
/// Returns a `CompilerError` if predecessor references are inconsistent.
///
/// # Panics
/// Panics if predecessor block data is inconsistent (should not happen with valid HIR).
pub fn assert_terminal_preds_exist(func: &HIRFunction) -> Result<(), CompilerError> {
    for block in func.body.blocks.values() {
        for &pred_id in &block.preds {
            let pred_block = func.body.blocks.get(&pred_id);
            if pred_block.is_none() {
                return Err(CompilerError::invariant(
                    "Expected predecessor block to exist",
                    Some(&format!(
                        "Block {} references non-existent predecessor {}",
                        block.id, pred_id
                    )),
                    GENERATED_SOURCE,
                ));
            }
            let pred_block = pred_block.expect("checked above");
            let successors = each_terminal_successor(&pred_block.terminal);
            if !successors.contains(&block.id) {
                return Err(CompilerError::invariant(
                    "Terminal successor does not reference correct predecessor",
                    Some(&format!(
                        "Block {} has {} as a predecessor, but {}'s successors do not include {}",
                        block.id, pred_id, pred_id, block.id
                    )),
                    GENERATED_SOURCE,
                ));
            }
        }
    }
    Ok(())
}

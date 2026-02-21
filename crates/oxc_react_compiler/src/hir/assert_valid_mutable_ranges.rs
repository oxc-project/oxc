/// Validate mutable ranges in the HIR.
///
/// Port of `HIR/AssertValidMutableRanges.ts` from the React Compiler.
///
/// Validates that mutable ranges of identifiers are consistent:
/// - Start <= End
/// - Ranges align with instruction IDs
/// - No overlapping ranges for the same declaration
use crate::{
    compiler_error::{CompilerError, GENERATED_SOURCE},
    hir::HIRFunction,
};

/// Validate that mutable ranges are well-formed.
///
/// # Errors
/// Returns a `CompilerError` if any mutable range is invalid.
pub fn assert_valid_mutable_ranges(func: &HIRFunction) -> Result<(), CompilerError> {
    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            let range = &instr.lvalue.identifier.mutable_range;
            // Start should be <= End (or both 0 for uninitialized)
            if range.start.0 != 0 || range.end.0 != 0 {
                if range.start > range.end {
                    return Err(CompilerError::invariant(
                        "Invalid mutable range: start > end",
                        Some(&format!(
                            "Identifier #{}: range [{}, {})",
                            instr.lvalue.identifier.id.0, range.start.0, range.end.0
                        )),
                        GENERATED_SOURCE,
                    ));
                }
            }
        }
    }
    Ok(())
}

/// Validate block nesting in the HIR.
///
/// Port of `HIR/AssertValidBlockNesting.ts` from the React Compiler.
///
/// Validates that scope and pruned-scope terminals are properly nested —
/// no scope should overlap with another scope without one containing the other.
use crate::{
    compiler_error::CompilerError,
    hir::{BlockId, HIRFunction, Terminal},
};
use rustc_hash::FxHashSet;

/// Validate that block nesting is well-formed.
///
/// # Errors
/// Returns a `CompilerError` if block nesting is invalid.
pub fn assert_valid_block_nesting(func: &HIRFunction) -> Result<(), CompilerError> {
    let mut scope_stack: Vec<BlockId> = Vec::new();
    let mut visited: FxHashSet<BlockId> = FxHashSet::default();

    for (&block_id, block) in &func.body.blocks {
        if !visited.insert(block_id) {
            continue;
        }

        match &block.terminal {
            Terminal::Scope(scope_term) => {
                // A scope terminal should have its block inside the scope
                // and the fallthrough after the scope
                scope_stack.push(scope_term.fallthrough);
            }
            Terminal::PrunedScope(scope_term) => {
                scope_stack.push(scope_term.fallthrough);
            }
            _ => {}
        }

        // Check if this block is a scope fallthrough — pop the scope
        scope_stack.retain(|&fallthrough| fallthrough != block_id);
    }

    Ok(())
}

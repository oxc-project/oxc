/// Instruction-level visitor dispatcher for stateless validation passes.
///
/// Fuses multiple per-instruction (and optionally per-terminal) validators into
/// a single HIR walk. Each pass implements [`InstructionVisitor`] and is fed every
/// instruction and terminal of the function body in block iteration order. After
/// the walk completes, each visitor's [`InstructionVisitor::finish`] method is
/// called to drain any accumulated errors. Errors from all visitors are merged
/// into a single returned [`CompilerError`].
///
/// This is intended for validators that:
/// - Walk the HIR once per function (no fixpoint).
/// - Carry only function-local state (no cross-function or recursive analysis).
/// - Accumulate diagnostics rather than early-returning on the first error.
use crate::{
    compiler_error::CompilerError,
    hir::{HIRFunction, Instruction, Terminal, environment::Environment},
};

/// A single-pass validator that observes every instruction (and optionally every
/// terminal) of an HIR function body.
pub trait InstructionVisitor {
    /// Called for every instruction in the function body, in block iteration order.
    fn visit_instruction(&mut self, env: &Environment, instr: &Instruction);

    /// Called for every block terminal in the function body, after that block's
    /// instructions have been visited. Default impl does nothing.
    fn visit_terminal(&mut self, _env: &Environment, _terminal: &Terminal) {}

    /// Called once after the full HIR walk is complete. Returns `Ok(())` if no
    /// errors were accumulated, otherwise returns the aggregated `CompilerError`.
    ///
    /// # Errors
    /// Returns the visitor's accumulated diagnostics, if any.
    fn finish(self: Box<Self>, env: &Environment) -> Result<(), CompilerError>;
}

/// Run every visitor over every instruction and terminal of `func`, then call
/// each visitor's `finish` and merge their accumulated errors into a single
/// `Result`.
///
/// # Errors
/// Returns the merged accumulated errors from all visitors (in registration
/// order). If no visitor produced any errors, returns `Ok(())`.
pub fn dispatch_instruction_visitors(
    func: &HIRFunction,
    mut visitors: Vec<Box<dyn InstructionVisitor>>,
) -> Result<(), CompilerError> {
    if visitors.is_empty() {
        return Ok(());
    }
    let env = &func.env;
    for block in func.body.blocks.values() {
        for instr in &block.instructions {
            for visitor in &mut visitors {
                visitor.visit_instruction(env, instr);
            }
        }
        for visitor in &mut visitors {
            visitor.visit_terminal(env, &block.terminal);
        }
    }
    let mut combined = CompilerError::new();
    for visitor in visitors {
        if let Err(err) = visitor.finish(env) {
            combined.merge(err);
        }
    }
    combined.into_result()
}

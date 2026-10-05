//! Comment assignment for the Oxc AST.
//!
//! Stores comment ownership on the source comments using existing AST node IDs.

mod engine;

use oxc_ast::ast::Program;

use engine::AssignmentVisitor;

/// Assign comments not already attached by the parser.
///
/// The AST must still match its source text and have unique node IDs, with
/// [`oxc_syntax::node::NodeId::ROOT`] reserved for Program.
/// Existing attachments must refer to valid owners in the unmodified AST.
pub fn assign_remaining(program: &mut Program<'_>) {
    if !program.comments.is_empty() {
        AssignmentVisitor::assign_remaining(program);
    }
}

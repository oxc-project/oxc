//! Comment assignment for the Oxc AST.
//!
//! Stores comment ownership on the source comments using existing AST node IDs.

mod engine;

use oxc_ast::ast::Program;

use engine::AssignmentVisitor;

/// Assigns comment ownership using the node IDs already present in the AST.
#[derive(Default)]
pub struct CommentAssignment;

impl CommentAssignment {
    /// Create a new assignment pass.
    #[inline]
    pub fn new() -> Self {
        Self
    }

    /// Assign ownership of every comment in `program`.
    ///
    /// The parser runs this automatically. Use this standalone entry point only
    /// when the AST still matches its source text.
    ///
    /// The AST must have unique node IDs, with [`oxc_syntax::node::NodeId::ROOT`]
    /// reserved for Program, as provided by the parser or semantic analysis.
    /// Existing node IDs and semantic data remain valid. Comment attachments are
    /// written directly; source order and token-boundary metadata are left intact.
    #[expect(
        clippy::unused_self,
        reason = "Preserve the assignment pass API without an ID counter"
    )]
    pub fn assign(self, program: &mut Program<'_>) {
        if !program.comments.is_empty() {
            AssignmentVisitor::assign::<false>(program);
        }
    }

    /// Assign comments not already attached by the parser.
    ///
    /// Existing attachments must refer to valid owners in the unmodified AST.
    #[expect(clippy::unused_self, reason = "Preserve the assignment pass API")]
    pub fn assign_remaining(self, program: &mut Program<'_>) {
        if !program.comments.is_empty() {
            AssignmentVisitor::assign::<true>(program);
        }
    }
}

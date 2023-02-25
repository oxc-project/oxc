mod id;
mod tree;

pub use id::AstNodeId;
use oxc_ast::AstKind;
pub use tree::AstNodes;

/// Indextree node containing a semantic node
pub type AstNode<'a> = indextree::Node<SemanticNode<'a>>;

/// Semantic node contains all the semantic information about an ast node.
#[derive(Debug, Clone, Copy)]
pub struct SemanticNode<'a> {
    /// A pointer to the ast node, which resides in the `bumpalo` memory arena.
    kind: AstKind<'a>,
}

impl<'a> SemanticNode<'a> {
    #[must_use]
    pub const fn new(kind: AstKind<'a>) -> Self {
        Self { kind }
    }

    pub const fn kind(&self) -> AstKind<'a> {
        self.kind
    }
}

mod nodes;

pub use nodes::AstNodes;

use oxc_allocator::{Address, GetAddress};
use oxc_ast::AstKind;
use oxc_span::{GetSpan, Span};
use oxc_syntax::{node::NodeId, scope::ScopeId};

/// Semantic node contains all the semantic information about an ast node.
#[derive(Debug, Clone, Copy)]
pub struct AstNode<'a> {
    id: NodeId,
    /// A pointer to the ast node, which resides in the `bumpalo` memory arena.
    kind: AstKind<'a>,

    /// Associated Scope (initialized by binding)
    scope_id: ScopeId,
}

impl<'a> AstNode<'a> {
    pub(crate) fn new(kind: AstKind<'a>, scope_id: ScopeId, id: NodeId) -> Self {
        Self { id, kind, scope_id }
    }

    /// This node's unique identifier.
    #[inline]
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// Access the underlying struct from [`oxc_ast`].
    #[inline]
    pub fn kind(&self) -> AstKind<'a> {
        self.kind
    }

    /// The scope in which this node was declared.
    ///
    /// It is important to note that this is _not_ the scope created _by_ the
    /// node. For example, given a function declaration, this is the scope where
    /// the function is declared, not the scope created by its body.
    #[inline]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id
    }
}

impl GetSpan for AstNode<'_> {
    #[inline]
    fn span(&self) -> Span {
        self.kind.span()
    }
}

impl GetAddress for AstNode<'_> {
    #[inline]
    fn address(&self) -> Address {
        self.kind.address()
    }
}

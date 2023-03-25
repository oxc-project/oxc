#![allow(non_upper_case_globals)] // for bitflags

mod id;
mod tree;

use bitflags::bitflags;
use oxc_ast::AstKind;

pub use self::{id::AstNodeId, tree::AstNodes};
use crate::scope::{Scope, ScopeId};

/// Indextree node containing a semantic node
pub type AstNode<'a> = indextree::Node<SemanticNode<'a>>;

/// Semantic node contains all the semantic information about an ast node.
#[derive(Debug, Clone, Copy)]
pub struct SemanticNode<'a> {
    /// A pointer to the ast node, which resides in the `bumpalo` memory arena.
    kind: AstKind<'a>,

    /// Associated Scope (initialized by binding)
    scope_id: ScopeId,

    flags: NodeFlags,
}

bitflags! {
    #[derive(Default)]
    pub struct NodeFlags: u8 {
        const JsDoc = 1 << 0; // If the Node has a JsDoc comment attached
        const Class = 1 << 1; // If Node is inside a class
    }
}

impl<'a> SemanticNode<'a> {
    #[must_use]
    pub fn new(kind: AstKind<'a>, scope_id: ScopeId, flags: NodeFlags) -> Self {
        Self { kind, scope_id, flags }
    }

    #[must_use]
    pub fn kind(&self) -> AstKind<'a> {
        self.kind
    }

    #[must_use]
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id
    }

    #[must_use]
    pub fn strict_mode(&self, scope: &Scope) -> bool {
        // All parts of a ClassDeclaration or a ClassExpression are strict mode code.
        scope.strict_mode() || self.in_class()
    }

    #[must_use]
    pub fn in_class(self) -> bool {
        self.flags.contains(NodeFlags::Class)
    }

    #[must_use]
    pub fn has_jsdoc(&self) -> bool {
        self.flags.contains(NodeFlags::JsDoc)
    }
}

//! Semantic Builder
//! This builds:
//!   * The untyped and flattened ast nodes into an indextree

use oxc_ast::{ast::Program, visit::Visit, AstKind};

use crate::{
    node::{AstNodeId, AstNodes, SemanticNode},
    Semantic,
};

#[derive(Debug)]
pub struct SemanticBuilder<'a> {
    nodes: AstNodes<'a>,
    current_node_id: AstNodeId,
}

impl<'a> SemanticBuilder<'a> {
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut nodes = AstNodes::default();
        let semantic_node = SemanticNode::new(AstKind::Root);
        let current_node_id = nodes.new_node(semantic_node).into();
        Self { nodes, current_node_id }
    }

    #[must_use]
    pub fn build(mut self, program: &'a Program<'a>) -> Semantic<'a> {
        // AST pass
        self.visit_program(program);
        Semantic { nodes: self.nodes }
    }

    fn create_ast_node(&mut self, kind: AstKind<'a>) {
        let ast_node = SemanticNode::new(kind);
        let node_id = self.nodes.new_node(ast_node);
        self.current_node_id.append(node_id, &mut self.nodes);
        self.current_node_id = node_id.into();
    }

    fn pop_ast_node(&mut self) {
        self.current_node_id =
            self.nodes[self.current_node_id.indextree_id()].parent().unwrap().into();
    }
}

impl<'a> Visit<'a> for SemanticBuilder<'a> {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        self.create_ast_node(kind);
    }

    fn leave_node(&mut self, _kind: AstKind<'a>) {
        self.pop_ast_node();
    }
}

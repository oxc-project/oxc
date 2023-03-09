//! Semantic Builder
//! This builds:
//!   * The untyped and flattened ast nodes into an indextree

use std::rc::Rc;

use oxc_ast::{ast::Program, visit::Visit, AstKind, SourceType, Trivias};

use crate::{
    node::{AstNodeId, AstNodes, NodeFlags, SemanticNode},
    scope::ScopeBuilder,
    Semantic,
};

pub struct SemanticBuilder<'a> {
    // states
    current_node_id: AstNodeId,
    current_node_flags: NodeFlags,

    // builders
    nodes: AstNodes<'a>,
    scope: ScopeBuilder,
}

impl<'a> SemanticBuilder<'a> {
    #[must_use]
    pub fn new(source_type: SourceType) -> Self {
        let scope = ScopeBuilder::new(source_type);
        let mut nodes = AstNodes::default();
        let semantic_node =
            SemanticNode::new(AstKind::Root, scope.current_scope_id, NodeFlags::empty());
        let current_node_id = nodes.new_node(semantic_node).into();
        Self { current_node_id, nodes, scope, current_node_flags: NodeFlags::empty() }
    }

    #[must_use]
    pub fn build(mut self, program: &'a Program<'a>, trivias: Rc<Trivias>) -> Semantic<'a> {
        // AST pass
        self.visit_program(program);
        Semantic { nodes: self.nodes, trivias }
    }

    fn create_ast_node(&mut self, kind: AstKind<'a>) {
        let ast_node =
            SemanticNode::new(kind, self.scope.current_scope_id, self.current_node_flags);
        let node_id = self.current_node_id.append_value(ast_node, &mut self.nodes);

        self.current_node_id = node_id.into();
    }

    fn pop_ast_node(&mut self) {
        self.current_node_id =
            self.nodes[self.current_node_id.indextree_id()].parent().unwrap().into();
    }

    fn try_enter_scope(&mut self, kind: AstKind<'a>) {
        if let Some(flags) = ScopeBuilder::scope_flags_from_ast_kind(kind) {
            self.scope.enter(flags);
        }
    }

    fn try_leave_scope(&mut self, kind: AstKind<'a>) {
        if ScopeBuilder::scope_flags_from_ast_kind(kind).is_some()
            || matches!(kind, AstKind::Program(_))
        {
            self.scope.leave();
        }
    }
}

impl<'a> Visit<'a> for SemanticBuilder<'a> {
    // Setup all the context for the binder,
    // the order is important here.
    fn enter_node(&mut self, kind: AstKind<'a>) {
        // create new self.scope.current_scope_id
        self.try_enter_scope(kind);

        // create new self.current_node_id
        self.create_ast_node(kind);
    }

    fn leave_node(&mut self, kind: AstKind<'a>) {
        self.pop_ast_node();
        self.try_leave_scope(kind);
    }
}

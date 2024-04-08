use std::borrow::Borrow;

use dashmap::DashMap;
use oxc_ast::ast::{Expression, Program, Statement};
use oxc_ast::AstOwnedKind;
use oxc_semantic::AstNodeId;
use oxc_span::Span;

use crate::tst_node::*;

pub struct Tst<'a> {
    /// Map of all nodes by their ID.
    nodes: DashMap<AstNodeId, TstNode<'a>>,

    /// The current node ID being transformed.
    /// We use this as a quick lookup for helper operations.
    current_id: AstNodeId,
}

// HELPERS ACCESSING NODES

impl<'a> Tst<'a> {
    pub fn check_parent(&self, op: impl FnOnce(&TstNode<'a>) -> bool) -> bool {
        let current = self.nodes.get(&self.current_id).unwrap();
        let current = current.borrow();

        let parent = self.nodes.get(&current.parent_id).unwrap();
        let parent = parent.borrow();

        op(parent)
    }

    pub fn check_parent_node(&self, op: impl FnOnce(&AstOwnedKind<'a>) -> bool) -> bool {
        self.check_parent(|node| op(&node.node))
    }

    pub fn check_ancestors(&self, op: impl Fn(&TstNode<'a>) -> bool) -> bool {
        let current = self.nodes.get(&self.current_id).unwrap();
        let current = current.borrow();

        for parent_id in &current.parent_ids {
            let parent = self.nodes.get(parent_id).unwrap();
            let parent = parent.borrow();

            if op(parent) {
                return true;
            }
        }

        false
    }

    pub fn check_ancestor_nodes(&self, op: impl Fn(&AstOwnedKind<'a>) -> bool) -> bool {
        self.check_ancestors(|node| op(&node.node))
    }
}

// CONVERTING FROM AN AST

#[derive(Debug)]
pub struct TstBuilder<'a> {
    nodes: DashMap<AstNodeId, TstNode<'a>>,
    current_id: AstNodeId,
    current_ids_stack: Vec<AstNodeId>,
}

impl<'a> TstBuilder<'a> {
    pub fn from_ast(program: Program<'a>) -> Self {
        let mut tst = Self {
            nodes: DashMap::default(),
            current_id: AstNodeId::new(0),
            current_ids_stack: Vec::new(),
        };
        let node = program.into_tst(&mut tst);
        tst.add_node(node);
        tst
    }

    pub fn to_ast(self) -> Option<Program<'a>> {
        None
    }

    pub fn create_node(&mut self) -> TstNode<'a> {
        let parent_id = self.current_ids_stack.last().cloned().unwrap_or(self.current_id);
        let parent_ids = self.current_ids_stack.clone();

        self.current_id += 1;

        TstNode {
            node: AstOwnedKind::Elision(Span::new(0, 0)),
            id: self.current_id,
            parent_id,
            parent_ids,
            children_ids: TstNodeChildren::None,
        }
    }

    pub fn push_parent(&mut self, id: AstNodeId) {
        self.current_ids_stack.push(id);
    }

    pub fn pop_parent(&mut self) {
        self.current_ids_stack.pop();
    }

    pub fn add_node(&mut self, node: TstNode<'a>) -> AstNodeId {
        let id = node.id.clone();

        self.nodes.insert(id, node);

        id
    }

    pub fn map_expression(&mut self, expr: Expression<'a>) -> AstNodeId {
        let node = match expr {
            Expression::NumericLiteral(lit) => lit.unbox().into_tst(self),
            _ => unreachable!(),
        };

        self.add_node(node)
    }

    pub fn map_statement(&mut self, stmt: Statement<'a>) -> AstNodeId {
        let node = match stmt {
            Statement::BlockStatement(block) => block.unbox().into_tst(self),
            Statement::ExpressionStatement(expr) => expr.unbox().into_tst(self),
            _ => unreachable!(),
        };

        self.add_node(node)
    }
}

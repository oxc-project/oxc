use std::borrow::Borrow;

use dashmap::DashMap;
use oxc_ast::ast::Program;
use oxc_ast::{AstOwnedKind, AstType};
use oxc_semantic::AstNodeId;

pub enum TstNodeChildren {
    None,
    One(AstNodeId),
    Many(Vec<AstNodeId>),
}

pub struct TstNode<'a> {
    /// Kind of node, for quick lookups.
    kind: AstType,

    /// The node itself.
    node: AstOwnedKind<'a>,

    /// ID of itself.
    id: AstNodeId,

    /// ID of direct parent.
    parent_id: AstNodeId,

    /// IDs of all ancestor parents.
    parent_ids: Vec<AstNodeId>,

    /// IDs of all children.
    children_ids: TstNodeChildren,
}

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
}

// CONVERTING FROM AN AST

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
        tst.add_node(AstOwnedKind::Program(program), AstNodeId::new(0));
        tst
    }

    pub fn to_ast(self) -> Option<Program<'a>> {
        None
    }

    pub fn add_node(&mut self, ast_node: AstOwnedKind<'a>, parent_id: AstNodeId) -> &mut Self {
        // Increment IDs before doing anything
        self.current_id += 1;

        let id = self.current_id;
        let parent_ids = self.current_ids_stack.clone();

        self.current_ids_stack.push(id);

        // Process the node by separating its children from itself,
        // allowing us to store them in separate locations. We then
        // keep a reference between the parent and children using IDs.

        match ast_node {
            AstOwnedKind::Program(mut program) => {
                // Extract the children
                let children = program.body.drain(..).collect::<Vec<_>>();

                // Process each child
                let children_ids =
                    children.into_iter().map(|_| AstNodeId::new(0)).collect::<Vec<_>>();

                // Insert the parent node
                self.nodes.insert(
                    id,
                    TstNode {
                        kind: AstType::Program,
                        node: AstOwnedKind::Program(program),
                        id,
                        parent_id,
                        parent_ids,
                        children_ids: TstNodeChildren::Many(children_ids),
                    },
                );
            }
            _ => {}
        };

        // Undo the stack after building
        self.current_ids_stack.pop();

        self
    }
}

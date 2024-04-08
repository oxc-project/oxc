use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::sync::RwLock;

use dashmap::DashMap;
use oxc_allocator::{Allocator, Box as ABox, String, Vec as AVec};
use oxc_ast::ast::{Expression, Program, Statement};
use oxc_ast::AstOwnedKind;
use oxc_semantic::AstNodeId;
use oxc_span::Atom;

use crate::tst_node::*;
use crate::tst_visit::VisitTransform;

type NodesMap<'a> = Rc<RwLock<HashMap<AstNodeId, TstNode<'a>>>>;

pub struct TstContext<'a> {
    allocator: &'a Allocator,

    /// Map of all nodes by their ID.
    // nodes: DashMap<AstNodeId, TstNode<'a>>,
    nodes: NodesMap<'a>,

    // Duplicate this data to avoid ownership issues
    current_id: AstNodeId,
    parent_id: AstNodeId,
    parent_ids: Vec<AstNodeId>,
    children_ids: TstNodeChildren,
}

// HELPERS ACCESSING NODES

impl<'a> TstContext<'a> {
    pub fn new(allocator: &'a Allocator, nodes: NodesMap<'a>) -> Self {
        Self {
            allocator,
            nodes,
            current_id: AstNodeId::new(0),
            parent_id: AstNodeId::new(0),
            parent_ids: vec![],
            children_ids: TstNodeChildren::None,
        }
    }

    #[inline]
    pub fn alloc<T>(&self, value: T) -> ABox<'a, T> {
        ABox(self.allocator.alloc(value))
    }

    #[inline]
    pub fn new_vec<T>(&self) -> AVec<'a, T> {
        AVec::new_in(self.allocator)
    }

    #[inline]
    pub fn new_vec_with_capacity<T>(&self, capacity: usize) -> AVec<'a, T> {
        AVec::with_capacity_in(capacity, self.allocator)
    }

    #[inline]
    pub fn new_vec_single<T>(&self, value: T) -> AVec<'a, T> {
        let mut vec = self.new_vec_with_capacity(1);
        vec.push(value);
        vec
    }

    #[inline]
    pub fn new_str(&self, value: &str) -> &'a str {
        String::from_str_in(value, self.allocator).into_bump_str()
    }

    #[inline]
    pub fn new_atom(&self, value: &str) -> Atom<'a> {
        Atom::from(String::from_str_in(value, self.allocator).into_bump_str())
    }

    pub fn check_parent(&self, op: impl FnOnce(&TstNode<'a>) -> bool) -> bool {
        let nodes = self.nodes.read().unwrap();
        let parent = nodes.get(&self.parent_id).unwrap();

        op(parent)
    }

    pub fn check_parent_node(&self, op: impl FnOnce(&AstOwnedKind<'a>) -> bool) -> bool {
        self.check_parent(|node| op(node.node.as_ref().unwrap()))
    }

    pub fn check_ancestors(&self, op: impl Fn(&TstNode<'a>) -> bool) -> bool {
        let nodes = self.nodes.read().unwrap();

        for parent_id in &self.parent_ids {
            let parent = nodes.get(parent_id).unwrap();

            if op(parent) {
                return true;
            }
        }

        false
    }

    pub fn check_ancestor_nodes(&self, op: impl Fn(&AstOwnedKind<'a>) -> bool) -> bool {
        self.check_ancestors(|node| op(&node.node.as_ref().unwrap()))
    }
}

// CONVERTING FROM AN AST

pub struct Tst<'a> {
    allocator: &'a Allocator,
    // nodes: Rc<DashMap<AstNodeId, TstNode<'a>>>,
    nodes: NodesMap<'a>,
    current_id: AstNodeId,
    current_ids_stack: Vec<AstNodeId>,
    transformers: Vec<Box<dyn VisitTransform<'a>>>,
}

impl<'a> fmt::Debug for Tst<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tst")
            .field("nodes", &self.nodes)
            .field("current_id", &self.current_id)
            .field("current_ids_stack", &self.current_ids_stack)
            .field("transformers", &self.transformers)
            .finish()
    }
}

impl<'a> Tst<'a> {
    pub fn from_ast(allocator: &'a Allocator, program: Program<'a>) -> Self {
        let mut tst = Self {
            allocator,
            // nodes: Rc::new(DashMap::default()),
            nodes: Rc::new(RwLock::new(HashMap::new())),
            current_id: AstNodeId::new(0),
            current_ids_stack: Vec::new(),
            transformers: vec![],
        };
        let node = program.into_tst(&mut tst);
        tst.add_node(node);
        tst
    }

    pub fn to_ast(self) -> Option<Program<'a>> {
        None
    }

    pub fn add_transformer(&mut self, transformer: impl VisitTransform<'a> + 'static) {
        self.transformers.push(Box::new(transformer));
    }

    pub fn create_node(&mut self) -> TstNode<'a> {
        let parent_id = self.current_ids_stack.last().cloned().unwrap_or(self.current_id);
        let parent_ids = self.current_ids_stack.clone();

        self.current_id += 1;

        TstNode {
            node: None,
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

        self.nodes.write().unwrap().insert(id, node);

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

    pub fn transform(&mut self) {
        self.do_transform(AstNodeId::new(1)); // Program is always 1!
    }

    pub fn do_transform(&mut self, id: AstNodeId) {
        // First, retrieve the node mutably, so that we can mutate
        // the inner node directly. Parent/children will be later!
        {
            let mut context = TstContext::new(&self.allocator, Rc::clone(&self.nodes));

            // Extract the inner node and ID references, then release
            // the lock so that we can acquire write access when
            // running the transformers!
            let mut inner_node = {
                let mut nodes = self.nodes.write().unwrap();
                let node = nodes.get_mut(&id).unwrap();

                context.current_id = id;
                context.parent_id = node.parent_id;
                context.parent_ids = node.parent_ids.clone();
                context.children_ids = node.children_ids.clone();

                node.node.take()
            };

            if let Some(inner_node) = &mut inner_node {
                match inner_node {
                    AstOwnedKind::Program(inner) => {
                        for t in &mut self.transformers {
                            t.transform_program(inner, &mut context);
                        }
                    }
                    AstOwnedKind::BlockStatement(inner) => {
                        for t in &mut self.transformers {
                            t.transform_block_statement(inner, &mut context);
                        }
                    }
                    AstOwnedKind::NumericLiteral(inner) => {
                        for t in &mut self.transformers {
                            t.transform_numeric_literal(inner, &mut context);
                        }
                    }
                    _ => {}
                };
            }

            // The node has potentially been mutated,
            // so let's inject the inner node back into our map!
            if inner_node.is_some() {
                self.nodes.write().unwrap().get_mut(&id).unwrap().node = inner_node;
            }
        }

        // Second, access the node immutably and process the children!
        let children_ids = { self.nodes.read().unwrap().get(&id).unwrap().children_ids.clone() };

        match children_ids {
            TstNodeChildren::None => {}
            TstNodeChildren::One(child_id) => {
                self.do_transform(child_id);
            }
            TstNodeChildren::Many(child_ids) => {
                for child_id in child_ids {
                    self.do_transform(child_id);
                }
            }
        }
    }
}

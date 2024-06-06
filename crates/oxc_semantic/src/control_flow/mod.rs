mod builder;
mod dot;

use oxc_span::CompactStr;
use oxc_syntax::operator::{
    AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator, UpdateOperator,
};
use petgraph::{
    stable_graph::NodeIndex,
    visit::{depth_first_search, Control, DfsEvent},
    Graph,
};

use crate::AstNodeId;

pub use builder::{ControlFlowGraphBuilder, CtxCursor, CtxFlags};
pub use dot::{DebugDot, DebugDotContext, DisplayDot};

pub type BasicBlockId = NodeIndex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Register {
    Index(u32),
    Return,
}

#[derive(Debug, Clone)]
pub enum ObjectPropertyAccessBy {
    PrivateProperty(CompactStr),
    Property(CompactStr),
    Expression(Register),
}

#[derive(Debug, Clone)]
pub struct CollectionAssignmentValue {
    pub id: AstNodeId,
    pub elements: Vec<Register>,
    pub spreads: Vec<usize>,
    pub collection_type: CollectionType,
}

#[derive(Debug, Clone)]
pub struct CalleeWithArgumentsAssignmentValue {
    pub id: AstNodeId,
    pub callee: Register,
    pub arguments: Vec<Register>,
    pub spreads: Vec<usize>,
    pub call_type: CallType,
}

#[derive(Debug, Clone)]
pub struct ObjectPropertyAccessAssignmentValue {
    pub id: AstNodeId,
    pub access_on: Register,
    pub access_by: ObjectPropertyAccessBy,
    pub optional: bool,
}

#[derive(Debug, Clone)]
pub struct BinaryAssignmentValue {
    pub id: AstNodeId,
    pub a: Register,
    pub b: Register,
    pub operator: BinaryOp,
}

#[derive(Debug, Clone)]
pub struct UpdateAssignmentValue {
    pub id: AstNodeId,
    pub expr: Register,
    pub op: UpdateOperator,
    pub prefix: bool,
}

#[derive(Debug, Clone)]
pub struct UnaryExpressioneAssignmentValue(pub AstNodeId, pub UnaryOperator, pub Register);

#[derive(Debug, Clone)]
pub enum AssignmentValue {
    ImplicitUndefined,
    NotImplicitUndefined,
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    BinaryOperator(BinaryOperator),
    LogicalOperator(LogicalOperator),
    AssignmentOperator(AssignmentOperator),
}

#[derive(Debug, Clone)]
pub enum CollectionType {
    Array,
    // Note: we do not currently track object names in objects.
    Object,
    JSXElement,
    JSXFragment,
    // doesn't use spreads
    Class,
    TemplateLiteral,
}

#[derive(Debug, Clone)]
pub enum CallType {
    New,
    CallExpression,
    // the callee is the yielded value, arguments are always empty
    // spreads are always empty
    Yield,
    // spreads are always empty
    TaggedTemplate,
    // spreads are always empty
    Import,
}

#[derive(Debug)]
pub struct BasicBlock {
    pub instructions: Vec<Instruction>,
}

impl BasicBlock {
    fn new() -> Self {
        BasicBlock { instructions: Vec::new() }
    }

    pub fn instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub kind: InstructionKind,
    pub node_id: Option<AstNodeId>,
}

impl Instruction {
    pub fn new(kind: InstructionKind, node_id: Option<AstNodeId>) -> Self {
        Self { kind, node_id }
    }
}

#[derive(Debug, Clone)]
pub enum InstructionKind {
    Unreachable,
    Statement,
    Return(ReturnInstructionKind),
    Break(LabeledInstruction),
    Continue(LabeledInstruction),
    Throw,
}

#[derive(Debug, Clone)]
pub enum ReturnInstructionKind {
    ImplicitUndefined,
    NotImplicitUndefined,
}

#[derive(Debug, Clone)]
pub enum LabeledInstruction {
    Labeled,
    Unlabeled,
}

#[derive(Debug, Clone)]
pub enum EdgeType {
    Jump,
    Normal,
    Backedge,
    NewFunction,
    Unreachable,
}

#[derive(Debug)]
pub struct ControlFlowGraph {
    pub graph: Graph<usize, EdgeType>,
    pub basic_blocks: Vec<BasicBlock>,
}

impl ControlFlowGraph {
    /// # Panics
    pub fn basic_block(&self, id: BasicBlockId) -> &BasicBlock {
        let ix = *self.graph.node_weight(id).expect("expected a valid node id in self.graph");
        self.basic_blocks.get(ix).expect("expected a valid node id in self.basic_blocks")
    }

    /// # Panics
    pub fn basic_block_mut(&mut self, id: BasicBlockId) -> &mut BasicBlock {
        let ix = *self.graph.node_weight(id).expect("expected a valid node id in self.graph");
        self.basic_blocks.get_mut(ix).expect("expected a valid node id in self.basic_blocks")
    }

    pub fn is_reachabale(&self, from: BasicBlockId, to: BasicBlockId) -> bool {
        if from == to {
            return true;
        }
        let graph = &self.graph;
        depth_first_search(&self.graph, Some(from), |event| match event {
            DfsEvent::TreeEdge(a, b) => {
                let unreachable = graph.edges_connecting(a, b).all(|edge| {
                    matches!(edge.weight(), EdgeType::NewFunction | EdgeType::Unreachable)
                });

                if unreachable {
                    Control::Prune
                } else if b == to {
                    return Control::Break(true);
                } else {
                    Control::Continue
                }
            }
            _ => Control::Continue,
        })
        .break_value()
        .unwrap_or(false)
    }

    pub fn is_cyclic(&self, node: BasicBlockId) -> bool {
        depth_first_search(&self.graph, Some(node), |event| match event {
            DfsEvent::BackEdge(_, id) if id == node => Err(()),
            _ => Ok(()),
        })
        .is_err()
    }

    pub fn has_conditional_path(&self, from: BasicBlockId, to: BasicBlockId) -> bool {
        let graph = &self.graph;
        // All nodes should be able to reach the `to` node, Otherwise we have a conditional/branching flow.
        petgraph::algo::dijkstra(graph, from, Some(to), |e| match e.weight() {
            EdgeType::NewFunction => 1,
            EdgeType::Jump | EdgeType::Unreachable | EdgeType::Backedge | EdgeType::Normal => 0,
        })
        .into_iter()
        .filter(|(_, val)| *val == 0)
        .any(|(f, _)| !self.is_reachabale(f, to))
    }
}

pub struct PreservedExpressionState {
    pub use_this_register: Option<Register>,
    pub store_final_assignments_into_this_array: Vec<Vec<Register>>,
}

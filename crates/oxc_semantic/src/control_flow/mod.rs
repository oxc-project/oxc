mod builder;
mod dot;

use itertools::Itertools;
use oxc_ast::AstKind;
use oxc_span::CompactStr;
use oxc_syntax::operator::{
    AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator, UpdateOperator,
};
use petgraph::{
    stable_graph::NodeIndex,
    visit::{depth_first_search, Control, DfsEvent, EdgeRef},
    Direction, Graph,
};

use crate::{AstNodeId, AstNodes};

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
    pub unreachable: bool,
}

impl BasicBlock {
    fn new() -> Self {
        BasicBlock { instructions: Vec::new(), unreachable: false }
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
    Condition,
    Iteration(IterationInstructionKind),
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
pub enum IterationInstructionKind {
    Of,
    In,
}

#[derive(Debug, Clone)]
pub enum EdgeType {
    /// Conditional jumps
    Jump,
    /// Normal control flow path
    Normal,
    /// Cyclic aka loops
    Backedge,
    /// Marks start of a function subgraph
    NewFunction,
    /// Finally
    Finalize,
    /// Error Path
    Error(ErrorEdgeKind),

    // misc edges
    Unreachable,
    /// Used to mark the end of a finalizer. It is an experimental approach might
    /// move to it's respective edge kind enum or get removed altogether.
    Join,
}

#[derive(Default, Debug, Clone, Copy)]
pub enum ErrorEdgeKind {
    /// Error kind for edges between a block which can throw, to it's respective catch block.
    Explicit,
    /// Any block that can throw would have an implicit error block connected using this kind.
    #[default]
    Implicit,
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

    pub fn is_reachable(&self, from: BasicBlockId, to: BasicBlockId) -> bool {
        self.is_reachable_filtered(from, to, |_| Control::Continue)
    }

    pub fn is_reachable_filtered<F: Fn(BasicBlockId) -> Control<bool>>(
        &self,
        from: BasicBlockId,
        to: BasicBlockId,
        filter: F,
    ) -> bool {
        if from == to {
            return true;
        }
        let graph = &self.graph;
        depth_first_search(&self.graph, Some(from), |event| match event {
            DfsEvent::TreeEdge(a, b) => {
                let filter_result = filter(a);
                if !matches!(filter_result, Control::Continue) {
                    return filter_result;
                }
                let unreachable = !graph.edges_connecting(a, b).any(|edge| {
                    !matches!(edge.weight(), EdgeType::NewFunction | EdgeType::Unreachable)
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

    /// Returns `None` the given node isn't the cyclic point of an infinite loop.
    /// Otherwise returns `Some(loop_start, loop_end)`.
    pub fn is_infinite_loop_start(
        &self,
        node: BasicBlockId,
        nodes: &AstNodes,
    ) -> Option<(BasicBlockId, BasicBlockId)> {
        enum EvalConstConditionResult {
            NotFound,
            Fail,
            Eval(bool),
        }
        fn try_eval_const_condition(
            instruction: &Instruction,
            nodes: &AstNodes,
        ) -> EvalConstConditionResult {
            use EvalConstConditionResult::{Eval, Fail, NotFound};
            match instruction {
                Instruction { kind: InstructionKind::Condition, node_id: Some(id) } => {
                    match nodes.kind(*id) {
                        AstKind::BooleanLiteral(lit) => Eval(lit.value),
                        _ => Fail,
                    }
                }
                _ => NotFound,
            }
        }

        fn get_jump_target(
            graph: &Graph<usize, EdgeType>,
            node: BasicBlockId,
        ) -> Option<BasicBlockId> {
            graph
                .edges_directed(node, Direction::Outgoing)
                .find_or_first(|e| matches!(e.weight(), EdgeType::Jump))
                .map(|it| it.target())
        }

        let basic_block = self.basic_block(node);
        let mut backedges = self
            .graph
            .edges_directed(node, Direction::Incoming)
            .filter(|e| matches!(e.weight(), EdgeType::Backedge));

        // if this node doesn't have an backedge it isn't a loop starting point.
        let backedge = backedges.next()?;

        debug_assert!(
            backedges.next().is_none(),
            "there should only be one backedge to each basic block."
        );

        // if instructions are empty we might be in a `for(;;)`.
        if basic_block.instructions().is_empty()
            && !self
                .graph
                .edges_directed(node, Direction::Outgoing)
                .any(|e| matches!(e.weight(), EdgeType::Backedge))
        {
            return get_jump_target(&self.graph, node).map(|it| (it, node));
        }

        // if there are more than one instruction in this block it can't be a valid loop start.
        let Ok(only_instruction) = basic_block.instructions().iter().exactly_one() else {
            return None;
        };

        // if there is exactly one and it is a condition instruction we are in a loop so we
        // check the condition to infer if it is always true.
        if let EvalConstConditionResult::Eval(true) =
            try_eval_const_condition(only_instruction, nodes)
        {
            get_jump_target(&self.graph, node).map(|it| (it, node))
        } else if let EvalConstConditionResult::Eval(true) =
            self.basic_block(backedge.source()).instructions().iter().exactly_one().map_or_else(
                |_| EvalConstConditionResult::NotFound,
                |it| try_eval_const_condition(it, nodes),
            )
        {
            get_jump_target(&self.graph, node).map(|it| (node, it))
        } else {
            None
        }
    }

    pub fn is_cyclic(&self, node: BasicBlockId) -> bool {
        depth_first_search(&self.graph, Some(node), |event| match event {
            DfsEvent::BackEdge(_, id) if id == node => Err(()),
            _ => Ok(()),
        })
        .is_err()
    }
}

mod context;

use context::Ctx;
pub use context::{CtxCursor, CtxFlags};
use oxc_index::IndexVec;
use oxc_syntax::node::NodeId;
use petgraph::Direction;

use super::{
    BasicBlock, BlockNodeId, ControlFlowGraph, EdgeType, ErrorEdgeKind, Graph, Instruction,
    InstructionKind, IterationInstructionKind, LabeledInstruction,
};
use crate::{BasicBlockId, ReturnInstructionKind};

#[derive(Debug, Default)]
struct ErrorHarness(ErrorEdgeKind, BlockNodeId);

#[derive(Debug, Default)]
pub struct ControlFlowGraphBuilder<'a> {
    pub graph: Graph,
    pub basic_blocks: IndexVec<BasicBlockId, BasicBlock>,
    pub current_node_ix: BlockNodeId,
    ctx_stack: Vec<Ctx<'a>>,
    /// Contains the error unwinding path represented as a stack of `ErrorHarness`es
    error_path: Vec<ErrorHarness>,
    /// Stack of finalizers, the top most element is always the appropriate one for current node.
    finalizers: Vec<Option<BlockNodeId>>,
}

impl<'a> ControlFlowGraphBuilder<'a> {
    pub fn build(self) -> ControlFlowGraph {
        ControlFlowGraph { graph: self.graph, basic_blocks: self.basic_blocks }
    }

    pub fn current_basic_block(&mut self) -> &mut BasicBlock {
        self.basic_block_mut(self.current_node_ix)
    }

    /// # Panics
    pub fn basic_block(&self, basic_block: BlockNodeId) -> &BasicBlock {
        let idx = *self
            .graph
            .node_weight(basic_block)
            .expect("expected `self.current_node_ix` to be a valid node index in self.graph");
        self.basic_blocks
            .get(idx)
            .expect("expected `self.current_node_ix` to be a valid node index in self.graph")
    }

    /// # Panics
    pub fn basic_block_mut(&mut self, basic_block: BlockNodeId) -> &mut BasicBlock {
        let idx = *self
            .graph
            .node_weight(basic_block)
            .expect("expected `self.current_node_ix` to be a valid node index in self.graph");
        self.basic_blocks
            .get_mut(idx)
            .expect("expected `self.current_node_ix` to be a valid node index in self.graph")
    }

    pub(self) fn new_basic_block(&mut self) -> BlockNodeId {
        // current length would be the index of block we are adding on the next line.
        let basic_block_ix = self.basic_blocks.push(BasicBlock::new());
        self.graph.add_node(basic_block_ix)
    }

    #[must_use]
    pub fn new_basic_block_function(&mut self) -> BlockNodeId {
        // we might want to differentiate between function blocks and normal blocks down the road.
        self.new_basic_block_normal()
    }

    /// # Panics
    /// if there is no error harness to attach to.
    #[must_use]
    pub fn new_basic_block_normal(&mut self) -> BlockNodeId {
        let graph_ix = self.new_basic_block();
        self.current_node_ix = graph_ix;

        // add an error edge to this block.
        let ErrorHarness(error_edge_kind, error_graph_ix) =
            self.error_path.last().expect("normal basic blocks need an error harness to attach to");
        self.add_edge(graph_ix, *error_graph_ix, EdgeType::Error(*error_edge_kind));

        if let Some(Some(finalizer)) = self.finalizers.last() {
            self.add_edge(graph_ix, *finalizer, EdgeType::Finalize);
        }

        graph_ix
    }

    pub fn add_edge(&mut self, a: BlockNodeId, b: BlockNodeId, weight: EdgeType) {
        if matches!(weight, EdgeType::NewFunction) {
            self.basic_block_mut(b).mark_as_reachable();
        } else if matches!(weight, EdgeType::Unreachable) || self.basic_block(a).is_unreachable() {
            if self.graph.edges_directed(b, Direction::Incoming).count() == 0 {
                self.basic_block_mut(b).mark_as_unreachable();
            }
        } else if !self
            .basic_block(b)
            .instructions()
            .iter()
            .any(|it| matches!(it, Instruction { kind: InstructionKind::Unreachable, .. }))
        {
            self.basic_block_mut(b).mark_as_reachable();
        }
        self.graph.add_edge(a, b, weight);
    }

    pub fn push_statement(&mut self, stmt: NodeId) {
        self.push_instruction(InstructionKind::Statement, Some(stmt));
    }

    pub fn push_return(&mut self, kind: ReturnInstructionKind, node: NodeId) {
        self.push_instruction(InstructionKind::Return(kind), Some(node));
    }

    /// Creates and push a new `BasicBlockId` onto `self.error_path` stack.
    /// Returns the `BasicBlockId` of the created error harness block.
    pub fn attach_error_harness(&mut self, kind: ErrorEdgeKind) -> BlockNodeId {
        let graph_ix = self.new_basic_block();
        self.error_path.push(ErrorHarness(kind, graph_ix));
        graph_ix
    }

    /// # Panics
    /// if there is no error harness pushed onto the stack,
    /// Or last harness doesn't match the expected `BasicBlockId`.
    pub fn release_error_harness(&mut self, expect: BlockNodeId) {
        let harness = self
            .error_path
            .pop()
            .expect("there is no error harness in the `self.error_path` stack");
        assert_eq!(
            harness.1, expect,
            "expected harness doesn't match the last harness pushed onto the stack."
        );
    }

    /// Creates and push a new `BasicBlockId` onto `self.finalizers` stack.
    /// Returns the `BasicBlockId` of the created finalizer block.
    pub fn attach_finalizer(&mut self) -> BlockNodeId {
        let graph_ix = self.new_basic_block();
        self.finalizers.push(Some(graph_ix));
        graph_ix
    }

    pub fn push_finalization_stack(&mut self) {
        self.finalizers.push(None);
    }

    pub fn pop_finalization_stack(&mut self) {
        let result = self.finalizers.pop();
        debug_assert!(result.as_ref().is_some_and(Option::is_none));
    }

    /// # Panics
    /// if last finalizer doesn't match the expected `BasicBlockId`.
    pub fn release_finalizer(&mut self, expect: BlockNodeId) {
        // return early if there is no finalizer.
        let Some(finalizer) = self.finalizers.pop() else { return };
        assert_eq!(
            finalizer,
            Some(expect),
            "expected finalizer doesn't match the last finalizer pushed onto the stack."
        );
    }

    pub fn append_condition_to(&mut self, block: BlockNodeId, node: Option<NodeId>) {
        self.push_instruction_to(block, InstructionKind::Condition, node);
    }

    pub fn append_iteration(&mut self, node: Option<NodeId>, kind: IterationInstructionKind) {
        self.push_instruction(InstructionKind::Iteration(kind), node);
    }

    pub fn append_throw(&mut self, node: NodeId) {
        self.push_instruction(InstructionKind::Throw, Some(node));
        self.append_unreachable();
    }

    pub fn append_break(&mut self, node: NodeId, label: Option<&'a str>) {
        let kind = match label {
            Some(_) => LabeledInstruction::Labeled,
            None => LabeledInstruction::Unlabeled,
        };

        let bb = self.current_node_ix;

        self.push_instruction(InstructionKind::Break(kind), Some(node));
        self.append_unreachable();

        self.ctx(label).r#break(bb);
    }

    pub fn append_continue(&mut self, node: NodeId, label: Option<&'a str>) {
        let kind = match label {
            Some(_) => LabeledInstruction::Labeled,
            None => LabeledInstruction::Unlabeled,
        };

        let bb = self.current_node_ix;

        self.push_instruction(InstructionKind::Continue(kind), Some(node));
        self.append_unreachable();

        self.ctx(label).r#continue(bb);
    }

    pub fn append_unreachable(&mut self) {
        let current_node_ix = self.current_node_ix;
        let basic_block_with_unreachable_graph_ix = self.new_basic_block_normal();
        self.push_instruction(InstructionKind::Unreachable, None);
        self.current_basic_block().mark_as_unreachable();
        self.add_edge(
            current_node_ix,
            basic_block_with_unreachable_graph_ix,
            EdgeType::Unreachable,
        );
    }

    /// # Panics
    #[inline]
    pub(self) fn push_instruction(&mut self, kind: InstructionKind, node_id: Option<NodeId>) {
        self.push_instruction_to(self.current_node_ix, kind, node_id);
    }

    #[inline]
    pub(self) fn push_instruction_to(
        &mut self,
        block: BlockNodeId,
        kind: InstructionKind,
        node_id: Option<NodeId>,
    ) {
        self.basic_block_mut(block).instructions.push(Instruction { kind, node_id });
    }

    pub fn enter_statement(&mut self, stmt: NodeId) {
        self.push_statement(stmt);
    }
}

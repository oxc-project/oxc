mod context;

use crate::ReturnInstructionKind;
use context::Ctx;

pub use context::{CtxCursor, CtxFlags};

use super::{
    AstNodeId, BasicBlock, BasicBlockId, CompactStr, ControlFlowGraph, EdgeType, ErrorEdgeKind,
    Graph, Instruction, InstructionKind, LabeledInstruction, PreservedExpressionState, Register,
};

#[derive(Debug, Default)]
struct ErrorHarness(ErrorEdgeKind, BasicBlockId);

#[derive(Debug, Default)]
pub struct ControlFlowGraphBuilder<'a> {
    pub graph: Graph<usize, EdgeType>,
    pub basic_blocks: Vec<BasicBlock>,
    pub current_node_ix: BasicBlockId,
    ctx_stack: Vec<Ctx<'a>>,
    /// Contains the error unwinding path represented as a stack of `ErrorHarness`es
    error_path: Vec<ErrorHarness>,
    /// Stack of finalizers, the top most element is always the appropiate one for current node.
    finalizers: Vec<BasicBlockId>,
    // note: this should only land in the big box for all things that take arguments
    // ie: callexpression, arrayexpression, etc
    // todo: add assert that it is used every time?
    pub use_this_register: Option<Register>,
    pub next_free_register: u32,
    pub store_assignments_into_this_array: Vec<Vec<Register>>,
    pub store_final_assignments_into_this_array: Vec<Vec<Register>>,
    // indexes of spreads in the store_assignments_into_this_array
    pub spread_indices: Vec<Vec<usize>>,
    // computed member expressions are only executed when we reach
    // that part of the chain, so we keep this vec to patch them in later
    pub should_save_stores_for_patching: bool,
    pub saved_store: Option<usize>,
    pub basic_blocks_with_breaks: Vec<Vec<BasicBlockId>>,
    pub basic_blocks_with_continues: Vec<Vec<BasicBlockId>>,
    // node indexes of the basic blocks of switch case conditions
    pub switch_case_conditions: Vec<Vec<BasicBlockId>>,
    pub next_label: Option<CompactStr>,
    pub label_to_ast_node_ix: Vec<(CompactStr, AstNodeId)>,
    pub ast_node_to_break_continue: Vec<(AstNodeId, usize, Option<usize>)>,
    pub after_throw_block: Option<BasicBlockId>,
}

impl<'a> ControlFlowGraphBuilder<'a> {
    pub fn build(self) -> ControlFlowGraph {
        ControlFlowGraph { graph: self.graph, basic_blocks: self.basic_blocks }
    }

    /// # Panics
    pub fn current_basic_block(&mut self) -> &mut BasicBlock {
        let idx = *self
            .graph
            .node_weight(self.current_node_ix)
            .expect("expected `self.current_node_ix` to be a valid node index in self.graph");
        self.basic_blocks
            .get_mut(idx)
            .expect("expected `self.current_node_ix` to be a valid node index in self.graph")
    }

    pub(self) fn new_basic_block(&mut self) -> BasicBlockId {
        // current length would be the index of block we are adding on the next line.
        let basic_block_ix = self.basic_blocks.len();
        self.basic_blocks.push(BasicBlock::new());
        self.graph.add_node(basic_block_ix)
    }

    #[must_use]
    pub fn new_basic_block_function(&mut self) -> BasicBlockId {
        // we might want to differentiate between function blocks and normal blocks down the road.
        self.new_basic_block_normal()
    }

    /// # Panics if there is no error harness to attach to.
    #[must_use]
    pub fn new_basic_block_normal(&mut self) -> BasicBlockId {
        let graph_ix = self.new_basic_block();
        self.current_node_ix = graph_ix;

        // add an error edge to this block.
        let ErrorHarness(error_edge_kind, error_graph_ix) =
            self.error_path.last().expect("normal basic blocks need an error harness to attach to");
        self.add_edge(graph_ix, *error_graph_ix, EdgeType::Error(*error_edge_kind));

        if let Some(finalizer) = self.finalizers.last() {
            self.add_edge(graph_ix, *finalizer, EdgeType::Finalize);
        }

        graph_ix
    }

    pub fn add_edge(&mut self, a: BasicBlockId, b: BasicBlockId, weight: EdgeType) {
        self.graph.add_edge(a, b, weight);
    }

    pub fn push_statement(&mut self, stmt: AstNodeId) {
        self.push_instruction(InstructionKind::Statement, Some(stmt));
    }

    pub fn push_return(&mut self, kind: ReturnInstructionKind, node: AstNodeId) {
        self.push_instruction(InstructionKind::Return(kind), Some(node));
    }

    /// Creates and push a new `BasicBlockId` onto `self.error_path` stack.
    /// Returns the `BasicBlockId` of the created error harness block.
    pub fn attach_error_harness(&mut self, kind: ErrorEdgeKind) -> BasicBlockId {
        let graph_ix = self.new_basic_block();
        self.error_path.push(ErrorHarness(kind, graph_ix));
        graph_ix
    }

    /// # Panics if there is no error harness pushed onto the stack,
    /// Or last harness doesn't match the expected `BasicBlockId`.
    pub fn release_error_harness(&mut self, expect: BasicBlockId) {
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
    pub fn attach_finalizer(&mut self) -> BasicBlockId {
        let graph_ix = self.new_basic_block();
        self.finalizers.push(graph_ix);
        graph_ix
    }

    /// # Panics if last finalizer doesn't match the expected `BasicBlockId`.
    pub fn release_finalizer(&mut self, expect: BasicBlockId) {
        // return early if there is no finalizer.
        let Some(finalizer) = self.finalizers.pop() else { return };
        assert_eq!(
            finalizer, expect,
            "expected finalizer doesn't match the last finalizer pushed onto the stack."
        );
    }

    pub fn append_throw(&mut self, node: AstNodeId) {
        self.push_instruction(InstructionKind::Throw, Some(node));
        self.append_unreachable();
    }

    pub fn append_break(&mut self, node: AstNodeId, label: Option<&'a str>) {
        let kind = match label {
            Some(_) => LabeledInstruction::Labeled,
            None => LabeledInstruction::Unlabeled,
        };

        let bb = self.current_node_ix;

        self.push_instruction(InstructionKind::Break(kind), Some(node));
        self.append_unreachable();

        self.ctx(label).r#break(bb);
    }

    pub fn append_continue(&mut self, node: AstNodeId, label: Option<&'a str>) {
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
        self.add_edge(
            current_node_ix,
            basic_block_with_unreachable_graph_ix,
            EdgeType::Unreachable,
        );
        self.push_instruction(InstructionKind::Unreachable, None);
    }

    #[inline]
    pub(self) fn push_instruction(&mut self, kind: InstructionKind, node_id: Option<AstNodeId>) {
        self.current_basic_block().instructions.push(Instruction { kind, node_id });
    }

    #[must_use]
    pub fn preserve_expression_state(&mut self) -> PreservedExpressionState {
        let use_this_register = self.use_this_register.take();
        let mut store_final_assignments_into_this_array = vec![];
        std::mem::swap(
            &mut store_final_assignments_into_this_array,
            &mut self.store_final_assignments_into_this_array,
        );

        // DO NOT preserve: saved_stores, should_save_stores_for_patching
        // should_save_stores_for_patching must always be active to catch
        // all stores, preserving will mess it up.
        PreservedExpressionState { use_this_register, store_final_assignments_into_this_array }
    }

    pub fn restore_expression_state(&mut self, mut preserved_state: PreservedExpressionState) {
        self.use_this_register = preserved_state.use_this_register.take();
        self.store_final_assignments_into_this_array =
            preserved_state.store_final_assignments_into_this_array;
    }

    pub fn enter_statement(&mut self, stmt: AstNodeId) {
        self.push_statement(stmt);
    }
}

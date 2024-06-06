mod context;

use crate::ReturnInstructionKind;
use context::Ctx;

pub use context::{CtxCursor, CtxFlags};

use super::{
    AstNodeId, BasicBlock, BasicBlockId, CompactStr, ControlFlowGraph, EdgeType, Graph,
    Instruction, InstructionKind, LabeledInstruction, PreservedExpressionState, Register,
};

#[derive(Debug, Default)]
pub struct ControlFlowGraphBuilder<'a> {
    pub graph: Graph<usize, EdgeType>,
    pub basic_blocks: Vec<BasicBlock>,
    pub current_node_ix: BasicBlockId,
    ctx_stack: Vec<Ctx<'a>>,
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

    #[must_use]
    pub fn new_basic_block_for_function(&mut self) -> BasicBlockId {
        self.basic_blocks.push(BasicBlock::new());
        let basic_block_id = self.basic_blocks.len() - 1;
        let graph_index = self.graph.add_node(basic_block_id);
        self.current_node_ix = graph_index;

        // todo: get smarter about what can throw, ie: return can't throw but it's expression can
        if let Some(after_throw_block) = self.after_throw_block {
            self.add_edge(graph_index, after_throw_block, EdgeType::NewFunction);
        }

        graph_index
    }

    #[must_use]
    pub fn new_basic_block(&mut self) -> BasicBlockId {
        self.basic_blocks.push(BasicBlock::new());
        let graph_index = self.graph.add_node(self.basic_blocks.len() - 1);
        self.current_node_ix = graph_index;

        // todo: get smarter about what can throw, ie: return can't throw but it's expression can
        if let Some(after_throw_block) = self.after_throw_block {
            self.add_edge(graph_index, after_throw_block, EdgeType::Normal);
        }

        graph_index
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

    pub fn push_throw(&mut self, node: AstNodeId) {
        self.push_instruction(InstructionKind::Throw, Some(node));
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
        let basic_block_with_unreachable_graph_ix = self.new_basic_block();
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

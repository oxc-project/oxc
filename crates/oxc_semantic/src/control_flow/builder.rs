use super::{
    AssignmentValue, AstNodeId, BasicBlockElement, BasicBlockId, CompactStr, ControlFlowGraph,
    EdgeType, Graph, PreservedExpressionState, PreservedStatementState, Register,
    StatementControlFlowType,
};

#[derive(Debug, Default)]
pub struct ControlFlowGraphBuilder {
    pub graph: Graph<usize, EdgeType>,
    pub basic_blocks: Vec<Vec<BasicBlockElement>>,
    pub current_node_ix: BasicBlockId,
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
    // (start, tail, last_register_used)
    pub saved_stores: Vec<(Vec<BasicBlockElement>, Option<Register>)>,
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

impl ControlFlowGraphBuilder {
    pub fn build(self) -> ControlFlowGraph {
        ControlFlowGraph { graph: self.graph, basic_blocks: self.basic_blocks }
    }

    /// # Panics
    pub fn current_basic_block(&mut self) -> &mut Vec<BasicBlockElement> {
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
        self.basic_blocks.push(Vec::new());
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
        self.basic_blocks.push(Vec::new());
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

    #[must_use]
    pub fn new_register(&mut self) -> Register {
        let register = Register::Index(self.next_free_register);
        self.next_free_register += 1;
        register
    }

    /// # Panics
    pub fn put_x_in_register(&mut self, asmt_value: AssignmentValue) {
        let register = self.use_this_register.take().unwrap_or_else(|| self.new_register());

        let basic_block_element = BasicBlockElement::Assignment(register, asmt_value);

        if self.should_save_stores_for_patching {
            let saved_store = &mut self.saved_stores[self.saved_store.expect(
                "there must be at least one saved store if should_save_stores_for_patching is true",
            )];

            saved_store.0.push(basic_block_element);
            saved_store.1 = Some(register);
        } else {
            self.current_basic_block().push(basic_block_element);
        }
        // used for storing the object base for MemberExpressions
        if let Some(arr) = self.store_assignments_into_this_array.last_mut() {
            arr.push(register);
        }

        if let Some(arr) = self.store_final_assignments_into_this_array.last_mut() {
            arr.push(register);
        }
    }

    pub fn put_throw(&mut self, throw_expr: Register) {
        self.current_basic_block().push(BasicBlockElement::Throw(throw_expr));
    }

    pub fn put_break(&mut self, label: Option<Register>) {
        self.current_basic_block().push(BasicBlockElement::Break(label));
    }

    pub fn put_unreachable(&mut self) {
        let current_node_ix = self.current_node_ix;
        let basic_block_with_unreachable_graph_ix = self.new_basic_block();
        self.add_edge(current_node_ix, basic_block_with_unreachable_graph_ix, EdgeType::Normal);
        self.current_basic_block().push(BasicBlockElement::Unreachable);
    }

    pub fn put_undefined(&mut self) {
        self.put_x_in_register(AssignmentValue::ImplicitUndefined);
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

    // note: could use type specialization rather than an enum
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn before_statement(
        &mut self,
        id: AstNodeId,
        control_flow_type: StatementControlFlowType,
    ) -> PreservedStatementState {
        let mut pss = PreservedStatementState { put_label: false };

        match control_flow_type {
            StatementControlFlowType::DoesNotUseContinue => {
                self.basic_blocks_with_breaks.push(vec![]);
                if let Some(next_label) = &self.next_label.take() {
                    self.label_to_ast_node_ix.push((next_label.clone(), id));
                    pss.put_label = true;
                    self.ast_node_to_break_continue.push((
                        id,
                        self.basic_blocks_with_breaks.len() - 1,
                        None,
                    ));
                }
            }
            StatementControlFlowType::UsesContinue => {
                self.basic_blocks_with_breaks.push(vec![]);
                self.basic_blocks_with_continues.push(vec![]);
                if let Some(next_label) = &self.next_label.take() {
                    self.label_to_ast_node_ix.push((next_label.clone(), id));
                    pss.put_label = true;
                    self.ast_node_to_break_continue.push((
                        id,
                        self.basic_blocks_with_breaks.len() - 1,
                        None,
                    ));
                }
            }
        }

        pss
    }

    /// # Panics
    pub fn after_statement(
        &mut self,
        preserved_state: &PreservedStatementState,
        id: AstNodeId,
        break_jump_position: BasicBlockId,
        continue_jump_position: Option<BasicBlockId>,
    ) {
        let basic_blocks_with_breaks = self
            .basic_blocks_with_breaks
            .pop()
            .expect("expected there to be a breaks array for this statement");

        for break_ in basic_blocks_with_breaks {
            // can this always be self.current_node_ix?
            self.add_edge(break_, break_jump_position, EdgeType::Normal);
        }

        if let Some(continue_jump_position) = continue_jump_position {
            let basic_blocks_with_continues = self.basic_blocks_with_continues.pop().expect(
                "expect there to be a basic block with continue directive for this statement",
            );

            for continue_ in basic_blocks_with_continues {
                self.add_edge(continue_, continue_jump_position, EdgeType::Normal);
            }
        }

        if preserved_state.put_label {
            let popped = self.label_to_ast_node_ix.pop();
            let popped_2 = self.ast_node_to_break_continue.pop();
            debug_assert_eq!(popped.unwrap().1, id);
            debug_assert_eq!(popped_2.unwrap().0, id);
        }
    }
}

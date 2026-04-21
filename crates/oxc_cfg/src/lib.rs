mod block;
mod builder;
pub mod dot;
pub mod visit;

use std::fmt;

use itertools::Itertools;
use oxc_index::{IndexVec, define_nonmax_u32_index_type};
use petgraph::{
    Direction,
    visit::{Control, DfsEvent, EdgeRef},
};

pub mod graph {
    pub use petgraph::*;
    pub mod visit {
        pub use petgraph::visit::*;

        pub use super::super::visit::*;
    }
}

pub use block::*;
pub use builder::{ControlFlowGraphBuilder, CtxCursor, CtxFlags};
pub use dot::DisplayDot;
use visit::set_depth_first_search;

pub type BlockNodeId = petgraph::stable_graph::NodeIndex;

define_nonmax_u32_index_type! {
    pub struct BasicBlockId;
}

impl PartialEq<u32> for BasicBlockId {
    #[inline]
    fn eq(&self, other: &u32) -> bool {
        self.0.get() == *other
    }
}

impl fmt::Display for BasicBlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub type Graph = petgraph::graph::DiGraph<BasicBlockId, EdgeType>;

/// Represents the different types of edges in the control flow graph.
///
/// Edges connect basic blocks and represent the possible paths of execution
/// through a program. Each edge type describes a specific kind of control flow
/// transition between blocks.
#[derive(Debug, Clone)]
pub enum EdgeType {
    /// Represents a conditional branch taken when a condition evaluates to a specific value.
    ///
    /// This edge connects the block containing a conditional statement (e.g., `if`, `while`, `for`)
    /// to the block that executes when the condition is satisfied. Jump edges are typically
    /// paired with Normal edges to represent the two possible outcomes of a conditional branch.
    ///
    /// # Example
    /// ```js
    /// if (x > 0) {  // Jump edge to consequent when true
    ///     foo();
    /// }
    /// ```
    Jump,

    /// Represents sequential control flow that follows the natural execution order.
    ///
    /// This is the default edge type for straightforward control flow transitions, such as
    /// falling through from one statement to the next, or the alternative path in a conditional
    /// branch. Normal edges represent the "else" path or continuation after a conditional.
    ///
    /// # Example
    /// ```js
    /// statement1;  // Normal edge to next statement
    /// statement2;
    /// ```
    Normal,

    /// Represents a backward edge that creates a cycle in the control flow graph.
    ///
    /// Backedges point from the end of a loop body back to the loop's entry point (the loop
    /// condition or loop header). These edges are essential for identifying loops and analyzing
    /// cyclic control flow patterns. Each loop header should have exactly one backedge pointing
    /// to it.
    ///
    /// # Example
    /// ```js
    /// while (condition) {  // Backedge from end of body back to condition
    ///     body;
    /// }
    /// ```
    Backedge,

    /// Marks the entry into a nested function's control flow subgraph.
    ///
    /// This edge type separates the control flow of nested function declarations or expressions
    /// from the containing function's flow. NewFunction edges help maintain proper scope
    /// boundaries and prevent incorrect reachability analysis across function boundaries.
    /// These edges are typically filtered out during reachability checks.
    ///
    /// # Example
    /// ```js
    /// function outer() {
    ///     function inner() {  // NewFunction edge to inner's CFG
    ///         // inner's control flow
    ///     }
    /// }
    /// ```
    NewFunction,

    /// Represents control flow into a `finally` block.
    ///
    /// This edge connects blocks that may transfer control to a `finally` clause, regardless
    /// of whether execution is normal or exceptional. Finalize edges ensure that cleanup code
    /// in `finally` blocks is properly represented in the control flow graph, even when the
    /// try or catch blocks contain early returns or throws.
    ///
    /// # Example
    /// ```js
    /// try {
    ///     risky();
    /// } catch (e) {
    ///     handle(e);
    /// } finally {  // Finalize edges from try and catch blocks
    ///     cleanup();
    /// }
    /// ```
    Finalize,

    /// Represents control flow along an error/exception path.
    ///
    /// Error edges connect blocks that may throw exceptions to their corresponding error
    /// handlers (catch blocks or finally blocks). The `ErrorEdgeKind` distinguishes between
    /// explicit throws and implicit error paths from operations that may throw.
    ///
    /// # Example
    /// ```js
    /// try {
    ///     mayThrow();  // Error edge to catch block
    /// } catch (e) {
    ///     handle(e);
    /// }
    /// ```
    Error(ErrorEdgeKind),

    /// Represents a control flow path that can never be taken.
    ///
    /// Unreachable edges mark portions of the control flow graph that are statically determined
    /// to be impossible to execute. These edges are filtered out during reachability analysis
    /// to avoid false positives. Common sources include code after unconditional returns or
    /// in branches with constant false conditions.
    ///
    /// # Example
    /// ```js
    /// return;
    /// unreachableCode();  // Unreachable edge to this block
    /// ```
    Unreachable,

    /// Marks the convergence point after a finalizer completes.
    ///
    /// This edge type is experimental and represents the point where control flow reconverges
    /// after executing a `finally` block. It helps distinguish between different paths through
    /// finally blocks (normal completion vs. exceptional completion). This variant may be
    /// refactored into a more specific edge kind enum or removed in future versions.
    ///
    /// # Example
    /// ```js
    /// try { a(); } finally { b(); }  // Join edge after finally completes
    /// c();  // Execution continues here
    /// ```
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

pub enum EvalConstConditionResult {
    NotFound,
    Fail,
    Eval(bool),
}

#[derive(Debug)]
pub struct ControlFlowGraph {
    pub graph: Graph,
    pub basic_blocks: IndexVec<BasicBlockId, BasicBlock>,
}

impl ControlFlowGraph {
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// # Panics
    pub fn basic_block(&self, id: BlockNodeId) -> &BasicBlock {
        let ix = *self.graph.node_weight(id).expect("expected a valid node id in self.graph");
        self.basic_blocks.get(ix).expect("expected a valid node id in self.basic_blocks")
    }

    /// # Panics
    pub fn basic_block_mut(&mut self, id: BlockNodeId) -> &mut BasicBlock {
        let ix = *self.graph.node_weight(id).expect("expected a valid node id in self.graph");
        self.basic_blocks.get_mut(ix).expect("expected a valid node id in self.basic_blocks")
    }

    pub fn is_reachable(&self, from: BlockNodeId, to: BlockNodeId) -> bool {
        self.is_reachable_filtered(from, to, |_| Control::Continue)
    }

    pub fn is_reachable_filtered<F: Fn(BlockNodeId) -> Control<bool>>(
        &self,
        from: BlockNodeId,
        to: BlockNodeId,
        filter: F,
    ) -> bool {
        if from == to {
            return true;
        }
        let graph = &self.graph;
        set_depth_first_search(&self.graph, Some(from), |event| match event {
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
                    Control::Break(true)
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
    ///
    /// # Panics
    ///
    /// * There should only be one backedge to each basic block.
    pub fn is_infinite_loop_start<F>(
        &self,
        node: BlockNodeId,
        try_eval_const_condition: F,
    ) -> Option<(BlockNodeId, BlockNodeId)>
    where
        F: Fn(&Instruction) -> EvalConstConditionResult,
    {
        fn get_jump_target(graph: &Graph, node: BlockNodeId) -> Option<BlockNodeId> {
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

        assert!(
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
        if matches!(
            try_eval_const_condition(only_instruction),
            EvalConstConditionResult::Eval(true)
        ) {
            get_jump_target(&self.graph, node).map(|it| (it, node))
        } else if matches!(
            self.basic_block(backedge.source())
                .instructions()
                .iter()
                .exactly_one()
                .map_or_else(|_| EvalConstConditionResult::NotFound, try_eval_const_condition),
            EvalConstConditionResult::Eval(true)
        ) {
            get_jump_target(&self.graph, node).map(|it| (node, it))
        } else {
            None
        }
    }

    pub fn is_cyclic(&self, node: BlockNodeId) -> bool {
        set_depth_first_search(&self.graph, Some(node), |event| match event {
            DfsEvent::BackEdge(_, id) if id == node => Err(()),
            _ => Ok(()),
        })
        .is_err()
    }
}

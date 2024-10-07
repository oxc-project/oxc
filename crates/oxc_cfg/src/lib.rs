mod block;
mod builder;
pub mod dot;
pub mod visit;

use std::fmt;

use itertools::Itertools;
use nonmax::NonMaxU32;
use oxc_index::{Idx, IndexVec};
use petgraph::{
    visit::{Control, DfsEvent, EdgeRef},
    Direction,
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
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BasicBlockId(NonMaxU32);

impl Idx for BasicBlockId {
    #[allow(clippy::cast_possible_truncation)]
    fn from_usize(idx: usize) -> Self {
        assert!(idx < u32::MAX as usize);
        // SAFETY: We just checked `idx` is valid for `NonMaxU32`
        Self(unsafe { NonMaxU32::new_unchecked(idx as u32) })
    }

    fn index(self) -> usize {
        self.0.get() as usize
    }
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
        if let EvalConstConditionResult::Eval(true) = try_eval_const_condition(only_instruction) {
            get_jump_target(&self.graph, node).map(|it| (it, node))
        } else if let EvalConstConditionResult::Eval(true) = self
            .basic_block(backedge.source())
            .instructions()
            .iter()
            .exactly_one()
            .map_or_else(|_| EvalConstConditionResult::NotFound, try_eval_const_condition)
        {
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

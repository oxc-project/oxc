//! Flow graph data structures for control flow analysis.
//!
//! The flow graph is a linked-list structure built during a pre-pass over each
//! function body and consumed via backward walk from each variable reference.
//! This mirrors typescript-go's approach but uses Rust-optimized data structures:
//! `IndexVec` instead of pointer-based linked lists, `SmallVec` for antecedents.

use oxc_index::define_index_type;
use rustc_hash::FxHashMap;
use oxc_syntax::node::NodeId;
use oxc_syntax::symbol::SymbolId;
use smallvec::SmallVec;

define_index_type! {
    /// Index into the flow graph's node storage.
    pub struct FlowNodeId = u32;
}

/// Caching metadata for a flow node. Tracks whether this node has been used
/// as an antecedent by multiple successors, making it worth caching during
/// the backward walk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheState {
    /// Not yet used as an antecedent.
    None,
    /// Used as an antecedent once.
    Referenced,
    /// Used as an antecedent 2+ times — cache-worthy.
    Shared,
}

/// A flow graph entry: the node kind plus orthogonal caching metadata.
#[derive(Debug)]
pub struct FlowEntry {
    pub cache_state: CacheState,
    pub kind: FlowNodeKind,
}

/// The different kinds of flow graph nodes. Each variant carries exactly
/// the data it needs — no optional fields.
#[derive(Debug)]
pub enum FlowNodeKind {
    /// Function/program entry point. Backward walk returns declared type here.
    Start,
    /// Dead code sentinel. Backward walk returns `never` here.
    Unreachable,
    /// Variable assignment. Resets the narrowed type for `symbol_id`.
    Assignment {
        node_id: NodeId,
        symbol_id: SymbolId,
        antecedent: FlowNodeId,
    },
    /// True branch of a narrowing condition (e.g., the `if` body for `if (x)`).
    TrueCondition {
        node_id: NodeId,
        antecedent: FlowNodeId,
    },
    /// False branch of a narrowing condition (e.g., the `else` body for `if (x)`).
    FalseCondition {
        node_id: NodeId,
        antecedent: FlowNodeId,
    },
    /// Non-looping junction (if/else merge point, switch merge, try/catch merge).
    BranchLabel {
        antecedents: SmallVec<[FlowNodeId; 2]>,
    },
    /// Looping junction (while/for back-edge target).
    LoopLabel {
        antecedents: SmallVec<[FlowNodeId; 2]>,
    },
}

/// Per-function flow graph. Stores all flow nodes in a contiguous `IndexVec`
/// with index-based references instead of pointers.
pub struct FlowGraph {
    /// All flow nodes for this function.
    pub nodes: oxc_index::IndexVec<FlowNodeId, FlowEntry>,
    /// Maps AST NodeId (of identifiers, statements) to their flow node.
    /// This replaces tsgo's approach of storing flow info on AST nodes directly.
    pub node_flow_map: FxHashMap<NodeId, FlowNodeId>,
    /// The start node (entry point of the function).
    pub start: FlowNodeId,
    /// The unreachable sentinel node.
    pub unreachable: FlowNodeId,
    /// The flow node at the end of the function/program body.
    /// If this is `unreachable`, all paths return/throw before reaching the end.
    pub end_of_flow: FlowNodeId,
}

impl FlowGraph {
    /// Create an empty flow graph (no mappings, lookups return None).
    /// Used as the default/placeholder when no function scope is active.
    pub fn empty() -> Self {
        let mut nodes = oxc_index::IndexVec::new();
        let start = nodes.push(FlowEntry {
            cache_state: CacheState::None,
            kind: FlowNodeKind::Start,
        });
        let unreachable = nodes.push(FlowEntry {
            cache_state: CacheState::None,
            kind: FlowNodeKind::Unreachable,
        });
        Self {
            nodes,
            node_flow_map: FxHashMap::default(),
            start,
            unreachable,
            end_of_flow: start,
        }
    }

    /// Get a flow node by its ID.
    #[inline]
    pub fn get(&self, id: FlowNodeId) -> &FlowEntry {
        &self.nodes[id]
    }

    /// Look up the flow node for an AST node (identifier reference).
    #[inline]
    pub fn get_flow_for_node(&self, node_id: NodeId) -> Option<FlowNodeId> {
        self.node_flow_map.get(&node_id).copied()
    }

    /// Returns true if control flow can reach the end of the function/program
    /// without hitting a return or throw statement.
    #[inline]
    pub fn is_end_reachable(&self) -> bool {
        !matches!(self.nodes[self.end_of_flow].kind, FlowNodeKind::Unreachable)
    }
}

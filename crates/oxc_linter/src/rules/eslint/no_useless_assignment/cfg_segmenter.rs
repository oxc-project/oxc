//! CFG Segmenter - splits the global CFG into per-function segments
//!
//! oxc builds a single global CFG for the entire program with function boundaries
//! marked by EdgeType::NewFunction edges. This module segments that global CFG
//! into function-specific subgraphs that can be analyzed independently.

use rustc_hash::FxHashSet;

use oxc_cfg::{
    BlockNodeId, ControlFlowGraph, EdgeType,
    graph::{
        Direction,
        visit::{EdgeRef, IntoNodeIdentifiers},
    },
};
use oxc_semantic::{AstNodes, ScopeId, Scoping};
use oxc_syntax::node::NodeId;

/// Represents a segment of the CFG corresponding to a single function or top-level code
#[derive(Debug)]
pub struct CFGSegment {
    /// Entry block for this segment (where execution starts)
    #[expect(dead_code)]
    pub entry_block: BlockNodeId,
    /// Exit blocks for this segment (blocks with no normal successors)
    pub exit_blocks: Vec<BlockNodeId>,
    /// All blocks belonging to this segment
    pub blocks: FxHashSet<BlockNodeId>,
    /// The scope ID corresponding to this segment
    pub scope_id: ScopeId,
}

/// Segment the global CFG into per-function/per-scope subgraphs
///
/// Algorithm:
/// 1. Start from the CFG root (program entry)
/// 2. DFS/BFS through the graph, collecting blocks
/// 3. When encountering EdgeType::NewFunction, start a new segment
/// 4. Map each segment to its corresponding ScopeId by examining instructions
///
/// Returns: Vec of CFG segments, one for module-level code and one per function
pub fn segment_cfg(
    cfg: &ControlFlowGraph,
    scoping: &Scoping,
    nodes: &AstNodes<'_>,
) -> Vec<CFGSegment> {
    let mut segments = Vec::new();
    let mut visited = FxHashSet::default();

    // Start with all nodes in the graph (we'll process them into segments)
    let all_nodes: Vec<_> = cfg.graph().node_identifiers().collect();

    for start_node in all_nodes {
        if visited.contains(&start_node) {
            continue;
        }

        // Check if this is the start of a new segment (entry point or after NewFunction edge)
        let is_segment_start = is_segment_entry(cfg, start_node, &visited);

        if !is_segment_start {
            continue;
        }

        // Collect all blocks in this segment
        let segment_blocks = collect_segment_blocks(cfg, start_node, &mut visited);

        if segment_blocks.is_empty() {
            continue;
        }

        // Determine the scope for this segment
        let scope_id = determine_scope(&segment_blocks, cfg, nodes, scoping);

        // Find exit blocks (blocks with no normal successors within the segment)
        let exit_blocks = find_exit_blocks(cfg, &segment_blocks);

        segments.push(CFGSegment {
            entry_block: start_node,
            exit_blocks,
            blocks: segment_blocks,
            scope_id,
        });
    }

    segments
}

/// Check if a node is the entry point of a CFG segment
fn is_segment_entry(
    cfg: &ControlFlowGraph,
    node: BlockNodeId,
    visited: &FxHashSet<BlockNodeId>,
) -> bool {
    // If already visited, not an entry
    if visited.contains(&node) {
        return false;
    }

    // Check incoming edges
    let incoming_edges: Vec<_> = cfg.graph().edges_directed(node, Direction::Incoming).collect();

    // If no incoming edges, this is the program entry
    if incoming_edges.is_empty() {
        return true;
    }

    // If all incoming edges are NewFunction edges, this is a function entry

    incoming_edges.iter().all(|e| matches!(e.weight(), EdgeType::NewFunction))
}

/// Collect all blocks belonging to a segment starting from an entry point
///
/// Stops at NewFunction edges (which mark function boundaries)
fn collect_segment_blocks(
    cfg: &ControlFlowGraph,
    start: BlockNodeId,
    visited: &mut FxHashSet<BlockNodeId>,
) -> FxHashSet<BlockNodeId> {
    let mut segment_blocks = FxHashSet::default();
    let mut worklist = vec![start];

    while let Some(node) = worklist.pop() {
        if visited.contains(&node) {
            continue;
        }

        visited.insert(node);
        segment_blocks.insert(node);

        // Follow outgoing edges, but stop at NewFunction edges
        for edge in cfg.graph().edges_directed(node, Direction::Outgoing) {
            // Skip NewFunction and Unreachable edges
            if matches!(edge.weight(), EdgeType::NewFunction | EdgeType::Unreachable) {
                continue;
            }

            let target = edge.target();
            if !visited.contains(&target) {
                worklist.push(target);
            }
        }
    }

    segment_blocks
}

/// Determine the scope ID for a segment by examining its instructions
///
/// Strategy:
/// 1. Look at instructions in blocks to find NodeIds
/// 2. Map NodeIds to their scopes using the AST
/// 3. Find the most specific common scope (usually a function scope or module scope)
fn determine_scope(
    blocks: &FxHashSet<BlockNodeId>,
    cfg: &ControlFlowGraph,
    nodes: &AstNodes<'_>,
    scoping: &Scoping,
) -> ScopeId {
    // Collect all node IDs from instructions in this segment
    let mut node_ids = Vec::new();

    for &block_node_id in blocks {
        let basic_block = cfg.basic_block(block_node_id);
        for instruction in basic_block.instructions() {
            if let Some(node_id) = instruction.node_id {
                node_ids.push(node_id);
            }
        }
    }

    // If we have no instructions, default to root scope
    if node_ids.is_empty() {
        return scoping.root_scope_id();
    }

    // Find the scope of the first node (usually representative)
    let first_node_id = node_ids[0];
    find_scope_for_node(first_node_id, nodes, scoping)
}

/// Find the scope containing a given AST node
fn find_scope_for_node(node_id: NodeId, nodes: &AstNodes<'_>, scoping: &Scoping) -> ScopeId {
    use oxc_ast::AstKind;

    // Walk up the AST to find the nearest scope
    let mut current_id = node_id;

    loop {
        let node = nodes.get_node(current_id);

        // Check if this node defines a scope
        match node.kind() {
            AstKind::Program(_) => {
                return scoping.root_scope_id();
            }
            AstKind::Function(func) => {
                if let Some(scope_id) = func.scope_id.get() {
                    return scope_id;
                }
            }
            AstKind::ArrowFunctionExpression(arrow) => {
                if let Some(scope_id) = arrow.scope_id.get() {
                    return scope_id;
                }
            }
            AstKind::BlockStatement(block) => {
                if let Some(scope_id) = block.scope_id.get() {
                    return scope_id;
                }
            }
            AstKind::Class(class) => {
                if let Some(scope_id) = class.scope_id.get() {
                    return scope_id;
                }
            }
            _ => {}
        }

        // Move to parent
        let parent_id = nodes.parent_id(current_id);
        if parent_id == NodeId::DUMMY {
            // Reached root, return root scope
            return scoping.root_scope_id();
        }
        current_id = parent_id;
    }
}

/// Find all exit blocks in a segment
///
/// An exit block is one that has no normal successors within the segment
fn find_exit_blocks(cfg: &ControlFlowGraph, blocks: &FxHashSet<BlockNodeId>) -> Vec<BlockNodeId> {
    let mut exit_blocks = Vec::new();

    for &block_node in blocks {
        // Check if this block has any successors within the segment
        let has_internal_successor =
            cfg.graph().edges_directed(block_node, Direction::Outgoing).any(|edge| {
                // Ignore special edges
                if matches!(edge.weight(), EdgeType::NewFunction | EdgeType::Unreachable) {
                    return false;
                }
                // Check if target is in the same segment
                blocks.contains(&edge.target())
            });

        if !has_internal_successor {
            exit_blocks.push(block_node);
        }
    }

    exit_blocks
}

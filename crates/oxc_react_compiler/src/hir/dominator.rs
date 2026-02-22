/// Dominator tree computation for HIR control-flow graphs.
///
/// Port of `HIR/Dominator.ts` from the React Compiler.
///
/// Computes the dominator tree using the algorithm from
/// "A Simple, Fast Dominance Algorithm" by Cooper, Harvey, and Kennedy (2001).
/// <https://www.cs.rice.edu/~keith/Embed/dom.pdf>
use rustc_hash::{FxHashMap, FxHashSet};

use super::hir_types::{BlockId, HIRFunction, Terminal};
use super::visitors::each_terminal_successor;

// =====================================================================================
// Internal graph representation
// =====================================================================================

struct Node {
    id: BlockId,
    index: usize,
    preds: FxHashSet<BlockId>,
    succs: FxHashSet<BlockId>,
}

struct Graph {
    entry: BlockId,
    nodes: FxHashMap<BlockId, Node>,
}

// =====================================================================================
// Dominator tree
// =====================================================================================

/// A dominator tree that stores the immediate dominator for each block.
///
/// A block X dominates block Y if all paths to Y must flow through X.
pub struct Dominator {
    entry: BlockId,
    nodes: FxHashMap<BlockId, BlockId>,
}

impl Dominator {
    /// Returns the entry node.
    pub fn entry(&self) -> BlockId {
        self.entry
    }

    /// Returns the immediate dominator of the given block.
    /// Returns `None` if the block is the entry (dominates itself).
    pub fn get(&self, id: BlockId) -> Option<BlockId> {
        let dominator = self.nodes.get(&id)?;
        if *dominator == id { None } else { Some(*dominator) }
    }
}

/// A post-dominator tree.
///
/// A block Y post-dominates block X if all paths from X to exit must flow through Y.
pub struct PostDominator {
    exit: BlockId,
    nodes: FxHashMap<BlockId, BlockId>,
}

impl PostDominator {
    /// Returns the exit node.
    pub fn exit(&self) -> BlockId {
        self.exit
    }

    /// Returns the immediate post-dominator of the given block.
    /// Returns `None` if the block is the exit (post-dominates itself).
    pub fn get(&self, id: BlockId) -> Option<BlockId> {
        let dominator = self.nodes.get(&id)?;
        if *dominator == id { None } else { Some(*dominator) }
    }
}

// =====================================================================================
// Public API
// =====================================================================================

/// Compute the dominator tree of the given function.
pub fn compute_dominator_tree(func: &HIRFunction) -> Dominator {
    let graph = build_graph(func);
    let nodes = compute_immediate_dominators(&graph);
    Dominator { entry: graph.entry, nodes }
}

/// Compute the post-dominator tree of the given function.
pub fn compute_post_dominator_tree(
    func: &HIRFunction,
    include_throws_as_exit: bool,
) -> PostDominator {
    let graph = build_reverse_graph(func, include_throws_as_exit);
    let mut nodes = compute_immediate_dominators(&graph);

    // For blocks not reachable to exit, add them with themselves as dominator
    if !include_throws_as_exit {
        for &id in func.body.blocks.keys() {
            nodes.entry(id).or_insert(id);
        }
    }

    PostDominator { exit: graph.entry, nodes }
}

// =====================================================================================
// Core algorithm
// =====================================================================================

/// Compute immediate dominators using the Cooper-Harvey-Kennedy algorithm.
fn compute_immediate_dominators(graph: &Graph) -> FxHashMap<BlockId, BlockId> {
    let mut nodes: FxHashMap<BlockId, BlockId> = FxHashMap::default();
    nodes.insert(graph.entry, graph.entry);

    let mut changed = true;
    while changed {
        changed = false;
        for (&id, node) in &graph.nodes {
            if node.id == graph.entry {
                continue;
            }

            // First processed predecessor
            let mut new_idom: Option<BlockId> = None;
            for &pred in &node.preds {
                if nodes.contains_key(&pred) {
                    new_idom = Some(pred);
                    break;
                }
            }
            let Some(mut new_idom) = new_idom else { continue }; // skip if no predecessor visited yet

            // For all other predecessors
            for &pred in &node.preds {
                if pred == new_idom {
                    continue;
                }
                if nodes.contains_key(&pred) {
                    new_idom = intersect(pred, new_idom, graph, &nodes);
                }
            }

            let current = nodes.get(&id).copied();
            if current != Some(new_idom) {
                nodes.insert(id, new_idom);
                changed = true;
            }
        }
    }

    nodes
}

fn intersect(
    a: BlockId,
    b: BlockId,
    graph: &Graph,
    nodes: &FxHashMap<BlockId, BlockId>,
) -> BlockId {
    let mut block1 = graph.nodes.get(&a);
    let mut block2 = graph.nodes.get(&b);

    while block1.map(|n| n.id) != block2.map(|n| n.id) {
        let idx1 = block1.map_or(0, |n| n.index);
        let idx2 = block2.map_or(0, |n| n.index);

        match idx1.cmp(&idx2) {
            std::cmp::Ordering::Greater => {
                let dom = nodes.get(&block1.map_or(BlockId(0), |n| n.id));
                block1 = dom.and_then(|d| graph.nodes.get(d));
            }
            std::cmp::Ordering::Less => {
                let dom = nodes.get(&block2.map_or(BlockId(0), |n| n.id));
                block2 = dom.and_then(|d| graph.nodes.get(d));
            }
            std::cmp::Ordering::Equal => {
                // Indices are equal but nodes different — shouldn't happen
                break;
            }
        }
    }

    block1.map_or(b, |n| n.id)
}

// =====================================================================================
// Graph construction
// =====================================================================================

fn build_graph(func: &HIRFunction) -> Graph {
    let mut nodes = FxHashMap::default();
    for (index, (&id, block)) in func.body.blocks.iter().enumerate() {
        nodes.insert(
            id,
            Node {
                id,
                index,
                preds: block.preds.clone(),
                succs: each_terminal_successor(&block.terminal).into_iter().collect(),
            },
        );
    }
    Graph { entry: func.body.entry, nodes }
}

fn build_reverse_graph(func: &HIRFunction, include_throws_as_exit: bool) -> Graph {
    let exit_id =
        BlockId(u32::try_from(func.body.blocks.len()).unwrap_or(u32::MAX).saturating_add(1000)); // synthetic exit block

    let mut nodes: FxHashMap<BlockId, Node> = FxHashMap::default();
    let mut exit_node =
        Node { id: exit_id, index: 0, preds: FxHashSet::default(), succs: FxHashSet::default() };

    for (&id, block) in &func.body.blocks {
        let mut node = Node {
            id,
            index: 0,
            preds: each_terminal_successor(&block.terminal).into_iter().collect(),
            succs: block.preds.clone(),
        };

        match &block.terminal {
            Terminal::Return(_) => {
                node.preds.insert(exit_id);
                exit_node.succs.insert(id);
            }
            Terminal::Throw(_) if include_throws_as_exit => {
                node.preds.insert(exit_id);
                exit_node.succs.insert(id);
            }
            _ => {}
        }

        nodes.insert(id, node);
    }
    nodes.insert(exit_id, exit_node);

    // Put nodes into RPO form via DFS
    let mut visited = FxHashSet::default();
    let mut postorder: Vec<BlockId> = Vec::new();
    dfs_visit(exit_id, &nodes, &mut visited, &mut postorder);
    postorder.reverse();

    let mut rpo_nodes = FxHashMap::default();
    for (index, &id) in postorder.iter().enumerate() {
        if let Some(mut node) = nodes.remove(&id) {
            node.index = index;
            rpo_nodes.insert(id, node);
        }
    }

    Graph { entry: exit_id, nodes: rpo_nodes }
}

fn dfs_visit(
    id: BlockId,
    nodes: &FxHashMap<BlockId, Node>,
    visited: &mut FxHashSet<BlockId>,
    postorder: &mut Vec<BlockId>,
) {
    if !visited.insert(id) {
        return;
    }
    if let Some(node) = nodes.get(&id) {
        for &succ in &node.succs {
            dfs_visit(succ, nodes, visited, postorder);
        }
    }
    postorder.push(id);
}

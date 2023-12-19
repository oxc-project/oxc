use std::collections::{HashMap, HashSet};

use petgraph::{stable_graph::NodeIndex, visit::EdgeRef, Direction, Graph};

use crate::control_flow::ControlFlowGraph;

/// # Panics
pub fn neighbors_filtered_by_edge_weight<
    State: Default + Copy + Clone,
    NodeWeight,
    EdgeWeight,
    F,
    G,
>(
    graph: &Graph<NodeWeight, EdgeWeight>,
    node: NodeIndex,
    edge_filter: &F,
    visitor: &mut G,
) -> Vec<State>
where
    F: Fn(&EdgeWeight) -> Option<State>,
    G: FnMut(&NodeWeight, State) -> (State, bool),
{
    let mut q = vec![];
    let mut final_states = vec![];

    // for initial node
    let (new_state, keep_walking_this_path) =
        visitor(graph.node_weight(node).unwrap(), Default::default());
    // if we will continue walking push this node
    if keep_walking_this_path {
        q.push((node, new_state));
    } else {
        final_states.push(new_state);
    }

    while let Some((graph_ix, state)) = q.pop() {
        let mut edges = 0;
        for edge in graph.edges_directed(graph_ix, Direction::Outgoing) {
            if let Some(result_of_edge_filtering) = edge_filter(edge.weight()) {
                final_states.push(result_of_edge_filtering);
            } else {
                let opposite_dir_of_edge_graph_ix = edge.target();
                let (new_state, keep_walking_this_path) =
                    visitor(graph.node_weight(opposite_dir_of_edge_graph_ix).unwrap(), state);
                if keep_walking_this_path {
                    q.push((opposite_dir_of_edge_graph_ix, new_state));
                } else {
                    final_states.push(new_state);
                }
                edges += 1;
            }
        }

        if edges == 0 {
            final_states.push(state);
        }
    }

    final_states
}

/// # Panics
pub fn replicate_tree_to_leaves(
    start_at: NodeIndex,
    cfg: &mut ControlFlowGraph,
) -> HashMap<NodeIndex, NodeIndex> {
    fn duplicate_graph_node(basic_block_id: usize, cfg: &mut ControlFlowGraph) -> NodeIndex {
        let new_basic_block_graph_ix = cfg.new_basic_block();

        // todo: what's a better way to do this?
        for i in 0..cfg.basic_blocks[basic_block_id].len() {
            let item = cfg.basic_blocks[basic_block_id][i].clone();
            cfg.basic_blocks[cfg.current_basic_block].push(item);
        }

        // todo: should we add the functions copied here to the function_to_node_ix?

        new_basic_block_graph_ix
    }

    let preserved_graph_ix = cfg.current_node_ix;
    let preserved_bb = cfg.current_basic_block;

    let mut old_to_new: HashMap<NodeIndex, NodeIndex> = HashMap::default();
    let mut q = vec![];

    let new_graph_start_ix = duplicate_graph_node(*cfg.graph.node_weight(start_at).unwrap(), cfg);
    old_to_new.insert(start_at, new_graph_start_ix);
    q.push((new_graph_start_ix, start_at));
    let mut edges_finished_already = HashSet::new();

    while let Some(node) = q.pop() {
        // todo: are there any incoming edges we won't cover?
        let edges = cfg
            .graph
            .edges_directed(node.1, Direction::Outgoing)
            .map(|x| (x.target(), x.weight().to_owned(), x.id()))
            .collect::<Vec<_>>();

        for (to, weight, id) in edges {
            if edges_finished_already.contains(&id) {
                continue;
            }

            edges_finished_already.insert(id);

            let new_graph_ix = if old_to_new.contains_key(&to) {
                old_to_new[&to]
            } else {
                let new_graph_ix = duplicate_graph_node(*cfg.graph.node_weight(to).unwrap(), cfg);
                old_to_new.insert(node.1, new_graph_start_ix);
                new_graph_ix
            };

            cfg.add_edge(node.0, new_graph_ix, weight);

            q.push((new_graph_ix, to));
        }
    }

    cfg.current_basic_block = preserved_bb;
    cfg.current_node_ix = preserved_graph_ix;
    old_to_new
}

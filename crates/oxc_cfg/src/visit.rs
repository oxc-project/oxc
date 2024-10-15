use std::hash::Hash;

use petgraph::{
    visit::{ControlFlow, DfsEvent, EdgeRef, IntoNeighbors, Time, VisitMap, Visitable},
    Direction, Graph,
};
use rustc_hash::FxHashSet;

use crate::BlockNodeId;

/// # Panics
pub fn neighbors_filtered_by_edge_weight<State: Default + Clone, NodeWeight, EdgeWeight, F, G>(
    graph: &Graph<NodeWeight, EdgeWeight>,
    node: BlockNodeId,
    edge_filter: &F,
    visitor: &mut G,
) -> Vec<State>
where
    F: Fn(&EdgeWeight) -> Option<State>,
    G: FnMut(&BlockNodeId, State) -> (State, bool),
{
    let mut q = vec![];
    let mut final_states = vec![];
    let mut visited = FxHashSet::default();

    // for initial node
    let (new_state, keep_walking_this_path) = visitor(&node, Default::default());
    // if we will continue walking push this node
    if keep_walking_this_path {
        q.push((node, new_state));
    } else {
        final_states.push(new_state);
    }

    while let Some((graph_ix, state)) = q.pop() {
        let mut edges = 0;

        for edge in graph.edges_directed(graph_ix, Direction::Outgoing) {
            if visited.contains(&edge.target()) {
                continue;
            }
            if let Some(result_of_edge_filtering) = edge_filter(edge.weight()) {
                final_states.push(result_of_edge_filtering);
            } else {
                let target = edge.target();
                let (new_state, keep_walking_this_path) = visitor(&target, state.clone());
                visited.insert(target);
                if keep_walking_this_path {
                    q.push((target, new_state.clone()));
                } else {
                    final_states.push(new_state.clone());
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

/// Copied from petgraph's `dfsvisit`.
/// Return if the expression is a break value, execute the provided statement
/// if it is a prune value.
macro_rules! try_control {
    ($e:expr, $p:stmt) => {
        try_control!($e, $p, ());
    };
    ($e:expr, $p:stmt, $q:stmt) => {
        match $e {
            x =>
            {
                #[allow(clippy::redundant_else)]
                if x.should_break() {
                    return x;
                } else if x.should_prune() {
                    $p
                } else {
                    $q
                }
            }
        }
    };
}

/// Similar to `depth_first_search` but uses a `HashSet` underneath. Ideal for small subgraphs.
pub fn set_depth_first_search<G, I, F, C, N>(graph: G, starts: I, mut visitor: F) -> C
where
    N: Copy + PartialEq + Eq + Hash,
    G: IntoNeighbors + Visitable<NodeId = N>,
    I: IntoIterator<Item = G::NodeId>,
    F: FnMut(DfsEvent<G::NodeId>) -> C,
    C: ControlFlow,
{
    let time = &mut Time(0);
    let discovered = &mut FxHashSet::<G::NodeId>::default();
    let finished = &mut FxHashSet::<G::NodeId>::default();

    for start in starts {
        try_control!(
            dfs_visitor(graph, start, &mut visitor, discovered, finished, time),
            unreachable!()
        );
    }
    C::continuing()
}

fn dfs_visitor<G, M, F, C>(
    graph: G,
    u: G::NodeId,
    visitor: &mut F,
    discovered: &mut M,
    finished: &mut M,
    time: &mut Time,
) -> C
where
    G: IntoNeighbors + Visitable,
    M: VisitMap<G::NodeId>,
    F: FnMut(DfsEvent<G::NodeId>) -> C,
    C: ControlFlow,
{
    if !discovered.visit(u) {
        return C::continuing();
    }

    try_control!(
        visitor(DfsEvent::Discover(u, time_post_inc(time))),
        {},
        for v in graph.neighbors(u) {
            if !discovered.is_visited(&v) {
                try_control!(visitor(DfsEvent::TreeEdge(u, v)), continue);
                try_control!(
                    dfs_visitor(graph, v, visitor, discovered, finished, time),
                    unreachable!()
                );
            } else if !finished.is_visited(&v) {
                try_control!(visitor(DfsEvent::BackEdge(u, v)), continue);
            } else {
                try_control!(visitor(DfsEvent::CrossForwardEdge(u, v)), continue);
            }
        }
    );
    let first_finish = finished.visit(u);
    debug_assert!(first_finish);
    try_control!(
        visitor(DfsEvent::Finish(u, time_post_inc(time))),
        panic!("Pruning on the `DfsEvent::Finish` is not supported!")
    );
    C::continuing()
}

fn time_post_inc(x: &mut Time) -> Time {
    let v = *x;
    x.0 += 1;
    v
}

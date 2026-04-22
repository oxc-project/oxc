use std::hash::Hash;

use petgraph::{
    Direction, Graph,
    visit::{ControlFlow, DfsEvent, EdgeRef, IntoNeighbors, Time, Visitable},
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

/// Similar to `depth_first_search` but uses a `HashSet` underneath and an iterative
/// implementation to avoid stack overflow on large graphs.
pub fn set_depth_first_search<G, I, F, C, N>(graph: G, starts: I, mut visitor: F) -> C
where
    N: Copy + PartialEq + Eq + Hash,
    G: IntoNeighbors<NodeId = N> + Visitable<NodeId = N>,
    I: IntoIterator<Item = G::NodeId>,
    F: FnMut(DfsEvent<G::NodeId>) -> C,
    C: ControlFlow,
{
    // Stack frame for iterative DFS.
    // Each frame represents a node being processed and its neighbors iterator.
    struct Frame<N, I> {
        node: N,
        neighbors: I,
        // Whether we've already emitted Discover for this node
        discovered_emitted: bool,
    }

    let mut time = Time(0);
    let mut discovered = FxHashSet::<G::NodeId>::default();
    let mut finished = FxHashSet::<G::NodeId>::default();
    let mut stack: Vec<Frame<N, <G as IntoNeighbors>::Neighbors>> = Vec::new();

    for start in starts {
        // Skip if already discovered from a previous start node
        if discovered.contains(&start) {
            continue;
        }

        // Push the start node onto the stack
        stack.push(Frame {
            node: start,
            neighbors: graph.neighbors(start),
            discovered_emitted: false,
        });

        while let Some(frame) = stack.last_mut() {
            let u = frame.node;

            // First time processing this frame: emit Discover event
            if !frame.discovered_emitted {
                let newly_discovered = discovered.insert(u);
                debug_assert!(
                    newly_discovered,
                    "DFS invariant violated: node on stack was already discovered"
                );

                let result = visitor(DfsEvent::Discover(u, time_post_inc(&mut time)));
                frame.discovered_emitted = true;

                if result.should_break() {
                    return result;
                }
                if result.should_prune() {
                    // Prune: skip children but still emit Finish
                    finished.insert(u);
                    let finish_result = visitor(DfsEvent::Finish(u, time_post_inc(&mut time)));
                    stack.pop();
                    if finish_result.should_break() {
                        return finish_result;
                    }
                    continue;
                }
            }

            // Process neighbors
            let mut found_unvisited = false;
            for v in frame.neighbors.by_ref() {
                if !discovered.contains(&v) {
                    // TreeEdge: edge to unvisited node
                    let result = visitor(DfsEvent::TreeEdge(u, v));
                    if result.should_break() {
                        return result;
                    }
                    if result.should_prune() {
                        // Prune this edge, continue to next neighbor
                        continue;
                    }

                    // Push the neighbor onto the stack to visit it
                    stack.push(Frame {
                        node: v,
                        neighbors: graph.neighbors(v),
                        discovered_emitted: false,
                    });
                    found_unvisited = true;
                    break;
                } else if !finished.contains(&v) {
                    // BackEdge: edge to node in current path (discovered but not finished)
                    let result = visitor(DfsEvent::BackEdge(u, v));
                    if result.should_break() {
                        return result;
                    }
                    // Continue to next neighbor (prune has no effect for BackEdge in original impl)
                } else {
                    // CrossForwardEdge: edge to already finished node
                    let result = visitor(DfsEvent::CrossForwardEdge(u, v));
                    if result.should_break() {
                        return result;
                    }
                    // Continue to next neighbor (prune has no effect for CrossForwardEdge in original impl)
                }
            }

            // If we didn't find an unvisited neighbor, we're done with this node
            if !found_unvisited {
                finished.insert(u);
                let result = visitor(DfsEvent::Finish(u, time_post_inc(&mut time)));
                stack.pop();
                if result.should_break() {
                    return result;
                }
                // Note: Pruning on Finish is not supported per original implementation
            }
        }
    }

    C::continuing()
}

fn time_post_inc(x: &mut Time) -> Time {
    let v = *x;
    x.0 += 1;
    v
}

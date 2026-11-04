/// Returns `(component_id, is_cyclic)` for each node, in input order.
///
/// Iterative Kosaraju keeps both passes linear and avoids overflowing the call stack on deep
/// dependency chains. Identifiers are node indices, so they are smaller than `edges.len()`.
pub fn cyclic_components(edges: &[Vec<usize>]) -> Vec<(usize, bool)> {
    // 1. Build reverse edges for the later walk back through the graph.
    let mut reverse_edges = vec![Vec::new(); edges.len()];
    for (node, dependencies) in edges.iter().enumerate() {
        for &dependency in dependencies {
            reverse_edges[dependency].push(node);
        }
    }

    // 2. Walk dependencies depth-first, recording nodes when their walks finish.
    let mut visited = vec![false; edges.len()];
    let mut finish_order = Vec::with_capacity(edges.len());
    let mut dfs_stack = Vec::new();
    for root in 0..edges.len() {
        if visited[root] {
            continue;
        }
        visited[root] = true;
        dfs_stack.push((root, 0));
        while let Some((node, next_edge)) = dfs_stack.last_mut() {
            if let Some(&dependency) = edges[*node].get(*next_edge) {
                *next_edge += 1;
                if !visited[dependency] {
                    visited[dependency] = true;
                    dfs_stack.push((dependency, 0));
                }
            } else {
                finish_order.push(*node);
                dfs_stack.pop();
            }
        }
    }

    // 3. Reverse finish order keeps each backward walk inside one unassigned SCC.
    let mut components = vec![(usize::MAX, false); edges.len()];
    let mut component_stack = Vec::new();
    let mut component_nodes = Vec::new();
    for &root in finish_order.iter().rev() {
        if components[root].0 != usize::MAX {
            continue;
        }
        components[root].0 = root;
        component_stack.push(root);
        while let Some(node) = component_stack.pop() {
            component_nodes.push(node);
            for &dependency in &reverse_edges[node] {
                if components[dependency].0 == usize::MAX {
                    components[dependency].0 = root;
                    component_stack.push(dependency);
                }
            }
        }
        // A singleton is cyclic only if it has a self-loop.
        let is_cyclic = component_nodes.len() > 1 || edges[root].contains(&root);
        for &node in &component_nodes {
            components[node].1 = is_cyclic;
        }
        component_nodes.clear();
    }
    components
}

#[cfg(test)]
mod tests {
    use super::cyclic_components;

    #[test]
    fn agrees_with_reachability_for_every_three_node_graph() {
        const N: usize = 3;
        assert_eq!(cyclic_components(&[]), []);
        for mask in 0..(1 << (N * N)) {
            let edges: Vec<Vec<usize>> = (0..N)
                .map(|node| (0..N).filter(|&dep| mask & (1 << (node * N + dep)) != 0).collect())
                .collect();
            let mut reachable = vec![vec![false; N]; N];
            for (node, dependencies) in edges.iter().enumerate() {
                for &dependency in dependencies {
                    reachable[node][dependency] = true;
                }
            }
            for pivot in 0..N {
                let via = reachable[pivot].clone();
                for row in &mut reachable {
                    if row[pivot] {
                        for (entry, &suffix) in row.iter_mut().zip(&via) {
                            *entry |= suffix;
                        }
                    }
                }
            }
            let components = cyclic_components(&edges);
            for (node, &(id, cyclic)) in components.iter().enumerate() {
                assert_eq!(cyclic, reachable[node][node], "graph {mask}, node {node}");
                for (other, &(other_id, _)) in components.iter().enumerate() {
                    assert_eq!(
                        id == other_id,
                        node == other || (reachable[node][other] && reachable[other][node]),
                        "graph {mask}, nodes {node} and {other}",
                    );
                }
            }
        }
    }

    #[test]
    fn deep_chain_leading_into_a_cycle_is_not_itself_cyclic() {
        let count = 50_000;
        let mut edges: Vec<_> = (1..count).map(|next| vec![next]).collect();
        edges.push(vec![count - 2]);
        let components = cyclic_components(&edges);
        assert!(components[..count - 2].iter().all(|&(_, cyclic)| !cyclic));
        assert!(components[count - 2].1);
        assert_eq!(components[count - 2], components[count - 1]);
    }
}

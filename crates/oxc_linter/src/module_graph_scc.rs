/// Returns a component identifier and whether that component is cyclic for each node.
///
/// Iterative Kosaraju keeps both passes linear and avoids overflowing the call stack on deep
/// dependency chains. Identifiers are node indices, so they are smaller than `edges.len()`.
pub fn cyclic_components(edges: &[Vec<usize>]) -> Vec<(usize, bool)> {
    let mut reversed = vec![Vec::new(); edges.len()];
    for (node, dependencies) in edges.iter().enumerate() {
        for &dependency in dependencies {
            reversed[dependency].push(node);
        }
    }

    let mut visited = vec![false; edges.len()];
    let mut order = Vec::with_capacity(edges.len());
    let mut stack = Vec::new();
    for root in 0..edges.len() {
        if visited[root] {
            continue;
        }
        visited[root] = true;
        stack.push((root, 0));
        while let Some((node, next_edge)) = stack.last_mut() {
            if let Some(&dependency) = edges[*node].get(*next_edge) {
                *next_edge += 1;
                if !visited[dependency] {
                    visited[dependency] = true;
                    stack.push((dependency, 0));
                }
            } else {
                order.push(*node);
                stack.pop();
            }
        }
    }

    let mut components = vec![(usize::MAX, false); edges.len()];
    let mut pending = Vec::new();
    let mut members = Vec::new();
    for &root in order.iter().rev() {
        if components[root].0 != usize::MAX {
            continue;
        }
        components[root].0 = root;
        pending.push(root);
        while let Some(node) = pending.pop() {
            members.push(node);
            for &dependency in &reversed[node] {
                if components[dependency].0 == usize::MAX {
                    components[dependency].0 = root;
                    pending.push(dependency);
                }
            }
        }
        let cyclic = members.len() > 1 || edges[root].contains(&root);
        for &node in &members {
            components[node].1 = cyclic;
        }
        members.clear();
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

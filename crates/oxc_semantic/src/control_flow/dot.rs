use petgraph::{dot, stable_graph::NodeIndex};

use crate::ControlFlowGraph;

use std::fmt::{self, Debug, Display};

impl ControlFlowGraph {
    /// Returns an object that implements [`Display`] for printing a
    /// [`GraphViz DOT`] representation of the control flow graph.
    ///
    /// [`GraphViz DOT`]: https://graphviz.org/doc/info/lang.html
    pub fn dot(&self) -> Dot<'_> {
        // Exposing our own struct instead of petgraph's allows us to control
        // our API surface.
        Dot { cfg: self }
    }

    pub(crate) fn fmt_dot(&self, f: &mut fmt::Formatter) -> fmt::Result {
        const CONFIG: &[dot::Config] = &[dot::Config::EdgeNoLabel, dot::Config::NodeNoLabel];

        let get_node_attributes = |_graph, node: (NodeIndex, &usize)| {
            let block = &self.basic_blocks[*node.1];

            format!(
                "label = \"{}\"",
                block.iter().map(|el| format!("{el}")).collect::<Vec<_>>().join("\\n")
            )
        };

        let dot = dot::Dot::with_attr_getters(
            &self.graph,
            CONFIG,
            &|_graph, _edge| String::new(),
            // todo: We currently do not print edge types into cfg dot diagram
            // so they aren't snapshotted, but we could by uncommenting this.
            // &|_graph, edge| format!("label = {:?}", edge.weight()),
            // &self.node_attributes,
            &get_node_attributes,
        );

        dot.fmt(f)
    }
}

/// Helper struct for rendering [`DOT`] diagrams of a [`ControlFlowGraph`].
/// Returned from [`ControlFlowGraph::dot`].
///
/// [`DOT`]: https://graphviz.org/doc/info/lang.html
///
#[derive(Clone, Copy)]
pub struct Dot<'a> {
    cfg: &'a ControlFlowGraph,
}

impl<'a> Debug for Dot<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.cfg.fmt_dot(f)
    }
}

impl<'a> Display for Dot<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.cfg.fmt_dot(f)
    }
}

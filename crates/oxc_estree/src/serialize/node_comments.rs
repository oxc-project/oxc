use oxc_data_structures::code_buffer::CodeBuffer;

/// Placement of a comment relative to the ESTree node which owns it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeCommentPlacement {
    Leading = 0,
    Trailing = 1,
    Dangling = 2,
}

/// Comments to serialize in the `comments` field of ESTree nodes, indexed by node ID.
///
/// Each comment is stored as pre-serialized JSON. Comments must be pushed in source order.
#[derive(Debug, Default)]
pub struct NodeComments {
    comments: Vec<String>,
    nodes: Vec<[Vec<u32>; 3]>,
}

impl NodeComments {
    /// Create an empty [`NodeComments`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a comment owned by the node with ID `node_id`.
    ///
    /// # Panics
    /// Panics if there are more than `u32::MAX` comments.
    pub fn push(&mut self, node_id: usize, placement: NodeCommentPlacement, comment_json: String) {
        let comment_index = u32::try_from(self.comments.len()).unwrap();
        self.comments.push(comment_json);
        if self.nodes.len() <= node_id {
            self.nodes.resize_with(node_id + 1, Default::default);
        }
        self.nodes[node_id][placement as usize].push(comment_index);
    }

    /// Print the `comments` field value for the nodes with IDs `node_ids`.
    pub(super) fn print(&self, node_ids: &[usize], buffer: &mut CodeBuffer) {
        let mut indices = [vec![], vec![], vec![]];
        for &node_id in node_ids {
            let Some(groups) = self.nodes.get(node_id) else { continue };
            for (group, node_group) in indices.iter_mut().zip(groups) {
                group.extend_from_slice(node_group);
            }
        }

        if indices.iter().all(Vec::is_empty) {
            buffer.print_str("null");
            return;
        }

        buffer.print_ascii_byte(b'{');
        for (index, (key, group)) in
            ["leading", "trailing", "dangling"].into_iter().zip(&mut indices).enumerate()
        {
            if index > 0 {
                buffer.print_ascii_byte(b',');
            }
            buffer.print_strs_array(["\"", key, "\":"]);
            if group.is_empty() {
                buffer.print_str("null");
                continue;
            }
            if node_ids.len() > 1 {
                group.sort_unstable();
            }
            buffer.print_ascii_byte(b'[');
            for (index, &comment_index) in group.iter().enumerate() {
                if index > 0 {
                    buffer.print_ascii_byte(b',');
                }
                buffer.print_str(&self.comments[comment_index as usize]);
            }
            buffer.print_ascii_byte(b']');
        }
        buffer.print_ascii_byte(b'}');
    }
}

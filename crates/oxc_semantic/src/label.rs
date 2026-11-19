use oxc_str::Ident;
use oxc_syntax::{label::LabelId, node::NodeId};

#[derive(Debug)]
pub struct LabeledScope<'a> {
    pub name: Ident<'a>,
    pub used: bool,
    pub node_id: NodeId,
    pub label_id: LabelId,
}

#[derive(Debug, Default)]
pub struct UnusedLabels<'a> {
    pub stack: Vec<LabeledScope<'a>>,
    /// Index in `stack` of the first label in the current function or static block.
    /// `break` and `continue` cannot target labels below it.
    pub jump_boundary: usize,
    pub labels: Vec<NodeId>,
}

impl<'a> UnusedLabels<'a> {
    pub fn add(&mut self, name: Ident<'a>, node_id: NodeId, label_id: LabelId) {
        self.stack.push(LabeledScope { name, used: false, node_id, label_id });
    }

    /// Start a function or static block, returning the previous boundary for [`Self::leave_jump_boundary`].
    pub fn enter_jump_boundary(&mut self) -> usize {
        std::mem::replace(&mut self.jump_boundary, self.stack.len())
    }

    pub fn leave_jump_boundary(&mut self, jump_boundary: usize) {
        self.jump_boundary = jump_boundary;
    }

    pub fn reference(&mut self, name: Ident<'_>) -> Option<LabelId> {
        for scope in self.stack[self.jump_boundary..].iter_mut().rev() {
            if scope.name == name {
                scope.used = true;
                return Some(scope.label_id);
            }
        }
        None
    }

    pub fn mark_unused(&mut self) {
        debug_assert!(
            !self.stack.is_empty(),
            "mark_unused called with empty label stack - this indicates mismatched add/mark_unused calls"
        );

        if let Some(scope) = self.stack.pop()
            && !scope.used
        {
            self.labels.push(scope.node_id);
        }
    }

    #[cfg(debug_assertions)]
    pub fn assert_empty(&self) {
        debug_assert!(
            self.stack.is_empty(),
            "Label stack not empty at end of processing - {} labels remaining. This indicates mismatched add/mark_unused calls",
            self.stack.len()
        );
    }
}

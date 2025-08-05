use oxc_syntax::node::NodeId;

#[derive(Debug)]
pub struct LabeledScope<'a> {
    pub name: &'a str,
    pub used: bool,
    pub node_id: NodeId,
}

#[derive(Debug, Default)]
pub struct UnusedLabels<'a> {
    pub stack: Vec<LabeledScope<'a>>,
    pub labels: Vec<NodeId>,
}

impl<'a> UnusedLabels<'a> {
    pub fn add(&mut self, name: &'a str, node_id: NodeId) {
        self.stack.push(LabeledScope { name, used: false, node_id });
    }

    pub fn reference(&mut self, name: &'a str) {
        for scope in self.stack.iter_mut().rev() {
            if scope.name == name {
                scope.used = true;
                return;
            }
        }
    }

    pub fn mark_unused(&mut self) {
        debug_assert!(
            !self.stack.is_empty(),
            "mark_unused called with empty label stack - this indicates mismatched add/mark_unused calls"
        );

        if let Some(scope) = self.stack.pop() {
            if !scope.used {
                self.labels.push(scope.node_id);
            }
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

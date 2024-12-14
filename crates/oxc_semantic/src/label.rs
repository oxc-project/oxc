use oxc_syntax::node::NodeId;

#[derive(Debug)]
pub struct LabeledScope<'a> {
    pub name: &'a str,
    pub used: bool,
    pub parent: usize,
}

#[derive(Debug, Default)]
pub struct UnusedLabels<'a> {
    pub scopes: Vec<LabeledScope<'a>>,
    pub curr_scope: usize,
    pub labels: Vec<NodeId>,
}

impl<'a> UnusedLabels<'a> {
    pub fn add(&mut self, name: &'a str) {
        self.scopes.push(LabeledScope { name, used: false, parent: self.curr_scope });
        self.curr_scope = self.scopes.len() - 1;
    }

    pub fn reference(&mut self, name: &'a str) {
        let scope = self.scopes.iter_mut().rev().find(|x| x.name == name);
        if let Some(scope) = scope {
            scope.used = true;
        }
    }

    pub fn mark_unused(&mut self, current_node_id: NodeId) {
        let scope = &self.scopes[self.curr_scope];
        if !scope.used {
            self.labels.push(current_node_id);
        }
        self.curr_scope = scope.parent;
    }
}

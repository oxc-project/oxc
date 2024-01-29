use oxc_ast::ast::LabeledStatement;
use oxc_span::Span;
use rustc_hash::FxHashSet;

use crate::AstNodeId;

#[derive(Debug)]
pub struct Label<'a> {
    pub id: AstNodeId,
    pub name: &'a str,
    pub span: Span,
    used: bool,
    /// depth is the number of nested labeled statements
    depth: usize,
    /// is accessible means that the label is accessible from the current position
    is_accessible: bool,
    /// is_inside_function_or_static_block means that the label is inside a function or static block
    is_inside_function_or_static_block: bool,
}

impl<'a> Label<'a> {
    pub fn new(
        id: AstNodeId,
        name: &'a str,
        span: Span,
        depth: usize,
        is_inside_function_or_static_block: bool,
    ) -> Self {
        Self {
            id,
            name,
            span,
            depth,
            is_inside_function_or_static_block,
            used: false,
            is_accessible: true,
        }
    }
}

#[derive(Default)]
pub struct LabelBuilder<'a> {
    pub labels: Vec<Vec<Label<'a>>>,
    depth: usize,
    pub unused_node_ids: FxHashSet<AstNodeId>,
}

impl<'a> LabelBuilder<'a> {
    pub fn enter(&mut self, stmt: &'a LabeledStatement<'a>, current_node_id: AstNodeId) {
        let is_empty = self.labels.last().map_or(false, Vec::is_empty);

        if !self.is_inside_labeled_statement() {
            self.labels.push(vec![]);
        }

        self.depth += 1;

        self.labels.last_mut().unwrap_or_else(|| unreachable!()).push(Label::new(
            current_node_id,
            stmt.label.name.as_str(),
            stmt.label.span,
            self.depth,
            is_empty,
        ));
    }

    pub fn leave(&mut self) {
        let depth = self.depth;

        // Mark labels at the current depth as inaccessible
        // ```ts
        // label: {} // leave here, mark label as inaccessible
        // break label // So we cannot find label here
        // ```
        for label in self.get_accessible_labels_mut() {
            if depth == label.depth {
                label.is_accessible = false;
            }
        }

        // If depth is 0, move last labels to the front of `labels` and set `depth` to the length of the last labels.
        // We need to do this because we're currently inside a function or static block
        while self.depth == 0 {
            if let Some(last_labels) = self.labels.pop() {
                if !last_labels.is_empty() {
                    self.labels.insert(0, last_labels);
                }
            }
            self.depth = self.labels.last().unwrap().len();
        }

        self.depth -= 1;

        // insert unused labels into `unused_node_ids`
        if self.depth == 0 {
            if let Some(labels) = self.labels.last() {
                for label in labels {
                    if !label.used {
                        self.unused_node_ids.insert(label.id);
                    }
                }
            }
        }
    }

    pub fn enter_function_or_static_block(&mut self) {
        if self.is_inside_labeled_statement() {
            self.depth = 0;
            self.labels.push(vec![]);
        }
    }

    pub fn leave_function_or_static_block(&mut self) {
        if self.is_inside_labeled_statement() {
            let labels = self.labels.pop().unwrap_or_else(|| unreachable!());
            if !labels.is_empty() {
                self.labels.insert(0, labels);
            }
            self.depth = self.labels.last().map_or(0, Vec::len);
        }
    }

    pub fn is_inside_labeled_statement(&self) -> bool {
        self.depth != 0 || self.labels.last().is_some_and(Vec::is_empty)
    }

    pub fn is_inside_function_or_static_block(&self) -> bool {
        self.labels
            .last()
            .is_some_and(|labels| labels.is_empty() || labels[0].is_inside_function_or_static_block)
    }

    pub fn get_accessible_labels(&self) -> impl DoubleEndedIterator<Item = &Label<'a>> {
        return self.labels.last().unwrap().iter().filter(|label| label.is_accessible).rev();
    }

    pub fn get_accessible_labels_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut Label<'a>> {
        return self
            .labels
            .last_mut()
            .unwrap()
            .iter_mut()
            .filter(|label| label.is_accessible)
            .rev();
    }

    pub fn mark_as_used(&mut self, label: &oxc_ast::ast::LabelIdentifier) {
        if self.is_inside_labeled_statement() {
            let label = self.get_accessible_labels_mut().find(|x| x.name == label.name);

            if let Some(label) = label {
                label.used = true;
            }
        }
    }
}

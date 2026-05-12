use oxc_ast::ast::{Function, PropertyDefinition, StaticBlock, ThisExpression};
use oxc_ast_visit::{Visit, walk};
use oxc_semantic::ScopeFlags;
use oxc_span::Span;

/// Finds `this` expressions without traversing into nested functions.
pub struct ThisExpressionFinder {
    spans: Vec<Span>,
    skip_static_blocks: bool,
    skip_property_definition_values: bool,
}

impl ThisExpressionFinder {
    pub fn new() -> Self {
        Self {
            spans: Vec::new(),
            skip_static_blocks: false,
            skip_property_definition_values: false,
        }
    }

    pub fn skip_static_blocks(mut self) -> Self {
        self.skip_static_blocks = true;
        self
    }

    pub fn skip_property_definition_values(mut self) -> Self {
        self.skip_property_definition_values = true;
        self
    }

    pub fn into_spans(self) -> Vec<Span> {
        self.spans
    }
}

impl<'a> Visit<'a> for ThisExpressionFinder {
    fn visit_this_expression(&mut self, expr: &ThisExpression) {
        self.spans.push(expr.span);
    }

    fn visit_function(&mut self, _func: &Function<'a>, _flags: ScopeFlags) {}

    fn visit_static_block(&mut self, block: &StaticBlock<'a>) {
        if !self.skip_static_blocks {
            walk::walk_static_block(self, block);
        }
    }

    fn visit_property_definition(&mut self, prop: &PropertyDefinition<'a>) {
        if self.skip_property_definition_values {
            self.visit_property_key(&prop.key);
        } else {
            walk::walk_property_definition(self, prop);
        }
    }
}

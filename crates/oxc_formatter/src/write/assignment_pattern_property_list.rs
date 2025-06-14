use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    formatter::{Format, FormatResult, Formatter, separated::FormatSeparatedIter},
    generated::ast_nodes::AstNode,
    options::{FormatTrailingCommas, TrailingSeparator},
};

pub struct AssignmentTargetPropertyList<'a, 'b> {
    properties: &'b AstNode<'a, Vec<'a, AssignmentTargetProperty<'a>>>,
    rest: Option<&'b AstNode<'a, AssignmentTargetRest<'a>>>,
}

impl<'a, 'b> AssignmentTargetPropertyList<'a, 'b> {
    pub fn new(
        properties: &'b AstNode<'a, Vec<'a, AssignmentTargetProperty<'a>>>,
        rest: Option<&'b AstNode<'a, AssignmentTargetRest<'a>>>,
    ) -> Self {
        Self { properties, rest }
    }
}

impl<'a> Format<'a> for AssignmentTargetPropertyList<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let has_trailing_rest = self.rest.is_some();
        let trailing_separator = if has_trailing_rest {
            TrailingSeparator::Disallowed
        } else {
            FormatTrailingCommas::ES5.trailing_separator(f.options())
        };
        let source_text = f.source_text();
        let entries = FormatSeparatedIter::new(self.properties.iter(), ",")
            .with_trailing_separator(trailing_separator)
            .zip(self.properties.iter());
        let mut join = f.join_nodes_with_soft_line();
        for (format_entry, node) in entries {
            join.entry(node.span(), source_text, &format_entry);
        }
        join.finish()
    }
}

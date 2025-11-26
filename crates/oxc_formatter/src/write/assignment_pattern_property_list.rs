use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::{AstNode, AstNodeIterator},
    formatter::{Format, Formatter},
    options::{FormatTrailingCommas, TrailingSeparator},
};

enum AssignmentTargetPropertyListNode<'a, 'b> {
    Property(&'b AstNode<'a, AssignmentTargetProperty<'a>>),
    Rest(&'b AstNode<'a, AssignmentTargetRest<'a>>),
}

impl GetSpan for AssignmentTargetPropertyListNode<'_, '_> {
    fn span(&self) -> Span {
        match self {
            AssignmentTargetPropertyListNode::Property(property) => property.span(),
            AssignmentTargetPropertyListNode::Rest(rest) => rest.span(),
        }
    }
}

impl<'a> Format<'a> for AssignmentTargetPropertyListNode<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        match self {
            AssignmentTargetPropertyListNode::Property(property) => property.fmt(f),
            AssignmentTargetPropertyListNode::Rest(rest) => rest.fmt(f),
        }
    }
}

struct AssignmentTargetPropertyListIter<'a, 'b> {
    properties: AstNodeIterator<'a, AssignmentTargetProperty<'a>>,
    rest: Option<&'b AstNode<'a, AssignmentTargetRest<'a>>>,
}

impl<'a, 'b> Iterator for AssignmentTargetPropertyListIter<'a, 'b> {
    type Item = AssignmentTargetPropertyListNode<'a, 'b>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(property) = self.properties.next() {
            Some(AssignmentTargetPropertyListNode::Property(property))
        } else {
            self.rest.take().map(AssignmentTargetPropertyListNode::Rest)
        }
    }
}

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
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let has_trailing_rest = self.rest.is_some();
        let trailing_separator = if has_trailing_rest {
            TrailingSeparator::Disallowed
        } else {
            FormatTrailingCommas::ES5.trailing_separator(f.options())
        };
        f.join_nodes_with_soft_line().entries_with_trailing_separator(
            AssignmentTargetPropertyListIter {
                properties: self.properties.iter(),
                rest: self.rest,
            },
            ",",
            trailing_separator,
        );
    }
}

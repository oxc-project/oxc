use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::{AstNode, AstNodeIterator},
    formatter::{Format, Formatter},
    options::{FormatTrailingCommas, TrailingSeparator},
};

enum AssignmentTargetPropertyListNode<'me, 'a> {
    Property(AstNode<'me, 'a, AssignmentTargetProperty<'a>>),
    Rest(AstNode<'me, 'a, AssignmentTargetRest<'a>>),
}

impl GetSpan for AssignmentTargetPropertyListNode<'_, '_> {
    fn span(&self) -> Span {
        match self {
            AssignmentTargetPropertyListNode::Property(property) => property.span(),
            AssignmentTargetPropertyListNode::Rest(rest) => rest.span(),
        }
    }
}

impl<'me, 'a> Format<'a> for AssignmentTargetPropertyListNode<'me, 'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        match self {
            AssignmentTargetPropertyListNode::Property(property) => property.fmt(f),
            AssignmentTargetPropertyListNode::Rest(rest) => rest.fmt(f),
        }
    }
}

struct AssignmentTargetPropertyListIter<'me, 'a, 'b> {
    properties: AstNodeIterator<'me, 'a, AssignmentTargetProperty<'a>>,
    rest: Option<&'b AstNode<'me, 'a, AssignmentTargetRest<'a>>>,
}

impl<'me, 'a, 'b> Iterator for AssignmentTargetPropertyListIter<'me, 'a, 'b> {
    type Item = AssignmentTargetPropertyListNode<'me, 'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(property) = self.properties.next() {
            Some(AssignmentTargetPropertyListNode::Property(property))
        } else {
            self.rest.take().map(AssignmentTargetPropertyListNode::Rest)
        }
    }
}

pub struct AssignmentTargetPropertyList<'me, 'a> {
    properties: AstNode<'me, 'a, Vec<'a, AssignmentTargetProperty<'a>>>,
    rest: Option<AstNode<'me, 'a, AssignmentTargetRest<'a>>>,
}

impl<'me, 'a> AssignmentTargetPropertyList<'me, 'a> {
    pub fn new(
        properties: AstNode<'me, 'a, Vec<'a, AssignmentTargetProperty<'a>>>,
        rest: Option<AstNode<'me, 'a, AssignmentTargetRest<'a>>>,
    ) -> Self {
        Self { properties, rest }
    }
}

impl<'me, 'a> Format<'a> for AssignmentTargetPropertyList<'me, 'a> {
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

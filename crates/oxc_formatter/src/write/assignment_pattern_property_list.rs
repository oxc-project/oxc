use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    formatter::{Buffer, Format, FormatResult, Formatter, separated::FormatSeparatedIter},
    generated::ast_nodes::{AstNode, AstNodeIterator},
    options::{FormatTrailingCommas, TrailingSeparator},
    write,
};

enum AssignmentTargetPropertyListNode<'a, 'b> {
    Property(&'b AstNode<'a, AssignmentTargetProperty<'a>>),
    Rest(&'b AstNode<'a, AssignmentTargetRest<'a>>),
}

impl AssignmentTargetPropertyListNode<'_, '_> {
    pub fn span(&self) -> Span {
        match self {
            AssignmentTargetPropertyListNode::Property(property) => property.span(),
            AssignmentTargetPropertyListNode::Rest(rest) => rest.span(),
        }
    }
}

impl<'a> Format<'a> for AssignmentTargetPropertyListNode<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
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
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let has_trailing_rest = self.rest.is_some();
        let trailing_separator = if has_trailing_rest {
            TrailingSeparator::Disallowed
        } else {
            FormatTrailingCommas::ES5.trailing_separator(f.options())
        };
        let source_text = f.source_text();
        let entries = FormatSeparatedIter::new(
            AssignmentTargetPropertyListIter {
                properties: self.properties.iter(),
                rest: self.rest,
            },
            ",",
        )
        .with_trailing_separator(trailing_separator);

        let mut join = f.join_nodes_with_soft_line();
        for entry in entries {
            join.entry(entry.element.span(), &entry);
        }

        join.finish()
    }
}

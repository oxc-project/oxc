use oxc_allocator::Vec;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::{AstNode, AstNodeIterator},
    formatter::{Format, Formatter},
    options::{FormatTrailingCommas, TrailingSeparator},
};

pub struct BindingPropertyList<'me, 'a, 'b> {
    properties: &'b AstNode<'me, 'a, Vec<'a, BindingProperty<'a>>>,
    rest: Option<&'b AstNode<'me, 'a, BindingRestElement<'a>>>,
}

enum BindingPropertyListNode<'me, 'a> {
    Property(AstNode<'me, 'a, BindingProperty<'a>>),
    Rest(AstNode<'me, 'a, BindingRestElement<'a>>),
}

impl GetSpan for BindingPropertyListNode<'_, '_> {
    fn span(&self) -> Span {
        match self {
            BindingPropertyListNode::Property(property) => property.span,
            BindingPropertyListNode::Rest(rest) => rest.span,
        }
    }
}

impl<'me, 'a> Format<'a> for BindingPropertyListNode<'me, 'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        match self {
            BindingPropertyListNode::Property(property) => property.fmt(f),
            BindingPropertyListNode::Rest(rest) => rest.fmt(f),
        }
    }
}

struct BindingPropertyListIter<'me, 'a, 'b> {
    properties: AstNodeIterator<'me, 'a, BindingProperty<'a>>,
    rest: Option<&'b AstNode<'me, 'a, BindingRestElement<'a>>>,
}

impl<'me, 'a, 'b> Iterator for BindingPropertyListIter<'me, 'a, 'b> {
    type Item = BindingPropertyListNode<'me, 'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(property) = self.properties.next() {
            Some(BindingPropertyListNode::Property(property))
        } else {
            self.rest.take().map(BindingPropertyListNode::Rest)
        }
    }
}

impl<'me, 'a, 'b> BindingPropertyList<'me, 'a, 'b> {
    pub fn new(
        properties: &'b AstNode<'me, 'a, Vec<'a, BindingProperty<'a>>>,
        rest: Option<&'b AstNode<'me, 'a, BindingRestElement<'a>>>,
    ) -> Self {
        Self { properties, rest }
    }
}

impl<'me, 'a> Format<'a> for BindingPropertyList<'me, 'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let has_trailing_rest = self.rest.is_some();
        let trailing_separator = if has_trailing_rest {
            TrailingSeparator::Disallowed
        } else {
            FormatTrailingCommas::ES5.trailing_separator(f.options())
        };

        f.join_nodes_with_soft_line().entries_with_trailing_separator(
            BindingPropertyListIter { properties: self.properties.iter(), rest: self.rest },
            ",",
            trailing_separator,
        );
    }
}

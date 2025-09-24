use oxc_allocator::{Box, Vec};
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    formatter::{
        Buffer, Format, FormatResult, Formatter,
        prelude::{format_once, soft_line_break_or_space, text},
        separated::FormatSeparatedIter,
    },
    generated::ast_nodes::{AstNode, AstNodeIterator},
    options::{FormatTrailingCommas, TrailingSeparator},
    write,
};

pub struct BindingPropertyList<'a, 'b> {
    properties: &'b AstNode<'a, Vec<'a, BindingProperty<'a>>>,
    rest: Option<&'b AstNode<'a, BindingRestElement<'a>>>,
}

enum BindingPropertyListNode<'a, 'b> {
    Property(&'b AstNode<'a, BindingProperty<'a>>),
    Rest(&'b AstNode<'a, BindingRestElement<'a>>),
}

impl GetSpan for BindingPropertyListNode<'_, '_> {
    fn span(&self) -> Span {
        match self {
            BindingPropertyListNode::Property(property) => property.span,
            BindingPropertyListNode::Rest(rest) => rest.span,
        }
    }
}

impl<'a> Format<'a> for BindingPropertyListNode<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match self {
            BindingPropertyListNode::Property(property) => property.fmt(f),
            BindingPropertyListNode::Rest(rest) => rest.fmt(f),
        }
    }
}

struct BindingPropertyListIter<'a, 'b> {
    properties: AstNodeIterator<'a, BindingProperty<'a>>,
    rest: Option<&'b AstNode<'a, BindingRestElement<'a>>>,
}

impl<'a, 'b> Iterator for BindingPropertyListIter<'a, 'b> {
    type Item = BindingPropertyListNode<'a, 'b>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(property) = self.properties.next() {
            Some(BindingPropertyListNode::Property(property))
        } else {
            self.rest.take().map(BindingPropertyListNode::Rest)
        }
    }
}

impl<'a, 'b> BindingPropertyList<'a, 'b> {
    pub fn new(
        properties: &'b AstNode<'a, Vec<'a, BindingProperty<'a>>>,
        rest: Option<&'b AstNode<'a, BindingRestElement<'a>>>,
    ) -> Self {
        Self { properties, rest }
    }
}

impl<'a> Format<'a> for BindingPropertyList<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let has_trailing_rest = self.rest.is_some();
        let trailing_separator = if has_trailing_rest {
            TrailingSeparator::Disallowed
        } else {
            FormatTrailingCommas::ES5.trailing_separator(f.options())
        };
        let source_text = f.source_text();

        f.join_nodes_with_soft_line()
            .entries_with_trailing_separator(
                BindingPropertyListIter { properties: self.properties.iter(), rest: self.rest },
                ",",
                trailing_separator,
            )
            .finish()
    }
}

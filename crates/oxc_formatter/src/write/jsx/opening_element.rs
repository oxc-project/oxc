use std::ops::Deref;

use oxc_ast::ast::{JSXAttributeItem, JSXAttributeValue, JSXOpeningElement, StringLiteral};
use oxc_span::GetSpan;

use crate::{
    ast_nodes::AstNode,
    formatter::{Formatter, prelude::*, trivia::FormatTrailingComments},
    write,
};

pub struct FormatOpeningElement<'a, 'b> {
    element: &'b AstNode<'a, JSXOpeningElement<'a>>,
    is_self_closing: bool,
}

impl<'a> Deref for FormatOpeningElement<'a, '_> {
    type Target = AstNode<'a, JSXOpeningElement<'a>>;

    fn deref(&self) -> &Self::Target {
        self.element
    }
}

impl<'a, 'b> FormatOpeningElement<'a, 'b> {
    pub fn new(element: &'b AstNode<'a, JSXOpeningElement<'a>>, is_self_closing: bool) -> Self {
        Self { element, is_self_closing }
    }

    fn compute_layout(&self, f: &Formatter<'_, 'a>) -> OpeningElementLayout {
        let attributes = self.element.attributes();

        let comments = f.context().comments();

        let last_attribute_has_comment = self
            .attributes
            .last()
            .is_some_and(|a| comments.has_comment_in_range(a.span().end, self.span.end));

        let type_arguments_or_name_end =
            self.type_arguments().map_or_else(|| self.name.span().end, |t| t.span.end);
        let first_attribute_start_or_element_end =
            self.attributes.first().map_or_else(|| self.span.end, |a| a.span().start);
        let name_has_comment = comments
            .has_comment_in_range(type_arguments_or_name_end, first_attribute_start_or_element_end);

        if self.is_self_closing && attributes.is_empty() && !name_has_comment {
            OpeningElementLayout::Inline
        } else if attributes.len() == 1
            && !name_has_comment
            && !last_attribute_has_comment
            && is_single_line_string_literal_attribute(&attributes[0])
        {
            OpeningElementLayout::SingleStringAttribute
        } else {
            OpeningElementLayout::IndentAttributes { name_has_comment, last_attribute_has_comment }
        }
    }
}

/// Returns `true` if this is an attribute with a [`StringLiteral`] initializer that contains at least one new line character.
fn is_multiline_string_literal_attribute(attribute: &JSXAttributeItem<'_>) -> bool {
    let JSXAttributeItem::Attribute(attr) = attribute else {
        return false;
    };
    attr.value.as_ref().is_some_and(|value| matches!(value, JSXAttributeValue::StringLiteral(string) if string.value.contains('\n')))
}

impl<'a> Format<'a> for FormatOpeningElement<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let layout = self.compute_layout(f);

        let attributes = self.attributes();

        let format_open = format_with(|f| write!(f, ["<", self.name(), self.type_arguments(),]));
        let format_close = format_with(|f| write!(f, [self.is_self_closing.then_some("/"), ">"]));

        match layout {
            OpeningElementLayout::Inline => {
                write!(f, [format_open, space(), format_close]);
            }
            OpeningElementLayout::SingleStringAttribute => {
                let attribute_spacing = if self.is_self_closing { Some(space()) } else { None };
                write!(
                    f,
                    [format_open, space(), self.attributes(), attribute_spacing, format_close]
                );
            }
            OpeningElementLayout::IndentAttributes {
                name_has_comment,
                last_attribute_has_comment,
            } => {
                let format_inner = format_with(|f| {
                    write!(f, [format_open, soft_line_indent_or_space(&self.attributes())]);

                    let comments = f.context().comments().comments_before(self.span.end);
                    FormatTrailingComments::Comments(comments).fmt(f);

                    let force_bracket_same_line = f.options().bracket_same_line.value();
                    let wants_bracket_same_line = attributes.is_empty() && !name_has_comment;

                    if self.is_self_closing {
                        write!(f, [soft_line_break_or_space(), format_close]);
                    } else if last_attribute_has_comment {
                        write!(f, [soft_line_break(), format_close]);
                    } else if (force_bracket_same_line && !self.attributes.is_empty())
                        || wants_bracket_same_line
                    {
                        write!(f, [format_close]);
                    } else {
                        write!(f, [soft_line_break(), format_close]);
                    }
                });

                let has_multiline_string_attribute = attributes
                    .iter()
                    .any(|attribute| is_multiline_string_literal_attribute(attribute));
                write!(f, [group(&format_inner).should_expand(has_multiline_string_attribute)]);
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum OpeningElementLayout {
    /// Don't create a group around the element to avoid it breaking ever.
    ///
    /// Applied for elements that have no attributes nor any comment attached to their name.
    ///
    /// ```javascript
    /// <ASuperLongComponentNameThatWouldBreakButDoesntSinceTheComponent<DonTBreakThis>></ASuperLongComponentNameThatWouldBreakButDoesntSinceTheComponent>
    /// ```
    Inline,

    /// Opening element with a single attribute that contains no line breaks, nor has comments.
    ///
    /// ```javascript
    /// <div tooltip="A very long tooltip text that would otherwise make the attribute break onto the same line but it is not because of the single string layout" more></div>;
    /// ```
    SingleStringAttribute,

    /// Default layout that indents the attributes and formats each attribute on its own line.
    ///
    /// ```javascript
    /// <div
    ///   oneAttribute
    ///   another="with value"
    ///   moreAttributes={withSomeExpression}
    /// ></div>;
    /// ```
    IndentAttributes { name_has_comment: bool, last_attribute_has_comment: bool },
}

/// Returns `true` if this is an attribute with a string literal initializer that does not contain any new line characters.
fn is_single_line_string_literal_attribute(attribute: &JSXAttributeItem) -> bool {
    as_string_literal_attribute_value(attribute).is_some_and(|string| !string.value.contains('\n'))
}

/// Returns `Some` if the initializer value of this attribute is a string literal.
/// Returns [None] otherwise.
fn as_string_literal_attribute_value<'a>(
    attribute: &'a JSXAttributeItem<'a>,
) -> Option<&'a StringLiteral<'a>> {
    match attribute {
        JSXAttributeItem::Attribute(attr) => {
            if let Some(JSXAttributeValue::StringLiteral(string)) = &attr.value {
                Some(string.as_ref())
            } else {
                None
            }
        }
        JSXAttributeItem::SpreadAttribute(_) => None,
    }
}

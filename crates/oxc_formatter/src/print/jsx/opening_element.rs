use std::ops::Deref;

use oxc_ast::ast::{JSXAttributeItem, JSXAttributeValue, JSXOpeningElement};
use oxc_span::GetSpan;

use crate::{
    ast_nodes::AstNode, formatter::prelude::*,
    utils::tailwindcss::tailwind_context_for_jsx_attribute, write,
};

pub struct FormatOpeningElement<'a, 'b> {
    element: &'b AstNode<'b, 'a, JSXOpeningElement<'a>>,
    is_self_closing: bool,
}

impl<'a, 'b> Deref for FormatOpeningElement<'a, 'b> {
    type Target = AstNode<'b, 'a, JSXOpeningElement<'a>>;

    fn deref(&self) -> &Self::Target {
        self.element
    }
}

impl<'a, 'b> FormatOpeningElement<'a, 'b> {
    pub fn new(element: &'b AstNode<'b, 'a, JSXOpeningElement<'a>>, is_self_closing: bool) -> Self {
        Self { element, is_self_closing }
    }

    fn compute_layout(&self, f: &JsFormatter<'_, 'a>) -> OpeningElementLayout {
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
            && string_literal_attribute_has_line_break(&attributes[0], f) == Some(false)
        {
            OpeningElementLayout::SingleStringAttribute
        } else {
            OpeningElementLayout::IndentAttributes { name_has_comment, last_attribute_has_comment }
        }
    }
}

/// Whether this attribute's string literal value is printed with a line break,
/// or `None` if the value isn't a string literal.
/// - Tailwind sorting without `preserveWhitespace` collapses line breaks,
///   so checking the source value alone would make formatting non-idempotent
fn string_literal_attribute_has_line_break(
    attribute: &JSXAttributeItem<'_>,
    f: &JsFormatter<'_, '_>,
) -> Option<bool> {
    let JSXAttributeItem::Attribute(attr) = attribute else {
        return None;
    };
    let Some(JSXAttributeValue::StringLiteral(string)) = &attr.value else {
        return None;
    };

    if !string.value.contains(|c: char| matches!(c, '\n' | '\r')) {
        return Some(false);
    }

    let collapses_whitespace = tailwind_context_for_jsx_attribute(&attr.name, f)
        .is_some_and(|ctx| !ctx.preserve_whitespace);
    Some(!collapses_whitespace)
}

impl<'a> Format<'a, JsFormatContext<'a>> for FormatOpeningElement<'a, '_> {
    fn fmt(&self, f: &mut JsFormatter<'_, 'a>) {
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
                    write!(f, [format_open]);

                    let attributes = self.attributes();
                    if !attributes.is_empty() {
                        write!(f, [soft_line_indent_or_space(&attributes)]);
                    }

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

                let has_multiline_string_attribute = attributes.iter().any(|attribute| {
                    string_literal_attribute_has_line_break(&attribute, f) == Some(true)
                });
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

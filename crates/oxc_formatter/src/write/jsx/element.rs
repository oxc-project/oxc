use oxc_allocator::Vec;
use oxc_ast::ast::{JSXChild, JSXElement, JSXExpression, JSXExpressionContainer, JSXFragment};
use oxc_span::{GetSpan, Span};

use crate::{
    ast_nodes::{AstNode, AstNodes},
    best_fitting, format_args,
    formatter::{Formatter, prelude::*, trivia::FormatTrailingComments},
    parentheses::NeedsParentheses,
    utils::{
        jsx::{WrapState, is_meaningful_jsx_text},
        suppressed::FormatSuppressedNode,
    },
    write,
    write::jsx::{FormatChildrenResult, FormatOpeningElement},
};

use super::{FormatJsxChildList, JsxChildListLayout};

/// Union type for JSX elements and fragments that have children
#[derive(Debug, Clone)]
pub enum AnyJsxTagWithChildren<'a, 'b> {
    Element(&'b AstNode<'a, JSXElement<'a>>),
    Fragment(&'b AstNode<'a, JSXFragment<'a>>),
}

impl<'a> AnyJsxTagWithChildren<'a, '_> {
    fn span(&self) -> Span {
        match self {
            Self::Element(element) => element.span(),
            Self::Fragment(fragment) => fragment.span(),
        }
    }

    fn format_leading_comments(&self, f: &mut Formatter<'_, 'a>) {
        match self {
            Self::Element(element) => element.format_leading_comments(f),
            Self::Fragment(fragment) => fragment.format_leading_comments(f),
        }
    }

    fn format_trailing_comments(&self, f: &mut Formatter<'_, 'a>) {
        let trailing_comments = if let AstNodes::ArrowFunctionExpression(arrow) =
            self.parent().parent().parent()
            && arrow.expression
        {
            f.context().comments().comments_before(arrow.span.end)
        } else {
            f.context().comments().end_of_line_comments_after(self.span().end)
        };
        FormatTrailingComments::Comments(trailing_comments).fmt(f);
    }

    /// Checks if a JSX Element should be wrapped in parentheses. Returns a [WrapState] which
    /// indicates when the element should be wrapped in parentheses.
    pub fn get_wrap_state(&self) -> WrapState {
        let parent = self.parent();
        // Call site has ensures that only non-nested JSX elements are passed.
        debug_assert!(!matches!(parent, AstNodes::JSXElement(_) | AstNodes::JSXFragment(_)));

        match parent {
            AstNodes::ArrayExpression(_)
            | AstNodes::JSXAttribute(_)
            | AstNodes::JSXExpressionContainer(_)
            | AstNodes::ConditionalExpression(_) => WrapState::NoWrap,
            AstNodes::StaticMemberExpression(member) => {
                if member.optional {
                    WrapState::NoWrap
                } else {
                    WrapState::WrapOnBreak
                }
            }
            // It is a argument of a call expression
            AstNodes::CallExpression(call) if call.is_argument_span(self.span()) => {
                WrapState::NoWrap
            }
            AstNodes::NewExpression(new) if new.is_argument_span(self.span()) => WrapState::NoWrap,
            AstNodes::ExpressionStatement(stmt) => {
                // `() => <div></div>`
                //        ^^^^^^^^^^^
                if stmt.is_arrow_function_body() {
                    WrapState::WrapOnBreak
                } else {
                    WrapState::NoWrap
                }
            }
            AstNodes::ComputedMemberExpression(member) => {
                if member.optional {
                    WrapState::NoWrap
                } else {
                    WrapState::WrapOnBreak
                }
            }
            _ => WrapState::WrapOnBreak,
        }
    }
}

impl<'a> Format<'a> for AnyJsxTagWithChildren<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        let is_suppressed = f.comments().is_suppressed(self.span().start);

        let format_tag = format_with(|f| {
            if is_suppressed {
                return FormatSuppressedNode(self.span()).fmt(f);
            }

            let format_opening = format_with(|f| self.fmt_opening(f));
            let format_closing = format_with(|f| self.fmt_closing(f));

            let layout = self.layout();

            match layout {
                ElementLayout::NoChildren => {
                    write!(f, [format_opening, format_closing]);
                }
                ElementLayout::Template(expression) => {
                    write!(f, [format_opening, expression, format_closing]);
                }
                ElementLayout::Default => {
                    let format_opening = format_opening.memoized();
                    let opening_breaks = format_opening.inspect(f).will_break();

                    let multiple_attributes = match self {
                        Self::Element(element) => element.opening_element.attributes.len() > 1,
                        Self::Fragment(_) => false,
                    };

                    let list_layout = if multiple_attributes || opening_breaks {
                        JsxChildListLayout::Multiline
                    } else {
                        JsxChildListLayout::BestFitting
                    };

                    let children = self.children();
                    let format_children = FormatJsxChildList::default()
                        .with_options(list_layout)
                        .fmt_children(children, f);

                    match format_children {
                        FormatChildrenResult::ForceMultiline(multiline) => {
                            write!(f, [format_opening, multiline, format_closing]);
                        }
                        FormatChildrenResult::BestFitting { flat_children, expanded_children } => {
                            let format_closing = format_closing.memoized();
                            write!(
                                f,
                                [best_fitting![
                                    format_args!(format_opening, flat_children, format_closing),
                                    format_args!(format_opening, expanded_children, format_closing)
                                ]]
                            );
                        }
                    }
                }
            }
        });

        // It's a nested JSX element or fragment, no need for parenthesis or wrapping.
        if matches!(self.parent(), AstNodes::JSXElement(_) | AstNodes::JSXFragment(_)) {
            return write!(f, [format_tag]);
        }

        let wrap = self.get_wrap_state();
        match wrap {
            WrapState::NoWrap => {
                write!(
                    f,
                    [
                        &format_with(|f| { self.format_leading_comments(f) }),
                        format_tag,
                        &format_with(|f| { self.format_trailing_comments(f) }),
                    ]
                );
            }
            WrapState::WrapOnBreak => {
                let should_expand = should_expand(self.parent());
                let needs_parentheses = self.needs_parentheses(f);

                let format_inner = format_with(|f| {
                    if !needs_parentheses {
                        write!(f, [if_group_breaks(&token("("))]);
                    }

                    write!(
                        f,
                        [soft_block_indent(&format_args!(
                            &format_with(|f| { self.format_leading_comments(f) }),
                            format_tag,
                            &format_with(|f| { self.format_trailing_comments(f) }),
                        ))]
                    );

                    if !needs_parentheses {
                        write!(f, [if_group_breaks(&token(")"))]);
                    }
                });

                write!(f, [group(&format_inner).should_expand(should_expand)]);
            }
        }
    }
}

/// This is a very special situation where we're returning a JsxElement
/// from an arrow function that's passed as an argument to a function,
/// which is itself inside a JSX expression child.
///
/// If you're wondering why this is the only other case, it's because
/// Prettier defines it to be that way.
///
/// ```jsx
///  let bar = <div>
///    {foo(() => <div> the quick brown fox jumps over the lazy dog </div>)}
///  </div>;
/// ```
pub fn should_expand(mut parent: &AstNodes<'_>) -> bool {
    if let AstNodes::ExpressionStatement(stmt) = parent {
        // If the parent is a JSXExpressionContainer, we need to check its parent
        // to determine if it should expand.
        parent = stmt.grand_parent();
    }
    let maybe_jsx_expression_child = match parent {
        AstNodes::ArrowFunctionExpression(arrow) if arrow.expression => match arrow.parent {
            AstNodes::CallExpression(call) => call.parent,
            _ => return false,
        },
        _ => return false,
    };
    matches!(
        maybe_jsx_expression_child.without_chain_expression(),
        AstNodes::JSXExpressionContainer(container)
        if matches!(container.parent, AstNodes::JSXElement(_) | AstNodes::JSXFragment(_))
    )
}

impl<'a, 'b> AnyJsxTagWithChildren<'a, 'b> {
    fn fmt_opening(&self, f: &mut Formatter<'_, 'a>) {
        match self {
            Self::Element(element) => {
                let is_self_closing = element.closing_element().is_none();
                let opening_formatter =
                    FormatOpeningElement::new(element.opening_element(), is_self_closing);
                write!(f, opening_formatter);
            }
            Self::Fragment(fragment) => {
                write!(f, fragment.opening_fragment());
            }
        }
    }

    fn fmt_closing(&self, f: &mut Formatter<'_, 'a>) {
        match self {
            Self::Element(element) => {
                write!(f, element.closing_element());
            }
            Self::Fragment(fragment) => {
                write!(f, fragment.closing_fragment());
            }
        }
    }

    fn children(&self) -> &'b AstNode<'a, Vec<'a, JSXChild<'a>>> {
        match self {
            Self::Element(element) => element.children(),
            Self::Fragment(fragment) => fragment.children(),
        }
    }

    fn parent(&self) -> &'b AstNodes<'a> {
        match self {
            Self::Element(element) => element.parent,
            Self::Fragment(fragment) => fragment.parent,
        }
    }

    fn needs_parentheses(&self, f: &Formatter<'_, 'a>) -> bool {
        match self {
            Self::Element(element) => element.needs_parentheses(f),
            Self::Fragment(fragment) => fragment.needs_parentheses(f),
        }
    }

    fn layout(&self) -> ElementLayout<'a, 'b> {
        let children = self.children();

        match children.len() {
            0 => ElementLayout::NoChildren,
            1 => {
                // Safe because of length check above
                let child = children.first().unwrap();

                match child.as_ast_nodes() {
                    AstNodes::JSXText(text) => {
                        if is_meaningful_jsx_text(&text.value) {
                            ElementLayout::Default
                        } else {
                            ElementLayout::NoChildren
                        }
                    }
                    AstNodes::JSXExpressionContainer(expression) => match &expression.expression {
                        JSXExpression::TemplateLiteral(_) => ElementLayout::Template(expression),
                        JSXExpression::TaggedTemplateExpression(_) => {
                            ElementLayout::Template(expression)
                        }
                        _ => ElementLayout::Default,
                    },
                    _ => ElementLayout::Default,
                }
            }
            _ => ElementLayout::Default,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ElementLayout<'a, 'b> {
    /// Empty Tag with no children or contains no meaningful text.
    NoChildren,

    /// Prefer breaking the template if it is the only child of the element
    /// ```javascript
    /// <div>{`A Long Template String That uses ${
    ///   5 + 4
    /// } that will eventually break across multiple lines ${(40 / 3) * 45}`}</div>;
    /// ```
    ///
    /// instead of
    ///
    /// ```javascript
    /// <div>
    ///   {`A Long Template String That uses ${
    ///     5 + 4
    ///   } that will eventually break across multiple lines ${(40 / 3) * 45}`}
    /// </div>;
    /// ```
    Template(&'b AstNode<'a, JSXExpressionContainer<'a>>),

    /// Default layout used for all elements that have children and [ElementLayout::Template] does not apply.
    ///
    /// ```javascript
    /// <Element2>
    ///   Some more content
    ///   <Sub />
    ///   <Sub />
    ///   <Sub />
    /// </Element2>;
    /// ```
    Default,
}

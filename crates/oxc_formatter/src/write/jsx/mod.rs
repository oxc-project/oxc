use oxc_allocator::Vec;
use oxc_ast::ast::*;

pub mod child_list;
pub mod element;
pub mod opening_element;

use child_list::{FormatChildrenResult, FormatJsxChildList, JsxChildListLayout};
use element::{AnyJsxTagWithChildren, ElementLayout};
use opening_element::{FormatOpeningElement, OpeningElementLayout};
use oxc_span::GetSpan;

use crate::{
    AttributePosition, Format, FormatResult,
    ast_nodes::{AstNode, AstNodes},
    format_args,
    formatter::{
        Comments, Formatter,
        prelude::*,
        trivia::{DanglingIndentMode, FormatDanglingComments, FormatTrailingComments},
    },
    utils::format_node_without_trailing_comments::FormatNodeWithoutTrailingComments,
    write,
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, JSXElement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        AnyJsxTagWithChildren::Element(self).fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXOpeningElement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        unreachable!("`AnyJsxTagWithChildren` will print it.")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXClosingElement<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let name = self.name();
        let mut name_has_leading_comment = false;
        let mut name_has_own_line_leading_comment = false;
        for leading_comment in f.comments().comments_before(name.span().start) {
            name_has_leading_comment = true;
            name_has_own_line_leading_comment =
                name_has_own_line_leading_comment || leading_comment.is_line();
        }

        let format_name = format_with(|f| {
            if name_has_own_line_leading_comment {
                write!(f, [hard_line_break()])?;
            } else if name_has_leading_comment {
                write!(f, [space()])?;
            }
            if name_has_own_line_leading_comment {
                write!(f, [block_indent(&name), hard_line_break()])
            } else {
                write!(f, [name])
            }
        });

        write!(f, ["</", &format_name, ">",])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXFragment<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        AnyJsxTagWithChildren::Fragment(self).fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXOpeningFragment> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let comments = f.context().comments().comments_before(self.span.end);
        let has_own_line_comment = comments.iter().any(|c| c.is_line());

        let format_comments = format_with(|f| {
            if has_own_line_comment {
                write!(f, [hard_line_break()])?;
            }

            write!(
                f,
                [FormatDanglingComments::Comments { comments, indent: DanglingIndentMode::None }]
            )
        });

        write!(
            f,
            ["<", indent(&format_comments), has_own_line_comment.then_some(hard_line_break()), ">"]
        )
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXClosingFragment> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let comments = f.context().comments().comments_before(self.span.end);
        let has_own_line_comment = comments.iter().any(|c| c.is_line());

        let format_comments = format_with(|f| {
            if has_own_line_comment {
                write!(f, [hard_line_break()])?;
            } else if !comments.is_empty() {
                write!(f, [space()])?;
            }

            write!(
                f,
                [FormatDanglingComments::Comments { comments, indent: DanglingIndentMode::None }]
            )
        });

        write!(
            f,
            [
                "</",
                indent(&format_comments),
                has_own_line_comment.then_some(hard_line_break()),
                ">"
            ]
        )
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXNamespacedName<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.namespace(), ":", self.name()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXMemberExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.object(), ".", self.property()])
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXExpressionContainer<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let has_comment = |f: &mut Formatter<'_, '_>| {
            let expression_span = self.expression.span();
            f.comments().has_comment_before(expression_span.start)
                || f.comments().has_comment_in_range(expression_span.end, self.span.end)
        };

        // Expression child
        if matches!(self.parent, AstNodes::JSXElement(_) | AstNodes::JSXFragment(_)) {
            if let JSXExpression::EmptyExpression(_) = self.expression {
                let comments = f.context().comments().comments_before(self.span.end);
                let has_line_comment = comments.iter().any(|c| c.is_line());

                write!(f, ["{"])?;

                if has_line_comment {
                    write!(
                        f,
                        [
                            FormatDanglingComments::Comments {
                                comments,
                                indent: DanglingIndentMode::Block
                            },
                            format_dangling_comments(self.span).with_block_indent(),
                            hard_line_break()
                        ]
                    )?;
                } else {
                    write!(
                        f,
                        [FormatDanglingComments::Comments {
                            comments,
                            indent: DanglingIndentMode::None
                        },]
                    )?;
                }

                write!(f, ["}"])
            } else {
                let comments = f.context().comments();
                let is_conditional_or_binary = matches!(
                    self.expression,
                    JSXExpression::ConditionalExpression(_)
                        | JSXExpression::LogicalExpression(_)
                        | JSXExpression::BinaryExpression(_)
                );

                let should_inline = !has_comment(f)
                    && (is_conditional_or_binary
                        || should_inline_jsx_expression(self, f.comments()));

                if should_inline {
                    write!(f, ["{", self.expression(), line_suffix_boundary(), "}"])
                } else {
                    write!(
                        f,
                        [group(&format_args!(
                            "{",
                            soft_block_indent(&self.expression()),
                            line_suffix_boundary(),
                            "}"
                        ))]
                    )
                }
            }
        } else {
            // JSXAttributeValue
            let should_inline = !has_comment(f) && should_inline_jsx_expression(self, f.comments());

            let format_expression = format_once(|f| {
                write!(f, FormatNodeWithoutTrailingComments(&self.expression()));
                let comments = f.context().comments().comments_before(self.span.end);
                write!(f, FormatTrailingComments::Comments(comments))
            });

            if should_inline {
                write!(f, ["{", format_expression, line_suffix_boundary(), "}"])
            } else {
                write!(
                    f,
                    [group(&format_args!(
                        "{",
                        soft_block_indent(&format_expression),
                        line_suffix_boundary(),
                        "}"
                    ))]
                )
            }
        }
    }
}

/// Tests if an expression inside of a [`JSXExpressionContainer`] should be inlined.
/// Good:
/// ```jsx
///  <ColorPickerPage
///     colors={[
///        "blue",
///        "brown",
///        "green",
///        "orange",
///        "purple",
///     ]} />
/// ```
///
/// Bad:
/// ```jsx
///  <ColorPickerPage
///     colors={
///       [
///         "blue",
///          "brown",
///         "green",
///         "orange",
///         "purple",
///       ]
///     } />
/// ```
pub fn should_inline_jsx_expression(
    container: &JSXExpressionContainer<'_>,
    comments: &Comments<'_>,
) -> bool {
    match &container.expression {
        JSXExpression::ArrayExpression(_)
        | JSXExpression::ObjectExpression(_)
        | JSXExpression::ArrowFunctionExpression(_)
        | JSXExpression::CallExpression(_)
        | JSXExpression::ImportExpression(_)
        | JSXExpression::MetaProperty(_)
        | JSXExpression::FunctionExpression(_)
        | JSXExpression::TemplateLiteral(_)
        | JSXExpression::TaggedTemplateExpression(_) => true,
        JSXExpression::ChainExpression(chain_expression) => {
            matches!(chain_expression.expression, ChainElement::CallExpression(_))
        }
        JSXExpression::AwaitExpression(await_expression) => {
            matches!(
                await_expression.argument,
                Expression::ArrayExpression(_)
                    | Expression::ObjectExpression(_)
                    | Expression::ArrowFunctionExpression(_)
                    | Expression::CallExpression(_)
                    | Expression::ImportExpression(_)
                    | Expression::MetaProperty(_)
                    | Expression::FunctionExpression(_)
                    | Expression::TemplateLiteral(_)
                    | Expression::TaggedTemplateExpression(_)
                    | Expression::JSXElement(_)
                    | Expression::JSXFragment(_)
            )
        }
        _ => false,
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXEmptyExpression> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        Ok(())
    }
}

impl<'a> Format<'a> for AstNode<'a, Vec<'a, JSXAttributeItem<'a>>> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let line_break = if f.options().attribute_position == AttributePosition::Multiline {
            hard_line_break()
        } else {
            soft_line_break_or_space()
        };

        f.join_with(&line_break).entries(self.iter()).finish()
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXAttribute<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.name())?;
        if let Some(value) = &self.value() {
            write!(f, ["=", value])?;
        }
        Ok(())
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXSpreadAttribute<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let comments = f.context().comments();
        let has_comment = comments.has_comment_before(self.argument.span().start)
            || comments.has_comment_in_range(self.argument.span().end, self.span.end);
        let format_inner = format_with(|f| {
            write!(f, [format_leading_comments(self.argument.span()), "..."])?;
            self.argument().fmt(f)
        });

        write!(f, ["{"])?;

        if has_comment {
            write!(f, [soft_block_indent(&format_inner)])?;
        } else {
            write!(f, [format_inner])?;
        }

        write!(f, "}")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXIdentifier<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.name().as_str()))
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXSpreadChild<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let comments = f.context().comments();
        let has_comment = comments.has_comment_before(self.expression.span().start)
            || comments.has_comment_in_range(self.expression.span().end, self.span.end);
        let format_inner = format_with(|f| {
            write!(f, [format_leading_comments(self.expression.span()), "..."])?;
            self.expression().fmt(f)
        });

        write!(f, "{")?;

        if has_comment {
            write!(f, [soft_block_indent(&format_inner)])?;
        } else {
            write!(f, [format_inner])?;
        }

        write!(f, "}")
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, JSXText<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, dynamic_text(self.value().as_str()))
    }
}

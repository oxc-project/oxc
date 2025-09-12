use std::ops::Deref;

use oxc_ast::ast::*;

use crate::{
    JsLabels, format_args,
    formatter::{
        Buffer, Format, FormatResult, Formatter, prelude::*, trivia::format_dangling_comments,
    },
    generated::ast_nodes::{AstNode, AstNodes},
    options::Expand,
    utils::member_chain::chain_member::FormatComputedMemberExpressionWithoutObject,
    write,
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'a, ComputedMemberExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, self.object())?;
        FormatComputedMemberExpressionWithoutObject(self).fmt(f)
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, StaticMemberExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        let is_member_chain = {
            let mut recording = f.start_recording();
            write!(recording, [self.object()])?;
            recording.stop().has_label(LabelId::of(JsLabels::MemberChain))
        };

        match layout(self, is_member_chain) {
            StaticMemberLayout::NoBreak => {
                let format_no_break =
                    format_with(|f| write!(f, [operator_token(self.optional()), self.property()]));

                if is_member_chain {
                    write!(f, [labelled(LabelId::of(JsLabels::MemberChain), &format_no_break)])
                } else {
                    write!(f, [format_no_break])
                }
            }
            StaticMemberLayout::BreakAfterObject => {
                write!(
                    f,
                    [group(&indent(&format_args!(
                        soft_line_break(),
                        operator_token(self.optional()),
                        self.property(),
                    )))]
                )
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum StaticMemberLayout {
    /// Forces that there's no line break between the object, operator, and member
    NoBreak,

    /// Breaks the static member expression after the object if the whole expression doesn't fit on a single line
    BreakAfterObject,
}

fn operator_token(optional: bool) -> &'static str {
    if optional { "?." } else { "." }
}

fn layout<'a>(
    node: &AstNode<'a, StaticMemberExpression<'a>>,
    is_member_chain: bool,
) -> StaticMemberLayout {
    let parent = node.parent;
    let object = &node.object;

    let is_nested = match parent {
        AstNodes::AssignmentExpression(_) | AstNodes::VariableDeclarator(_) => {
            let no_break = match object {
                Expression::CallExpression(call_expression) => {
                    !call_expression.arguments.is_empty()
                }
                Expression::TSNonNullExpression(non_null_assertion) => {
                    match &non_null_assertion.expression {
                        Expression::CallExpression(call_expression) => {
                            !call_expression.arguments.is_empty()
                        }
                        _ => false,
                    }
                }
                _ => false,
            };

            if no_break || is_member_chain {
                return StaticMemberLayout::NoBreak;
            }
            true
        }
        AstNodes::StaticMemberExpression(_) | AstNodes::ComputedMemberExpression(_) => true,
        _ => false,
    };

    if !is_nested && matches!(object, Expression::Identifier(_)) {
        return StaticMemberLayout::NoBreak;
    }

    let mut first_non_static_member_ancestor = parent;
    while matches!(
        first_non_static_member_ancestor,
        AstNodes::StaticMemberExpression(_) | AstNodes::ComputedMemberExpression(_)
    ) {
        first_non_static_member_ancestor = first_non_static_member_ancestor.parent();
    }

    match first_non_static_member_ancestor {
        AstNodes::NewExpression(_) => StaticMemberLayout::NoBreak,
        AstNodes::JSXExpressionContainer(_) => {
            // Strategic fix: Keep member expressions inline in JSX text contexts
            StaticMemberLayout::NoBreak
        }
        AstNodes::AssignmentExpression(assignment) => {
            if matches!(assignment.left, AssignmentTarget::AssignmentTargetIdentifier(_)) {
                StaticMemberLayout::BreakAfterObject
            } else {
                StaticMemberLayout::NoBreak
            }
        }
        _ => StaticMemberLayout::BreakAfterObject,
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, PrivateFieldExpression<'a>> {
    fn write(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        write!(f, [self.object(), self.optional().then_some("?"), ".", self.field()])
    }
}

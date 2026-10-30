use oxc_ast::ast::*;
use oxc_formatter_core::Format;
use oxc_span::{GetSpan, Span};

use crate::{
    JsLabels,
    ast_nodes::{AstNode, AstNodeKind, AstNodes},
    format_args,
    formatter::{JsFormatter, prelude::*, trivia::FormatLeadingComments},
    parentheses::NeedsParentheses,
    utils::{
        expression::as_call_expression_without_chain_wrappers,
        member_chain::chain_member::FormatComputedMemberExpressionWithoutObject,
        typecast::is_cast_target,
    },
    write,
};

use super::FormatWrite;

impl<'a> FormatWrite<'a> for AstNode<'_, 'a, ComputedMemberExpression<'a>> {
    fn write(&self, f: &mut JsFormatter<'_, 'a>) {
        write!(f, self.object());
        FormatComputedMemberExpressionWithoutObject(self).fmt(f);
    }
}

impl<'a> FormatWrite<'a> for AstNode<'_, 'a, StaticMemberExpression<'a>> {
    fn write(&self, f: &mut JsFormatter<'_, 'a>) {
        let printed_len = self.property.name.len();
        write_non_computed_member(self.object(), self.optional(), self.property(), printed_len, f);
    }
}

/// `a.#b` is laid out the same as `a.b`.
impl<'a> FormatWrite<'a> for AstNode<'_, 'a, PrivateFieldExpression<'a>> {
    fn write(&self, f: &mut JsFormatter<'_, 'a>) {
        // `#` + name
        let printed_len = self.field.name.len() + 1;
        write_non_computed_member(self.object(), self.optional(), self.field(), printed_len, f);
    }
}

/// Writes `a.b` or `a.#b`.
///
/// `printed_len` is the byte length the property prints with;
/// it equals the property's source length exactly when the source has no escape.
fn write_non_computed_member<'b, 'a, P: GetSpan>(
    object: AstNode<'b, 'a, Expression<'a>>,
    optional: bool,
    property: AstNode<'b, 'a, P>,
    printed_len: usize,
    f: &mut JsFormatter<'_, 'a>,
) where
    AstNode<'b, 'a, P>: Format<'a, JsFormatContext<'a>>,
{
    // Before the recording: printing the object consumes its cast comment
    let object_is_cast_target = is_cast_target(object.span(), f);
    let property_start = property.span().start;
    // The member expression itself
    let member = property.parent();

    if !object_is_cast_target
        && let Some(text) =
            single_word_member_source(&object, optional, property.span(), printed_len, member, f)
    {
        write!(f, text_without_whitespace(text));
        return;
    }

    let is_member_chain = {
        let start = f.elements().len();
        write!(f, [object]);
        f.elements()[start..].has_label(LabelId::of(JsLabels::MemberChain))
    };

    // The operator and the property, printed as their source text in one element when it is the same text
    let format_operator_and_property = format_with(|f| {
        if let Some(text) =
            operator_and_property_source(optional, property.span(), printed_len, member.span(), f)
        {
            write!(f, text_without_whitespace(text));
        } else {
            write!(f, [operator_token(optional), property]);
        }
    });

    match layout(&object, property_start, member, is_member_chain, object_is_cast_target, f) {
        StaticMemberLayout::NoBreak => {
            let format_no_break = format_with(|f| write!(f, [format_operator_and_property]));

            if is_member_chain {
                write!(f, [labelled(LabelId::of(JsLabels::MemberChain), &format_no_break)]);
            } else {
                write!(f, [format_no_break]);
            }
        }
        StaticMemberLayout::BreakAfterObject => {
            write!(
                f,
                [group(&indent(&format_args!(
                    soft_line_break(),
                    &format_with(|f| {
                        if f.comments().has_leading_own_line_comment(property_start) {
                            let comments = f.context().comments().comments_before(property_start);
                            write!(
                                f,
                                [FormatLeadingComments::Comments(comments), soft_line_break()]
                            );
                        }
                    }),
                    format_operator_and_property,
                )))]
            );
        }
    }
}

/// The source text of the operator and the property (`.b`, `?.#b`),
/// when the property prints exactly that text right after the operator:
/// - the operator directly precedes the property in the source
/// - the property's source has no escape (`printed_len` is its whole span)
/// - the property prints no comment, like its generated `fmt`'s quiet path
fn operator_and_property_source<'a>(
    optional: bool,
    property_span: Span,
    printed_len: usize,
    member_span: Span,
    f: &JsFormatter<'_, 'a>,
) -> Option<&'a str> {
    let operator = operator_token(optional);
    #[expect(clippy::cast_possible_truncation)] // The operator is at most 2 bytes
    let start = property_span.start.checked_sub(operator.len() as u32)?;
    (property_span.size() as usize == printed_len
        && f.source_text().bytes_range(start, property_span.start) == operator.as_bytes()
        && f.comments().is_quiet(property_span, || member_span, 0, false))
    .then(|| f.source_text().slice_range(start, property_span.end))
}

/// The source text of a whole `a.b` / `this.b` (`?.` and `#b` too),
/// when the member prints exactly that text on one line:
/// - the object is a single word printing its source text (no escape, no parentheses, no comment, like its quiet `fmt`)
/// - the operator and the property directly follow it (see [`operator_and_property_source`])
/// - the layout keeps them together ([`StaticMemberLayout::NoBreak`])
fn single_word_member_source<'a>(
    object: &AstNode<'_, 'a, Expression<'a>>,
    optional: bool,
    property_span: Span,
    printed_len: usize,
    member: &AstNodes<'_, 'a>,
    f: &JsFormatter<'_, 'a>,
) -> Option<&'a str> {
    let object_span = object.span();
    let word_len = match object.as_ref() {
        Expression::Identifier(identifier) => identifier.name.len(),
        Expression::ThisExpression(_) => "this".len(),
        _ => return None,
    };
    #[expect(clippy::cast_possible_truncation)] // The operator is at most 2 bytes
    let operator_len = operator_token(optional).len() as u32;
    let member_span = member.span();
    let needs_parentheses = || match object.as_kind() {
        AstNodeKind::IdentifierReference(identifier) => identifier.needs_parentheses(f),
        AstNodeKind::ThisExpression(this) => this.needs_parentheses(f),
        _ => true,
    };
    (object_span.size() as usize == word_len
        && object_span.end + operator_len == property_span.start
        && operator_and_property_source(optional, property_span, printed_len, member_span, f)
            .is_some()
        && f.comments().is_quiet(object_span, || member_span, 0, true)
        && !needs_parentheses()
        && matches!(
            layout(object, property_span.start, member, false, false, f),
            StaticMemberLayout::NoBreak
        ))
    .then(|| f.source_text().slice_range(object_span.start, property_span.end))
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

/// An identifier, `this` or `super`: a member of it is not split unless nested (see [`layout`]),
/// which `is_inert_operand` (`binary_like_expression.rs`) relies on.
pub(super) fn is_single_word_object(object: &Expression<'_>) -> bool {
    matches!(
        object,
        Expression::Identifier(_) | Expression::ThisExpression(_) | Expression::Super(_)
    )
}

/// `object_is_cast_target`:
/// a cast-target object is neither a call nor an identifier to the rules below (see `is_cast_target`).
fn layout<'a>(
    object: &Expression<'a>,
    property_start: u32,
    member: &AstNodes<'_, 'a>,
    is_member_chain: bool,
    object_is_cast_target: bool,
    f: &JsFormatter<'_, 'a>,
) -> StaticMemberLayout {
    if f.comments().has_leading_own_line_comment(property_start) {
        return StaticMemberLayout::BreakAfterObject;
    }

    // `a.b.c!` and `a.b?.c`
    // `TSNonNullExpression` is a wrapper node for `!`, and `ChainExpression` is a wrapper node for `?.`,
    // so we need to skip them to find the real parent node.
    let mut parent = member.parent();
    while matches!(parent, AstNodes::TSNonNullExpression(_) | AstNodes::ChainExpression(_)) {
        parent = parent.parent();
    }

    let is_nested = match parent {
        AstNodes::AssignmentExpression(_) | AstNodes::VariableDeclarator(_) => {
            let no_break = !object_is_cast_target
                && as_call_expression_without_chain_wrappers(object)
                    .is_some_and(|call| !call.arguments.is_empty());

            if no_break || is_member_chain {
                return StaticMemberLayout::NoBreak;
            }

            false
        }
        _ => parent.is_member_expression(),
    };

    // A single-level member is not split when its object is a single word (an identifier, `this` or `super`):
    // the break would leave the word alone on its line for a single property.
    // A longer path (`a.b.c`) can break into a list of properties instead.
    // NOTE: `import.meta` and `new.target` cannot break inside either, so the same reasoning applies to them.
    // They are not included yet, and break like Prettier (`import.meta` + break + `.resolve(...)`).
    if !is_nested && !object_is_cast_target && is_single_word_object(object) {
        return StaticMemberLayout::NoBreak;
    }

    let mut first_non_static_member_ancestor = parent;
    while first_non_static_member_ancestor.is_member_expression()
        || matches!(
            first_non_static_member_ancestor,
            AstNodes::ChainExpression(_) | AstNodes::TSNonNullExpression(_)
        )
    {
        first_non_static_member_ancestor = first_non_static_member_ancestor.parent();
    }

    match first_non_static_member_ancestor {
        AstNodes::NewExpression(expr) if expr.callee.span().contains_inclusive(member.span()) => {
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

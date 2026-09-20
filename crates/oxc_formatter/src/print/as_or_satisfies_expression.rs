use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{
    ast_nodes::{AstNode, AstNodes},
    format_args,
    formatter::{prelude::*, trivia::FormatTrailingComments},
    print::{FormatNodeWithoutTrailingComments, FormatWrite, union_prints_itself},
    write,
};

impl<'a> FormatWrite<'a> for AstNode<'a, TSAsExpression<'a>> {
    fn write(&self, f: &mut JsFormatter<'_, 'a>) {
        let is_callee_or_object = is_callee_or_object_context(self.span(), self.parent());
        format_as_or_satisfies_expression(
            self.expression(),
            self.type_annotation(),
            is_callee_or_object,
            "as",
            f,
        );
    }
}

impl<'a> FormatWrite<'a> for AstNode<'a, TSSatisfiesExpression<'a>> {
    fn write(&self, f: &mut JsFormatter<'_, 'a>) {
        let is_callee_or_object = is_callee_or_object_context(self.span(), self.parent());
        format_as_or_satisfies_expression(
            self.expression(),
            self.type_annotation(),
            is_callee_or_object,
            "satisfies",
            f,
        );
    }
}

fn format_as_or_satisfies_expression<'a>(
    expression: &AstNode<'a, Expression>,
    type_annotation: &AstNode<'a, TSType>,
    is_callee_or_object: bool,
    operation: &'static str,
    f: &mut JsFormatter<'_, 'a>,
) {
    let format_inner = format_with(|f| {
        let expression_end = expression.span().end;
        let type_start = type_annotation.span().start;

        // The operator gap's comment slots are grammar-defined:
        // no line terminator may precede the operator (a multiline comment's interior counts),
        // so once the expression's source parens are dropped,
        // only same-line single-line block comments can stay on the expression side.
        // Riding line comment flushes behind the operator, everything else normalizes to the type side.
        // Within those slots, comments keep their source side and their line-start side,
        // the head-body comment policy applied to the operator gap (see DIVERGENCES.md#binary-cast-own-line-comment).
        let comments = f.context().comments().comments_in_range(expression_end, type_start);
        // A suppression comment must stay visible to the type it targets:
        // leave the whole gap to the type's leading pass instead of consuming it
        // (the break decision below still applies, keeping own-line ones own-line)
        let has_suppression_comment =
            comments.iter().any(|c| f.context().comments().is_suppression_comment(c));
        let before_operator_count = if comments.is_empty() {
            0
        } else {
            // The gap holds only trivia and `)`, so the operator's first byte splits the comments at the operator
            f.context()
                .comments()
                .comments_before_character(expression_end, operation.as_bytes()[0])
                .len()
        };
        // Both scans start from the unprinted cursor at `expression_end`, so the count indexes `comments`
        debug_assert!(before_operator_count <= comments.len());
        let pre_glued_count = comments[..before_operator_count]
            .iter()
            .take_while(|c| !c.preceded_by_newline() && !c.is_multiline_block())
            .count();
        // Comments the pre-operator slot cannot hold; they print on the type side
        let pre_moved = &comments[pre_glued_count..before_operator_count];

        // A union claims the after-operator run and the moved pre-operator comments as its leading comments
        // and places them itself (see the cast arms of `should_indent` in `union_type.rs`);
        // this site then breaks only for a riding line comment before the operator.
        let is_union = union_prints_itself(type_annotation.as_ref(), f.comments());

        let after_operator_comments = &comments[before_operator_count..];
        // The run still on the operator's line;
        // a line-ending multiline block is promoted out of it (printed own-line above the type).
        // With unprinted pre-operator comments pending, nothing glues: everything leads the type in source order.
        let glued_count = if is_union || !pre_moved.is_empty() {
            0
        } else {
            after_operator_comments
                .iter()
                .take_while(|c| {
                    !(c.preceded_by_newline()
                        || (c.is_multiline_block() && c.followed_by_newline()))
                })
                .count()
        };
        let promoted = |c: &Comment| {
            c.followed_by_newline() && (c.preceded_by_newline() || c.is_multiline_block())
        };
        let type_on_own_line = comments[..pre_glued_count].iter().any(|c| c.is_line())
            || after_operator_comments[..glued_count].iter().any(|c| c.is_line())
            || (!is_union
                && (pre_moved.iter().any(promoted)
                    || after_operator_comments[glued_count..].iter().any(promoted)));

        // The non-glued rest stays unprinted and leads the type;
        // the expression must not claim the after-operator comments (it would pull them backward across the operator)
        write!(f, [FormatNodeWithoutTrailingComments(expression)]);
        if !has_suppression_comment {
            write!(f, [FormatTrailingComments::Comments(&comments[..pre_glued_count])]);
        }
        write!(f, [space(), token(operation)]);
        if !has_suppression_comment {
            write!(f, [FormatTrailingComments::Comments(&after_operator_comments[..glued_count])]);
        }
        if type_on_own_line {
            // The hard break also flushes a glued riding line comment
            write!(f, [indent(&format_args!(hard_line_break(), type_annotation))]);
        } else {
            write!(f, [space(), type_annotation]);
        }
    });

    if is_callee_or_object {
        write!(f, [group(&soft_block_indent(&format_inner))]);
    } else {
        write!(f, [format_inner]);
    }
}

fn is_callee_or_object_context(span: Span, parent: &AstNodes<'_>) -> bool {
    match parent {
        // Static member
        AstNodes::StaticMemberExpression(_) => true,
        AstNodes::ComputedMemberExpression(member) => member.object.span() == span,
        // Or CallExpression callee (Not NewExpression, to align with Prettier)
        // https://github.com/prettier/prettier/blob/fdfa6701767f5140a85902ecc9fb6444f5b4e3f8/src/language-js/print/cast-expression.js#L28-L33
        // NOTE: We may revert this if resolved: https://github.com/prettier/prettier/issues/18406
        // _ => parent.is_call_like_callee_span(span),
        AstNodes::CallExpression(call) => call.callee.span() == span,
        _ => false,
    }
}

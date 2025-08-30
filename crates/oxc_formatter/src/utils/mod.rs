pub mod assignment_like;
pub mod call_expression;
pub mod conditional;
pub mod expression;
pub mod jsx;
pub mod member_chain;
pub mod object;
pub mod string_utils;
pub mod suppressed;

use oxc_allocator::Address;
use oxc_ast::{AstKind, ast::CallExpression};
use oxc_span::{GetSpan, Span};

use crate::{
    Format, FormatResult, FormatTrailingCommas, format_args,
    formatter::{Formatter, prelude::soft_line_break_or_space},
    generated::ast_nodes::{AstNode, AstNodes},
};

/// Tests if expression is a long curried call
///
/// ```javascript
/// `connect(a, b, c)(d)`
/// ```
pub fn is_long_curried_call(call: &AstNode<'_, CallExpression<'_>>) -> bool {
    if let AstNodes::CallExpression(parent_call) = call.parent {
        let parent_args_len = parent_call.arguments().len();
        // Fast path: if parent has no arguments, it's not a curried call
        if parent_args_len == 0 {
            return false;
        }
        return call.arguments().len() > parent_args_len;
    }

    false
}

/// Context-aware version that can use argument context when available during formatting
pub fn is_expression_used_as_call_argument_fast(
    span: Span,
    parent: &AstNodes,
    f: &Formatter<'_, '_>,
) -> bool {
    // Fast path: Use O(1) context check when available during formatting
    if f.is_in_arguments() {
        match parent {
            AstNodes::CallExpression(call) => {
                // If this is the callee, it's not an argument
                if call.callee.span() == span {
                    return false;
                }
                // If we're in argument context and this isn't the callee, it's likely an argument
                return true;
            }
            AstNodes::NewExpression(new_expr) => {
                // If this is the callee, it's not an argument
                if new_expr.callee.span() == span {
                    return false;
                }
                // If we're in argument context and this isn't the callee, it's likely an argument
                return true;
            }
            _ => {
                // For other parent types, fall through to span-based detection
            }
        }
    }

    // Fallback to existing span-based detection when context isn't available
    is_expression_used_as_call_argument(span, parent)
}

/// Check if an expression is used as a call argument by examining the parent node.
/// This replaces the missing AstKind::Argument detection capability.
pub fn is_expression_used_as_call_argument(span: Span, parent: &AstNodes) -> bool {
    match parent {
        AstNodes::CallExpression(call) => {
            // Fast path: if callee matches, it's not an argument
            if call.callee.span() == span {
                return false;
            }
            // Only check arguments if there are any
            if call.arguments.is_empty() {
                return false;
            }

            // Phase 1 Optimization: Single argument fast path
            if call.arguments.len() == 1 {
                let arg_span = call.arguments[0].span();
                return arg_span == span || arg_span.contains_inclusive(span);
            }

            // Phase 1 Optimization: Span bounds checking
            if let (Some(first), Some(last)) = (call.arguments.first(), call.arguments.last()) {
                let first_start = first.span().start;
                let last_end = last.span().end;

                // If target span is completely outside the arguments range, it can't be an argument
                if span.end < first_start || span.start > last_end {
                    return false;
                }
            }

            // Phase 1 Optimization: Early exit iteration with span equality first
            for arg in &call.arguments {
                let arg_span = arg.span();
                if arg_span == span {
                    return true; // Exact match - most common case
                }
                if arg_span.contains_inclusive(span) {
                    return true; // Containment match
                }
            }
            false
        }
        AstNodes::NewExpression(new_expr) => {
            // Fast path: if callee matches, it's not an argument
            if new_expr.callee.span() == span {
                return false;
            }
            // Only check arguments if there are any
            if new_expr.arguments.is_empty() {
                return false;
            }

            // Phase 1 Optimization: Single argument fast path
            if new_expr.arguments.len() == 1 {
                let arg_span = new_expr.arguments[0].span();
                return arg_span == span || arg_span.contains_inclusive(span);
            }

            // Phase 1 Optimization: Span bounds checking
            if let (Some(first), Some(last)) =
                (new_expr.arguments.first(), new_expr.arguments.last())
            {
                let first_start = first.span().start;
                let last_end = last.span().end;

                // If target span is completely outside the arguments range, it can't be an argument
                if span.end < first_start || span.start > last_end {
                    return false;
                }
            }

            // Phase 1 Optimization: Early exit iteration with span equality first
            for arg in &new_expr.arguments {
                let arg_span = arg.span();
                if arg_span == span {
                    return true; // Exact match - most common case
                }
                if arg_span.contains_inclusive(span) {
                    return true; // Containment match
                }
            }
            false
        }
        _ => false,
    }
}

pub mod array;
pub mod assignment_like;
pub mod call_expression;
pub mod conditional;
pub mod expression;
pub mod format_node_without_trailing_comments;
pub mod jsx;
pub mod member_chain;
pub mod object;
pub mod statement_body;
pub mod string_utils;
pub mod suppressed;
pub mod typecast;
pub mod typescript;

use oxc_allocator::Address;
use oxc_ast::{AstKind, ast::CallExpression};
use oxc_span::{GetSpan, Span};

use crate::{
    Format, FormatResult, FormatTrailingCommas,
    ast_nodes::{AstNode, AstNodes},
    format_args,
    formatter::{Formatter, prelude::soft_line_break_or_space},
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

/// Check if an expression is used as a call argument by examining the parent node.
#[inline(always)]
pub fn is_expression_used_as_call_argument(span: Span, parent: &AstNodes) -> bool {
    match parent {
        AstNodes::CallExpression(call) => {
            if call.arguments.is_empty() {
                return false;
            }
            if call.callee.span().eq_fast(span) {
                return false;
            }

            // Unrolled loop optimization for common argument counts (95% of cases)
            match call.arguments.len() {
                1 => {
                    // Single argument: most common case after empty
                    let arg_span = call.arguments[0].span();
                    arg_span.eq_fast(span) || arg_span.contains_inclusive(span)
                }
                2 => {
                    // Two arguments: second most common
                    let arg0_span = call.arguments[0].span();
                    let arg1_span = call.arguments[1].span();
                    arg0_span.eq_fast(span)
                        || arg0_span.contains_inclusive(span)
                        || arg1_span.eq_fast(span)
                        || arg1_span.contains_inclusive(span)
                }
                3 => {
                    // Three arguments: unroll for cache efficiency
                    let spans = [
                        call.arguments[0].span(),
                        call.arguments[1].span(),
                        call.arguments[2].span(),
                    ];
                    spans.iter().any(|&arg_span| {
                        arg_span.eq_fast(span) || arg_span.contains_inclusive(span)
                    })
                }
                _ => {
                    // Rare case (>3 arguments): use cold path
                    check_many_arguments_cold(span, &call.arguments)
                }
            }
        }
        AstNodes::NewExpression(new_expr) => {
            // Branch prediction: Most expressions are not arguments
            if new_expr.arguments.is_empty() {
                return false;
            }
            if new_expr.callee.span().eq_fast(span) {
                return false;
            }

            // Same unrolled optimization as CallExpression
            match new_expr.arguments.len() {
                1 => {
                    let arg_span = new_expr.arguments[0].span();
                    arg_span.eq_fast(span) || arg_span.contains_inclusive(span)
                }
                2 => {
                    let arg0_span = new_expr.arguments[0].span();
                    let arg1_span = new_expr.arguments[1].span();
                    arg0_span.eq_fast(span)
                        || arg0_span.contains_inclusive(span)
                        || arg1_span.eq_fast(span)
                        || arg1_span.contains_inclusive(span)
                }
                3 => {
                    let spans = [
                        new_expr.arguments[0].span(),
                        new_expr.arguments[1].span(),
                        new_expr.arguments[2].span(),
                    ];
                    spans.iter().any(|&arg_span| {
                        arg_span.eq_fast(span) || arg_span.contains_inclusive(span)
                    })
                }
                _ => check_many_arguments_cold(span, &new_expr.arguments),
            }
        }
        _ => false,
    }
}

/// Cold path for checking many arguments - rarely called, optimized for code size not speed
#[cold]
#[inline(never)]
fn check_many_arguments_cold(span: Span, arguments: &[oxc_ast::ast::Argument]) -> bool {
    // Iterator for rare complex cases (>3 arguments)
    arguments.iter().any(|arg| {
        let arg_span = arg.span();
        arg_span.eq_fast(span) || arg_span.contains_inclusive(span)
    })
}

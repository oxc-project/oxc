pub mod assignment_like;
pub mod call_expression;
pub mod conditional;
pub mod expression;
pub mod member_chain;
pub mod object;
pub mod string_utils;

use oxc_allocator::Address;
use oxc_ast::{AstKind, ast::CallExpression};
use oxc_span::{GetSpan, Span};

use crate::{
    Format, FormatResult, FormatTrailingCommas, format_args,
    formatter::{Formatter, prelude::soft_line_break_or_space},
    generated::ast_nodes::{AstNode, AstNodes},
};

/// This function is in charge to format the call arguments.
pub fn write_arguments_multi_line<'a, S: Format<'a>, I>(
    separated: I,
    f: &mut Formatter<'_, 'a>,
) -> FormatResult<()>
where
    I: Iterator<Item = S>,
{
    let mut iterator = separated.peekable();
    let mut join_with = f.join_with(soft_line_break_or_space());

    // Fast path for empty arguments
    if iterator.peek().is_none() {
        return join_with.finish();
    }

    while let Some(element) = iterator.next() {
        let is_last = iterator.peek().is_none();

        if is_last {
            join_with.entry(&format_args!(element, FormatTrailingCommas::All));
        } else {
            join_with.entry(&element);
        }
    }

    join_with.finish()
}

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
            // Use direct span equality first (faster), fall back to contains_inclusive only if needed
            call.arguments.iter().any(|arg| {
                let arg_span = arg.span();
                arg_span == span || arg_span.contains_inclusive(span)
            })
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
            // Use direct span equality first (faster), fall back to contains_inclusive only if needed
            new_expr.arguments.iter().any(|arg| {
                let arg_span = arg.span();
                arg_span == span || arg_span.contains_inclusive(span)
            })
        }
        _ => false,
    }
}

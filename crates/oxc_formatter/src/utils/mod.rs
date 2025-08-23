pub mod assignment_like;
pub mod call_expression;
pub mod conditional;
pub mod expression;
pub mod member_chain;
pub mod object;
pub mod string_utils;

use oxc_allocator::Address;
use oxc_ast::{AstKind, ast::CallExpression};

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

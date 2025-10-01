pub mod assignment_like;
pub mod call_expression;
pub mod conditional;
pub mod format_node_without_trailing_comments;
pub mod jsx;
pub mod member_chain;
pub mod object;
pub mod string_utils;
pub mod suppressed;
pub mod typecast;
pub mod typescript;

use oxc_allocator::Address;
use oxc_ast::{AstKind, ast::CallExpression};

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
        return call.arguments().len() > parent_call.arguments().len()
            && !parent_call.arguments().is_empty();
    }

    false
}

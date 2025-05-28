pub mod member_chain;

use oxc_allocator::Address;
use oxc_ast::{AstKind, ast::CallExpression};

use crate::{
    Format, FormatResult, FormatTrailingCommas, format_args,
    formatter::{Formatter, parent_stack::ParentStack, prelude::soft_line_break_or_space},
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

    while let Some(element) = iterator.next() {
        let last = iterator.peek().is_none();

        if last {
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
pub fn is_long_curried_call(parent: AstKind<'_>, f: &mut Formatter<'_, '_>) -> bool {
    if let AstKind::CallExpression(call) = parent {
        if let Some(parent_call) = f.parent_kind_of(Address::from_ptr(call)).as_call_expression() {
            return call.arguments.len() > parent_call.arguments.len()
                && !parent_call.arguments.is_empty();
        }
    }

    false
}

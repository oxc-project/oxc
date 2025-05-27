use oxc_ast::ast::CallExpression;

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
pub fn is_long_curried_call(parent_stack: &ParentStack<'_>) -> bool {
    // if let Some(expression) = expression {
    //     if let Some(parent_call) = expression.parent::<JsCallExpression>() {
    //         if let (Ok(arguments), Ok(parent_arguments)) =
    //             (expression.arguments(), parent_call.arguments())
    //         {
    //             let is_callee = matches!(
    //                 parent_call.syntax().kind(),
    //                 JsSyntaxKind::JS_CALL_EXPRESSION | JsSyntaxKind::JS_NEW_EXPRESSION
    //             );
    //             return is_callee
    //                 && arguments.args().len() > parent_arguments.args().len()
    //                 && !parent_arguments.args().is_empty();
    //         }
    //     }
    // }

    false
}

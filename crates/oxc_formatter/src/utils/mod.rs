pub mod assignment_like;
pub mod call_expression;
pub mod conditional;
pub mod expression;
pub mod jsx;
pub mod member_chain;
pub mod object;
pub mod string_utils;
pub mod suppressed;
pub mod typecast;

use oxc_allocator::Address;
use oxc_ast::{AstKind, ast::*};
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

/// Helper function to check if an AST node is used as an argument\
/// This replaces the missing AstNodes::Argument(_) functionality
pub fn is_node_argument(span: Span, parent: &AstNodes) -> bool {
    is_expression_used_as_call_argument(span, parent)
}

/// Check if an expression is used as a call argument by examining the parent node.
/// Comprehensive approach that replaces AstNodes::Argument behavior fully.
pub fn is_expression_used_as_call_argument(span: Span, parent: &AstNodes) -> bool {
    match parent {
        AstNodes::CallExpression(call) => {
            if call.callee.span() == span {
                return false; // This is the callee, not an argument
            }

            // Check every argument for exact or containment match
            call.arguments.iter().any(|arg| {
                let arg_span = arg.span();
                arg_span == span || arg_span.contains_inclusive(span)
            })
        }
        AstNodes::NewExpression(new_expr) => {
            if new_expr.callee.span() == span {
                return false; // This is the callee, not an argument
            }

            // Check every argument for exact or containment match
            new_expr.arguments.iter().any(|arg| {
                let arg_span = arg.span();
                arg_span == span || arg_span.contains_inclusive(span)
            })
        }
        // Expanded detection to replace AstKind::Argument functionality
        AstNodes::TemplateLiteral(template) => {
            // Template literal expressions can contain call arguments
            template
                .expressions
                .iter()
                .any(|expr| expr.span() == span || expr.span().contains_inclusive(span))
        }
        AstNodes::TaggedTemplateExpression(tagged) => {
            // Tagged template expressions can have call-like behavior
            tagged
                .quasi
                .expressions
                .iter()
                .any(|expr| expr.span() == span || expr.span().contains_inclusive(span))
        }
        AstNodes::ArrayExpression(array) => {
            // Array elements can be call arguments in functional programming patterns
            array.elements.iter().any(|elem| {
                let elem_span = elem.span();
                elem_span == span || elem_span.contains_inclusive(span)
            })
        }
        AstNodes::ObjectExpression(object) => {
            // Object property values can be call arguments
            object.properties.iter().any(|prop| match prop {
                ObjectPropertyKind::ObjectProperty(obj_prop) => {
                    obj_prop.value.span() == span || obj_prop.value.span().contains_inclusive(span)
                }
                ObjectPropertyKind::SpreadProperty(_) => false,
            })
        }
        _ => false,
    }
}

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
pub mod string;
pub mod suppressed;
pub mod tailwindcss;
pub mod typecast;
pub mod typescript;

use oxc_ast::ast::{CallExpression, PropertyKey};

use crate::ast_nodes::{AstNode, AstNodes};

/// Tests if the property key is an identifier named `static`, `get` or `set`,
/// which would parse as a modifier or accessor of the following member
/// if the separating semicolon were omitted, e.g. `get` + `<T>(): T` -> `get <T>(): T`.
/// Computed keys are never `StaticIdentifier`, so they need no extra check.
///
/// Used by the class and interface member no-semi rules, like Prettier's `isKeywordProperty`.
pub fn is_keyword_property_key(key: &PropertyKey<'_>) -> bool {
    matches!(key, PropertyKey::StaticIdentifier(ident) if matches!(ident.name.as_str(), "static" | "get" | "set"))
}

/// Tests if expression is a long curried call
///
/// ```javascript
/// `connect(a, b, c)(d)`
/// ```
pub fn is_long_curried_call(call: &AstNode<'_, CallExpression<'_>>) -> bool {
    if let AstNodes::CallExpression(parent_call) = call.parent()
        && parent_call.is_callee_span(call.span)
    {
        return call.arguments().len() > parent_call.arguments().len()
            && !parent_call.arguments().is_empty();
    }

    false
}

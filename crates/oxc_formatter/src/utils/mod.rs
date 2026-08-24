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

use oxc_ast::ast::{
    CallExpression, Declaration, Decorator, ExportDeclaration, ExportDefaultDeclaration,
    ExportDefaultDeclarationKind, PropertyKey, Statement,
};
use oxc_span::Span;

use crate::ast_nodes::{AstNode, AstNodes};

/// Statements the formatter drops from statement lists: they produce no output,
/// so following-span computation and emptiness checks must treat them as absent.
pub fn is_dropped_statement(stmt: &Statement<'_>) -> bool {
    matches!(stmt, Statement::EmptyStatement(_))
}

/// Span of an export statement extended over class decorators placed before the `export` keyword
/// (`@dec export class C {}`), which the parser leaves outside the export node's span.
pub fn export_declaration_span(export: &ExportDeclaration<'_>) -> Span {
    if let Declaration::ClassDeclaration(class) = &export.declaration {
        span_with_decorators_before_export(&class.decorators, export.span)
    } else {
        export.span
    }
}

/// See [`export_declaration_span`].
pub fn export_default_declaration_span(export: &ExportDefaultDeclaration<'_>) -> Span {
    if let ExportDefaultDeclarationKind::ClassDeclaration(class) = &export.declaration {
        span_with_decorators_before_export(&class.decorators, export.span)
    } else {
        export.span
    }
}

/// First-decorator start when the decorators precede the `export` keyword;
/// `None` for `export @dec class C {}` (decorators inside the export span).
pub fn decorators_before_export_start(decorators: &[Decorator<'_>], span: Span) -> Option<u32> {
    decorators.first().map(|decorator| decorator.span.start).filter(|start| *start < span.start)
}

fn span_with_decorators_before_export(decorators: &[Decorator<'_>], span: Span) -> Span {
    decorators_before_export_start(decorators, span)
        .map_or(span, |start| Span::new(start, span.end))
}

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

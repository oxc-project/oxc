//! Identifier disambiguation logic for ESTree to oxc AST conversion.

use super::context::ConversionContext;
use super::error::{ConversionError, ConversionResult, Span};
use super::types::EstreeIdentifier;

/// The kind of identifier in oxc AST.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentifierKind {
    Binding,
    Reference,
    Name,
    Label,
}

/// Convert an ESTree Identifier to the appropriate oxc identifier type.
///
/// This uses context-aware disambiguation with optional JavaScript-side hints.
/// The conversion follows this priority:
/// 1. JavaScript-added hint (`_oxc_identifierKind`) if present
/// 2. Context-based rules (parent node type + field name)
/// 3. Default to `IdentifierReference` (safest fallback)
pub fn convert_identifier(
    estree_id: &EstreeIdentifier,
    context: &ConversionContext,
    source_text: &str,
) -> ConversionResult<IdentifierKind> {
    // Check for JavaScript-added hint first (if present)
    // Format: _oxc_identifierKind (designed for potential future standardization)
    // ESLint-compatible: Unknown properties are ignored by ESLint
    if let Some(kind) = &estree_id._oxc_identifierKind {
        match kind.as_str() {
            "binding" => return Ok(IdentifierKind::Binding),
            "reference" => return Ok(IdentifierKind::Reference),
            "name" => return Ok(IdentifierKind::Name),
            "label" => return Ok(IdentifierKind::Label),
            _ => {
                // Invalid hint value, fall through to context-based
            }
        }
    }

    // Fall back to context-based conversion
    match (context.parent_type.as_deref(), context.field_name.as_deref()) {
        // Binding contexts - identifiers that declare/bind variables
        (Some("VariableDeclarator"), Some("id")) => Ok(IdentifierKind::Binding),
        (Some("FunctionDeclaration"), Some("id")) => Ok(IdentifierKind::Binding),
        (Some("FunctionExpression"), Some("id")) => Ok(IdentifierKind::Binding),
        (Some("ClassDeclaration"), Some("id")) => Ok(IdentifierKind::Binding),
        (Some("ClassExpression"), Some("id")) => Ok(IdentifierKind::Binding),
        (Some("CatchClause"), Some("param")) => Ok(IdentifierKind::Binding),
        (Some("Property"), Some("key")) if context.is_shorthand => Ok(IdentifierKind::Binding),
        // Object/Array destructuring patterns
        (Some("ObjectPattern"), Some("properties")) => Ok(IdentifierKind::Binding),
        (Some("ArrayPattern"), Some("elements")) => Ok(IdentifierKind::Binding),
        (Some("RestElement"), Some("argument")) => Ok(IdentifierKind::Binding),
        (Some("AssignmentPattern"), Some("left")) => Ok(IdentifierKind::Binding),
        // For loop patterns
        (Some("ForInStatement"), Some("left")) => Ok(IdentifierKind::Binding),
        (Some("ForOfStatement"), Some("left")) => Ok(IdentifierKind::Binding),

        // Name contexts - identifiers used as property names, not variable references
        (Some("MemberExpression"), Some("property")) if !context.is_computed => {
            Ok(IdentifierKind::Name)
        }
        (Some("Property"), Some("key")) if !context.is_shorthand => Ok(IdentifierKind::Name),
        (Some("MethodDefinition"), Some("key")) => Ok(IdentifierKind::Name),
        (Some("PropertyDefinition"), Some("key")) => Ok(IdentifierKind::Name),
        (Some("ExportSpecifier"), Some("exported")) => Ok(IdentifierKind::Name),
        (Some("ImportSpecifier"), Some("imported")) => Ok(IdentifierKind::Name),
        (Some("ImportDefaultSpecifier"), Some("local")) => Ok(IdentifierKind::Name),
        (Some("ImportNamespaceSpecifier"), Some("local")) => Ok(IdentifierKind::Name),

        // Label contexts - identifiers used as labels
        (Some("LabeledStatement"), Some("label")) => Ok(IdentifierKind::Label),
        (Some("BreakStatement"), Some("label")) => Ok(IdentifierKind::Label),
        (Some("ContinueStatement"), Some("label")) => Ok(IdentifierKind::Label),

        // Default: IdentifierReference (safest fallback)
        // This covers all expression contexts, call arguments, etc.
        _ => Ok(IdentifierKind::Reference),
    }
}

/// Get the span for an ESTree identifier as (start, end) byte offsets.
pub fn get_identifier_span(estree_id: &EstreeIdentifier) -> Span {
    estree_id
        .range
        .map(|r| (r[0] as u32, r[1] as u32))
        .unwrap_or((0, 0))
}


use oxc_ast::ast::{TSType, TSUnionType};
use oxc_span::GetSpan;

use crate::formatter::Formatter;

/// Check if a TSType is a simple type (primitives, keywords, simple references)
pub fn is_simple_type(ty: &TSType) -> bool {
    match ty {
        TSType::TSAnyKeyword(_)
        | TSType::TSNullKeyword(_)
        | TSType::TSThisType(_)
        | TSType::TSVoidKeyword(_)
        | TSType::TSNumberKeyword(_)
        | TSType::TSBooleanKeyword(_)
        | TSType::TSBigIntKeyword(_)
        | TSType::TSStringKeyword(_)
        | TSType::TSSymbolKeyword(_)
        | TSType::TSNeverKeyword(_)
        | TSType::TSObjectKeyword(_)
        | TSType::TSUndefinedKeyword(_)
        | TSType::TSTemplateLiteralType(_)
        | TSType::TSLiteralType(_)
        | TSType::TSUnknownKeyword(_) => true,
        TSType::TSTypeReference(reference) => {
            // Simple reference without type arguments
            reference.type_arguments.is_none()
        }
        _ => false,
    }
}

/// Check if a TSType is object-like (object literal, mapped type, etc.)
pub fn is_object_like_type(ty: &TSType) -> bool {
    matches!(ty, TSType::TSTypeLiteral(_) | TSType::TSMappedType(_))
}

pub fn should_hug_type(node: &TSUnionType<'_>, f: &Formatter<'_, '_>) -> bool {
    let types = &node.types;

    if types.len() == 1 {
        return true;
    }

    let has_object_type =
        types.iter().any(|t| matches!(t, TSType::TSTypeLiteral(_) | TSType::TSTypeReference(_)));

    if !has_object_type {
        return false;
    }

    let void_count = types
        .iter()
        .filter(|t| matches!(t, TSType::TSVoidKeyword(_) | TSType::TSNullKeyword(_)))
        .count();

    if types.len() - 1 != void_count {
        return false;
    }

    // `{ a: string } /* comment */ | null | /* comment */ */ undefined`
    //                ^^^^^^^^^^^^           ^^^^^^^^^^^^
    // Check whether there are comments between the types, if so, we should not hug
    let mut start = node.span.start;
    for t in types {
        if f.comments().has_comment_in_range(start, t.span().start) {
            return false;
        }
        start = t.span().end;
    }

    true
}

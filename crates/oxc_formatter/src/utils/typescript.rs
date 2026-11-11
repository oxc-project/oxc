use oxc_ast::ast::TSType;

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

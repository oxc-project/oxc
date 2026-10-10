//! Ports of internal/utils/ts_api_utils.go helpers, added as rules need them.

use super::*;

/// ts_api_utils.go IsTypeNullType.
pub fn is_type_null_type(t: P<Type>) -> bool {
    is_type_flag_set(t, TypeFlags::Null)
}

/// ts_api_utils.go IsTypeUndefinedType.
pub fn is_type_undefined_type(t: P<Type>) -> bool {
    is_type_flag_set(t, TypeFlags::Undefined)
}

/// ts_api_utils.go IsTypeVoidType.
pub fn is_type_void_type(t: P<Type>) -> bool {
    is_type_flag_set(t, TypeFlags::Void)
}

/// ts_api_utils.go IsNullLiteral.
pub fn is_null_literal(node: Option<P<Node>>) -> bool {
    node.is_some_and(|n| n.kind() == Kind::NullKeyword)
}

/// ts_api_utils.go IsUndefinedIdentifier.
pub fn is_undefined_identifier(node: Option<P<Node>>) -> bool {
    node.is_some_and(|n| ast::is_identifier(n) && n.text() == "undefined")
}

/// ts_api_utils.go IsVoidExpression.
pub fn is_void_expression(node: Option<P<Node>>) -> bool {
    node.is_some_and(ast::is_void_expression)
}

/// ts_api_utils.go IsUndefinedLiteral.
pub fn is_undefined_literal(node: Option<P<Node>>) -> bool {
    is_undefined_identifier(node) || is_void_expression(node)
}

/// ts_api_utils.go IsNullishLiteral.
pub fn is_nullish_literal(node: Option<P<Node>>) -> bool {
    is_null_literal(node) || is_undefined_literal(node)
}

/// ts_api_utils.go IsPropertyOrElementAccess.
pub fn is_property_or_element_access(node: P<Node>) -> bool {
    ast::is_property_access_expression(node) || ast::is_element_access_expression(node)
}

/// ts_api_utils.go IsAccessExpression.
pub fn is_access_expression(node: P<Node>) -> bool {
    ast::is_property_access_expression(node)
        || ast::is_element_access_expression(node)
        || ast::is_call_expression(node)
}

/// ts_api_utils.go GetConstructSignatures.
pub fn get_construct_signatures(c: &mut Checker, t: P<Type>) -> &'static [P<Signature>] {
    c.get_signatures_of_type(t, SignatureKind::Construct)
}

/// ts_api_utils.go IsIntrinsicType.
pub fn is_intrinsic_type(t: P<Type>) -> bool {
    is_type_flag_set(t, TypeFlags::Intrinsic)
}

/// ts_api_utils.go IsIntrinsicErrorType.
pub fn is_intrinsic_error_type(t: P<Type>) -> bool {
    is_intrinsic_type(t) && t.as_intrinsic_type().intrinsic_name() == "error"
}

/// ts_api_utils.go GetWellKnownSymbolPropertyOfType.
pub fn get_well_known_symbol_property_of_type(
    t: P<Type>,
    name: &str,
    c: &mut Checker,
) -> Option<P<Symbol>> {
    let property_name = c.get_property_name_for_known_symbol_name(name);
    c.get_property_of_type(t, &property_name)
}

/// ts_api_utils.go IsIntrinsicVoidType.
pub fn is_intrinsic_void_type(t: P<Type>) -> bool {
    is_type_flag_set(t, TypeFlags::Void)
}

/// ts_api_utils.go IsNullableType.
pub fn is_nullable_type(c: &mut Checker, t: P<Type>) -> bool {
    c.is_nullable_type(t)
}

/// ts_api_utils.go IsNullLiteralOrUndefinedIdentifier.
pub fn is_null_literal_or_undefined_identifier(node: Option<P<Node>>) -> bool {
    is_null_literal(node) || is_undefined_identifier(node)
}

/// ts_api_utils.go IsObjectType.
pub fn is_object_type(t: P<Type>) -> bool {
    is_type_flag_set(t, TypeFlags::Object)
}

/// ts_api_utils.go CollectAllCallSignatures (ex. getCallSignaturesOfType).
pub fn collect_all_call_signatures(c: &mut Checker, t: P<Type>) -> Vec<P<Signature>> {
    if is_union_type(t) {
        let mut signatures = Vec::new();
        for &subtype in t.types() {
            signatures.extend_from_slice(get_call_signatures(c, subtype));
        }
        return signatures;
    }
    if is_intersection_type(t) {
        let mut signatures: Option<&'static [P<Signature>]> = None;
        for &subtype in t.types() {
            let sig = get_call_signatures(c, subtype);
            if !sig.is_empty() {
                if signatures.is_some() {
                    return Vec::new();
                }
                signatures = Some(sig);
            }
        }
        return signatures.map(|s| s.to_vec()).unwrap_or_default();
    }
    c.get_signatures_of_type(t, SignatureKind::Call).to_vec()
}

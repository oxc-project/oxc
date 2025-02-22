use oxc_allocator::Allocator;
use oxc_ast::{AstBuilder, ast::*};
use oxc_ecmascript::{
    ToJsString,
    is_global_reference::{IsGlobalReference, WithoutGlobalReferenceInformation},
};
use oxc_span::SPAN;

struct GlobalReferenceInformation {
    is_undefined_shadowed: bool,
}

impl IsGlobalReference for GlobalReferenceInformation {
    fn is_global_reference(&self, ident: &IdentifierReference<'_>) -> Option<bool> {
        if ident.name == "undefined" { Some(!self.is_undefined_shadowed) } else { None }
    }
}

#[test]
fn test() {
    let allocator = Allocator::default();
    let ast = AstBuilder::new(&allocator);

    let undefined = ast.expression_identifier(SPAN, "undefined");
    let shadowed_undefined_string =
        undefined.to_js_string(&GlobalReferenceInformation { is_undefined_shadowed: true });
    let global_undefined_string =
        undefined.to_js_string(&GlobalReferenceInformation { is_undefined_shadowed: false });

    let empty_object = ast.expression_object(SPAN, ast.vec(), None);
    let object_with_to_string = ast.expression_object(
        SPAN,
        ast.vec1(ast.object_property_kind_object_property(
            SPAN,
            PropertyKind::Init,
            ast.property_key_static_identifier(SPAN, "toString"),
            ast.expression_string_literal(SPAN, "foo", None),
            false,
            false,
            false,
        )),
        None,
    );
    let empty_object_string = empty_object.to_js_string(&WithoutGlobalReferenceInformation {});
    let object_with_to_string_string =
        object_with_to_string.to_js_string(&WithoutGlobalReferenceInformation {});

    let bigint_with_separators = ast.expression_big_int_literal(SPAN, "1_0n", BigintBase::Decimal);
    let bigint_with_separators_string =
        bigint_with_separators.to_js_string(&WithoutGlobalReferenceInformation {});

    assert_eq!(shadowed_undefined_string, None);
    assert_eq!(global_undefined_string, Some("undefined".into()));
    assert_eq!(empty_object_string, Some("[object Object]".into()));
    assert_eq!(object_with_to_string_string, None);
    assert_eq!(bigint_with_separators_string, Some("10".into()));
}

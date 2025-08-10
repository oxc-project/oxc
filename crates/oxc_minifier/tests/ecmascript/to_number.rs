use oxc_allocator::Allocator;
use oxc_ast::{AstBuilder, ast::*};
use oxc_ecmascript::{GlobalContext, ToNumber, WithoutGlobalReferenceInformation};
use oxc_span::SPAN;

struct GlobalReferenceInformation {
    is_undefined_shadowed: bool,
}

impl<'a> GlobalContext<'a> for GlobalReferenceInformation {
    fn is_global_reference(&self, ident: &IdentifierReference<'a>) -> Option<bool> {
        if ident.name == "undefined" { Some(!self.is_undefined_shadowed) } else { None }
    }
}

#[test]
fn test() {
    let allocator = Allocator::default();
    let ast = AstBuilder::new(&allocator);

    let undefined = ast.expression_identifier(SPAN, "undefined");
    let shadowed_undefined_number =
        undefined.to_number(&GlobalReferenceInformation { is_undefined_shadowed: true });
    let global_undefined_number =
        undefined.to_number(&GlobalReferenceInformation { is_undefined_shadowed: false });

    let empty_object = ast.expression_object(SPAN, ast.vec());
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
    );
    let empty_object_number = empty_object.to_number(&WithoutGlobalReferenceInformation {});
    let object_with_to_string_number =
        object_with_to_string.to_number(&WithoutGlobalReferenceInformation {});

    assert_eq!(shadowed_undefined_number, None);
    assert!(global_undefined_number.is_some_and(f64::is_nan));
    assert!(empty_object_number.is_some_and(f64::is_nan));
    assert_eq!(object_with_to_string_number, None);
}

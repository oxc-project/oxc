use oxc_allocator::Allocator;
use oxc_ast::{AstBuilder, ast::*};
use oxc_ecmascript::{ToNumber, WithoutGlobalReferenceInformation};
use oxc_span::SPAN;

use super::GlobalReferenceInformation;

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

    // Test arrays with boolean elements - should convert to NaN
    let false_literal = ast.alloc_boolean_literal(SPAN, false);
    let true_literal = ast.alloc_boolean_literal(SPAN, true);
    let array_with_false =
        ast.expression_array(SPAN, ast.vec1(ArrayExpressionElement::BooleanLiteral(false_literal)));
    let array_with_true =
        ast.expression_array(SPAN, ast.vec1(ArrayExpressionElement::BooleanLiteral(true_literal)));
    let array_with_false_number = array_with_false.to_number(&WithoutGlobalReferenceInformation {});
    let array_with_true_number = array_with_true.to_number(&WithoutGlobalReferenceInformation {});

    // [false].toString() = "false", Number("false") = NaN
    assert!(array_with_false_number.is_some_and(f64::is_nan));
    // [true].toString() = "true", Number("true") = NaN
    assert!(array_with_true_number.is_some_and(f64::is_nan));
}

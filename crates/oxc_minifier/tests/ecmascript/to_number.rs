use oxc_allocator::{Allocator, ArenaVec};
use oxc_ast::{ast::*, builder::AstBuilder};
use oxc_ecmascript::{ToNumber, WithoutGlobalReferenceInformation};
use oxc_span::SPAN;

use super::GlobalReferenceInformation;

#[test]
fn test() {
    let allocator = Allocator::default();
    let ast = AstBuilder::new(&allocator);
    let ast = &ast;

    let undefined = Expression::new_identifier(SPAN, "undefined", ast);
    let shadowed_undefined_number =
        undefined.to_number(&GlobalReferenceInformation { is_undefined_shadowed: true });
    let global_undefined_number =
        undefined.to_number(&GlobalReferenceInformation { is_undefined_shadowed: false });

    let empty_object = Expression::new_object_expression(SPAN, [], ast);
    let object_with_to_string = Expression::new_object_expression(
        SPAN,
        ArenaVec::from_value_in(
            ObjectPropertyKind::new_object_property(
                SPAN,
                PropertyKind::Init,
                PropertyKey::new_static_identifier(SPAN, "toString", ast),
                Expression::new_string_literal(SPAN, "foo", None, ast),
                false,
                false,
                false,
                ast,
            ),
            ast,
        ),
        ast,
    );
    let empty_object_number = empty_object.to_number(&WithoutGlobalReferenceInformation {});
    let object_with_to_string_number =
        object_with_to_string.to_number(&WithoutGlobalReferenceInformation {});

    assert_eq!(shadowed_undefined_number, None);
    assert!(global_undefined_number.is_some_and(f64::is_nan));
    assert!(empty_object_number.is_some_and(f64::is_nan));
    assert_eq!(object_with_to_string_number, None);

    // Test arrays with boolean elements - should convert to NaN
    let false_literal = ArrayExpressionElement::new_boolean_literal(SPAN, false, ast);
    let true_literal = ArrayExpressionElement::new_boolean_literal(SPAN, true, ast);
    let array_with_false =
        Expression::new_array_expression(SPAN, ArenaVec::from_value_in(false_literal, ast), ast);
    let array_with_true =
        Expression::new_array_expression(SPAN, ArenaVec::from_value_in(true_literal, ast), ast);
    let array_with_false_number = array_with_false.to_number(&WithoutGlobalReferenceInformation {});
    let array_with_true_number = array_with_true.to_number(&WithoutGlobalReferenceInformation {});

    // [false].toString() = "false", Number("false") = NaN
    assert!(array_with_false_number.is_some_and(f64::is_nan));
    // [true].toString() = "true", Number("true") = NaN
    assert!(array_with_true_number.is_some_and(f64::is_nan));
}

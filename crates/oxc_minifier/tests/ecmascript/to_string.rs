use oxc_allocator::Allocator;
use oxc_ast::{AstBuilder, ast::*};
use oxc_ecmascript::{ToJsString, WithoutGlobalReferenceInformation};
use oxc_span::SPAN;

use super::GlobalReferenceInformation;

#[test]
fn test() {
    let allocator = Allocator::default();
    let ast = AstBuilder::new(&allocator);

    let undefined = ast.expression_identifier(SPAN, "undefined");
    let shadowed_undefined_string =
        undefined.to_js_string(&GlobalReferenceInformation { is_undefined_shadowed: true });
    let global_undefined_string =
        undefined.to_js_string(&GlobalReferenceInformation { is_undefined_shadowed: false });

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
    let empty_object_string = empty_object.to_js_string(&WithoutGlobalReferenceInformation {});
    let object_with_to_string_string =
        object_with_to_string.to_js_string(&WithoutGlobalReferenceInformation {});

    let bigint_with_separators =
        ast.expression_big_int_literal(SPAN, "10", Some(Atom::from("1_0n")), BigintBase::Decimal);
    let bigint_with_separators_string =
        bigint_with_separators.to_js_string(&WithoutGlobalReferenceInformation {});

    assert_eq!(shadowed_undefined_string, None);
    assert_eq!(global_undefined_string, Some("undefined".into()));
    assert_eq!(empty_object_string, Some("[object Object]".into()));
    assert_eq!(object_with_to_string_string, None);
    assert_eq!(bigint_with_separators_string, Some("10".into()));

    let num_cases = [
        (0.0, "0"),
        (-0.0, "0"),
        (1.0, "1"),
        (-1.0, "-1"),
        (f64::NAN, "NaN"),
        (-f64::NAN, "NaN"),
        (f64::INFINITY, "Infinity"),
        (f64::NEG_INFINITY, "-Infinity"),
    ];
    for (num, expected) in num_cases {
        let num_lit = ast.expression_numeric_literal(SPAN, num, None, NumberBase::Decimal);
        assert_eq!(
            num_lit.to_js_string(&WithoutGlobalReferenceInformation {}),
            Some(expected.into())
        );
    }
}

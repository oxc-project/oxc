use oxc_allocator::{Allocator, ArenaVec};
use oxc_ast::{ast::*, builder::AstBuilder};
use oxc_ecmascript::{ToJsString, WithoutGlobalReferenceInformation};
use oxc_span::SPAN;

use super::GlobalReferenceInformation;

#[test]
fn test() {
    let allocator = Allocator::default();
    let ast = AstBuilder::new(&allocator);
    let ast = &ast;

    let undefined = Expression::new_identifier(SPAN, "undefined", ast);
    let shadowed_undefined_string =
        undefined.to_js_string(&GlobalReferenceInformation { is_undefined_shadowed: true });
    let global_undefined_string =
        undefined.to_js_string(&GlobalReferenceInformation { is_undefined_shadowed: false });

    let empty_object = Expression::new_object_expression(SPAN, ArenaVec::new_in(ast), ast);
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
    let empty_object_string = empty_object.to_js_string(&WithoutGlobalReferenceInformation {});
    let object_with_to_string_string =
        object_with_to_string.to_js_string(&WithoutGlobalReferenceInformation {});

    let bigint_with_separators = Expression::new_big_int_literal(
        SPAN,
        "10",
        Some(Str::from("1_0n")),
        BigintBase::Decimal,
        ast,
    );
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
        let num_lit = Expression::new_numeric_literal(SPAN, num, None, NumberBase::Decimal, ast);
        assert_eq!(
            num_lit.to_js_string(&WithoutGlobalReferenceInformation {}),
            Some(expected.into())
        );
    }
}

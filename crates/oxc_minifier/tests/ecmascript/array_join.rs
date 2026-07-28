use oxc_allocator::{Allocator, ArenaBox, CloneIn};
use oxc_ast::{ast::*, builder::AstBuilder};
use oxc_ecmascript::{ArrayJoin, WithoutGlobalReferenceInformation};
use oxc_span::SPAN;

#[test]
fn test() {
    let allocator = Allocator::default();
    let ast = AstBuilder::new(&allocator);
    let ast = &ast;

    let array = ArrayExpression::new(
        SPAN,
        [
            ArrayExpressionElement::new_elision(SPAN, ast),
            ArrayExpressionElement::new_null_literal(SPAN, ast),
            ArrayExpressionElement::new_numeric_literal(
                SPAN,
                42f64,
                None,
                NumberBase::Decimal,
                ast,
            ),
            ArrayExpressionElement::new_string_literal(SPAN, "foo", None, ast),
            ArrayExpressionElement::new_boolean_literal(SPAN, true, ast),
            ArrayExpressionElement::new_big_int_literal(
                SPAN,
                "42",
                Some(Str::from("42n")),
                BigintBase::Decimal,
                ast,
            ),
        ],
        ast,
    );
    let mut array2 = array.clone_in(&allocator);
    array2.elements.push(ArrayExpressionElement::ArrayExpression(ArenaBox::new_in(array, ast)));
    array2.elements.push(ArrayExpressionElement::new_object_expression(SPAN, [], ast));
    let joined = array2.array_join(&WithoutGlobalReferenceInformation {}, Some("_"));
    assert_eq!(joined, Some("__42_foo_true_42_,,42,foo,true,42_[object Object]".to_string()));

    let joined2 = array2.array_join(&WithoutGlobalReferenceInformation {}, None);
    // By default, in `Array.prototype.toString`, the separator is a comma. However, in `Array.prototype.join`, the separator is none if not given.
    assert_eq!(joined2, Some(",,42,foo,true,42,,,42,foo,true,42,[object Object]".to_string()));
}

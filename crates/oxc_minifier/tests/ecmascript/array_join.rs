use oxc_allocator::{Allocator, CloneIn};
use oxc_ast::{ast::*, AstBuilder};
use oxc_ecmascript::ArrayJoin;
use oxc_span::SPAN;

#[test]
fn test() {
    let allocator = Allocator::default();
    let ast = AstBuilder::new(&allocator);
    let mut elements = ast.vec();
    elements.push(ast.array_expression_element_elision(SPAN));
    elements.push(ArrayExpressionElement::NullLiteral(ast.alloc(ast.null_literal(SPAN))));
    elements.push(ArrayExpressionElement::NumericLiteral(ast.alloc(ast.numeric_literal(
        SPAN,
        42f64,
        None,
        NumberBase::Decimal,
    ))));
    elements.push(ArrayExpressionElement::StringLiteral(
        ast.alloc(ast.string_literal(SPAN, "foo", None)),
    ));
    elements
        .push(ArrayExpressionElement::BooleanLiteral(ast.alloc(ast.boolean_literal(SPAN, true))));
    elements.push(ArrayExpressionElement::BigIntLiteral(ast.alloc(ast.big_int_literal(
        SPAN,
        "42n",
        BigintBase::Decimal,
    ))));
    let array = ast.array_expression(SPAN, elements.clone_in(&allocator), None);
    let mut array2 = array.clone_in(&allocator);
    array2.elements.push(ArrayExpressionElement::ArrayExpression(ast.alloc(array)));
    array2.elements.push(ArrayExpressionElement::ObjectExpression(
        ast.alloc(ast.object_expression(SPAN, ast.vec(), None)),
    ));
    let joined = array2.array_join(Some("_"));
    assert_eq!(joined, Some("__42_foo_true_42_,,42,foo,true,42_[object Object]".to_string()));

    let joined2 = array2.array_join(None);
    // By default, in `Array.prototype.toString`, the separator is a comma. However, in `Array.prototype.join`, the separator is none if not given.
    assert_eq!(joined2, Some(",,42,foo,true,42,,,42,foo,true,42,[object Object]".to_string()));
}

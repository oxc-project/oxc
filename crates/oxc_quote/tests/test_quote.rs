use oxc_quote::{jsquote_expr, private::*};

#[test]
fn playground() {
    let a = Allocator::default();
    let span = Span::empty(0);

    let expr: Expression = jsquote_expr!(&a, span, { 1234.5 + 2 });

    assert!(expr.content_eq(&Expression::BinaryExpression(Box::new_in(
        BinaryExpression {
            span,
            left: Expression::NumericLiteral(Box::new_in(
                NumericLiteral {
                    span,
                    value: 1234.5,
                    raw: Some(Atom::new_const("1234.5"),),
                    base: NumberBase::Float,
                },
                &a
            )),
            operator: BinaryOperator::Addition,
            right: Expression::NumericLiteral(Box::new_in(
                NumericLiteral {
                    span,
                    value: 2.0,
                    raw: Some(Atom::new_const("2"),),
                    base: NumberBase::Decimal,
                },
                &a
            )),
        },
        &a
    ))));
}

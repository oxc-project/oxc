use oxc_ast::{ast::*, AstBuilder};
use oxc_span::Span;
use oxc_syntax::operator::BinaryOperator;

use std::rc::Rc;

/// ES2016: Exponentiation Operator
///
/// References:
/// * <https://babel.dev/docs/babel-plugin-transform-exponentiation-operator>
/// * <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-exponentiation-operator>
///
pub struct ExponentiationOperator<'a> {
    ast: Rc<AstBuilder<'a>>,
}

impl<'a> ExponentiationOperator<'a> {
    pub fn new(ast: Rc<AstBuilder<'a>>) -> Self {
        Self { ast }
    }

    pub fn transform_expression<'b>(&mut self, expr: &'b mut Expression<'a>) {
        let Expression::BinaryExpression(binary_expression) = expr else { return };
        if binary_expression.operator != BinaryOperator::Exponential {
            return;
        }
        // left **  right
        let left = self.ast.move_expression(&mut binary_expression.left);
        let right = self.ast.move_expression(&mut binary_expression.right);
        // Math.pow
        let ident_math = IdentifierReference::new("Math".into(), Span::default());
        let object = self.ast.identifier_expression(ident_math);
        let property = IdentifierName::new(Span::default(), "pow".into());
        let callee = self.ast.static_member_expression(Span::default(), object, property, false);
        // Math.pow(left, right)
        let mut arguments = self.ast.new_vec_with_capacity(2);
        arguments.push(Argument::Expression(left));
        arguments.push(Argument::Expression(right));
        let call_expr = self.ast.call_expression(Span::default(), callee, arguments, false, None);
        *expr = call_expr;
    }
}

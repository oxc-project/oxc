use std::rc::Rc;

use oxc_ast::{ast::*, AstBuilder};
use oxc_span::Span;
use oxc_syntax::operator::{AssignmentOperator, LogicalOperator};

use crate::options::{TransformOptions, TransformTarget};

/// ES2021: Logical Assignment Operators
///
/// References:
/// * <https://babel.dev/docs/babel-plugin-transform-logical-assignment-operators>
/// * <https://github.com/babel/babel/blob/main/packages/babel-plugin-transform-logical-assignment-operator>
pub struct LogicalAssignmentOperators<'a> {
    ast: Rc<AstBuilder<'a>>,
}

impl<'a> LogicalAssignmentOperators<'a> {
    pub fn new(ast: Rc<AstBuilder<'a>>, options: &TransformOptions) -> Option<Self> {
        (options.target < TransformTarget::ES2021 || options.logical_assignment_operators)
            .then(|| Self { ast })
    }

    pub fn transform_expression<'b>(&mut self, expr: &'b mut Expression<'a>) {
        let Expression::AssignmentExpression(assignment_expr) = expr else { return };

        let operator = match assignment_expr.operator {
            AssignmentOperator::LogicalAnd => LogicalOperator::And,
            AssignmentOperator::LogicalOr => LogicalOperator::Or,
            AssignmentOperator::LogicalNullish => LogicalOperator::Coalesce,
            _ => return,
        };

        // Create the left hand side
        // a || (a = b)
        // ^     ^
        let left1: AssignmentTarget<'a> = self.ast.copy(&assignment_expr.left);
        let left2 = match &assignment_expr.left {
            AssignmentTarget::SimpleAssignmentTarget(target) => match target {
                SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) => {
                    self.ast.identifier_reference_expression((*ident).clone())
                }
                SimpleAssignmentTarget::MemberAssignmentTarget(member_expr) => {
                    let member_expr = self.ast.copy(&**member_expr);
                    self.ast.member_expression(member_expr)
                }
                // All other are TypeScript syntax.
                _ => return,
            },
            // It is a Syntax Error if AssignmentTargetType of LeftHandSideExpression is not simple.
            // So safe to return here.
            AssignmentTarget::AssignmentTargetPattern(_) => return,
        };

        // Create the right hand side
        // a || (a = b)
        //      ^^^^^^^
        let assign_op = AssignmentOperator::Assign;
        let right = self.ast.copy(&assignment_expr.right);
        let right = self.ast.assignment_expression(Span::default(), assign_op, left1, right);
        let right = self.ast.parenthesized_expression(Span::default(), right);

        let logical_expr = self.ast.logical_expression(Span::default(), left2, operator, right);
        *expr = logical_expr;
    }
}

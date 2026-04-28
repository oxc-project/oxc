use std::iter;

use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::ast_nodes::{AstNode, AstNodes};

#[derive(Debug, Clone, Copy)]
pub enum ExpressionLeftSide<'me, 'a, 'b> {
    Expression(&'b AstNode<'me, 'a, Expression<'a>>),
    AssignmentTarget(&'b AstNode<'me, 'a, AssignmentTarget<'a>>),
    SimpleAssignmentTarget(&'b AstNode<'me, 'a, SimpleAssignmentTarget<'a>>),
}

impl<'me, 'a, 'b> From<&'b AstNode<'me, 'a, Expression<'a>>> for ExpressionLeftSide<'me, 'a, 'b> {
    fn from(value: &'b AstNode<'me, 'a, Expression<'a>>) -> Self {
        Self::Expression(value)
    }
}

impl<'me, 'a, 'b> From<&'b AstNode<'me, 'a, AssignmentTarget<'a>>> for ExpressionLeftSide<'me, 'a, 'b> {
    fn from(value: &'b AstNode<'me, 'a, AssignmentTarget<'a>>) -> Self {
        Self::AssignmentTarget(value)
    }
}

impl<'me, 'a, 'b> From<&'b AstNode<'me, 'a, SimpleAssignmentTarget<'a>>> for ExpressionLeftSide<'me, 'a, 'b> {
    fn from(value: &'b AstNode<'me, 'a, SimpleAssignmentTarget<'a>>) -> Self {
        Self::SimpleAssignmentTarget(value)
    }
}

impl<'me, 'a, 'b> ExpressionLeftSide<'me, 'a, 'b> {
    pub fn leftmost(
        expression: &'b AstNode<'me, 'a, Expression<'a>>,
    ) -> &'b AstNode<'me, 'a, Expression<'a>> {
        let current: Self = expression.into();

        current.iter_expression().last().unwrap()
    }

    /// Returns the left side of an expression (an expression where the first child is a `Node` or [None]
    /// if the expression has no left side.
    pub fn left(&self) -> Option<Self> {
        match self {
            Self::Expression(expression) => match &expression.inner {
                Expression::SequenceExpression(b) => {
                    let expr = expression.with_inner(b.as_ref());
                    expr.expressions().first().map(Into::into)
                }
                Expression::StaticMemberExpression(b) => {
                    let expr = expression.with_inner(b.as_ref());
                    Some(expr.object().into())
                }
                Expression::ComputedMemberExpression(b) => {
                    let expr = expression.with_inner(b.as_ref());
                    Some(expr.object().into())
                }
                Expression::PrivateFieldExpression(b) => {
                    let expr = expression.with_inner(b.as_ref());
                    Some(expr.object().into())
                }
                Expression::TaggedTemplateExpression(b) => {
                    let expr = expression.with_inner(b.as_ref());
                    Some(expr.tag().into())
                }
                Expression::NewExpression(b) => {
                    let expr = expression.with_inner(b.as_ref());
                    Some(expr.callee().into())
                }
                Expression::CallExpression(b) => {
                    let expr = expression.with_inner(b.as_ref());
                    Some(expr.callee().into())
                }
                Expression::ConditionalExpression(b) => {
                    let expr = expression.with_inner(b.as_ref());
                    Some(expr.test().into())
                }
                Expression::TSAsExpression(b) => {
                    let expr = expression.with_inner(b.as_ref());
                    Some(expr.expression().into())
                }
                Expression::TSSatisfiesExpression(b) => {
                    let expr = expression.with_inner(b.as_ref());
                    Some(expr.expression().into())
                }
                Expression::TSNonNullExpression(b) => {
                    let expr = expression.with_inner(b.as_ref());
                    Some(expr.expression().into())
                }
                Expression::AssignmentExpression(b) => {
                    let expr = expression.with_inner(b.as_ref());
                    Some(Self::AssignmentTarget(expr.left()))
                }
                Expression::UpdateExpression(b) => {
                    let expr = expression.with_inner(b.as_ref());
                    if expr.prefix {
                        None
                    } else {
                        Some(Self::SimpleAssignmentTarget(expr.argument()))
                    }
                }
                Expression::BinaryExpression(b) => {
                    let binary = expression.with_inner(b.as_ref());
                    Some(binary.left().into())
                }
                Expression::LogicalExpression(b) => {
                    let logical = expression.with_inner(b.as_ref());
                    Some(logical.left().into())
                }
                Expression::ChainExpression(b) => {
                    let chain = expression.with_inner(b.as_ref());
                    let chain_expr = chain.expression();
                    match &chain_expr.inner {
                        Expression::CallExpression(b2) => {
                            let e = chain_expr.with_inner(b2.as_ref());
                            Some(e.callee().into())
                        }
                        Expression::TSNonNullExpression(b2) => {
                            let e = chain_expr.with_inner(b2.as_ref());
                            Some(e.expression().into())
                        }
                        Expression::ComputedMemberExpression(b2) => {
                            let e = chain_expr.with_inner(b2.as_ref());
                            Some(e.object().into())
                        }
                        Expression::StaticMemberExpression(b2) => {
                            let e = chain_expr.with_inner(b2.as_ref());
                            Some(e.object().into())
                        }
                        Expression::PrivateFieldExpression(b2) => {
                            let e = chain_expr.with_inner(b2.as_ref());
                            Some(e.object().into())
                        }
                        _ => unreachable!(),
                    }
                }
                _ => None,
            },
            // TODO: Restore as_ast_nodes-equivalent for AssignmentTarget — requires inline
            // match against AssignmentTarget enum variants. See NORTH_STAR.md.
            Self::AssignmentTarget(_target) => None,
            Self::SimpleAssignmentTarget(_target) => None,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = ExpressionLeftSide<'me, 'a, 'b>> {
        iter::successors(Some(*self), |f| match f {
            ExpressionLeftSide::Expression(expression) => Self::Expression(expression).left(),
            ExpressionLeftSide::AssignmentTarget(target) => Self::AssignmentTarget(target).left(),
            ExpressionLeftSide::SimpleAssignmentTarget(target) => {
                Self::SimpleAssignmentTarget(target).left()
            }
        })
    }

    pub fn iter_expression(&self) -> impl Iterator<Item = &'b AstNode<'me, 'a, Expression<'a>>> {
        self.iter().filter_map(|left| match left {
            ExpressionLeftSide::Expression(expression) => Some(expression),
            _ => None,
        })
    }

    pub fn span(&self) -> Span {
        match self {
            ExpressionLeftSide::Expression(expression) => expression.span(),
            ExpressionLeftSide::AssignmentTarget(target) => target.span(),
            ExpressionLeftSide::SimpleAssignmentTarget(target) => target.span(),
        }
    }

    fn get_left_side_of_assignment(node: &'b AstNodes<'me, 'a>) -> Option<ExpressionLeftSide<'me, 'a, 'b>> {
        match node {
            Expression::TSAsExpression(b) => {
                    let expr = expression.with_inner(b.as_ref());
                    Some(expr.expression().into())
                }
            Expression::TSSatisfiesExpression(b) => {
                    let expr = expression.with_inner(b.as_ref());
                    Some(expr.expression().into())
                }
            Expression::TSNonNullExpression(b) => {
                    let expr = expression.with_inner(b.as_ref());
                    Some(expr.expression().into())
                }
            Expression::TSTypeAssertion(b) => {
                    let expr = expression.with_inner(b.as_ref());
                    Some(expr.expression().into())
                }
            Expression::ComputedMemberExpression(b) => {
                    let expr = expression.with_inner(b.as_ref());
                    Some(expr.object().into())
                }
            Expression::StaticMemberExpression(b) => {
                    let expr = expression.with_inner(b.as_ref());
                    Some(expr.object().into())
                }
            Expression::PrivateFieldExpression(b) => {
                    let expr = expression.with_inner(b.as_ref());
                    Some(expr.object().into())
                }
            _ => None,
        }
    }
}

use std::iter;

use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::ast_nodes::{AstNode, AstNodes};

#[derive(Debug, Clone, Copy)]
pub enum ExpressionLeftSide<'a, 'b> {
    Expression(&'b AstNode<'a, Expression<'a>>),
    AssignmentTarget(&'b AstNode<'a, AssignmentTarget<'a>>),
    SimpleAssignmentTarget(&'b AstNode<'a, SimpleAssignmentTarget<'a>>),
}

impl<'a, 'b> From<&'b AstNode<'a, Expression<'a>>> for ExpressionLeftSide<'a, 'b> {
    fn from(value: &'b AstNode<'a, Expression<'a>>) -> Self {
        Self::Expression(value)
    }
}

impl<'a, 'b> From<&'b AstNode<'a, AssignmentTarget<'a>>> for ExpressionLeftSide<'a, 'b> {
    fn from(value: &'b AstNode<'a, AssignmentTarget<'a>>) -> Self {
        Self::AssignmentTarget(value)
    }
}

impl<'a, 'b> From<&'b AstNode<'a, SimpleAssignmentTarget<'a>>> for ExpressionLeftSide<'a, 'b> {
    fn from(value: &'b AstNode<'a, SimpleAssignmentTarget<'a>>) -> Self {
        Self::SimpleAssignmentTarget(value)
    }
}

impl<'a, 'b> ExpressionLeftSide<'a, 'b> {
    pub fn leftmost(
        expression: &'b AstNode<'a, Expression<'a>>,
    ) -> &'b AstNode<'a, Expression<'a>> {
        let mut current: Self = expression.into();

        current.iter_expression().last().unwrap()
    }

    /// Returns the left side of an expression (an expression where the first child is a `Node` or [None]
    /// if the expression has no left side.
    pub fn left(&self) -> Option<Self> {
        match self {
            Self::Expression(expression) => match expression.as_ast_nodes() {
                AstNodes::SequenceExpression(expr) => expr.expressions().first().map(Into::into),
                AstNodes::StaticMemberExpression(expr) => Some(expr.object().into()),
                AstNodes::ComputedMemberExpression(expr) => Some(expr.object().into()),
                AstNodes::PrivateFieldExpression(expr) => Some(expr.object().into()),
                AstNodes::TaggedTemplateExpression(expr) => Some(expr.tag().into()),
                AstNodes::NewExpression(expr) => Some(expr.callee().into()),
                AstNodes::CallExpression(expr) => Some(expr.callee().into()),
                AstNodes::ConditionalExpression(expr) => Some(expr.test().into()),
                AstNodes::TSAsExpression(expr) => Some(expr.expression().into()),
                AstNodes::TSSatisfiesExpression(expr) => Some(expr.expression().into()),
                AstNodes::TSNonNullExpression(expr) => Some(expr.expression().into()),
                AstNodes::AssignmentExpression(expr) => Some(Self::AssignmentTarget(expr.left())),
                AstNodes::UpdateExpression(expr) => {
                    if expr.prefix {
                        None
                    } else {
                        Some(Self::SimpleAssignmentTarget(expr.argument()))
                    }
                }
                AstNodes::BinaryExpression(binary) => Some(binary.left().into()),
                AstNodes::LogicalExpression(logical) => Some(logical.left().into()),
                AstNodes::ChainExpression(chain) => match &chain.expression().as_ast_nodes() {
                    AstNodes::CallExpression(expr) => Some(expr.callee().into()),
                    AstNodes::TSNonNullExpression(expr) => Some(expr.expression().into()),
                    AstNodes::ComputedMemberExpression(expr) => Some(expr.object().into()),
                    AstNodes::StaticMemberExpression(expr) => Some(expr.object().into()),
                    AstNodes::PrivateFieldExpression(expr) => Some(expr.object().into()),
                    _ => {
                        unreachable!()
                    }
                },
                _ => None,
            },
            Self::AssignmentTarget(target) => {
                Self::get_left_side_of_assignment(target.as_ast_nodes())
            }
            Self::SimpleAssignmentTarget(target) => {
                Self::get_left_side_of_assignment(target.as_ast_nodes())
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = ExpressionLeftSide<'a, 'b>> {
        iter::successors(Some(*self), |f| match f {
            ExpressionLeftSide::Expression(expression) => Self::Expression(expression).left(),
            ExpressionLeftSide::AssignmentTarget(target) => Self::AssignmentTarget(target).left(),
            ExpressionLeftSide::SimpleAssignmentTarget(target) => {
                Self::SimpleAssignmentTarget(target).left()
            }
            _ => None,
        })
    }

    pub fn iter_expression(&self) -> impl Iterator<Item = &'b AstNode<'a, Expression<'a>>> {
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

    fn get_left_side_of_assignment(node: &'b AstNodes<'a>) -> Option<ExpressionLeftSide<'a, 'b>> {
        match node {
            AstNodes::TSAsExpression(expr) => Some(expr.expression().into()),
            AstNodes::TSSatisfiesExpression(expr) => Some(expr.expression().into()),
            AstNodes::TSNonNullExpression(expr) => Some(expr.expression().into()),
            AstNodes::TSTypeAssertion(expr) => Some(expr.expression().into()),
            AstNodes::ComputedMemberExpression(expr) => Some(expr.object().into()),
            AstNodes::StaticMemberExpression(expr) => Some(expr.object().into()),
            AstNodes::PrivateFieldExpression(expr) => Some(expr.object().into()),
            _ => None,
        }
    }
}

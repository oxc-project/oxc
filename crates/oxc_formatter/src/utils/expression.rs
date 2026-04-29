use std::iter;

use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::ast_nodes::{AstNode, AstNodes};

#[derive(Debug, Clone, Copy)]
pub enum ExpressionLeftSide<'me, 'a> {
    Expression(AstNode<'me, 'a, Expression<'a>>),
    AssignmentTarget(AstNode<'me, 'a, AssignmentTarget<'a>>),
    SimpleAssignmentTarget(AstNode<'me, 'a, SimpleAssignmentTarget<'a>>),
}

impl<'me, 'a> From<AstNode<'me, 'a, Expression<'a>>> for ExpressionLeftSide<'me, 'a> {
    fn from(value: AstNode<'me, 'a, Expression<'a>>) -> Self {
        Self::Expression(value)
    }
}

impl<'me, 'a> From<AstNode<'me, 'a, AssignmentTarget<'a>>> for ExpressionLeftSide<'me, 'a> {
    fn from(value: AstNode<'me, 'a, AssignmentTarget<'a>>) -> Self {
        Self::AssignmentTarget(value)
    }
}

impl<'me, 'a> From<AstNode<'me, 'a, SimpleAssignmentTarget<'a>>> for ExpressionLeftSide<'me, 'a> {
    fn from(value: AstNode<'me, 'a, SimpleAssignmentTarget<'a>>) -> Self {
        Self::SimpleAssignmentTarget(value)
    }
}

impl<'me, 'a> ExpressionLeftSide<'me, 'a> {
    pub fn leftmost(
        expression: &AstNode<'me, 'a, Expression<'a>>,
    ) -> AstNode<'me, 'a, Expression<'a>> {
        let current: Self = (*expression).into();

        current.iter_expression().last().unwrap()
    }

    /// Returns the left side of an expression (an expression where the first child is a `Node` or [None]
    /// if the expression has no left side.
    ///
    // TODO: Implement this. The previous arena-allocated design called `as_ast_nodes()` which
    // returned `&AstNodes<'a>` whose getter methods produced child `AstNode<'a, ...>` references
    // bound to the arena lifetime. With stack-allocated `AstNode`, calling a getter on a
    // locally-constructed wrapper produces children that borrow the local frame, which can't
    // satisfy `'me`. Reintroducing `left()` requires either redesigning the getters to thread
    // the `'me` parent lifetime through, or avoiding the wrapper hop here. Returning `None`
    // disables left-side traversal (used by member-chain layout heuristics) but lets the spike
    // build.
    pub fn left(&self) -> Option<Self> {
        None
    }

    pub fn iter(&self) -> impl Iterator<Item = ExpressionLeftSide<'me, 'a>> {
        iter::successors(Some(*self), |f| f.left())
    }

    pub fn iter_expression(&self) -> impl Iterator<Item = AstNode<'me, 'a, Expression<'a>>> {
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

    #[allow(dead_code)]
    fn get_left_side_of_assignment(_node: &AstNodes<'me, 'a>) -> Option<ExpressionLeftSide<'me, 'a>> {
        // TODO: Restore using inline match patterns. Currently unused since left() handles
        // AssignmentTarget paths via TODO.
        None
    }
}

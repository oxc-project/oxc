use std::iter;

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::ast_nodes::{AstNode, AstNodeKind, AstNodes};

#[derive(Debug, Clone, Copy)]
pub enum ExpressionLeftSide<'a, 'b> {
    Expression(AstNode<'b, 'a, Expression<'a>>),
    AssignmentTarget(AstNode<'b, 'a, AssignmentTarget<'a>>),
    SimpleAssignmentTarget(AstNode<'b, 'a, SimpleAssignmentTarget<'a>>),
}

impl<'a, 'b> From<AstNode<'b, 'a, Expression<'a>>> for ExpressionLeftSide<'a, 'b> {
    fn from(value: AstNode<'b, 'a, Expression<'a>>) -> Self {
        Self::Expression(value)
    }
}

impl<'a, 'b> From<AstNode<'b, 'a, AssignmentTarget<'a>>> for ExpressionLeftSide<'a, 'b> {
    fn from(value: AstNode<'b, 'a, AssignmentTarget<'a>>) -> Self {
        Self::AssignmentTarget(value)
    }
}

impl<'a, 'b> From<AstNode<'b, 'a, SimpleAssignmentTarget<'a>>> for ExpressionLeftSide<'a, 'b> {
    fn from(value: AstNode<'b, 'a, SimpleAssignmentTarget<'a>>) -> Self {
        Self::SimpleAssignmentTarget(value)
    }
}

impl<'a, 'b> ExpressionLeftSide<'a, 'b> {
    /// Returns the leftmost expression that starts `expression` (`expression` itself if none).
    pub fn leftmost<'e>(expression: &'e Expression<'e>) -> &'e Expression<'e> {
        LeftSide::Expression(expression)
            .iter()
            .filter_map(|left| match left {
                LeftSide::Expression(expression) => Some(expression),
                _ => None,
            })
            .last()
            .unwrap()
    }

    /// See [Self::left_of].
    ///
    /// The node of `self` is in `allocator`, as it is the parent of the left side.
    pub fn left(&self, allocator: &'a Allocator) -> Option<Self> {
        let (node, assignment) = match self {
            Self::Expression(expression) => (expression.as_kind(), false),
            Self::AssignmentTarget(target) => (target.as_kind(), true),
            Self::SimpleAssignmentTarget(target) => (target.as_kind(), true),
        };
        // Allocate only when the walk continues
        LeftSide::of(&node.as_ast_nodes(), assignment)?;
        Self::left_of(node.in_arena(allocator), assignment, allocator)
    }

    /// [LeftSide::of] with [AstNode]s, for a walk that needs the parents.
    fn left_of(
        node: &'b AstNodeKind<'b, 'a>,
        assignment: bool,
        allocator: &'a Allocator,
    ) -> Option<Self> {
        let left = match node {
            AstNodeKind::SequenceExpression(expr) => expr.expressions().first().map(Into::into),
            AstNodeKind::StaticMemberExpression(expr) => Some(expr.object().into()),
            AstNodeKind::ComputedMemberExpression(expr) => Some(expr.object().into()),
            AstNodeKind::PrivateFieldExpression(expr) => Some(expr.object().into()),
            AstNodeKind::TaggedTemplateExpression(expr) => Some(expr.tag().into()),
            AstNodeKind::CallExpression(expr) => Some(expr.callee().into()),
            AstNodeKind::ConditionalExpression(expr) => Some(expr.test().into()),
            AstNodeKind::TSAsExpression(expr) => Some(expr.expression().into()),
            AstNodeKind::TSSatisfiesExpression(expr) => Some(expr.expression().into()),
            AstNodeKind::TSNonNullExpression(expr) => Some(expr.expression().into()),
            AstNodeKind::TSInstantiationExpression(expr) => Some(expr.expression().into()),
            AstNodeKind::TSTypeAssertion(expr) if assignment => Some(expr.expression().into()),
            AstNodeKind::AssignmentExpression(expr) => Some(expr.left().into()),
            AstNodeKind::UpdateExpression(expr) => {
                if expr.prefix {
                    None
                } else {
                    Some(expr.argument().into())
                }
            }
            AstNodeKind::BinaryExpression(binary) => Some(binary.left().into()),
            AstNodeKind::LogicalExpression(logical) => Some(logical.left().into()),
            AstNodeKind::ChainExpression(chain) => {
                Self::left_of(chain.expression().as_kind().in_arena(allocator), false, allocator)
            }
            _ => None,
        };
        debug_assert_eq!(
            left.map(|left| left.span()),
            LeftSide::of(&node.as_ast_nodes(), assignment).map(|left| left.span())
        );
        left
    }

    /// Same as [Self::left_of], but only reads the span, so no [AstNode] is created.
    pub fn left_span_of(node: &AstNodes<'_, '_>) -> Option<Span> {
        LeftSide::of(node, false).map(|left| left.span())
    }

    pub fn iter(
        &self,
        allocator: &'a Allocator,
    ) -> impl Iterator<Item = ExpressionLeftSide<'a, 'b>> {
        iter::successors(Some(*self), move |left| left.left(allocator))
    }

    pub fn span(&self) -> Span {
        match self {
            ExpressionLeftSide::Expression(expression) => expression.span(),
            ExpressionLeftSide::AssignmentTarget(target) => target.span(),
            ExpressionLeftSide::SimpleAssignmentTarget(target) => target.span(),
        }
    }
}

/// [ExpressionLeftSide] over the AST, without [AstNode]s.
#[derive(Debug, Clone, Copy)]
pub enum LeftSide<'a> {
    Expression(&'a Expression<'a>),
    AssignmentTarget(&'a AssignmentTarget<'a>),
    SimpleAssignmentTarget(&'a SimpleAssignmentTarget<'a>),
}

impl<'a> LeftSide<'a> {
    /// Returns the child that starts `node` (e.g. `a` of `a.b`, `a + b`, `a ? b : c`),
    /// or [None] if `node` starts with its own token.
    ///
    /// With `assignment` (`node` is an assignment target),
    /// a `TSTypeAssertion` is walked into as well (`(<T>a) = b`),
    /// as `get_left_side_of_assignment` did.
    ///
    /// The table of the left sides, [ExpressionLeftSide::left_of] follows it with [AstNode]s.
    fn of(node: &AstNodes<'_, 'a>, assignment: bool) -> Option<Self> {
        let left = match node {
            AstNodes::SequenceExpression(expr) => expr.inner().expressions.first()?,
            AstNodes::StaticMemberExpression(expr) => &expr.inner().object,
            AstNodes::ComputedMemberExpression(expr) => &expr.inner().object,
            AstNodes::PrivateFieldExpression(expr) => &expr.inner().object,
            AstNodes::TaggedTemplateExpression(expr) => &expr.inner().tag,
            AstNodes::CallExpression(expr) => &expr.inner().callee,
            AstNodes::ConditionalExpression(expr) => &expr.inner().test,
            AstNodes::TSAsExpression(expr) => &expr.inner().expression,
            AstNodes::TSSatisfiesExpression(expr) => &expr.inner().expression,
            AstNodes::TSNonNullExpression(expr) => &expr.inner().expression,
            AstNodes::TSInstantiationExpression(expr) => &expr.inner().expression,
            AstNodes::TSTypeAssertion(expr) if assignment => &expr.inner().expression,
            AstNodes::AssignmentExpression(expr) => {
                return Some(Self::AssignmentTarget(&expr.inner().left));
            }
            AstNodes::UpdateExpression(expr) => {
                let expr = expr.inner();
                return (!expr.prefix).then_some(Self::SimpleAssignmentTarget(&expr.argument));
            }
            AstNodes::BinaryExpression(binary) => &binary.inner().left,
            AstNodes::LogicalExpression(logical) => &logical.inner().left,
            AstNodes::ChainExpression(chain) => match &chain.inner().expression {
                ChainElement::CallExpression(call) => &call.callee,
                ChainElement::TSNonNullExpression(expr) => &expr.expression,
                element => element.as_member_expression()?.object(),
            },
            _ => return None,
        };
        Some(Self::Expression(left))
    }

    /// See [Self::of].
    pub fn left(self) -> Option<Self> {
        let (node, assignment) = match self {
            Self::Expression(expression) => {
                (AstNode::new(expression, AstNodes::Dummy()).as_kind(), false)
            }
            Self::AssignmentTarget(target) => {
                (AstNode::new(target, AstNodes::Dummy()).as_kind(), true)
            }
            Self::SimpleAssignmentTarget(target) => {
                (AstNode::new(target, AstNodes::Dummy()).as_kind(), true)
            }
        };
        Self::of(&node.as_ast_nodes(), assignment)
    }

    pub fn iter(self) -> impl Iterator<Item = Self> {
        iter::successors(Some(self), |left| left.left())
    }
}

impl GetSpan for LeftSide<'_> {
    fn span(&self) -> Span {
        match self {
            Self::Expression(expression) => expression.span(),
            Self::AssignmentTarget(target) => target.span(),
            Self::SimpleAssignmentTarget(target) => target.span(),
        }
    }
}

/// Strip `ChainExpression` / `TSNonNullExpression` wrappers
/// and return the inner [`CallExpression`], if that is what they wrap.
///
/// Equivalent to Prettier's `isCallExpression(stripChainElementWrappers(node))`.
/// A general "strip" cannot return `&Expression` because [`ChainElement`] is a separate enum,
/// hence the call-expression-specialized shape;
/// see also the member-expression sibling [`is_member_expression_without_chain_wrappers`].
pub fn as_call_expression_without_chain_wrappers<'a, 'b>(
    expression: &'b Expression<'a>,
) -> Option<&'b CallExpression<'a>> {
    match expression {
        Expression::CallExpression(call) => Some(call),
        Expression::TSNonNullExpression(non_null) => {
            as_call_expression_without_chain_wrappers(&non_null.expression)
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::CallExpression(call) => Some(call),
            ChainElement::TSNonNullExpression(non_null) => {
                as_call_expression_without_chain_wrappers(&non_null.expression)
            }
            _ => None,
        },
        _ => None,
    }
}

/// Strip `ChainExpression` / `TSNonNullExpression` wrappers
/// and check whether they wrap a [`MemberExpression`].
///
/// Equivalent to Prettier's `isMemberExpression(stripChainElementWrappers(node))`;
/// the member-expression sibling of [`as_call_expression_without_chain_wrappers`].
pub fn is_member_expression_without_chain_wrappers(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::TSNonNullExpression(non_null) => {
            is_member_expression_without_chain_wrappers(&non_null.expression)
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::TSNonNullExpression(non_null) => {
                is_member_expression_without_chain_wrappers(&non_null.expression)
            }
            element => element.as_member_expression().is_some(),
        },
        _ => expression.as_member_expression().is_some(),
    }
}

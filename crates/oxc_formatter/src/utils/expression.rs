use std::iter;

use oxc_ast::ast::*;
use oxc_span::GetSpan;

use crate::{ast_nodes::AstNode, formatter::Formatter};

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
        f: &Formatter<'_, 'a>,
    ) -> AstNode<'me, 'a, Expression<'a>> {
        let current: Self = (*expression).into();

        current.iter_expression(f).last().unwrap()
    }

    /// Return the left-side child of `self` (the expression whose first child is this node's
    /// own first child), or `None` if this node has no left side.
    ///
    /// Each step promotes the relevant typed wrapper into the arena (via `f.allocator()`) so the
    /// returned wrapper carries a `'me`-lifetime parent reference.
    pub fn left(&self, f: &Formatter<'_, 'a>) -> Option<Self> {
        match self {
            Self::Expression(expression) => match &expression.inner {
                Expression::SequenceExpression(b) => {
                    let spec = f.allocator().alloc(expression.with_inner(b.as_ref()));
                    spec.expressions().first().map(Into::into)
                }
                Expression::StaticMemberExpression(b) => {
                    let spec = f.allocator().alloc(expression.with_inner(b.as_ref()));
                    Some(spec.object().into())
                }
                Expression::ComputedMemberExpression(b) => {
                    let spec = f.allocator().alloc(expression.with_inner(b.as_ref()));
                    Some(spec.object().into())
                }
                Expression::PrivateFieldExpression(b) => {
                    let spec = f.allocator().alloc(expression.with_inner(b.as_ref()));
                    Some(spec.object().into())
                }
                Expression::TaggedTemplateExpression(b) => {
                    let spec = f.allocator().alloc(expression.with_inner(b.as_ref()));
                    Some(spec.tag().into())
                }
                Expression::NewExpression(b) => {
                    let spec = f.allocator().alloc(expression.with_inner(b.as_ref()));
                    Some(spec.callee().into())
                }
                Expression::CallExpression(b) => {
                    let spec = f.allocator().alloc(expression.with_inner(b.as_ref()));
                    Some(spec.callee().into())
                }
                Expression::ConditionalExpression(b) => {
                    let spec = f.allocator().alloc(expression.with_inner(b.as_ref()));
                    Some(spec.test().into())
                }
                Expression::TSAsExpression(b) => {
                    let spec = f.allocator().alloc(expression.with_inner(b.as_ref()));
                    Some(spec.expression().into())
                }
                Expression::TSSatisfiesExpression(b) => {
                    let spec = f.allocator().alloc(expression.with_inner(b.as_ref()));
                    Some(spec.expression().into())
                }
                Expression::TSNonNullExpression(b) => {
                    let spec = f.allocator().alloc(expression.with_inner(b.as_ref()));
                    Some(spec.expression().into())
                }
                Expression::AssignmentExpression(b) => {
                    let spec = f.allocator().alloc(expression.with_inner(b.as_ref()));
                    Some(Self::AssignmentTarget(spec.left()))
                }
                Expression::UpdateExpression(b) => {
                    if b.prefix {
                        None
                    } else {
                        let spec = f.allocator().alloc(expression.with_inner(b.as_ref()));
                        Some(Self::SimpleAssignmentTarget(spec.argument()))
                    }
                }
                Expression::BinaryExpression(b) => {
                    let spec = f.allocator().alloc(expression.with_inner(b.as_ref()));
                    Some(spec.left().into())
                }
                Expression::LogicalExpression(b) => {
                    let spec = f.allocator().alloc(expression.with_inner(b.as_ref()));
                    Some(spec.left().into())
                }
                Expression::ChainExpression(b) => {
                    let chain_spec = f.allocator().alloc(expression.with_inner(b.as_ref()));
                    let chain_inner = chain_spec.expression();
                    match &chain_inner.inner {
                        ChainElement::CallExpression(c) => {
                            let spec = f.allocator().alloc(chain_inner.with_inner(c.as_ref()));
                            Some(spec.callee().into())
                        }
                        ChainElement::TSNonNullExpression(c) => {
                            let spec = f.allocator().alloc(chain_inner.with_inner(c.as_ref()));
                            Some(spec.expression().into())
                        }
                        ChainElement::ComputedMemberExpression(c) => {
                            let spec = f.allocator().alloc(chain_inner.with_inner(c.as_ref()));
                            Some(spec.object().into())
                        }
                        ChainElement::StaticMemberExpression(c) => {
                            let spec = f.allocator().alloc(chain_inner.with_inner(c.as_ref()));
                            Some(spec.object().into())
                        }
                        ChainElement::PrivateFieldExpression(c) => {
                            let spec = f.allocator().alloc(chain_inner.with_inner(c.as_ref()));
                            Some(spec.object().into())
                        }
                    }
                }
                _ => None,
            },
            Self::AssignmentTarget(target) => Self::get_left_side_of_assignment_target(target, f),
            Self::SimpleAssignmentTarget(target) => {
                Self::get_left_side_of_simple_assignment_target(target, f)
            }
        }
    }

    pub fn iter<'f>(
        &self,
        f: &'f Formatter<'_, 'a>,
    ) -> impl Iterator<Item = ExpressionLeftSide<'me, 'a>> + 'f
    where
        'me: 'f,
        'a: 'f,
    {
        iter::successors(Some(*self), move |s| s.left(f))
    }

    pub fn iter_expression<'f>(
        &self,
        f: &'f Formatter<'_, 'a>,
    ) -> impl Iterator<Item = AstNode<'me, 'a, Expression<'a>>> + 'f
    where
        'me: 'f,
        'a: 'f,
    {
        self.iter(f).filter_map(|left| match left {
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

    fn get_left_side_of_assignment_target(
        target: &AstNode<'me, 'a, AssignmentTarget<'a>>,
        f: &Formatter<'_, 'a>,
    ) -> Option<Self> {
        match &target.inner {
            AssignmentTarget::TSAsExpression(b) => {
                let spec = f.allocator().alloc(target.with_inner(b.as_ref()));
                Some(spec.expression().into())
            }
            AssignmentTarget::TSSatisfiesExpression(b) => {
                let spec = f.allocator().alloc(target.with_inner(b.as_ref()));
                Some(spec.expression().into())
            }
            AssignmentTarget::TSNonNullExpression(b) => {
                let spec = f.allocator().alloc(target.with_inner(b.as_ref()));
                Some(spec.expression().into())
            }
            AssignmentTarget::TSTypeAssertion(b) => {
                let spec = f.allocator().alloc(target.with_inner(b.as_ref()));
                Some(spec.expression().into())
            }
            AssignmentTarget::ComputedMemberExpression(b) => {
                let spec = f.allocator().alloc(target.with_inner(b.as_ref()));
                Some(spec.object().into())
            }
            AssignmentTarget::StaticMemberExpression(b) => {
                let spec = f.allocator().alloc(target.with_inner(b.as_ref()));
                Some(spec.object().into())
            }
            AssignmentTarget::PrivateFieldExpression(b) => {
                let spec = f.allocator().alloc(target.with_inner(b.as_ref()));
                Some(spec.object().into())
            }
            _ => None,
        }
    }

    fn get_left_side_of_simple_assignment_target(
        target: &AstNode<'me, 'a, SimpleAssignmentTarget<'a>>,
        f: &Formatter<'_, 'a>,
    ) -> Option<Self> {
        match &target.inner {
            SimpleAssignmentTarget::TSAsExpression(b) => {
                let spec = f.allocator().alloc(target.with_inner(b.as_ref()));
                Some(spec.expression().into())
            }
            SimpleAssignmentTarget::TSSatisfiesExpression(b) => {
                let spec = f.allocator().alloc(target.with_inner(b.as_ref()));
                Some(spec.expression().into())
            }
            SimpleAssignmentTarget::TSNonNullExpression(b) => {
                let spec = f.allocator().alloc(target.with_inner(b.as_ref()));
                Some(spec.expression().into())
            }
            SimpleAssignmentTarget::TSTypeAssertion(b) => {
                let spec = f.allocator().alloc(target.with_inner(b.as_ref()));
                Some(spec.expression().into())
            }
            SimpleAssignmentTarget::ComputedMemberExpression(b) => {
                let spec = f.allocator().alloc(target.with_inner(b.as_ref()));
                Some(spec.object().into())
            }
            SimpleAssignmentTarget::StaticMemberExpression(b) => {
                let spec = f.allocator().alloc(target.with_inner(b.as_ref()));
                Some(spec.object().into())
            }
            SimpleAssignmentTarget::PrivateFieldExpression(b) => {
                let spec = f.allocator().alloc(target.with_inner(b.as_ref()));
                Some(spec.object().into())
            }
            _ => None,
        }
    }
}

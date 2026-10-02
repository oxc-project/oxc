use oxc_allocator::ArenaVec;
use oxc_ast::ast::{Expression, SequenceExpression};
use oxc_span::GetSpan;

use crate::TraverseCtx;

use super::PeepholeOptimizations;

impl<'a> PeepholeOptimizations {
    /// Joins two expressions into a sequence, preserving evaluation order.
    pub fn join_sequence(
        a: Expression<'a>,
        b: Expression<'a>,
        ctx: &TraverseCtx<'a>,
    ) -> Expression<'a> {
        if let Expression::SequenceExpression(mut sequence_expr) = a {
            // `(a, b); c`
            sequence_expr.expressions.push(b);
            return Expression::SequenceExpression(sequence_expr);
        }
        let span = a.span();
        let exprs = if let Expression::SequenceExpression(sequence_expr) = b {
            // `a; (b, c)`
            ArenaVec::from_iter_in(std::iter::once(a).chain(sequence_expr.unbox().expressions), ctx)
        } else {
            // `a; b`
            ArenaVec::from_array_in([a, b], ctx)
        };
        Expression::new_sequence_expression(span, exprs, ctx)
    }

    /// Returns the final expression, descending through non-empty trailing sequences.
    /// For `(a, (b, c))`, returns `c`. An empty outer sequence returns [`None`];
    /// an empty trailing sequence is returned as-is.
    pub fn last_expression_in_sequence<'s>(
        seq: &'s SequenceExpression<'a>,
    ) -> Option<&'s Expression<'a>> {
        let mut last = seq.expressions.last()?;
        while let Expression::SequenceExpression(inner) = last {
            let Some(next) = inner.expressions.last() else { break };
            last = next;
        }
        Some(last)
    }

    /// Removes and returns the final expression, descending through trailing sequences
    /// and discarding any that become empty.
    /// For `(a, (b, c))`, returns `c` and leaves `(a, (b))`.
    /// Returns [`None`] if the outer sequence or a trailing nested sequence is already empty.
    pub fn pop_expression_from_sequence(
        seq: &mut SequenceExpression<'a>,
    ) -> Option<Expression<'a>> {
        if let Some(Expression::SequenceExpression(inner)) = seq.expressions.last_mut() {
            let last = Self::pop_expression_from_sequence(inner)?;
            if inner.expressions.is_empty() {
                seq.expressions.pop();
            }
            return Some(last);
        }

        seq.expressions.pop()
    }
}

use oxc_ast::ast::{Expression, SequenceExpression};

pub(crate) trait SequenceExpressionExt<'a> {
    /// Returns the final expression, descending through non-empty trailing sequences.
    /// For `(a, (b, c))`, returns `c`. An empty outer sequence returns [`None`];
    /// an empty trailing sequence is returned as-is.
    fn last_expression(&self) -> Option<&Expression<'a>>;

    /// Removes and returns the final expression, descending through trailing sequences
    /// and discarding any that become empty.
    /// For `(a, (b, c))`, returns `c` and leaves `(a, (b))`.
    /// Returns [`None`] if the outer sequence or a trailing nested sequence is already empty.
    fn pop_last_expression(&mut self) -> Option<Expression<'a>>;
}

impl<'a> SequenceExpressionExt<'a> for SequenceExpression<'a> {
    fn last_expression(&self) -> Option<&Expression<'a>> {
        let last = self.expressions.last()?;
        match last {
            Expression::SequenceExpression(seq) if !seq.expressions.is_empty() => {
                seq.last_expression()
            }
            last => Some(last),
        }
    }

    fn pop_last_expression(&mut self) -> Option<Expression<'a>> {
        if let Some(Expression::SequenceExpression(seq)) = self.expressions.last_mut() {
            let last = seq.pop_last_expression()?;
            if seq.expressions.is_empty() {
                self.expressions.pop();
            }
            return Some(last);
        }

        self.expressions.pop()
    }
}
